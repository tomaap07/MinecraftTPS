import type {IncomingMessage,ServerResponse} from 'node:http';
import {createHash,createHmac,timingSafeEqual} from 'node:crypto';
import {ApiError,environment,rpc} from './db.js';
export type ApiRequest=IncomingMessage&{body?:unknown};
export type ApiResponse=ServerResponse;
export const header=(req:ApiRequest,name:string)=>{const value=req.headers[name.toLowerCase()];return Array.isArray(value)?value[0]??'':value??'';};
export function response(res:ApiResponse,status:number,value:unknown){res.statusCode=status;res.setHeader('Content-Type','application/json; charset=utf-8');res.end(JSON.stringify(value));}
export function prepare(req:ApiRequest,res:ApiResponse,methods:string[]):boolean{
  res.setHeader('X-Content-Type-Options','nosniff');res.setHeader('Referrer-Policy','no-referrer');
  const origin=header(req,'origin'),allowed=(process.env.TP_ALLOWED_ORIGINS??'').split(',').map(value=>value.trim()).filter(Boolean);
  res.setHeader('Vary','Origin, X-API-Key');
  if(!allowed.length)res.setHeader('Access-Control-Allow-Origin','*');
  else if(origin&&allowed.includes(origin))res.setHeader('Access-Control-Allow-Origin',origin);
  res.setHeader('Access-Control-Allow-Methods',methods.join(', '));res.setHeader('Access-Control-Allow-Headers','Content-Type, X-API-Key, X-Client-ID');
  if(req.method==='OPTIONS'){res.statusCode=204;res.end();return false;}
  if(!methods.includes(req.method??'')){res.setHeader('Allow',methods.join(', '));response(res,405,{error:'Método no permitido.'});return false;}
  res.setHeader('Cache-Control','no-store');return true;
}
export function fail(res:ApiResponse,error:unknown){
  const status=error instanceof ApiError?error.status:500;
  response(res,status,{error:error instanceof ApiError?error.message:'No se pudo completar la operación.'});
}
export async function body(req:ApiRequest):Promise<Record<string,unknown>>{
  if(Number(header(req,'content-length')||0)>4096)throw new ApiError(413,'El formulario es demasiado grande.');
  let parsed:unknown=req.body;
  if(parsed===undefined){const chunks:Buffer[]=[];let size=0;for await(const part of req){size+=part.length;if(size>4096)throw new ApiError(413,'El formulario es demasiado grande.');chunks.push(Buffer.from(part));}parsed=Buffer.concat(chunks).toString('utf8');}
  if(typeof parsed==='string'){if(Buffer.byteLength(parsed)>4096)throw new ApiError(413,'El formulario es demasiado grande.');try{parsed=JSON.parse(parsed);}catch{throw new ApiError(400,'Envía un formulario JSON válido.');}}
  if(!parsed||typeof parsed!=='object'||Array.isArray(parsed))throw new ApiError(400,'Envía un formulario JSON válido.');
  if(Buffer.byteLength(JSON.stringify(parsed))>4096)throw new ApiError(413,'El formulario es demasiado grande.');
  return parsed as Record<string,unknown>;
}
export function query(req:ApiRequest){return new URL(req.url??'/', 'https://tp-api.local').searchParams;}
export function integer(value:string|null,fallback:number,min:number,max:number){if(value===null)return fallback;const number=Number(value);if(!Number.isInteger(number)||number<min||number>max)throw new ApiError(400,'Un parámetro numérico no es válido.');return number;}
export function clean(value:unknown,max:number,fallback=''){return typeof value==='string'?value.replace(/[\u0000-\u001f\u007f]/g,'').trim().slice(0,max):fallback;}
export async function authorizeRead(req:ApiRequest,res:ApiResponse,scope='list'){
  const env=environment();

  const ip=header(req,'x-vercel-forwarded-for').split(',')[0]?.trim()||req.socket.remoteAddress||'unknown';
  const subject=createHmac('sha256',env.master).update(`${scope}:${ip}`).digest('hex');
  const allowed=await rpc<{allowed:boolean}>('tp_check_rate_limit',{p_subject:subject,p_limit:scope==='submit'?5:60,p_seconds:60});
  if(!allowed.allowed){res.setHeader('Retry-After','60');throw new ApiError(429,'Has realizado demasiadas consultas. Espera un minuto.');}
  const key=header(req,'x-api-key');
  if(key){if(key.length>200)throw new ApiError(401,'API Key no válida.');const result=await rpc<{allowed:boolean;reason?:string;remaining?:number}>('tp_consume_api_key',{p_hash:createHash('sha256').update(key).digest('hex')});
    if(!result.allowed)throw new ApiError(result.reason==='limit'?429:401,result.reason==='limit'?'Se alcanzó el límite diario de esta API Key.':'API Key no válida.');
    res.setHeader('X-RateLimit-Remaining',String(result.remaining??0));
  }
}
export function masterAuthorized(req:ApiRequest){
  const provided=header(req,'authorization').replace(/^Bearer /i,'');
  const input=Buffer.from(provided);
  return [process.env.TP_MASTER_SECRET,process.env.CRON_SECRET].some(secret=>{if(!secret||secret.length<24)return false;const expected=Buffer.from(secret);return input.length===expected.length&&timingSafeEqual(input,expected);});
}
