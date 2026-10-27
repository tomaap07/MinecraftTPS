import {lookup,resolveSrv} from 'node:dns/promises';
import {isIP,Socket} from 'node:net';
import {performance} from 'node:perf_hooks';
import {ApiError} from './db.js';
import type {MotdSpan,ServerState} from './types.js';

export type Address={host:string;port:number;explicitPort:boolean};
export function parseAddress(input:string,providedPort?:unknown):Address{
  const value=input.trim().toLowerCase();
  if(!value||value.length>320||/[\s\/@?#\\]/.test(value))throw new ApiError(400,'Introduce un dominio o IP sin URL ni espacios.');
  let host=value,embedded:string|undefined;
  if(value.startsWith('[')){const end=value.indexOf(']');if(end<0)throw new ApiError(400,'IPv6 no válida.');host=value.slice(1,end);if(value.length>end+1){if(value[end+1]!==':')throw new ApiError(400,'Puerto no válido.');embedded=value.slice(end+2);}if(isIP(host)!==6)throw new ApiError(400,'IPv6 no válida.');}
  else if(isIP(value)!==6&&value.includes(':')){const pieces=value.split(':');if(pieces.length!==2)throw new ApiError(400,'Dirección no válida.');[host,embedded]=pieces;}
  host=host.replace(/\.$/,'');
  if(!isIP(host)&&(!host.includes('.')||!host.split('.').every(part=>/^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/.test(part))))throw new ApiError(400,'El dominio del servidor no es válido.');
  const port=Number(providedPort??embedded??25565);
  if(!Number.isInteger(port)||port<1||port>65535||(embedded!==undefined&&providedPort!==undefined&&Number(embedded)!==port))throw new ApiError(400,'El puerto debe estar entre 1 y 65535.');
  return {host,port,explicitPort:providedPort!==undefined||embedded!==undefined};
}
function ipv6Number(value:string):bigint|null{
  if(value.includes('.')||value.includes('%'))return null;
  const halves=value.split('::');if(halves.length>2)return null;
  const left=halves[0]?halves[0].split(':'):[],right=halves.length===2&&halves[1]?halves[1].split(':'):[];
  const fill=8-left.length-right.length;if(fill<0||(halves.length===1&&fill!==0))return null;
  const all=[...left,...Array(fill).fill('0'),...right];if(all.length!==8||all.some(part=>!/^[0-9a-f]{1,4}$/i.test(part)))return null;
  return all.reduce((number,part)=>(number<<16n)|BigInt(parseInt(part,16)),0n);
}
export function isPublicAddress(address:string):boolean{
  if(isIP(address)===4){
    const parts=address.split('.').map(Number),number=parts.reduce((total,part)=>(total*256+part)>>>0,0);
    const blocked:[string,number][]=[['0.0.0.0',8],['10.0.0.0',8],['100.64.0.0',10],['127.0.0.0',8],['169.254.0.0',16],['172.16.0.0',12],['192.0.0.0',24],['192.0.2.0',24],['192.88.99.0',24],['192.168.0.0',16],['198.18.0.0',15],['198.51.100.0',24],['203.0.113.0',24],['224.0.0.0',4],['240.0.0.0',4]];
    return !blocked.some(([base,bits])=>{const target=base.split('.').map(Number).reduce((total,part)=>(total*256+part)>>>0,0);return (number>>>(32-bits))===(target>>>(32-bits));});
  }
  const value=ipv6Number(address);if(value===null||isIP(address)!==6)return false;

  if((value>>125n)!==1n)return false;
  return ![['2001:db8::',32],['2001::',32],['2002::',16],['2001:2::',48],['2001:10::',28],['2001:20::',28]].some(([base,bits])=>{
    const shift=128n-BigInt(bits);return (value>>shift)===(ipv6Number(String(base))!>>shift);
  });
}
const withTimeout=<T>(promise:Promise<T>,ms:number):Promise<T>=>new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(new ApiError(422,'El DNS del servidor no respondió a tiempo.')),ms);promise.then(value=>{clearTimeout(timer);resolve(value);},error=>{clearTimeout(timer);reject(error);});});
async function destination(address:Address){
  let host=address.host,port=address.port;
  if(!address.explicitPort&&!isIP(host)){
    try{const records=await withTimeout(resolveSrv(`_minecraft._tcp.${host}`),1800);records.sort((a,b)=>a.priority-b.priority||b.weight-a.weight);if(records.length){host=records[0].name;port=records[0].port;}}
    catch{ }
  }
  if(port<1||port>65535)throw new ApiError(422,'El registro SRV publica un puerto no válido.');
  const addresses=isIP(host)?[{address:host,family:isIP(host)}]:await withTimeout(lookup(host,{all:true,verbatim:true}),2500);
  if(!addresses.length||addresses.some(item=>!isPublicAddress(item.address)))throw new ApiError(422,'Solo se admiten servidores públicos de Internet.');
  return {...(addresses.find(item=>item.family===4)??addresses[0]),port};
}
export function varint(value:number):Buffer{const bytes:number[]=[];let rest=value>>>0;do{let next=rest&127;rest>>>=7;if(rest)next|=128;bytes.push(next);}while(rest);return Buffer.from(bytes);}
export function readVarint(buffer:Buffer,offset=0):{value:number;size:number}|null{let value=0;for(let i=0;i<5;i++){if(offset+i>=buffer.length)return null;const byte=buffer[offset+i];value|=(byte&127)<<(7*i);if(!(byte&128))return {value:value>>>0,size:i+1};}throw new ApiError(422,'Respuesta de Minecraft no válida.');}
const textPacket=(value:string)=>{const bytes=Buffer.from(value);return Buffer.concat([varint(bytes.length),bytes]);};
const frame=(body:Buffer)=>Buffer.concat([varint(body.length),body]);
const COLORS:Record<string,string>={black:'#000000',dark_blue:'#0000aa',dark_green:'#00aa00',dark_aqua:'#00aaaa',dark_red:'#aa0000',dark_purple:'#aa00aa',gold:'#ffaa00',gray:'#aaaaaa',dark_gray:'#555555',blue:'#5555ff',green:'#55ff55',aqua:'#55ffff',red:'#ff5555',light_purple:'#ff55ff',yellow:'#ffff55',white:'#ffffff'};
const colorCodes='0123456789abcdef';
const baseSpan=():MotdSpan=>({text:'',color:'#aaaaaa',bold:false,italic:false,underlined:false,strikethrough:false});
export function motdSpans(input:unknown):MotdSpan[]{
  const result:MotdSpan[]=[];let remaining=2048;
  const add=(text:string,style:MotdSpan)=>{const clipped=text.replace(/[\u0000-\u0008\u000b-\u001f\u007f]/g,'').slice(0,remaining);remaining-=clipped.length;if(clipped&&result.length<128)result.push({...style,text:clipped});};
  const legacy=(text:string,base:MotdSpan)=>{let style={...base},current='';const flush=()=>{add(current,style);current='';};for(let i=0;i<text.length&&remaining>0;i++){if(text[i]==='§'&&i+1<text.length){flush();const code=text[++i].toLowerCase(),index=colorCodes.indexOf(code);if(index>=0)style={...baseSpan(),color:Object.values(COLORS)[index]};else if(code==='l')style.bold=true;else if(code==='o')style.italic=true;else if(code==='n')style.underlined=true;else if(code==='m')style.strikethrough=true;else if(code==='r')style={...base};}else current+=text[i];}flush();};
  const walk=(part:unknown,style:MotdSpan,depth=0)=>{if(depth>12||remaining<=0)return;if(typeof part==='string'){legacy(part,style);return;}if(Array.isArray(part)){for(const next of part.slice(0,128))walk(next,style,depth+1);return;}if(!part||typeof part!=='object')return;const value=part as Record<string,unknown>,next={...style};if(typeof value.color==='string')next.color=COLORS[value.color]??(/^#[0-9a-f]{6}$/i.test(value.color)?value.color:style.color);for(const key of ['bold','italic','underlined','strikethrough'] as const)if(typeof value[key]==='boolean')next[key]=value[key];if(typeof value.text==='string')legacy(value.text,next);if(Array.isArray(value.extra))for(const extra of value.extra.slice(0,128))walk(extra,next,depth+1);};
  walk(input,baseSpan());return result;
}
export function validFavicon(value:unknown):string{
  if(typeof value!=='string'||!value.startsWith('data:image/png;base64,')||value.length>90_000)return '';
  const encoded=value.slice(22);if(!/^[A-Za-z0-9+/]+={0,2}$/.test(encoded))return '';
  const bytes=Buffer.from(encoded,'base64');if(bytes.length<33||bytes.length>65536||!bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))||bytes.toString('ascii',12,16)!=='IHDR'||bytes.readUInt32BE(16)!==64||bytes.readUInt32BE(20)!==64)return '';
  return `data:image/png;base64,${bytes.toString('base64')}`;
}
export function stateFromJson(value:unknown,latency:number):ServerState{
  if(!value||typeof value!=='object')throw new ApiError(422,'El destino no respondió como servidor Minecraft.');
  const data=value as {players?:{online?:unknown;max?:unknown};version?:{name?:unknown;protocol?:unknown};description?:unknown;favicon?:unknown};
  if(!data.version||!data.players)throw new ApiError(422,'El destino no publica un estado Minecraft válido.');
  const count=(number:unknown)=>typeof number==='number'&&Number.isInteger(number)&&number>=0?Math.min(number,2147483647):0;
  const motd=motdSpans(data.description);
  return {is_online:true,players_online:count(data.players.online),players_max:count(data.players.max),version_text:String(data.version.name??'Minecraft').replace(/§[0-9a-fk-orx]/gi,'').slice(0,60),protocol:count(data.version.protocol),motd,motd_clean:motd.map(span=>span.text).join(''),favicon_url:validFavicon(data.favicon),latency_ms:Math.max(0,Math.round(latency))};
}
export async function pingMinecraft(address:Address):Promise<ServerState>{
  const target=await destination(address);
  return new Promise((resolve,reject)=>{
    const socket=new Socket();let bytes=Buffer.alloc(0),finished=false;const start=performance.now();
    const finish=(error?:unknown,state?:ServerState)=>{if(finished)return;finished=true;clearTimeout(timer);socket.destroy();if(error)reject(error);else resolve(state!);};
    const timer=setTimeout(()=>finish(new ApiError(422,'El servidor no respondió al protocolo Minecraft en 6 segundos.')),6000);
    socket.once('error',()=>finish(new ApiError(422,'No se pudo conectar con el servidor Minecraft.')));
    socket.once('close',()=>{if(!finished)finish(new ApiError(422,'El servidor cerró la conexión sin publicar su estado.'));});
    socket.once('connect',()=>{const port=Buffer.alloc(2);port.writeUInt16BE(address.port);socket.write(Buffer.concat([frame(Buffer.concat([varint(0),varint(-1),textPacket(address.host),port,varint(1)])),frame(varint(0))]));});
    socket.on('data',(chunk:Buffer)=>{try{
      if(bytes.length+chunk.length>1_048_576)throw new ApiError(422,'La respuesta del servidor supera el límite permitido.');bytes=Buffer.concat([bytes,chunk]);
      const length=readVarint(bytes);if(!length)return;if(length.value>1_000_000)throw new ApiError(422,'El paquete del servidor es demasiado grande.');if(bytes.length<length.size+length.value)return;
      const packet=bytes.subarray(length.size,length.size+length.value),id=readVarint(packet);if(!id||id.value!==0)throw new ApiError(422,'Paquete de estado Minecraft no válido.');const size=readVarint(packet,id.size);if(!size||size.value>900_000||id.size+size.size+size.value>packet.length)throw new ApiError(422,'Mensaje de estado Minecraft no válido.');
      const json=JSON.parse(packet.toString('utf8',id.size+size.size,id.size+size.size+size.value));finish(undefined,stateFromJson(json,performance.now()-start));
    }catch(error){finish(error);}});

    socket.connect({host:target.address,port:target.port,family:target.family});
  });
}
