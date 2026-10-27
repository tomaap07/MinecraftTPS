
create table if not exists public.tp_servers (
  id uuid primary key default gen_random_uuid(),
  name varchar(120) not null check (length(btrim(name)) > 0),
  ip varchar(255) not null check (length(btrim(ip)) > 0),
  port integer not null default 25565 check (port between 1 and 65535),
  category varchar(50) not null default 'Survival',
  region varchar(20) not null default 'Global',
  language varchar(10) not null default 'ES',
  version_text varchar(60) not null default '',
  is_online boolean not null default false,
  players_online integer not null default 0 check (players_online >= 0),
  players_max integer not null default 0 check (players_max >= 0),
  motd_clean text not null default '' check (length(motd_clean) <= 2048),
  motd jsonb not null default '[]'::jsonb check (jsonb_typeof(motd) = 'array'),
  favicon_url text not null default '' check (length(favicon_url) <= 90000),
  latency_ms integer not null default 0 check (latency_ms >= 0),
  protocol integer not null default 0,
  is_featured boolean not null default false,
  featured_until timestamptz,
  sponsor_url text,
  priority_tier smallint not null default 2 check (priority_tier between 1 and 3),
  fail_streak integer not null default 0 check (fail_streak >= 0),
  last_pinged_at timestamptz,
  next_ping_at timestamptz not null default now(),
  lease_token uuid,
  lease_until timestamptz,
  source_url text,
  created_at timestamptz not null default now(),
  search_document tsvector generated always as (to_tsvector('simple',name || ' ' || ip)) stored,
  unique (ip,port),
  check (ip = lower(ip)),
  check (not is_featured or featured_until is not null),
  check (sponsor_url is null or sponsor_url ~ '^https://')
);
create index if not exists idx_tp_servers_listing on public.tp_servers (is_featured desc,is_online desc,players_online desc,id);
create index if not exists idx_tp_servers_filters on public.tp_servers (category,language,region);
create index if not exists idx_tp_servers_polling on public.tp_servers (next_ping_at,priority_tier,last_pinged_at);
create index if not exists idx_tp_servers_search on public.tp_servers using gin (search_document);

create table if not exists public.tp_api_keys (
  id uuid primary key default gen_random_uuid(),
  owner_name varchar(100) not null,
  api_key_hash char(64) not null unique check (api_key_hash ~ '^[a-f0-9]{64}$'),
  tier varchar(20) not null default 'free' check (tier in ('free','partner','admin')),
  daily_limit integer not null default 5000 check (daily_limit between 1 and 10000000),
  requests_today integer not null default 0 check (requests_today >= 0),
  usage_day date not null default (timezone('utc',now())::date),
  is_active boolean not null default true,
  created_at timestamptz not null default now()
);
create table if not exists public.tp_rate_limits (
  subject_hash char(64) not null check (subject_hash ~ '^[a-f0-9]{64}$'),
  window_start timestamptz not null,
  hits integer not null default 1,
  primary key (subject_hash,window_start)
);
create index if not exists idx_tp_rate_limits_cleanup on public.tp_rate_limits (window_start);

alter table public.tp_servers enable row level security;
alter table public.tp_api_keys enable row level security;
alter table public.tp_rate_limits enable row level security;
revoke all on public.tp_servers,public.tp_api_keys,public.tp_rate_limits from anon,authenticated;
grant select (id,name,ip,port,category,region,language,version_text,is_online,players_online,players_max,motd_clean,motd,favicon_url,latency_ms,protocol,is_featured,featured_until,sponsor_url,last_pinged_at,created_at) on public.tp_servers to anon,authenticated;
grant all on public.tp_servers,public.tp_api_keys,public.tp_rate_limits to service_role;
drop policy if exists tp_servers_public_read on public.tp_servers;
create policy tp_servers_public_read on public.tp_servers for select to anon,authenticated using (true);

create or replace function public.tp_check_rate_limit(p_subject text,p_limit integer,p_seconds integer default 60)
returns jsonb language plpgsql security invoker set search_path = public,pg_temp as $$
declare bucket timestamptz; hits_count integer;
begin
  if p_limit not between 1 and 100000 or p_seconds not between 1 and 86400 then raise exception 'invalid rate limit'; end if;
  bucket := to_timestamp(floor(extract(epoch from clock_timestamp())/p_seconds)*p_seconds);
  insert into public.tp_rate_limits (subject_hash,window_start,hits) values (p_subject,bucket,1)
    on conflict (subject_hash,window_start) do update set hits=least(tp_rate_limits.hits+1,2147483647) returning hits into hits_count;
  return jsonb_build_object('allowed',hits_count<=p_limit,'remaining',greatest(0,p_limit-hits_count));
end $$;

