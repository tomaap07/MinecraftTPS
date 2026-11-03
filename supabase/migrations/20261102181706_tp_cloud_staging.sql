begin;
create schema if not exists tp_private;
revoke all on schema tp_private from public, anon;
grant usage on schema tp_private to authenticated, service_role;

create table if not exists public.tp_cloud_profiles (
  user_id uuid primary key references auth.users(id) on delete cascade,
  handle varchar(24) not null unique check(handle ~ '^[a-z0-9_]{3,24}$'),
  display_name varchar(48) not null check(length(btrim(display_name)) between 1 and 48),
  created_at timestamptz not null default now()
);
create table if not exists tp_private.admin_roles (
  user_id uuid primary key references auth.users(id) on delete cascade,
  role text not null check(role in ('owner','moderator')),
  granted_at timestamptz not null default now()
);
create table if not exists public.tp_cloud_blocks (
  owner_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  blocked_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  created_at timestamptz not null default now(), primary key(owner_id,blocked_id), check(owner_id<>blocked_id)
);
create index if not exists tp_cloud_blocked_idx on public.tp_cloud_blocks(blocked_id,owner_id);
create table if not exists public.tp_cloud_friend_requests (
  id uuid primary key default gen_random_uuid(),
  sender_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  recipient_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  status text not null default 'pending' check(status in ('pending','accepted','rejected','cancelled')),
  created_at timestamptz not null default now(), decided_at timestamptz,
  check(sender_id<>recipient_id)
);
create index if not exists tp_cloud_friend_recipient_idx on public.tp_cloud_friend_requests(recipient_id,status,created_at desc);
create index if not exists tp_cloud_friend_sender_idx on public.tp_cloud_friend_requests(sender_id,status,created_at desc);
create unique index if not exists tp_cloud_friend_pair_idx on public.tp_cloud_friend_requests(least(sender_id,recipient_id),greatest(sender_id,recipient_id)) where status in ('pending','accepted');

