import {createHash,randomUUID} from 'node:crypto';
import {PNG} from 'pngjs';
import {ApiError} from './db.js';

export type CloudEnvironment={origin:string;publishable:string;service:string;allowedOrigins:string[]};
export function cloudEnvironment():CloudEnvironment{
 if(process.env.TP_CLOUD_ENABLED!=='true')throw new ApiError(503,'TP Cloud no está configurado. La comunidad remota aún no está activada.');
 const origin=process.env.TP_CLOUD_SUPABASE_URL?.trim()??'',publishable=process.env.TP_CLOUD_PUBLISHABLE_KEY?.trim()??'',service=process.env.TP_CLOUD_SERVICE_ROLE_KEY?.trim()??'';
 if(!/^https:\/\/[a-z0-9]{20}\.supabase\.co$/.test(origin)||!publishable||!service)throw new ApiError(503,'TP Cloud necesita su proyecto de staging y sus credenciales privadas de servidor.');
 return{origin,publishable,service,allowedOrigins:(process.env.TP_CLOUD_ALLOWED_ORIGINS??'').split(',').map(value=>value.trim()).filter(Boolean)};
}
export function validatedCapePng(input:Buffer):{bytes:Buffer;sha256:string;width:number;height:number}{
 if(input.length<33||input.length>1024*1024||!input.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))||input.toString('ascii',12,16)!=='IHDR')throw new ApiError(400,'Selecciona un PNG real de hasta 1 MB.');
 const width=input.readUInt32BE(16),height=input.readUInt32BE(20);
 if(![64,128,256,512].includes(width)||height!==width/2)throw new ApiError(400,'La capa debe medir 64×32, 128×64, 256×128 o 512×256.');
 let image:PNG;
 try{image=PNG.sync.read(input,{checkCRC:true});}catch{throw new ApiError(400,'El PNG está dañado o no se puede decodificar.');}
 if(image.width!==width||image.height!==height||image.data.length!==width*height*4)throw new ApiError(400,'El PNG tiene un formato no válido.');
 const normalized=new PNG({width,height});normalized.data=image.data;
 const bytes=PNG.sync.write(normalized,{colorType:6,inputColorType:6,bitDepth:8});
 if(bytes.length>1024*1024)throw new ApiError(400,'La imagen normalizada supera el límite.');
 return{bytes,sha256:createHash('sha256').update(bytes).digest('hex'),width,height};
}
async function jsonResponse(response:Response,maximum=65536):Promise<unknown>{
 if(response.headers.get('content-length')&&Number(response.headers.get('content-length'))>maximum)throw new ApiError(502,'La respuesta del servicio es demasiado grande.');
 const reader=response.body?.getReader();if(!reader)throw new ApiError(502,'El servicio no devolvió una respuesta.');
 let size=0;const chunks:Uint8Array[]=[];
 try{while(true){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>maximum)throw new ApiError(502,'La respuesta del servicio es demasiado grande.');chunks.push(value);}}finally{await reader.cancel();reader.releaseLock();}
 try{return JSON.parse(Buffer.concat(chunks).toString('utf8'));}catch{throw new ApiError(502,'El servicio devolvió una respuesta no válida.');}
}
export const CLOUD_ACTIONS=new Set(['profile','capabilities','friend_request','friend_response','block','submit_cape','moderate','report','claim','partner','room_create','room_invite','room_join','room_close']);
export async function cloudCommand(environment:CloudEnvironment,bearer:string,action:unknown,args:unknown,request:typeof fetch=fetch):Promise<unknown>{
 if(typeof action!=='string'||!CLOUD_ACTIONS.has(action)||!args||typeof args!=='object'||Array.isArray(args)||Buffer.byteLength(JSON.stringify(args))>4096)throw new ApiError(400,'La operación de TP Cloud no es válida.');
 await verifiedCloudUser(environment,bearer,request);
 const response=await request(`${environment.origin}/rest/v1/rpc/tp_cloud_command`,{method:'POST',redirect:'error',signal:AbortSignal.timeout(10000),headers:{apikey:environment.publishable,Authorization:bearer,'Content-Type':'application/json'},body:JSON.stringify({p_action:action,p_args:args})});
 if(!response.ok)throw new ApiError(response.status===401?401:response.status===403?403:response.status===429?429:400,'TP Cloud rechazó la operación. Revisa los permisos, los límites y el estado de los datos.');
 return await jsonResponse(response);
}
const CLOUD_READS:Record<string,{table:string;fields:string;order:string;public?:boolean}>={
 gallery:{table:'tp_cloud_cosmetics',fields:'id,title,kind,source,sha256,author_id,created_at',order:'created_at.desc,id.asc',public:true},
 partners:{table:'tp_cloud_partners',fields:'id,name,official_url',order:'id.asc',public:true},
 profiles:{table:'tp_cloud_profiles',fields:'user_id,handle,display_name',order:'handle.asc'},
 friends:{table:'tp_cloud_friend_requests',fields:'id,sender_id,recipient_id,status,created_at,decided_at',order:'created_at.desc,id.asc'},
 submissions:{table:'tp_cloud_submissions',fields:'id,user_id,title,state,reason,created_at,reviewed_at',order:'created_at.desc,id.asc'},
 reports:{table:'tp_cloud_reports',fields:'id,reporter_id,cosmetic_id,reason,created_at',order:'created_at.desc,id.asc'},
 inventory:{table:'tp_cloud_entitlements',fields:'cosmetic_id,source,granted_at',order:'granted_at.desc,cosmetic_id.asc'},
 quests:{table:'tp_cloud_quest_claims',fields:'quest_id,cosmetic_id,claimed_at',order:'claimed_at.desc,quest_id.asc'},
 rooms:{table:'tp_cloud_rooms',fields:'id,host_id,game_version,loader,state,max_members,created_at,expires_at',order:'created_at.desc,id.asc'},
 members:{table:'tp_cloud_room_members',fields:'room_id,user_id,state,joined_at',order:'joined_at.desc,user_id.asc'},
};
export async function cloudRows(environment:CloudEnvironment,bearer:string,resource:string,page:number,request:typeof fetch=fetch):Promise<{items:unknown[];page:number;limit:number;hasMore:boolean}>{
 const specification=CLOUD_READS[resource];
 if(!specification||!Number.isInteger(page)||page<1||page>10000)throw new ApiError(400,'El recurso o la página no son válidos.');
 if(!specification.public)await verifiedCloudUser(environment,bearer,request);
 const params=new URLSearchParams({select:specification.fields,order:specification.order,limit:'25',offset:String((page-1)*24)});
 if(resource==='gallery'){params.set('approved','eq.true');params.set('withdrawn','eq.false');}
 if(resource==='partners')params.set('active','eq.true');
 const response=await request(`${environment.origin}/rest/v1/${specification.table}?${params}`,{redirect:'error',signal:AbortSignal.timeout(10000),headers:{apikey:environment.publishable,...(!specification.public?{Authorization:bearer}:{})}});
 if(!response.ok)throw new ApiError(response.status===401?401:403,'No se pudo consultar este recurso de TP Cloud.');
 const rows=await jsonResponse(response,256*1024);
 if(!Array.isArray(rows)||rows.length>25)throw new ApiError(502,'El catálogo devolvió una página no válida.');
 return{items:rows.slice(0,24),page,limit:24,hasMore:rows.length>24};
}
export async function verifiedCloudUser(environment:CloudEnvironment,bearer:string,request:typeof fetch=fetch):Promise<string>{
 if(!/^Bearer [A-Za-z0-9._~-]{20,8192}$/.test(bearer))throw new ApiError(401,'Inicia sesión en tu cuenta TP para presentar una capa.');
 const response=await request(`${environment.origin}/auth/v1/user`,{redirect:'error',signal:AbortSignal.timeout(10000),headers:{apikey:environment.publishable,Authorization:bearer}});
 if(!response.ok)throw new ApiError(401,'La sesión de TP Cloud no es válida o ha caducado.');
 const user=await jsonResponse(response) as {id?:unknown;is_anonymous?:unknown;email_confirmed_at?:unknown};
 if(typeof user.id!=='string'||!/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(user.id)||user.is_anonymous===true||typeof user.email_confirmed_at!=='string')throw new ApiError(403,'Verifica el correo de tu cuenta TP antes de presentar contenido.');
 return user.id;
}
export async function storeValidatedCape(environment:CloudEnvironment,userId:string,input:Buffer,request:typeof fetch=fetch){
 const image=validatedCapePng(input),path=`${userId}/${randomUUID()}.png`;
 const headers={apikey:environment.service,Authorization:`Bearer ${environment.service}`};
 const upload=await request(`${environment.origin}/storage/v1/object/tp-community-capes/${path}`,{method:'POST',redirect:'error',signal:AbortSignal.timeout(15000),headers:{...headers,'Content-Type':'image/png','x-upsert':'false'},body:new Uint8Array(image.bytes)});
 if(!upload.ok)throw new ApiError(503,'No se pudo guardar la capa en el almacenamiento privado.');
 try{
  const response=await request(`${environment.origin}/rest/v1/rpc/tp_cloud_validated_upload`,{method:'POST',redirect:'error',signal:AbortSignal.timeout(10000),headers:{...headers,'Content-Type':'application/json'},body:JSON.stringify({p_user:userId,p_sha:image.sha256,p_object:path,p_width:image.width,p_height:image.height,p_size:image.bytes.length})});
  if(!response.ok)throw new ApiError(response.status===400?429:503,'No se pudo registrar la validación de la capa. Espera antes de volver a intentarlo.');
  const ticket=await jsonResponse(response);
  if(typeof ticket!=='string'||!/^[a-f0-9-]{36}$/i.test(ticket))throw new ApiError(502,'No se recibió un comprobante válido de la capa.');
  return{ticket,sha256:image.sha256,width:image.width,height:image.height,state:'private_validated'};
 }catch(error){
  await request(`${environment.origin}/storage/v1/object/tp-community-capes`,{method:'DELETE',redirect:'error',signal:AbortSignal.timeout(10000),headers:{...headers,'Content-Type':'application/json'},body:JSON.stringify({prefixes:[path]})}).catch(()=>{});
  throw error;
 }
}