create or replace function public.tp_consume_api_key(p_hash text)
returns jsonb language plpgsql security invoker set search_path = public,pg_temp as $$
declare key_row public.tp_api_keys; today date := timezone('utc',now())::date;
begin
  select * into key_row from public.tp_api_keys where api_key_hash=p_hash and is_active for update;
  if not found then return jsonb_build_object('allowed',false,'reason','invalid'); end if;
  if key_row.usage_day<>today then key_row.requests_today:=0; end if;
  if key_row.requests_today>=key_row.daily_limit then return jsonb_build_object('allowed',false,'reason','limit'); end if;
  update public.tp_api_keys set requests_today=key_row.requests_today+1,usage_day=today where id=key_row.id;
  return jsonb_build_object('allowed',true,'tier',key_row.tier,'remaining',key_row.daily_limit-key_row.requests_today-1);
end $$;

create or replace function public.tp_list_servers(p_page integer default 1,p_limit integer default 12,p_category text default 'all',p_language text default 'all',p_region text default 'all',p_search text default '',p_version text default '')
returns jsonb language sql stable security invoker set search_path = public,pg_temp as $
  with filtered as (
    select id,name,ip,port,category,region,language,version_text,is_online,players_online,players_max,motd_clean,motd,favicon_url,latency_ms,protocol,
      (is_featured and featured_until>now()) as is_featured,featured_until,sponsor_url,last_pinged_at,created_at
    from public.tp_servers
    where (p_category='all' or category ilike '%'||p_category||'%')
      and (p_language='all' or lower(language)=lower(p_language))
      and (p_region='all' or lower(region) like '%'||lower(p_region)||'%')
      and (p_search='' or search_document @@ websearch_to_tsquery('simple',p_search) or ip ilike '%'||p_search||'%')
      and (p_version='' or version_text ilike '%'||p_version||'%')
  ), listed as (
    select * from filtered order by is_featured desc,is_online desc,players_online desc,id
    limit greatest(1,least(p_limit,48)) offset (greatest(p_page,1)-1)*greatest(1,least(p_limit,48))
  ) select jsonb_build_object('servers',coalesce((select jsonb_agg(to_jsonb(listed)) from listed),'[]'::jsonb),'total',(select count(*) from filtered));
$;

create or replace function public.tp_claim_servers(p_limit integer,p_lease uuid)
returns setof public.tp_servers language plpgsql security invoker set search_path = public,pg_temp as $$
begin
  update public.tp_servers set is_featured=false where is_featured and featured_until<=now();
  delete from public.tp_rate_limits where window_start<now()-interval '2 days';
  return query with candidates as (
    select id from public.tp_servers where next_ping_at<=now() and (lease_until is null or lease_until<now())
    order by priority_tier,next_ping_at,id for update skip locked limit greatest(1,least(p_limit,96))
  ) update public.tp_servers s set lease_token=p_lease,lease_until=now()+interval '3 minutes'
    from candidates c where s.id=c.id returning s.*;
end $$;

create or replace function public.tp_finish_server_ping(p_id uuid,p_lease uuid,p_state jsonb)
returns boolean language plpgsql security invoker set search_path = public,pg_temp as $$
declare current_row public.tp_servers; online_state boolean := coalesce((p_state->>'is_online')::boolean,false); next_tier smallint; failures integer;
begin
  select * into current_row from public.tp_servers where id=p_id and lease_token=p_lease for update;
  if not found then return false; end if;
  failures:=case when online_state then 0 else current_row.fail_streak+1 end;
  next_tier:=case when (current_row.is_featured and current_row.featured_until>now()) or (online_state and (p_state->>'players_online')::integer>50) then 1 when not online_state and failures>=5 then 3 when online_state then 2 else current_row.priority_tier end;
  update public.tp_servers set is_online=online_state,players_online=case when online_state then greatest(0,(p_state->>'players_online')::integer) else 0 end,
    players_max=case when online_state then greatest(0,(p_state->>'players_max')::integer) else players_max end,
    version_text=case when online_state then left(p_state->>'version_text',60) else version_text end,
    motd_clean=case when online_state then left(p_state->>'motd_clean',2048) else motd_clean end,
    motd=case when online_state then p_state->'motd' else motd end,
    favicon_url=case when online_state then p_state->>'favicon_url' else favicon_url end,
    latency_ms=case when online_state then greatest(0,(p_state->>'latency_ms')::integer) else 0 end,
    protocol=case when online_state then coalesce((p_state->>'protocol')::integer,0) else protocol end,
    fail_streak=failures,priority_tier=next_tier,last_pinged_at=now(),
    next_ping_at=now()+case next_tier when 1 then interval '5 minutes' when 2 then interval '30 minutes' else interval '12 hours' end,
    lease_token=null,lease_until=null where id=p_id and lease_token=p_lease;
  return true;
end $$;

revoke execute on function public.tp_check_rate_limit(text,integer,integer),public.tp_consume_api_key(text),public.tp_list_servers(integer,integer,text,text,text,text,text),public.tp_claim_servers(integer,uuid),public.tp_finish_server_ping(uuid,uuid,jsonb) from public,anon,authenticated;
grant execute on function public.tp_check_rate_limit(text,integer,integer),public.tp_consume_api_key(text),public.tp_list_servers(integer,integer,text,text,text,text,text),public.tp_claim_servers(integer,uuid),public.tp_finish_server_ping(uuid,uuid,jsonb) to service_role;