create table if not exists tp_private.validated_capes (
  id uuid primary key default gen_random_uuid(), user_id uuid not null references auth.users(id) on delete cascade,
  sha256 char(64) not null check(sha256 ~ '^[a-f0-9]{64}$'),
  object_path text not null unique check(length(object_path)<200),
  width integer not null check(width in (64,128,256,512)), height integer not null check(height=width/2),
  byte_size integer not null check(byte_size between 64 and 1048576),
  created_at timestamptz not null default now(), used_at timestamptz
);
create table if not exists public.tp_cloud_cosmetics (
  id text primary key check(id ~ '^[a-z0-9-]{3,80}$'), title varchar(80) not null,
  kind text not null check(kind in ('cape','wings','crown','aura')),
  source text not null check(source in ('welcome','quest','community')),
  approved boolean not null default false, withdrawn boolean not null default false,
  sha256 char(64), object_path text, author_id uuid references public.tp_cloud_profiles(user_id) on delete set null,
  created_at timestamptz not null default now()
);
create table if not exists public.tp_cloud_submissions (
  id uuid primary key default gen_random_uuid(), user_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  ticket_id uuid not null unique references tp_private.validated_capes(id), title varchar(80) not null check(length(btrim(title)) between 3 and 80),
  state text not null default 'pending' check(state in ('pending','approved','rejected','withdrawn')),
  reason varchar(1000), created_at timestamptz not null default now(), reviewed_at timestamptz,
  reviewed_by uuid references auth.users(id) on delete set null
);
create index if not exists tp_cloud_submissions_author_idx on public.tp_cloud_submissions(user_id,state,created_at desc);
create index if not exists tp_cloud_submissions_queue_idx on public.tp_cloud_submissions(state,created_at);
create table if not exists public.tp_cloud_reports (
  id uuid primary key default gen_random_uuid(), reporter_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  cosmetic_id text not null references public.tp_cloud_cosmetics(id), reason varchar(1000) not null check(length(btrim(reason)) between 10 and 1000),
  created_at timestamptz not null default now(), unique(reporter_id,cosmetic_id)
);
create table if not exists tp_private.moderation_audit (
  id uuid primary key default gen_random_uuid(), actor_id uuid references auth.users(id) on delete set null,
  submission_id uuid references public.tp_cloud_submissions(id) on delete set null, action text not null,
  reason text, created_at timestamptz not null default now()
);
create table if not exists tp_private.quest_events (
  event_id uuid primary key, user_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  kind text not null check(kind in ('active_minutes','room_completed')),
  quantity integer not null check(quantity between 1 and 60), created_at timestamptz not null default now()
);
create index if not exists tp_cloud_quest_events_user_idx on tp_private.quest_events(user_id,kind);
create table if not exists public.tp_cloud_entitlements (
  user_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  cosmetic_id text not null references public.tp_cloud_cosmetics(id),
  source text not null, granted_at timestamptz not null default now(), primary key(user_id,cosmetic_id)
);
create table if not exists public.tp_cloud_quest_claims (
  user_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  quest_id text not null check(quest_id in ('friends-3','adventure-60','artist','rooms-5')),
  cosmetic_id text not null references public.tp_cloud_cosmetics(id), claimed_at timestamptz not null default now(),
  primary key(user_id,quest_id)
);
create table if not exists public.tp_cloud_partners (
  id uuid primary key default gen_random_uuid(), name varchar(80) not null,
  official_url text not null check(official_url ~ '^https://[A-Za-z0-9-]+([.][A-Za-z0-9-]+)+(:443)?(/[^[:cntrl:]\\]*)?$' and official_url !~ '[[:cntrl:]]'),
  permission_evidence text not null check(length(btrim(permission_evidence)) between 10 and 2000),
  active boolean not null default false, updated_at timestamptz not null default now()
);
create table if not exists public.tp_cloud_rooms (
  id uuid primary key default gen_random_uuid(), host_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  game_version varchar(48) not null check(game_version ~ '^[a-zA-Z0-9._+-]{1,48}$'),
  loader text not null check(loader in ('vanilla','fabric','forge','neoforge','quilt','optifine')),
  state text not null default 'waiting_lan' check(state in ('waiting_lan','closed')),
  max_members integer not null default 8 check(max_members between 2 and 8),
  created_at timestamptz not null default now(), expires_at timestamptz not null default now()+interval '4 hours'
);
create index if not exists tp_cloud_rooms_host_idx on public.tp_cloud_rooms(host_id,state,created_at desc);
create table if not exists public.tp_cloud_room_members (
  room_id uuid not null references public.tp_cloud_rooms(id) on delete cascade,
  user_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  state text not null default 'authorized' check(state in ('authorized','revoked')),
  joined_at timestamptz not null default now(), primary key(room_id,user_id)
);
create index if not exists tp_cloud_room_member_user_idx on public.tp_cloud_room_members(user_id,room_id);
create table if not exists tp_private.room_invites (
  code_hash bytea primary key, room_id uuid not null references public.tp_cloud_rooms(id) on delete cascade,
  recipient_id uuid not null references public.tp_cloud_profiles(user_id) on delete cascade,
  created_at timestamptz not null default now(), expires_at timestamptz not null default now()+interval '15 minutes',
  used_at timestamptz, revoked_at timestamptz
);

insert into public.tp_cloud_cosmetics(id,title,kind,source,approved) values
 ('nebula-infinity','Nebulosa Infinita','cape','welcome',true),
 ('celestial-seraph','Serafín Celestial','wings','welcome',true),
 ('elemental-eclipse','Eclipse Elemental','crown','welcome',true),
 ('friendship','Amistad','cape','quest',true),('first-adventure','Primera aventura','aura','quest',true),
 ('digital-artist','Artista digital','crown','quest',true),('shared-adventure','Aventura compartida','wings','quest',true)
on conflict(id) do nothing;

create or replace function tp_private.cloud_user() returns uuid language plpgsql stable security definer set search_path='' as $$
declare actor uuid := auth.uid(); begin
 if actor is null or not exists(select 1 from auth.users u where u.id=actor and u.email_confirmed_at is not null and not coalesce(u.is_anonymous,false)) then raise exception 'TP_AUTH_REQUIRED' using errcode='42501'; end if;
 return actor;
end $$;
create or replace function tp_private.cloud_owner() returns boolean language sql stable security definer set search_path='' as $$
 select auth.uid() is not null and exists(select 1 from tp_private.admin_roles r where r.user_id=auth.uid() and r.role='owner')
$$;
create or replace function tp_private.cloud_blocked(a uuid,b uuid) returns boolean language sql stable security definer set search_path='' as $$
 select exists(select 1 from public.tp_cloud_blocks x where (x.owner_id=a and x.blocked_id=b) or (x.owner_id=b and x.blocked_id=a))
