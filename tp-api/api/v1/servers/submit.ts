import {ApiError,database} from '../../../lib/db.js';
import {authorizeRead,body,clean,fail,prepare,response,type ApiRequest,type ApiResponse} from '../../../lib/http.js';
import {parseAddress,pingMinecraft} from '../../../lib/minecraft.js';
import {PUBLIC_COLUMNS,type PublicServer} from '../../../lib/types.js';

export default async function handler(req:ApiRequest,res:ApiResponse){
  if(!prepare(req,res,['POST']))return;
  try{
    await authorizeRead(req,res,'submit');
    const input=await body(req),name=clean(input.name,120);
    if(!name)throw new ApiError(400,'Escribe el nombre del servidor.');
    const address=parseAddress(clean(input.ip,320),input.port);
    const state=await pingMinecraft(address);
    const now=new Date(),tier=state.players_online>50?1:2;
    const item={name,ip:address.host,port:address.port,category:clean(input.category,50,'Survival')||'Survival',region:clean(input.region,20,'Global')||'Global',language:clean(input.language??input.lang,10,'ES').toUpperCase()||'ES',...state,
      priority_tier:tier,last_pinged_at:now.toISOString(),next_ping_at:new Date(now.getTime()+(tier===1?5:30)*60_000).toISOString()};
    const inserted=await database<PublicServer[]>(`tp_servers?on_conflict=ip,port&select=${PUBLIC_COLUMNS}`,{method:'POST',headers:{Prefer:'resolution=ignore-duplicates,return=representation'},body:JSON.stringify(item)});
    const server=inserted[0]??(await database<PublicServer[]>(`tp_servers?ip=eq.${encodeURIComponent(address.host)}&port=eq.${address.port}&select=${PUBLIC_COLUMNS}&limit=1`))[0];
    response(res,inserted.length?201:200,{server,created:inserted.length>0});
  }catch(error){fail(res,error);}
}
