

const EXPECTED_DIGEST = Deno.env.get('TP_DB_BRIDGE_TOKEN_SHA256') ?? 'SET_AT_DEPLOYMENT';
const allowed = new Set(['tp_servers','tp_api_keys','rpc/tp_list_servers','rpc/tp_check_rate_limit',
  'rpc/tp_consume_api_key','rpc/tp_claim_servers','rpc/tp_finish_server_ping']);
const json = (status:number,error:string) => new Response(JSON.stringify({error}),{status,headers:{'Content-Type':'application/json','Cache-Control':'no-store'}});

Deno.serve(async request => {
  const token=request.headers.get('X-TP-Bridge-Token')??'';
  if(token.length<32||token.length>256||EXPECTED_DIGEST==='SET_AT_DEPLOYMENT')return json(401,'Unauthorized');
  const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(token))),byte=>byte.toString(16).padStart(2,'0')).join('');
  let difference=hash.length^EXPECTED_DIGEST.length;
  for(let index=0;index<hash.length;index++)difference|=hash.charCodeAt(index)^(EXPECTED_DIGEST.charCodeAt(index)||0);
  if(difference)return json(401,'Unauthorized');
  if(!['GET','POST','PATCH'].includes(request.method))return json(405,'Method not allowed');
  const path=new URL(request.url).searchParams.get('path')??'';
  if(path.length>6000||path.startsWith('/')||path.includes('..')||path.includes('#'))return json(400,'Invalid resource');
  const resource=path.split('?')[0];
  if(!allowed.has(resource)||(resource.startsWith('rpc/')&&request.method!=='POST'))return json(403,'Resource not allowed');
  const origin=Deno.env.get('SUPABASE_URL'),service=Deno.env.get('SUPABASE_SERVICE_ROLE_KEY');
  if(!origin||!service)return json(503,'Database configuration missing');
  const headers=new Headers({'apikey':service,'Authorization':`Bearer ${service}`,'Content-Type':'application/json'});
  for(const name of ['Prefer','Range','Range-Unit','Accept']){const value=request.headers.get(name);if(value)headers.set(name,value);}
  try{
    let body:Uint8Array|undefined;
    if(request.method!=='GET'){
      if(Number(request.headers.get('content-length'))>1024*1024)return json(413,'Payload too large');
      const reader=request.body?.getReader(),chunks:Uint8Array[]=[];let size=0;
      if(reader)while(true){const chunk=await reader.read();if(chunk.done)break;size+=chunk.value.length;if(size>1024*1024){await reader.cancel();return json(413,'Payload too large');}chunks.push(chunk.value);}
      body=new Uint8Array(size);let offset=0;for(const chunk of chunks){body.set(chunk,offset);offset+=chunk.length;}
    }
    const response=await fetch(`${origin}/rest/v1/${path}`,{method:request.method,headers,body,signal:AbortSignal.timeout(20_000),redirect:'error'});
    const outgoing=new Headers({'Cache-Control':'no-store'});
    for(const name of ['Content-Type','Content-Range','Preference-Applied']){const value=response.headers.get(name);if(value)outgoing.set(name,value);}
    return new Response(response.body,{status:response.status,headers:outgoing});
  }catch{return json(503,'Database temporarily unavailable');}
});
