import {cloudEnvironment,cloudCommand,cloudRows} from '../../../lib/cloud.js';
import {ApiError} from '../../../lib/db.js';
import {body,header,response,fail,query,integer,type ApiRequest,type ApiResponse} from '../../../lib/http.js';

export default async function handler(req:ApiRequest,res:ApiResponse){
 res.setHeader('Cache-Control','no-store');res.setHeader('X-Content-Type-Options','nosniff');res.setHeader('Vary','Origin');
 try{
  const env=cloudEnvironment(),origin=header(req,'origin');
  if(origin&&!env.allowedOrigins.includes(origin))throw new ApiError(403,'Este origen no está autorizado para TP Cloud.');
  if(origin)res.setHeader('Access-Control-Allow-Origin',origin);
  res.setHeader('Access-Control-Allow-Methods','GET, POST, OPTIONS');res.setHeader('Access-Control-Allow-Headers','Content-Type, Authorization');
  if(req.method==='OPTIONS'){res.statusCode=204;res.end();return;}
  const bearer=header(req,'authorization');
  if(req.method==='GET'){
   const params=query(req);response(res,200,await cloudRows(env,bearer,params.get('resource')??'gallery',integer(params.get('page'),1,1,10000)));return;
  }
  if(req.method==='POST'){
   const value=await body(req);response(res,200,await cloudCommand(env,bearer,value.action,value.args??{}));return;
  }
  res.setHeader('Allow','GET, POST, OPTIONS');throw new ApiError(405,'Método no permitido.');
 }catch(error){fail(res,error);}
}
