import {mkdir,readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve,join} from 'node:path';

const root=resolve(import.meta.dirname,'..');
const output=join(root,'.codex-tmp','version-audit');
const cache=join(output,'metadata');
const metadataOnly=process.argv.includes('--metadata-only');
const single=process.argv.find(arg=>arg.startsWith('--version='))?.slice(10);
const sha1=bytes=>createHash('sha1').update(bytes).digest('hex');
const official=url=>{try{const u=new URL(url);return u.protocol==='https:'&&!u.username&&!u.password&&(['piston-meta.mojang.com','piston-data.mojang.com','launchermeta.mojang.com','launcher.mojang.com','libraries.minecraft.net','resources.download.minecraft.net'].includes(u.hostname)||u.hostname==='s3.amazonaws.com'&&u.pathname.startsWith('/Minecraft.Download/'));}catch{return false;}};
const links=new Map();
const addLink=(url,id,kind)=>{if(!official(url))throw Error('Fuente no reconocida: '+url);const entry=links.get(url)??{url,kind,versions:[]};if(!entry.versions.includes(id))entry.versions.push(id);links.set(url,entry);};
async function request(url,method='GET'){
 if(!official(url))throw Error('URL no oficial');
 let last;
 for(let attempt=0;attempt<3;attempt++){
  try{
   const response=await fetch(url,{method,redirect:'error',signal:AbortSignal.timeout(20000),headers:{'User-Agent':'TP-LAUNCHER/0.1.1 version-validation'}});
   if(response.ok)return response;
   const status=response.status;await response.body?.cancel();
   if(status!==429&&status<500)throw Object.assign(Error('HTTP '+status),{permanent:true});
   last=Error('HTTP '+status);
  }catch(error){last=error;if(error.permanent)throw error;}
  await new Promise(resolve=>setTimeout(resolve,500*(attempt+1)));
 }
 throw last;
}
async function pool(items,concurrency,run){let index=0;await Promise.all(Array.from({length:concurrency},async()=>{for(;;){const at=index++;if(at>=items.length)return;await run(items[at],at);}}));}
function allowed(lib){let allowed=!lib.rules;for(const rule of lib.rules??[]){const os=rule.os;const matches=(!os||(!os.name||os.name==='windows')&&(!os.arch||['x86_64','amd64'].includes(os.arch))&&(!os.version||new RegExp(os.version).test('10.0')))&&(!rule.features||Object.entries(rule.features).every(([key,value])=>value===({has_custom_resolution:true}[key]??false)));if(matches)allowed=rule.action==='allow';}return allowed;}
await mkdir(cache,{recursive:true});
const manifestBytes=Buffer.from(await(await request('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json')).arrayBuffer());
const manifest=JSON.parse(manifestBytes);
const versions=manifest.versions.filter(v=>!single||v.id===single);
if(!versions.length)throw Error('No hay versiones para revisar');
const report={generated_at:new Date().toISOString(),mode:metadataOnly?'metadata':'metadata-and-download-endpoints',platform:'Windows x64',launcher_version:'0.1.1',manifest_sha1:sha1(manifestBytes),total:versions.length,versions:[],links:[],scope:'Valida manifiestos, hashes de JSON y respuestas de descarga. No equivale a arrancar cada cliente ni a comprobar todos sus recursos dentro del juego.'};
let completed=0;
await pool(versions,8,async entry=>{
 const result={id:entry.id,kind:entry.type,status:'pass',java_major:null,errors:[]};
 try{
  const file=join(cache,sha1(Buffer.from(entry.id))+'.json');let bytes;
  try{const cached=await readFile(file);if(entry.sha1&&sha1(cached)===entry.sha1)bytes=cached;}catch{}
  if(!bytes){bytes=Buffer.from(await(await request(entry.url)).arrayBuffer());if(bytes.length>8*1024*1024)throw Error('JSON demasiado grande');if(entry.sha1&&sha1(bytes)!==entry.sha1)throw Error('SHA1 del JSON no coincide');await writeFile(file,bytes);}
  const version=JSON.parse(bytes);
  if(version.id!==entry.id)throw Error('ID del JSON no coincide');
  if(!version.mainClass)throw Error('Falta mainClass');
  if(!version.downloads?.client?.url)throw Error('Falta cliente oficial');
  result.java_major=version.javaVersion?.majorVersion??8;
  result.java_source=version.javaVersion?'manifest':'legacy-default';
  result.main_class=version.mainClass;
  addLink(version.downloads.client.url,entry.id,'client');
  if(version.assetIndex?.url)addLink(version.assetIndex.url,entry.id,'asset-index');
  for(const library of version.libraries??[]){
   if(!allowed(library))continue;
   if(library.downloads?.artifact?.url)addLink(library.downloads.artifact.url,entry.id,'library');
   const native=library.natives?.windows?.replaceAll('${arch}','64');
   if(native){const artifact=library.downloads?.classifiers?.[native];if(artifact?.url)addLink(artifact.url,entry.id,'native');else result.errors.push('Falta descriptor nativo de '+library.name+' / '+native);}
  }
  if(result.errors.length)result.status='fail';
 }catch(error){result.status='fail';result.errors.push(error.message);}
 report.versions.push(result);
 if(++completed%100===0||completed===versions.length)console.log('Metadatos '+completed+'/'+versions.length);
});
if(!metadataOnly){
 let count=0;const unique=[...links.values()];
 await pool(unique,10,async item=>{
  const result={...item,status:'pass',http_status:null};
  try{let response;try{response=await request(item.url,'HEAD');}catch(error){if(!/HTTP (403|405)/.test(error.message))throw error;response=await request(item.url);}
   result.http_status=response.status;result.size=Number(response.headers.get('content-length'))||null;await response.body?.cancel();
  }catch(error){result.status='fail';result.error=error.message;}
  report.links.push(result);
  if(++count%200===0||count===unique.length)console.log('Descargas '+count+'/'+unique.length);
 });
}
for(const link of report.links.filter(link=>link.status==='fail'))for(const id of link.versions){const row=report.versions.find(row=>row.id===id);row.status='fail';row.errors.push(link.kind+': '+link.error+' ('+link.url+')');}
report.versions.sort((a,b)=>versions.findIndex(v=>v.id===a.id)-versions.findIndex(v=>v.id===b.id));
report.passed=report.versions.filter(row=>row.status==='pass').length;report.failed=report.total-report.passed;report.unique_downloads=links.size;
const file=join(output,'report.json');await writeFile(file,JSON.stringify(report,null,2)+'\n');
const csv=['version,type,java,status,errors',...report.versions.map(row=>[row.id,row.kind,row.java_major,row.status,row.errors.join(' | ')].map(value=>'"'+String(value??'').replaceAll('"','""')+'"').join(','))].join('\n');await writeFile(join(output,'versions.csv'),csv+'\n');
console.log(JSON.stringify({total:report.total,passed:report.passed,failed:report.failed,download_endpoints:report.unique_downloads,report:file}));
if(report.failed)process.exitCode=1;
