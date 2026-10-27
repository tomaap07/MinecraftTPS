export class ApiError extends Error {constructor(public status:number,message:string){super(message);}}
export function environment(){
  const url=process.env.SUPABASE_URL?.trim(),key=process.env.SUPABASE_SERVICE_ROLE_KEY?.trim(),master=process.env.TP_MASTER_SECRET?.trim();
  const bridge=process.env.TP_DB_BRIDGE_URL?.trim(),bridgeToken=process.env.TP_DB_BRIDGE_TOKEN?.trim(),anon=process.env.SUPABASE_ANON_KEY?.trim();
  if(!url||(!key&&(!bridge||!bridgeToken||!anon))||!master||master.length<24)throw new ApiError(503,'TP API todavía no tiene su configuración privada completa.');
  const parsed=new URL(url);
  if(parsed.protocol!=='https:'||parsed.username||parsed.password)throw new ApiError(503,'La configuración de base de datos no es válida.');
  if(bridge){const bridgeUrl=new URL(bridge);if(bridgeUrl.protocol!=='https:'||bridgeUrl.username||bridgeUrl.password||bridgeUrl.origin!==parsed.origin)throw new ApiError(503,'La conexión privada con la base de datos no es válida.');}
  return {url:parsed.origin,key,master,bridge,bridgeToken,anon};
}
export async function database<T>(path:string,init:RequestInit={}):Promise<T>{
  const env=environment();
  const endpoint=env.key?`${env.url}/rest/v1/${path}`:`${env.bridge}?path=${encodeURIComponent(path)}`;
  const auth:Record<string,string>=env.key?{'apikey':env.key,'Authorization':`Bearer ${env.key}`}:
    {'apikey':env.anon!,'Authorization':`Bearer ${env.anon!}`,'X-TP-Bridge-Token':env.bridgeToken!};
  const response=await fetch(endpoint,{...init,signal:AbortSignal.timeout(15_000),headers:{...auth,'Content-Type':'application/json',...init.headers}});
  if(!response.ok){console.error('TP database request failed',{path:path.split('?')[0],status:response.status});throw new ApiError(503,'La base de datos del directorio no está disponible.');}
  if(response.status===204)return undefined as T;
  return await response.json() as T;
}
export const rpc=<T>(name:string,args:Record<string,unknown>)=>database<T>(`rpc/${name}`,{method:'POST',body:JSON.stringify(args)});