$$;
create or replace function tp_private.cloud_room_member(room uuid) returns boolean language sql stable security definer set search_path='' as $$
 select auth.uid() is not null and exists(select 1 from public.tp_cloud_rooms r where r.id=room and r.expires_at>now() and r.state<>'closed' and (r.host_id=auth.uid() or exists(select 1 from public.tp_cloud_room_members m where m.room_id=r.id and m.user_id=auth.uid() and m.state='authorized')))
$$;

create or replace function tp_private.cloud_command(action text,args jsonb) returns jsonb language plpgsql security definer set search_path='' as $$
declare actor uuid:=tp_private.cloud_user(); other uuid; ident uuid; code text; item text; count_value bigint; row_value record; ticket record; existing record;
begin
 if action is null or length(action)>40 or args is null or jsonb_typeof(args)<>'object' or pg_column_size(args)>4096 then raise exception 'TP_INVALID_COMMAND' using errcode='22023'; end if;
 if action='profile' then
  insert into public.tp_cloud_profiles(user_id,handle,display_name) values(actor,lower(args->>'handle'),args->>'displayName')
  on conflict(user_id) do update set handle=excluded.handle,display_name=excluded.display_name;
  insert into public.tp_cloud_entitlements(user_id,cosmetic_id,source) select actor,c.id,'welcome' from public.tp_cloud_cosmetics c where c.source='welcome' and c.approved and not c.withdrawn on conflict do nothing;
  return jsonb_build_object('userId',actor);
 end if;
 if not exists(select 1 from public.tp_cloud_profiles p where p.user_id=actor) then raise exception 'TP_PROFILE_REQUIRED' using errcode='42501'; end if;
 if action='capabilities' then return jsonb_build_object('owner',tp_private.cloud_owner(),'transport','not_configured'); end if;
 if action='friend_request' then
  other:=(args->>'recipient')::uuid;
  if actor=other or tp_private.cloud_blocked(actor,other) or not exists(select 1 from public.tp_cloud_profiles p where p.user_id=other) then raise exception 'TP_REQUEST_UNAVAILABLE' using errcode='42501'; end if;
  perform pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtextextended(actor::text,3));
  perform pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtextextended(least(actor,other)::text||greatest(actor,other)::text,0));
  if (select count(*) from public.tp_cloud_friend_requests f where f.sender_id=actor and f.created_at>now()-interval '10 minutes')>=10 then raise exception 'TP_RATE_LIMIT' using errcode='54000'; end if;
  insert into public.tp_cloud_friend_requests(sender_id,recipient_id) values(actor,other) returning id into ident;
  return jsonb_build_object('id',ident,'status','pending');
 elsif action='friend_response' then
  select * into row_value from public.tp_cloud_friend_requests f where f.id=(args->>'request')::uuid for update;
  if not found or row_value.recipient_id<>actor or row_value.status<>'pending' or tp_private.cloud_blocked(actor,row_value.sender_id) then raise exception 'TP_REQUEST_UNAVAILABLE' using errcode='42501'; end if;
  update public.tp_cloud_friend_requests set status=case when (args->>'accept')::boolean then 'accepted' else 'rejected' end,decided_at=now() where id=row_value.id;
  return jsonb_build_object('id',row_value.id);
 elsif action='block' then
  other:=(args->>'user')::uuid;
  if other=actor then raise exception 'TP_INVALID_TARGET'; end if;
  if (args->>'blocked')::boolean then
   insert into public.tp_cloud_blocks(owner_id,blocked_id) values(actor,other) on conflict do nothing;
   update public.tp_cloud_friend_requests set status='cancelled',decided_at=now() where status in ('pending','accepted') and ((sender_id=actor and recipient_id=other) or(sender_id=other and recipient_id=actor));
   update tp_private.room_invites set revoked_at=now() where recipient_id in(actor,other) and room_id in(select r.id from public.tp_cloud_rooms r where r.host_id in(actor,other));
   update public.tp_cloud_room_members set state='revoked' where (user_id=other and room_id in(select id from public.tp_cloud_rooms where host_id=actor)) or(user_id=actor and room_id in(select id from public.tp_cloud_rooms where host_id=other));
  else delete from public.tp_cloud_blocks where owner_id=actor and blocked_id=other; end if;
  return jsonb_build_object('blocked',(args->>'blocked')::boolean);
 elsif action='submit_cape' then
  if not coalesce((args->>'rightsAccepted')::boolean,false) then raise exception 'TP_RIGHTS_REQUIRED' using errcode='42501'; end if;
  perform pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtextextended(actor::text,1));
  if (select count(*) from public.tp_cloud_submissions s where s.user_id=actor and s.state='pending')>=3 then raise exception 'TP_PENDING_LIMIT' using errcode='54000'; end if;
  select * into ticket from tp_private.validated_capes c where c.id=(args->>'ticket')::uuid and c.user_id=actor and c.used_at is null and c.created_at>now()-interval '1 hour' for update;
  if not found then raise exception 'TP_VALIDATED_UPLOAD_REQUIRED' using errcode='42501'; end if;
  insert into public.tp_cloud_submissions(user_id,ticket_id,title) values(actor,ticket.id,args->>'title') returning id into ident;
  update tp_private.validated_capes set used_at=now() where id=ticket.id;
  return jsonb_build_object('id',ident,'state','pending');
 elsif action='moderate' then
  if not tp_private.cloud_owner() then raise exception 'TP_OWNER_REQUIRED' using errcode='42501'; end if;
  if args->>'decision' not in ('approved','rejected','withdrawn') or length(btrim(coalesce(args->>'reason','')))<3 then raise exception 'TP_MODERATION_REASON_REQUIRED'; end if;
  select * into row_value from public.tp_cloud_submissions s where s.id=(args->>'submission')::uuid for update;
  if not found then raise exception 'TP_SUBMISSION_NOT_FOUND'; end if;
  select * into ticket from tp_private.validated_capes c where c.id=row_value.ticket_id;
  item:='community-'||row_value.id::text;
  if args->>'decision'='approved' then
   if row_value.state<>'pending' then raise exception 'TP_SUBMISSION_NOT_PENDING'; end if;
   insert into public.tp_cloud_cosmetics(id,title,kind,source,approved,sha256,object_path,author_id) values(item,row_value.title,'cape','community',true,ticket.sha256,ticket.object_path,row_value.user_id);
  elsif args->>'decision'='withdrawn' then update public.tp_cloud_cosmetics set withdrawn=true where id=item;
  elsif row_value.state<>'pending' then raise exception 'TP_SUBMISSION_NOT_PENDING'; end if;
  update public.tp_cloud_submissions set state=args->>'decision',reason=args->>'reason',reviewed_at=now(),reviewed_by=actor where id=row_value.id;
  insert into tp_private.moderation_audit(actor_id,submission_id,action,reason) values(actor,row_value.id,args->>'decision',args->>'reason');
  return jsonb_build_object('id',row_value.id,'state',args->>'decision');
 elsif action='report' then
  insert into public.tp_cloud_reports(reporter_id,cosmetic_id,reason) values(actor,args->>'cosmetic',args->>'reason') on conflict(reporter_id,cosmetic_id) do update set reason=excluded.reason;
  return jsonb_build_object('received',true);
 elsif action='claim' then
  item:=case args->>'quest' when 'friends-3' then 'friendship' when 'adventure-60' then 'first-adventure' when 'artist' then 'digital-artist' when 'rooms-5' then 'shared-adventure' else null end;
  if item is null then raise exception 'TP_UNKNOWN_QUEST'; end if;
  if args->>'quest'='friends-3' then
   select count(distinct case when f.sender_id=actor then f.recipient_id else f.sender_id end) into count_value from public.tp_cloud_friend_requests f where f.status='accepted' and actor in(f.sender_id,f.recipient_id);
   if count_value<3 then raise exception 'TP_QUEST_INCOMPLETE' using errcode='42501'; end if;
  elsif args->>'quest'='artist' then
   if not exists(select 1 from public.tp_cloud_submissions s where s.user_id=actor) then raise exception 'TP_QUEST_INCOMPLETE' using errcode='42501'; end if;
  else
   select coalesce(sum(e.quantity),0) into count_value from tp_private.quest_events e where e.user_id=actor and e.kind=case args->>'quest' when 'adventure-60' then 'active_minutes' else 'room_completed' end;
   if count_value<(case args->>'quest' when 'adventure-60' then 60 else 5 end) then raise exception 'TP_QUEST_INCOMPLETE' using errcode='42501'; end if;
  end if;
  insert into public.tp_cloud_quest_claims(user_id,quest_id,cosmetic_id) values(actor,args->>'quest',item) on conflict do nothing;
  insert into public.tp_cloud_entitlements(user_id,cosmetic_id,source) values(actor,item,'quest') on conflict do nothing;
  return jsonb_build_object('cosmeticId',item,'claimed',true);
 elsif action='partner' then
  if not tp_private.cloud_owner() then raise exception 'TP_OWNER_REQUIRED' using errcode='42501'; end if;
  ident:=coalesce(nullif(args->>'id','')::uuid,gen_random_uuid());
  insert into public.tp_cloud_partners(id,name,official_url,permission_evidence,active) values(ident,args->>'name',args->>'url',args->>'evidence',(args->>'active')::boolean)
  on conflict(id) do update set name=excluded.name,official_url=excluded.official_url,permission_evidence=excluded.permission_evidence,active=excluded.active,updated_at=now();
  return jsonb_build_object('id',ident);
 elsif action='room_create' then
  perform pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtextextended(actor::text,4));
  if (select count(*) from public.tp_cloud_rooms r where r.host_id=actor and r.state<>'closed' and r.expires_at>now())>=2 then raise exception 'TP_ROOM_LIMIT' using errcode='54000'; end if;
  insert into public.tp_cloud_rooms(host_id,game_version,loader) values(actor,args->>'game',args->>'loader') returning id into ident;
  return jsonb_build_object('id',ident,'state','waiting_lan','transport','not_configured');
 elsif action='room_invite' then
  other:=(args->>'recipient')::uuid;
  select * into row_value from public.tp_cloud_rooms r where r.id=(args->>'room')::uuid and r.host_id=actor and r.state<>'closed' and r.expires_at>now() for update;
  if not found or actor=other or tp_private.cloud_blocked(actor,other) or not exists(select 1 from public.tp_cloud_friend_requests f where f.status='accepted' and ((f.sender_id=actor and f.recipient_id=other) or(f.sender_id=other and f.recipient_id=actor))) then raise exception 'TP_INVITE_UNAVAILABLE' using errcode='42501'; end if;
  if (select count(*) from tp_private.room_invites i where i.room_id=row_value.id and i.created_at>now()-interval '1 minute')>=10 then raise exception 'TP_RATE_LIMIT' using errcode='54000'; end if;
  code:=replace(gen_random_uuid()::text||gen_random_uuid()::text,'-','');
  insert into tp_private.room_invites(code_hash,room_id,recipient_id) values(pg_catalog.sha256(convert_to(code,'UTF8')),row_value.id,other);
  return jsonb_build_object('code',code,'expiresInSeconds',900,'transport','not_configured');
 elsif action='room_join' then
  if length(coalesce(args->>'code',''))<>64 then raise exception 'TP_INVITE_UNAVAILABLE' using errcode='42501'; end if;
  select * into existing from tp_private.room_invites i where i.code_hash=pg_catalog.sha256(convert_to(args->>'code','UTF8')) and i.recipient_id=actor and i.expires_at>now() and i.used_at is null and i.revoked_at is null for update;
  if not found then raise exception 'TP_INVITE_UNAVAILABLE' using errcode='42501'; end if;
  select * into row_value from public.tp_cloud_rooms r where r.id=existing.room_id and r.expires_at>now() and r.state<>'closed' for update;
  if not found or tp_private.cloud_blocked(actor,row_value.host_id) then raise exception 'TP_INVITE_UNAVAILABLE' using errcode='42501'; end if;
  if row_value.game_version is distinct from args->>'game' or row_value.loader is distinct from args->>'loader' then raise exception 'TP_VERSION_MISMATCH'; end if;
  if (select count(*) from public.tp_cloud_room_members m where m.room_id=row_value.id and m.state='authorized')>=row_value.max_members-1 then raise exception 'TP_ROOM_FULL'; end if;
  insert into public.tp_cloud_room_members(room_id,user_id) values(row_value.id,actor) on conflict(room_id,user_id) do update set state='authorized',joined_at=now();
  update tp_private.room_invites set used_at=now() where code_hash=existing.code_hash;
  return jsonb_build_object('id',row_value.id,'state','authorized','transport','not_configured');
 elsif action='room_close' then
  update public.tp_cloud_rooms set state='closed' where id=(args->>'room')::uuid and host_id=actor returning id into ident;
  if ident is null then raise exception 'TP_ROOM_UNAVAILABLE' using errcode='42501'; end if;
  update public.tp_cloud_room_members set state='revoked' where room_id=ident;
  update tp_private.room_invites set revoked_at=now() where room_id=ident;
  return jsonb_build_object('id',ident,'state','closed');
 end if;
 raise exception 'TP_UNKNOWN_ACTION';
