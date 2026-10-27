import {prepare,response,type ApiRequest,type ApiResponse} from '../../lib/http.js';
import {environment} from '../../lib/db.js';
export default function handler(req:ApiRequest,res:ApiResponse){
  if(!prepare(req,res,['GET']))return;
  try{environment();response(res,200,{service:'TP API',configured:true,version:'0.1.0'});}
  catch{response(res,503,{service:'TP API',configured:false,version:'0.1.0'});}
}
