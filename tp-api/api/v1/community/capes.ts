import {cloudEnvironment,verifiedCloudUser,storeValidatedCape} from '../../../lib/cloud.js';
import {ApiError} from '../../../lib/db.js';
import {header,response,fail,type ApiRequest,type ApiResponse} from '../../../lib/http.js';

export default async function handler(req:ApiRequest,res:ApiResponse){
 res.setHeader('Cache-Control','no-store');res.setHeader('X-Content-Type-Options','nosniff');res.setHeader('Vary','Origin');
 try{
  const env=cloudEnvironment(),origin=header(req,'origin');
  if(origin&&!env.allowedOrigins.includes(origin))throw new ApiError(403,'Este origen no está autorizado para TP Cloud.');
  if(origin)res.setHeader('Access-Control-Allow-Origin',origin);
  res.setHeader('Access-Control-Allow-Methods','POST, OPTIONS');res.setHeader('Access-Control-Allow-Headers','Content-Type, Authorization');
  if(req.method==='OPTIONS'){res.statusCode=204;res.end();return;}
  if(req.method!=='POST'){res.setHeader('Allow','POST, OPTIONS');throw new ApiError(405,'Método no permitido.');}
  const user=await verifiedCloudUser(env,header(req,'authorization'));
  if(Number(header(req,'content-length')||0)>1450000)throw new ApiError(413,'La imagen supera el límite.');
  let value:unknown=req.body;
  if(value===undefined){let size=0;const parts:Buffer[]=[];for await(const part of req){size+=part.length;if(size>1450000)throw new ApiError(413,'La imagen supera el límite.');parts.push(Buffer.from(part));}value=Buffer.concat(parts).toString('utf8');}
  if(typeof value==='string'){if(Buffer.byteLength(value)>1450000)throw new ApiError(413,'La imagen supera el límite.');try{value=JSON.parse(value);}catch{throw new ApiError(400,'Envía la imagen como JSON.');}}
  if(value&&typeof value==='object'&&Buffer.byteLength(JSON.stringify(value))>1450000)throw new ApiError(413,'La imagen supera el límite.');
  if(!value||typeof value!=='object'||Array.isArray(value)||!('pngBase64' in value)||typeof value.pngBase64!=='string'||value.pngBase64.length>1400000||!/^[A-Za-z0-9+/]+={0,2}$/.test(value.pngBase64))throw new ApiError(400,'No se recibió una imagen PNG válida.');
  response(res,201,await storeValidatedCape(env,user,Buffer.from(value.pngBase64,'base64')));
 }catch(error){fail(res,error);}
}