end $$;

create or replace function public.tp_cloud_command(p_action text,p_args jsonb default '{}'::jsonb) returns jsonb language sql security invoker set search_path='' as $$ select tp_private.cloud_command(p_action,p_args) $$;
create or replace function tp_private.cloud_validated_upload(p_user uuid,p_sha text,p_object text,p_width int,p_height int,p_size int) returns uuid language plpgsql security definer set search_path='' as $$
declare ticket uuid; begin
 perform pg_catalog.pg_advisory_xact_lock(pg_catalog.hashtextextended(p_user::text,2));
 if (select count(*) from tp_private.validated_capes c where c.user_id=p_user and c.created_at>now()-interval '10 minutes')>=6 then raise exception 'TP_UPLOAD_RATE_LIMIT' using errcode='54000'; end if;
 if p_object !~ ('^'||p_user::text||'/[a-f0-9-]{36}\.png$') then raise exception 'TP_UPLOAD_PATH_INVALID'; end if;
 insert into tp_private.validated_capes(user_id,sha256,object_path,width,height,byte_size) values(p_user,p_sha,p_object,p_width,p_height,p_size) returning id into ticket;return ticket;
end $$;
create or replace function public.tp_cloud_validated_upload(p_user uuid,p_sha text,p_object text,p_width int,p_height int,p_size int) returns uuid language sql security invoker set search_path='' as $$ select tp_private.cloud_validated_upload(p_user,p_sha,p_object,p_width,p_height,p_size) $$;
create or replace function tp_private.cloud_record_event(p_user uuid,p_event uuid,p_kind text,p_quantity int) returns boolean language plpgsql security definer set search_path='' as $$
begin
 if p_kind='room_completed' and p_quantity<>1 then raise exception 'TP_EVENT_INVALID'; end if;
 insert into tp_private.quest_events(event_id,user_id,kind,quantity) values(p_event,p_user,p_kind,p_quantity) on conflict(event_id) do nothing;
 return found;
