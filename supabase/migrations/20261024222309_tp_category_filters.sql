create or replace function public.tp_list_servers(p_page integer default 1,p_limit integer default 12,p_category text default 'all',p_language text default 'all',p_region text default 'all',p_search text default '',p_version text default '')
returns jsonb language sql stable security invoker set search_path = public,pg_temp as $$
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
$$;
