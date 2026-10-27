import {rpc} from '../../../lib/db.js';
import {authorizeRead,clean,fail,header,integer,prepare,query,response,type ApiRequest,type ApiResponse} from '../../../lib/http.js';
import type {PublicServer} from '../../../lib/types.js';
import {categoryFilter} from '../../../lib/filters.js';

export default async function handler(req:ApiRequest,res:ApiResponse){
  if(!prepare(req,res,['GET']))return;
  try{
    await authorizeRead(req,res);
    const params=query(req),page=integer(params.get('page'),1,1,10000),limit=integer(params.get('limit'),12,1,48);
    const filter=(name:string,max:number)=>clean(params.get(name),max,'all')||'all';
    const result=await rpc<{servers:PublicServer[];total:number}>('tp_list_servers',{
      p_page:page,p_limit:limit,p_category:categoryFilter(filter('category',50)),p_language:filter('lang',10),p_region:filter('region',20),p_search:clean(params.get('search'),120),p_version:clean(params.get('version'),32)
    });
    if(!header(req,'x-api-key'))res.setHeader('Cache-Control','public, s-maxage=60, stale-while-revalidate=300');
    response(res,200,{...result,page,limit,pages:Math.max(1,Math.ceil(result.total/limit)),source:'TP API'});
  }catch(error){fail(res,error);}
}
