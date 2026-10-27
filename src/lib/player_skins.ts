import {invoke} from '@tauri-apps/api/core';
import {FALLBACK_SKIN} from './poses_catalog';

export type DefaultSkinVariant='steve'|'alex';
export type PublicPlayerAppearance={name:string;skin_data:string;cape_data:string;avatar_data:string;model:'default'|'slim'};
type AppearanceLoader=(variant:DefaultSkinVariant)=>Promise<PublicPlayerAppearance>;
type AppearanceResolver=(variant?:DefaultSkinVariant)=>Promise<PublicPlayerAppearance>;
type Entry={expires:number;value?:PublicPlayerAppearance;error?:unknown;pending?:Promise<PublicPlayerAppearance>};
const SUCCESS_TTL=30*60*1000,FAILURE_TTL=30*1000;



export function createDefaultSkinResolver(load:AppearanceLoader,now:()=>number=Date.now):AppearanceResolver{
 const cache=new Map<DefaultSkinVariant,Entry>();
 return (variant:DefaultSkinVariant='steve')=>{
  if(variant!=='steve'&&variant!=='alex')return Promise.reject(new Error('Modelo predeterminado no válido.'));
  const existing=cache.get(variant);
  if(existing?.pending)return existing.pending;
  if(existing&&existing.expires>now())return existing.value?Promise.resolve(existing.value):Promise.reject(existing.error);
  const entry:Entry={expires:0};cache.set(variant,entry);
  const request=Promise.resolve().then(()=>load(variant)).then(value=>{
   if(typeof value.skin_data!=='string'||!value.skin_data.startsWith('data:image/png;base64,')||typeof value.avatar_data!=='string'||!value.avatar_data.startsWith('data:image/png;base64,')||!['default','slim'].includes(value.model))throw new Error('La skin predeterminada no es válida.');
   entry.value=value;entry.error=undefined;entry.expires=now()+SUCCESS_TTL;entry.pending=undefined;return value;
  }).catch(error=>{entry.error=error;entry.expires=now()+FAILURE_TTL;entry.pending=undefined;throw error;});
  entry.pending=request;return request;
 };
}

export const defaultPlayerAppearance=createDefaultSkinResolver(async(variant)=>{
 if(typeof window==='undefined')throw new Error('La apariencia se prepara al abrir el launcher.');
 return invoke<PublicPlayerAppearance>('get_default_player_appearance',{variant});
});

export function isFallbackSkin(source?:string):boolean{return !source||source===FALLBACK_SKIN;}

export function crafatarSkinUrl(uuid:string,variant:DefaultSkinVariant='steve'):string|null{
 const id=uuid.trim().replaceAll('-','').toLowerCase();
 if(!/^[a-f0-9]{32}$/.test(id)||/^0{32}$/.test(id))return null;
 return `https://crafatar.com/skins/${id}?default=${variant==='alex'?'MHF_Alex':'MHF_Steve'}`;
}

const images=new Map<string,{pending:Promise<HTMLImageElement>;expires:number}>();

export function loadSkinImage(source:string):Promise<HTMLImageElement>{
 if(typeof Image==='undefined')return Promise.reject(new Error('El visor todavía no está disponible.'));
 const prior=images.get(source);if(prior&&prior.expires>Date.now())return prior.pending;
 const pending=new Promise<HTMLImageElement>((resolve,reject)=>{
  const image=new Image();image.crossOrigin='anonymous';image.decoding='async';
  const timeout=setTimeout(()=>{image.onload=image.onerror=null;image.src='';reject(new Error('La descarga de la skin tardó demasiado.'));},15000);
  image.onload=()=>{clearTimeout(timeout);image.onload=image.onerror=null;
   if(image.width<64||image.width>2048||image.width%64!==0||![image.width,image.width/2].includes(image.height)){reject(new Error('La imagen no tiene formato de skin.'));return;}resolve(image);
  };
  image.onerror=()=>{clearTimeout(timeout);image.onload=image.onerror=null;reject(new Error('No se pudo leer la textura.'));};image.src=source;
 });
 const entry={pending,expires:Date.now()+5*60*1000};images.delete(source);images.set(source,entry);
 while(images.size>16)images.delete(images.keys().next().value!);
 void pending.catch(()=>{if(images.get(source)===entry)images.delete(source);});return pending;
}
