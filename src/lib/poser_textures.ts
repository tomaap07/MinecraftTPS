import type * as THREE from 'three';
import {buildLocalItemModel,type LocalItemModel,type ItemPixels} from './item_models';
type Prepared={model:LocalItemModel;images:Map<string,ItemPixels>};
const prepared=new Map<string,Prepared>();let revision=0,currentModels:Record<string,LocalItemModel>|undefined,pending:Promise<void>|undefined;
const EMPTY:Record<string,LocalItemModel>={};
async function decode(source:string):Promise<ItemPixels>{
 if(!source.startsWith('data:image/png;base64,')||source.length>3*1024*1024)throw new Error('Textura local no válida.');
 const image=await new Promise<HTMLImageElement>((resolve,reject)=>{const image=new Image();const timeout=setTimeout(()=>{image.onload=image.onerror=null;reject(new Error('Textura local no disponible.'));},5000);image.onload=()=>{clearTimeout(timeout);resolve(image);};image.onerror=()=>{clearTimeout(timeout);reject(new Error('Textura local dañada.'));};image.src=source;});
 if(image.width<1||image.width>2048||image.height<1||image.height>16384||image.width*image.height>4*1024*1024)throw new Error('Textura local demasiado grande.');
 const canvas=document.createElement('canvas');canvas.width=image.width;canvas.height=image.height;const context=canvas.getContext('2d');if(!context)throw new Error('Canvas no disponible.');context.drawImage(image,0,0);return{width:image.width,height:image.height,data:context.getImageData(0,0,image.width,image.height).data};
}

export function preparePoserTextures(_textures:Record<string,string>={},models:Record<string,LocalItemModel>=EMPTY):Promise<void>{
 if(currentModels===models&&pending)return pending;currentModels=models;const request=++revision;
 pending=(async()=>{const next=new Map<string,Prepared>();
  if(typeof Image!=='undefined')await Promise.all(Object.entries(models).slice(0,64).map(async([id,model])=>{
   if(!model||!/^[a-z0-9_]+$/.test(id)||!['sprite','model','shield','trident'].includes(model.kind)||!model.textures||!Array.isArray(model.elements)||Object.keys(model.textures).length>16||model.elements.length>128)return;
   try{const images=new Map<string,ItemPixels>();await Promise.all(Object.entries(model.textures).map(async([key,source])=>{images.set(key,await decode(source));}));if(images.size)next.set(id,{model,images});}catch{ }
  }));
  if(request===revision){prepared.clear();for(const[id,value]of next)prepared.set(id,value);}
 })();return pending;
}
export function localItemModel(id:string):LocalItemModel|undefined{return prepared.get(id)?.model;}
export function voxelItem(three:typeof import('three'),id:string):THREE.Group|null{const item=prepared.get(id);return item?buildLocalItemModel(three,item.model,item.images):null;}
