import catalog from './data/hud_metrics.json';
import type{OverlaySnapshot}from './launcher_extensions';
export type MetricModuleId=`m_${string}`;
export type MetricDescriptor={id:MetricModuleId;name:string;group:string;metric:string;source:'world'|'derived';format:string;unit:string;description:string};



export const METRIC_MODULES=catalog as MetricDescriptor[];
const byId=new Map(METRIC_MODULES.map(row=>[row.id,row]));
const number=(n:number,digits=0)=>n.toLocaleString(undefined,{maximumFractionDigits:digits});
export function metricValue(id:string,frame:OverlaySnapshot|null,preview=false):string|null{
 const row=byId.get(id as MetricModuleId);if(!row)return null;
 if(preview)return row.source==='derived'?({'nether_coords':'16 / −32','chunk_coords':'8 / −16','chunk_local':'0 / 0','world_dimension':'Overworld'} as Record<string,string>)[row.metric]??'Ejemplo':format(({percent:.72,hearts:18,damage:120,cm:3825,bytes:734003200,ticks:140,decimal:2.65,bool:1}as Record<string,number>)[row.format]??64,row);
 const t=frame?.telemetry;
 if(row.source==='world'){if(!t?.has_world)return null;const value=t.world_metrics?.[row.metric];return value==null?null:format(value,row);}
 const positions=!!t?.has_world&&!t.coordinates_restricted&&t.capabilities?.includes('coordinates');
 switch(row.metric){
 case'nether_coords':return positions?`${number(t!.x/(t!.dimension==='minecraft:the_nether'?.125:8),1)} / ${number(t!.z/(t!.dimension==='minecraft:the_nether'?.125:8),1)}`:null;
 case'chunk_coords':return positions?`${Math.floor(t!.x/16)} / ${Math.floor(t!.z/16)}`:null;
 case'chunk_local':return positions?`${((Math.floor(t!.x)%16)+16)%16} / ${((Math.floor(t!.z)%16)+16)%16}`:null;
 case'world_dimension':return t?.has_world&&t.dimension?t.dimension.replace('minecraft:','').replaceAll('_',' '):null;
 default:return null;
 }
}
function format(n:number,row:MetricDescriptor):string{
 if(!Number.isFinite(n))return'—';let result:string;
 switch(row.format){case'bool':return n?'Sí':'No';case'percent':result=number(n*100,1);break;case'hearts':result=number(n/2,1);break;case'damage':result=number(n/20,1);break;case'cm':result=number(n/100,1);break;case'ticks':result=number(Math.max(0,n)/20,1);break;case'decimal':result=number(n,2);break;default:result=number(n);}
 return result+row.unit;
}
