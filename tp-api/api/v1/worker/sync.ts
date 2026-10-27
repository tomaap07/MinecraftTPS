import {fail,masterAuthorized,prepare,response,type ApiRequest,type ApiResponse} from '../../../lib/http.js';
import {syncBatch} from '../../../lib/worker.js';
export default async function handler(req:ApiRequest,res:ApiResponse){
  if(!prepare(req,res,['GET','POST']))return;
  if(!masterAuthorized(req)){response(res,401,{error:'No autorizado.'});return;}
  try{response(res,200,{ok:true,...await syncBatch()});}catch(error){fail(res,error);}
}