end $$;
create or replace function public.tp_cloud_record_event(p_user uuid,p_event uuid,p_kind text,p_quantity int) returns boolean language sql security invoker set search_path='' as $$ select tp_private.cloud_record_event(p_user,p_event,p_kind,p_quantity) $$;

do $$ declare tbl text; begin
 foreach tbl in array array['tp_cloud_profiles','tp_cloud_blocks','tp_cloud_friend_requests','tp_cloud_cosmetics','tp_cloud_submissions','tp_cloud_reports','tp_cloud_entitlements','tp_cloud_quest_claims','tp_cloud_partners','tp_cloud_rooms','tp_cloud_room_members'] loop
  execute format('alter table public.%I enable row level security',tbl);
  execute format('revoke all on public.%I from public,anon,authenticated',tbl);
  execute format('grant select on public.%I to authenticated',tbl);
 end loop;
 foreach tbl in array array['admin_roles','validated_capes','moderation_audit','quest_events','room_invites'] loop execute format('alter table tp_private.%I enable row level security',tbl); execute format('revoke all on tp_private.%I from public,anon,authenticated',tbl); end loop;
end $$;

drop policy if exists tp_cloud_profile_read on public.tp_cloud_profiles;
create policy tp_cloud_profile_read on public.tp_cloud_profiles for select to authenticated using((select auth.uid()) is not null);
drop policy if exists tp_cloud_blocks_own on public.tp_cloud_blocks;
create policy tp_cloud_blocks_own on public.tp_cloud_blocks for select to authenticated using(owner_id=(select auth.uid()));
drop policy if exists tp_cloud_friend_own on public.tp_cloud_friend_requests;
create policy tp_cloud_friend_own on public.tp_cloud_friend_requests for select to authenticated using((select auth.uid()) in(sender_id,recipient_id));
drop policy if exists tp_cloud_cosmetics_read on public.tp_cloud_cosmetics;
create policy tp_cloud_cosmetics_read on public.tp_cloud_cosmetics for select to anon,authenticated using(approved and not withdrawn);
grant select on public.tp_cloud_cosmetics to anon;
drop policy if exists tp_cloud_submissions_own on public.tp_cloud_submissions;
create policy tp_cloud_submissions_own on public.tp_cloud_submissions for select to authenticated using(user_id=(select auth.uid()) or (select tp_private.cloud_owner()));
drop policy if exists tp_cloud_reports_own on public.tp_cloud_reports;
create policy tp_cloud_reports_own on public.tp_cloud_reports for select to authenticated using(reporter_id=(select auth.uid()) or (select tp_private.cloud_owner()));
drop policy if exists tp_cloud_entitlements_own on public.tp_cloud_entitlements;
create policy tp_cloud_entitlements_own on public.tp_cloud_entitlements for select to authenticated using(user_id=(select auth.uid()));
drop policy if exists tp_cloud_claims_own on public.tp_cloud_quest_claims;
create policy tp_cloud_claims_own on public.tp_cloud_quest_claims for select to authenticated using(user_id=(select auth.uid()));
drop policy if exists tp_cloud_partners_read on public.tp_cloud_partners;
create policy tp_cloud_partners_read on public.tp_cloud_partners for select to anon,authenticated using(active);
grant select(id,name,official_url) on public.tp_cloud_partners to anon;
revoke select on public.tp_cloud_partners from authenticated;
grant select(id,name,official_url,active,updated_at) on public.tp_cloud_partners to authenticated;
drop policy if exists tp_cloud_rooms_member on public.tp_cloud_rooms;
create policy tp_cloud_rooms_member on public.tp_cloud_rooms for select to authenticated using(host_id=(select auth.uid()) or tp_private.cloud_room_member(id));
drop policy if exists tp_cloud_room_members_own on public.tp_cloud_room_members;
create policy tp_cloud_room_members_own on public.tp_cloud_room_members for select to authenticated using(user_id=(select auth.uid()) or tp_private.cloud_room_member(room_id));

