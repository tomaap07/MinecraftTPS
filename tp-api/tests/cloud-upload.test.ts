import test from 'node:test';
import assert from 'node:assert/strict';
import {PNG} from 'pngjs';
import {cloudEnvironment,validatedCapePng,verifiedCloudUser,storeValidatedCape,type CloudEnvironment} from '../lib/cloud.js';

const environment:CloudEnvironment={origin:'https://aaaaaaaaaaaaaaaaaaaa.supabase.co',publishable:'public-fixture',service:'private-fixture-server-only',allowedOrigins:[]};
const user='10000000-0000-4000-8000-000000000001';
function cape(){const image=new PNG({width:64,height:32});image.data=Buffer.alloc(64*32*4,180);return PNG.sync.write(image);}
test('cape uploads decode, verify CRC and normalize pixels before receiving a validation ticket',()=>{
 const image=validatedCapePng(cape());assert.equal(image.width,64);assert.equal(image.height,32);assert.match(image.sha256,/^[a-f0-9]{64}$/);assert.equal(PNG.sync.read(image.bytes).data.length,64*32*4);
 for(const bytes of [Buffer.from('not png'),Buffer.alloc(1024*1024+1)])assert.throws(()=>validatedCapePng(bytes));
 const corrupt=cape();corrupt[corrupt.length-8]^=1;assert.throws(()=>validatedCapePng(corrupt));
 const huge=cape();huge.writeUInt32BE(65535,16);assert.throws(()=>validatedCapePng(huge),/64×32/);
});
test('TP community auth uses a verified TP session, never a user-supplied UUID or user metadata role',async()=>{
 let calls=0;
 const request=(async(url:RequestInfo|URL,options?:RequestInit)=>{calls++;assert.equal(String(url),`${environment.origin}/auth/v1/user`);assert.equal(new Headers(options?.headers).get('apikey'),environment.publishable);return Response.json({id:user,email_confirmed_at:'2026-11-02T00:00:00Z',is_anonymous:false,user_metadata:{role:'owner'}});}) as typeof fetch;
 assert.equal(await verifiedCloudUser(environment,'Bearer fixture-valid-tp-auth-session',request),user);assert.equal(calls,1);
 await assert.rejects(verifiedCloudUser(environment,'Bearer forged',request),/Inicia sesión/);assert.equal(calls,1);
 await assert.rejects(verifiedCloudUser(environment,'Bearer fixture-valid-tp-auth-session',(async()=>Response.json({id:user,is_anonymous:true})) as typeof fetch),/Verifica/);
});
test('only private bucket paths are uploaded and failed ticket issuance removes the orphan',async()=>{
 const calls:Array<{url:string;options?:RequestInit}>=[];
 const request=(async(url:RequestInfo|URL,options?:RequestInit)=>{calls.push({url:String(url),options});if(String(url).endsWith('/rpc/tp_cloud_validated_upload'))return Response.json({error:'rate limited'},{status:400});return Response.json({ok:true});}) as typeof fetch;
 await assert.rejects(storeValidatedCape(environment,user,cape(),request),/registrar/);
 assert.equal(calls.length,3);assert.ok(calls[0]!.url.startsWith(`${environment.origin}/storage/v1/object/tp-community-capes/${user}/`));assert.equal(calls[2]!.options?.method,'DELETE');
 const cleanup=JSON.parse(String(calls[2]!.options?.body));assert.equal(cleanup.prefixes.length,1);assert.ok(cleanup.prefixes[0].startsWith(`${user}/`));
});
test('successful upload returns no server credential and remains private until moderation',async()=>{
 const ticket='20000000-0000-4000-8000-000000000001';
 const request=(async(url:RequestInfo|URL)=>Response.json(String(url).endsWith('/rpc/tp_cloud_validated_upload')?ticket:{ok:true})) as typeof fetch;
 const result=await storeValidatedCape(environment,user,cape(),request);assert.equal(result.ticket,ticket);assert.equal(result.state,'private_validated');assert.ok(!JSON.stringify(result).includes(environment.service));
});
test('cloud upload fails closed without an explicit staging configuration',()=>{
 const old=process.env.TP_CLOUD_ENABLED;delete process.env.TP_CLOUD_ENABLED;try{assert.throws(cloudEnvironment,/no está configurado/);}finally{if(old!==undefined)process.env.TP_CLOUD_ENABLED=old;}
});
