import {readFile} from 'node:fs/promises';
import {database,environment} from '../lib/db.js';
import {parseAddress,isPublicAddress} from '../lib/minecraft.js';
import {isIP} from 'node:net';
import {syncBatch} from '../lib/worker.js';

type Seed={name:string;ip:string;port?:number;category?:string;region?:string;lang?:string;language?:string};
const args=process.argv.slice(2),flag=(name:string)=>args.includes(name),option=(name:string)=>args.find(value=>value.startsWith(`${name}=`))?.slice(name.length+1);
const MAX_IMPORT=2000;
async function jsonSource(url:string):Promise<unknown>{
  const parsed=new URL(url);
  if(parsed.protocol!=='https:'||!['raw.githubusercontent.com','www.mc-monitor.app'].includes(parsed.hostname)||parsed.username||parsed.password)throw new Error('Las fuentes deben usar HTTPS en raw.githubusercontent.com o www.mc-monitor.app.');
  const response=await fetch(parsed,{redirect:'error',signal:AbortSignal.timeout(20_000),headers:{Accept:'application/json','User-Agent':'TP-API/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)'}});
  if(!response.ok)throw new Error(`La fuente pública devolvió HTTP ${response.status}.`);
  const reader=response.body?.getReader();if(!reader)throw new Error('Fuente pública vacía.');const chunks:Uint8Array[]=[];let size=0;
  while(true){const next=await reader.read();if(next.done)break;size+=next.value.byteLength;if(size>5*1024*1024){await reader.cancel();throw new Error('La fuente pública supera 5 MiB.');}chunks.push(next.value);}
  return JSON.parse(Buffer.concat(chunks).toString('utf8'));
}
function records(value:unknown):Seed[]{
  const data=Array.isArray(value)?value:(value&&typeof value==='object'?(value as {servers?:unknown}).servers:[]);
  if(!Array.isArray(data))throw new Error('La fuente no contiene una lista JSON de servidores.');
  return data.slice(0,MAX_IMPORT).flatMap(row=>{
    if(!row||typeof row!=='object')return [];const item=row as Record<string,unknown>;
    const ip=typeof item.ip==='string'?item.ip:typeof item.address==='string'?item.address:'';
    if(!ip)return [];
    const tags=Array.isArray(item.tags)?item.tags.flatMap(tag=>tag&&typeof tag==='object'&&typeof tag.tag==='string'?[tag.tag]:[]):[];
    if(tags.includes('bedrock')&&!tags.includes('java')&&!tags.includes('crossplay'))return [];
    const categories=[['survival','Survival'],['pvp','PvP'],['minigames','Minijuegos'],['rpg','RPG'],['anarchy','Anarquía'],['skyblock','SkyBlock']].filter(([tag])=>tags.includes(tag)).map(([,label])=>label).join(', ');
    return [{name:typeof item.name==='string'?item.name:ip,ip,port:typeof item.port==='number'?item.port:undefined,
      category:typeof item.category==='string'?item.category:categories||'General',region:typeof item.region==='string'?item.region:'Global',language:typeof item.language==='string'?item.language:'all'}];
  });
}
async function main(){
  environment();
  const base=JSON.parse(await readFile(new URL('../../data/seed-servers.json',import.meta.url),'utf8')) as Seed[];
  const input:{entry:Seed;source:string}[]=base.map(entry=>({entry,source:'https://github.com/tomaap07/MinecraftTPS'}));
  const urls:string[]=[];
  if(flag('--import-public')&&!option('--import-url'))throw new Error('--import-public requiere --import-url=<URL JSON de un repositorio público verificado>.');
  if(option('--import-url'))urls.push(option('--import-url')!);
  if(flag('--import-directory'))urls.push('https://www.mc-monitor.app/api/servers?limit=500&sort=players');
  for(const url of urls){try{input.push(...records(await jsonSource(url)).map(entry=>({entry,source:url})));}catch(error){console.error('Importación omitida:',error instanceof Error?error.message:'Error de fuente');}}
  const unique=new Map<string,Record<string,unknown>>();let invalid=0;
  for(const {entry,source} of input){try{
    const parsed=parseAddress(entry.ip,entry.port),key=`${parsed.host}:${parsed.port}`;
    if(isIP(parsed.host)&&!isPublicAddress(parsed.host))throw new Error('Dirección no pública.');
    if(!unique.has(key))unique.set(key,{name:entry.name.trim().slice(0,120)||parsed.host,ip:parsed.host,port:parsed.port,
      category:(entry.category??'Survival').slice(0,50),region:(entry.region??'Global').slice(0,20),language:(entry.language??entry.lang??'EN').slice(0,10).toUpperCase(),source_url:source});
  }catch{invalid++;}}
  const rows=[...unique.values()].slice(0,MAX_IMPORT);let inserted=0;
  for(let i=0;i<rows.length;i+=100){const result=await database<{id:string}[]>('tp_servers?on_conflict=ip,port&select=id',{method:'POST',headers:{Prefer:'resolution=ignore-duplicates,return=representation'},body:JSON.stringify(rows.slice(i,i+100))});inserted+=result.length;}
  console.log(JSON.stringify({selected:rows.length,inserted,already_present:rows.length-inserted,invalid,featured_promotions_created:0}));
  if(flag('--sync')){
    const requested=Number(option('--sync-batches')??1),batches=Number.isInteger(requested)?Math.max(1,Math.min(100,requested)):1;
    for(let i=0;i<batches;i++){const result=await syncBatch({batch:24,budgetMs:55_000});console.log(JSON.stringify({batch:i+1,...result}));if(!result.claimed)break;}
  }
}
void main().catch(error=>{console.error(error instanceof Error?error.message:'No se pudo cargar la semilla.');process.exitCode=1;});