revoke execute on function tp_private.cloud_user(),tp_private.cloud_owner(),tp_private.cloud_blocked(uuid,uuid),tp_private.cloud_room_member(uuid),tp_private.cloud_command(text,jsonb),tp_private.cloud_validated_upload(uuid,text,text,int,int,int),tp_private.cloud_record_event(uuid,uuid,text,int) from public,anon,authenticated,service_role;
grant execute on function tp_private.cloud_user(),tp_private.cloud_owner(),tp_private.cloud_room_member(uuid),tp_private.cloud_command(text,jsonb) to authenticated;
grant execute on function tp_private.cloud_validated_upload(uuid,text,text,int,int,int),tp_private.cloud_record_event(uuid,uuid,text,int) to service_role;
revoke execute on function public.tp_cloud_command(text,jsonb) from public,anon,service_role;
grant execute on function public.tp_cloud_command(text,jsonb) to authenticated;
revoke execute on function public.tp_cloud_validated_upload(uuid,text,text,int,int,int),public.tp_cloud_record_event(uuid,uuid,text,int) from public,anon,authenticated;
grant execute on function public.tp_cloud_validated_upload(uuid,text,text,int,int,int),public.tp_cloud_record_event(uuid,uuid,text,int) to service_role;
do $$ begin
 if to_regclass('storage.buckets') is not null and to_regclass('storage.objects') is not null then
  insert into storage.buckets(id,name,public,file_size_limit,allowed_mime_types) values('tp-community-capes','tp-community-capes',false,1048576,array['image/png']) on conflict(id) do nothing;
  if exists(select 1 from storage.buckets where id='tp-community-capes' and public) then raise exception 'TP_CAPE_BUCKET_MUST_BE_PRIVATE'; end if;
  execute 'drop policy if exists tp_cloud_capes_read on storage.objects';
  execute $policy$create policy tp_cloud_capes_read on storage.objects for select to anon,authenticated using(bucket_id='tp-community-capes' and (name like (select auth.uid())::text||'/%' or exists(select 1 from public.tp_cloud_cosmetics c where c.object_path=name and c.approved and not c.withdrawn)))$policy$;
  execute 'drop policy if exists tp_cloud_capes_owner_review on storage.objects';
  execute $policy$create policy tp_cloud_capes_owner_review on storage.objects for select to authenticated using(bucket_id='tp-community-capes' and (select tp_private.cloud_owner()))$policy$;
  grant select on storage.objects to anon,authenticated;
 end if;
end $$;
commit;
