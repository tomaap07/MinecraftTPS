import test from 'node:test';
import assert from 'node:assert/strict';
import {cloudCommand,cloudRows,type CloudEnvironment} from '../lib/cloud.js';

const environment:CloudEnvironment={origin:'https://aaaaaaaaaaaaaaaaaaaa.supabase.co',publishable:'public-fixture',service:'private-server-fixture',allowedOrigins:[]};
const bearer='Bearer fixture-valid-tp-session-token';
const user='10000000-0000-4000-8000-000000000001';

test('cloud commands authenticate the TP session and use its RLS identity, never the service role',async()=>{
 const calls:Array<{url:string;options?:RequestInit}>=[];
 const request=(async(url:RequestInfo|URL,options?:RequestInit)=>{calls.push({url:String(url),options});return String(url).endsWith('/auth/v1/user')?Response.json({id:user,email_confirmed_at:'2026-11-02T00:00:00Z',is_anonymous:false}):Response.json({state:'pending'});}) as typeof fetch;
 assert.deepEqual(await cloudCommand(environment,bearer,'friend_request',{recipient:user},request),{state:'pending'});
 assert.equal(calls.length,2);assert.ok(calls[1]!.url.endsWith('/rpc/tp_cloud_command'));
 const headers=new Headers(calls[1]!.options?.headers);assert.equal(headers.get('apikey'),environment.publishable);assert.equal(headers.get('authorization'),bearer);
 assert.ok(!JSON.stringify(calls).includes(environment.service));
 await assert.rejects(cloudCommand(environment,bearer,'tp_cloud_record_event',{complete:true},request),/no es válida/);
 await assert.rejects(cloudCommand(environment,bearer,'profile',[],request),/no es válida/);
 await assert.rejects(cloudCommand(environment,bearer,'profile',{payload:'x'.repeat(4097)},request),/no es válida/);
 assert.equal(calls.length,2,'invalid operations stop before touching auth or the database');
});

test('unverified TP identities cannot invoke RPC even with a plausible user UUID',async()=>{
 let calls=0;
 const request=(async()=>{calls++;return Response.json({id:user,is_anonymous:true});}) as typeof fetch;
 await assert.rejects(cloudCommand(environment,bearer,'moderate',{user_id:user,role:'owner'},request),/Verifica/);assert.equal(calls,1);
});

test('public catalogue requests only approved rows and emits bounded pages without private object paths',async()=>{
 const request=(async(url:RequestInfo|URL,options?:RequestInit)=>{
  const target=new URL(String(url));assert.equal(target.pathname,'/rest/v1/tp_cloud_cosmetics');assert.equal(target.searchParams.get('approved'),'eq.true');assert.equal(target.searchParams.get('withdrawn'),'eq.false');assert.equal(target.searchParams.get('limit'),'25');assert.equal(target.searchParams.get('offset'),'24');assert.ok(!target.searchParams.get('select')!.includes('object_path'));assert.equal(new Headers(options?.headers).get('authorization'),null);
  return Response.json(Array.from({length:25},(_,index)=>({id:`cape-${index}`})));
 }) as typeof fetch;
 const page=await cloudRows(environment,'','gallery',2,request);assert.equal(page.items.length,24);assert.equal(page.hasMore,true);assert.equal(page.page,2);
 await assert.rejects(cloudRows(environment,'','admin_roles',1,request),/no son válidos/);
 await assert.rejects(cloudRows(environment,'','gallery',0,request),/no son válidos/);
});

test('private submissions are fetched under the verified user bearer and explicit columns',async()=>{
 let calls=0;
 const request=(async(url:RequestInfo|URL,options?:RequestInit)=>{calls++;if(calls===1)return Response.json({id:user,email_confirmed_at:'2026-11-02T00:00:00Z',is_anonymous:false});const target=new URL(String(url));assert.equal(target.pathname,'/rest/v1/tp_cloud_submissions');assert.ok(!target.searchParams.get('select')!.includes('ticket_id'));assert.equal(new Headers(options?.headers).get('authorization'),bearer);return Response.json([]);}) as typeof fetch;
 const result=await cloudRows(environment,bearer,'submissions',1,request);assert.deepEqual(result.items,[]);assert.equal(result.hasMore,false);assert.equal(calls,2);
});
