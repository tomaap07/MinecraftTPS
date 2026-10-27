create extension if not exists pg_cron;
create extension if not exists pg_net;
select cron.schedule('tp-api-smart-polling','*/5 * * * *',$job$
  select net.http_post(
    url := 'https://tp-api-hazel.vercel.app/api/v1/worker/sync',
    headers := jsonb_build_object('Content-Type','application/json','Authorization','Bearer ' || (select decrypted_secret from vault.decrypted_secrets where name='tp_api_cron_secret' limit 1)),
    body := '{}'::jsonb, timeout_milliseconds := 60000
  );
$job$);
