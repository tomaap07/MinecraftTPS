import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {PGlite} from '@electric-sql/pglite';

test('TP Cloud staging: PostgreSQL roles, RLS and transactional authorization',async t=>{
 const db=await PGlite.create();
 const users={a:'10000000-0000-4000-8000-000000000001',b:'10000000-0000-4000-8000-000000000002',c:'10000000-0000-4000-8000-000000000003',d:'10000000-0000-4000-8000-000000000004',owner:'10000000-0000-4000-8000-000000000005',unverified:'10000000-0000-4000-8000-000000000006'};
 const sql=await readFile(new URL('../../../supabase/migrations/20261102181706_tp_cloud_staging.sql',import.meta.url),'utf8').catch(()=>readFile(new URL('../../../../supabase/migrations/20261102181706_tp_cloud_staging.sql',import.meta.url),'utf8'));
 try{
  await db.exec(`create role anon; create role authenticated; create role service_role; create schema auth; grant usage on schema auth to anon,authenticated,service_role;
   create table auth.users(id uuid primary key,email_confirmed_at timestamptz,is_anonymous boolean default false);
   create function auth.uid() returns uuid language sql stable as $$select nullif(current_setting('request.jwt.claim.sub',true),'')::uuid$$;
   create schema storage; create table storage.buckets(id text primary key,name text,public boolean,file_size_limit int,allowed_mime_types text[]);
   create table storage.objects(id uuid default gen_random_uuid(),bucket_id text,name text); alter table storage.objects enable row level security; grant usage on schema storage to anon,authenticated,service_role;`);
  await db.exec(sql);
  for(const [name,id] of Object.entries(users))await db.query('insert into auth.users(id,email_confirmed_at) values($1,$2)',[id,name==='unverified'?null:new Date().toISOString()]);
  async function user<T>(id:string,callback:()=>Promise<T>):Promise<T>{await db.exec('set role authenticated');await db.query("select set_config('request.jwt.claim.sub',$1,false)",[id]);try{return await callback();}finally{await db.exec('reset role');await db.exec("select set_config('request.jwt.claim.sub','',false)");}}
  async function command(action:string,args:Record<string,unknown>={}){return (await db.query<{result:Record<string,unknown>}>('select public.tp_cloud_command($1,$2::jsonb) result',[action,JSON.stringify(args)])).rows[0]!.result;}
  async function service<T>(callback:()=>Promise<T>){await db.exec('set role service_role');try{return await callback();}finally{await db.exec('reset role');}}
  await t.test('unverified and anonymous callers cannot create profiles',async()=>{
   await assert.rejects(user(users.unverified,()=>command('profile',{handle:'notverified',displayName:'Denied'})),/TP_AUTH_REQUIRED/);
   await db.exec('set role anon');await assert.rejects(command('profile',{handle:'anonymous',displayName:'Denied'}),/permission denied/);await db.exec('reset role');
  });
  for(const [name,id] of Object.entries(users).filter(([name])=>name!=='unverified'))await user(id,()=>command('profile',{handle:`player_${name}`,displayName:`Player ${name}`}));
  await db.query("insert into tp_private.admin_roles(user_id,role) values($1,'owner')",[users.owner]);
  await t.test('welcome gifts are free, unique, and only visible to their owner',async()=>{
   await user(users.a,async()=>{await command('profile',{handle:'player_a',displayName:'A'});const rows=await db.query<{user_id:string}>('select user_id from public.tp_cloud_entitlements');assert.equal(rows.rows.length,3);assert.ok(rows.rows.every(row=>row.user_id===users.a));await assert.rejects(db.query("insert into public.tp_cloud_entitlements(user_id,cosmetic_id,source) values($1,'friendship','forged')",[users.a]),/permission denied/);});
  });
  let friendRequest='';
  await t.test('only the recipient accepts a request; third parties cannot inspect it',async()=>{
   friendRequest=String((await user(users.a,()=>command('friend_request',{recipient:users.b}))).id);
   await user(users.c,async()=>{assert.equal((await db.query('select id from public.tp_cloud_friend_requests')).rows.length,0);await assert.rejects(command('friend_response',{request:friendRequest,accept:true}),/TP_REQUEST_UNAVAILABLE/);});
   await user(users.b,()=>command('friend_response',{request:friendRequest,accept:true}));
   await assert.rejects(user(users.b,()=>command('friend_request',{recipient:users.a})),/duplicate key/);
  });
  let submission='',ticketId='';
  await t.test('only validated private uploads can enter the pending queue',async()=>{
   await user(users.a,async()=>{await assert.rejects(db.query("select public.tp_cloud_validated_upload($1,$2,$3,64,32,512)",[users.a,'a'.repeat(64),`${users.a}/20000000-0000-4000-8000-000000000001.png`]),/permission denied/);await assert.rejects(command('submit_cape',{ticket:'20000000-0000-4000-8000-000000000001',title:'Forgery',rightsAccepted:true}),/TP_VALIDATED_UPLOAD_REQUIRED/);});
   ticketId=String((await service(()=>db.query<{ticket:string}>('select public.tp_cloud_validated_upload($1,$2,$3,64,32,512) ticket',[users.a,'a'.repeat(64),`${users.a}/20000000-0000-4000-8000-000000000001.png`]))).rows[0]!.ticket);
   submission=String((await user(users.a,()=>command('submit_cape',{ticket:ticketId,title:'Original cape',rightsAccepted:true}))).id);
   await user(users.b,async()=>{assert.equal((await db.query('select id from public.tp_cloud_submissions')).rows.length,0);await assert.rejects(command('submit_cape',{ticket:ticketId,title:'Stolen ticket',rightsAccepted:true}),/TP_VALIDATED_UPLOAD_REQUIRED/);});
   await user(users.a,()=>assert.rejects(command('submit_cape',{ticket:ticketId,title:'Duplicate',rightsAccepted:true}),/TP_VALIDATED_UPLOAD_REQUIRED/));
  });
  await t.test('editing metadata or direct SQL cannot grant moderator or publication rights',async()=>{
   await user(users.a,async()=>{await db.query("select set_config('request.jwt.claims',$1,false)",[JSON.stringify({user_metadata:{role:'owner'},app_metadata:{role:'owner'}})]);await assert.rejects(command('moderate',{submission,decision:'approved',reason:'Forged approval'}),/TP_OWNER_REQUIRED/);await assert.rejects(db.query("update public.tp_cloud_submissions set state='approved' where id=$1",[submission]),/permission denied/);await assert.rejects(db.exec('select * from tp_private.admin_roles'),/permission denied/);});
  });
  await t.test('owner approval publishes only that validated cape; withdrawal removes it',async()=>{
   const path=`${users.a}/20000000-0000-4000-8000-000000000001.png`;
   await db.query("insert into storage.objects(bucket_id,name) values('tp-community-capes',$1)",[path]);
   await user(users.owner,async()=>{assert.equal((await db.query('select name from storage.objects')).rows.length,1,'owner can preview private pending images');});
   await user(users.b,async()=>{assert.equal((await db.query('select name from storage.objects')).rows.length,0,'other authenticated users cannot preview a pending image');});
   await db.exec('set role anon');assert.equal((await db.query("select id from public.tp_cloud_cosmetics where source='community'")).rows.length,0);assert.equal((await db.query('select name from storage.objects')).rows.length,0);await db.exec('reset role');
   await user(users.owner,()=>command('moderate',{submission,decision:'approved',reason:'Original work verified by owner'}));
   await db.exec('set role anon');assert.equal((await db.query("select id from public.tp_cloud_cosmetics where source='community'")).rows.length,1);assert.equal((await db.query('select name from storage.objects')).rows.length,1);await db.exec('reset role');
   await user(users.owner,()=>command('moderate',{submission,decision:'withdrawn',reason:'Copyright report under review'}));
   await db.exec('set role anon');assert.equal((await db.query("select id from public.tp_cloud_cosmetics where source='community'")).rows.length,0);assert.equal((await db.query('select name from storage.objects')).rows.length,0);await db.exec('reset role');
  });
  await t.test('quest completion cannot be asserted by the frontend and event replay is ignored',async()=>{
   await user(users.a,async()=>{await assert.rejects(command('claim',{quest:'adventure-60',complete:true}),/TP_QUEST_INCOMPLETE/);await assert.rejects(db.query("select public.tp_cloud_record_event($1,$2,'active_minutes',60)",[users.a,'30000000-0000-4000-8000-000000000001']),/permission denied/);});
   const event=()=>service(()=>db.query<{accepted:boolean}>("select public.tp_cloud_record_event($1,$2,'active_minutes',60) accepted",[users.a,'30000000-0000-4000-8000-000000000001']));
   assert.equal((await event()).rows[0]!.accepted,true);assert.equal((await event()).rows[0]!.accepted,false);
   await user(users.a,async()=>{await command('claim',{quest:'adventure-60'});await command('claim',{quest:'adventure-60'});assert.equal((await db.query("select * from public.tp_cloud_quest_claims where quest_id='adventure-60'")).rows.length,1);assert.equal((await db.query("select * from public.tp_cloud_entitlements where cosmetic_id='first-adventure'")).rows.length,1);});
  });
  await t.test('rooms stay unconnected without a transport; invites are recipient-bound and single-use',async()=>{
   const room=await user(users.a,()=>command('room_create',{game:'1.21.1',loader:'fabric'}));assert.equal(room.transport,'not_configured');
   const invite=await user(users.a,()=>command('room_invite',{room:room.id,recipient:users.b}));
   await assert.rejects(user(users.c,()=>command('room_join',{code:invite.code,game:'1.21.1',loader:'fabric'})),/TP_INVITE_UNAVAILABLE/);
   await assert.rejects(user(users.b,()=>command('room_join',{code:invite.code,game:'1.20.1',loader:'fabric'})),/TP_VERSION_MISMATCH/);
   await assert.rejects(user(users.b,()=>command('room_join',{code:invite.code})),/TP_VERSION_MISMATCH/);
   const joined=await user(users.b,()=>command('room_join',{code:invite.code,game:'1.21.1',loader:'fabric'}));assert.equal(joined.state,'authorized');assert.equal(joined.transport,'not_configured');
   await assert.rejects(user(users.b,()=>command('room_join',{code:invite.code,game:'1.21.1',loader:'fabric'})),/TP_INVITE_UNAVAILABLE/);
   await user(users.a,()=>command('room_close',{room:room.id}));assert.equal((await user(users.b,()=>db.query("select id from public.tp_cloud_rooms"))).rows.length,0);
  });
  await t.test('blocking cancels friendships and rejects subsequent invitations',async()=>{
   await user(users.b,()=>command('block',{user:users.a,blocked:true}));
   await assert.rejects(user(users.a,()=>command('friend_request',{recipient:users.b})),/TP_REQUEST_UNAVAILABLE/);
   const room=await user(users.a,()=>command('room_create',{game:'1.21.1',loader:'fabric'}));
   await assert.rejects(user(users.a,()=>command('room_invite',{room:room.id,recipient:users.b})),/TP_INVITE_UNAVAILABLE/);
  });
  await t.test('partners require owner evidence and expose no private authorization document',async()=>{
   await assert.rejects(user(users.a,()=>command('partner',{name:'False partner',url:'https://example.com',evidence:'Unverified permission',active:true})),/TP_OWNER_REQUIRED/);
   await user(users.owner,()=>command('partner',{name:'Fixture hosting',url:'https://example.com',evidence:'TEST ONLY: documented permission fixture',active:true}));
   await db.exec('set role anon');assert.equal((await db.query('select id,name,official_url from public.tp_cloud_partners')).rows.length,1);await assert.rejects(db.query('select permission_evidence from public.tp_cloud_partners'),/permission denied/);await db.exec('reset role');
  });
  await t.test('all public cloud tables have RLS and migration can be applied twice',async()=>{
   const rows=await db.query<{relrowsecurity:boolean}>("select relrowsecurity from pg_class where relname like 'tp_cloud_%' and relkind='r'");assert.ok(rows.rows.length>=11);assert.ok(rows.rows.every(row=>row.relrowsecurity));
   await db.exec(sql);
  });
  await t.test('account deletion does not get trapped by authorship or moderation foreign keys',async()=>{
   await db.query('delete from auth.users where id=$1',[users.a]);
   assert.equal((await db.query('select user_id from public.tp_cloud_profiles where user_id=$1',[users.a])).rows.length,0);
   assert.equal((await db.query('select user_id from public.tp_cloud_submissions where user_id=$1',[users.a])).rows.length,0);
   const audits=await db.query<{submission_id:string|null}>('select submission_id from tp_private.moderation_audit');assert.ok(audits.rows.length>0);assert.ok(audits.rows.every(row=>row.submission_id===null));
  });
 } finally{await db.close();}
});
