import {writable} from 'svelte/store';
import {invoke} from '@tauri-apps/api/core';
import type {LocalItemModel} from './item_models';

export type LocalGameArtwork={textures:Record<string,string>;models:Record<string,LocalItemModel>;source:string;unavailable?:string[]};
const empty:LocalGameArtwork={textures:{},models:{},source:'Instala una versión para usar sus recursos locales'};
export const localGameArtwork=writable<LocalGameArtwork>(empty);


export const localVersionArtwork=writable<LocalGameArtwork>(empty);
let sequence=0;
let haveVersionIcons=false;
function normalize(result:LocalGameArtwork):LocalGameArtwork{
 const textures=Object.fromEntries(Object.entries(result.textures??{}).filter(([id,data])=>/^[a-z0-9_]+$/.test(id)&&typeof data==='string'&&data.startsWith('data:image/png;base64,')&&data.length<700000));
 const models=Object.fromEntries(Object.entries(result.models??{}).filter(([id,model])=>/^[a-z0-9_]+$/.test(id)&&model&&['sprite','model','shield','trident'].includes(model.kind)));
 return{textures,models,source:result.source,unavailable:result.unavailable};
}

export async function refreshLocalGameArtwork(version:string,iconFallbackVersion?:string):Promise<void>{
 const request=++sequence;localGameArtwork.set(empty);
 try{
  const result=normalize(await invoke<LocalGameArtwork>('get_poser_assets',{versionId:version}));
  if(request!==sequence)return;
  localGameArtwork.set(result);
  if(['stone_block','grass_block','redstone','cobblestone_block'].every(id=>result.textures[id])){localVersionArtwork.set(result);haveVersionIcons=true;}
  else if(!haveVersionIcons&&iconFallbackVersion&&iconFallbackVersion!==version){
   const fallback=normalize(await invoke<LocalGameArtwork>('get_poser_assets',{versionId:iconFallbackVersion}));
   if(request===sequence&&Object.keys(fallback.textures).length){localVersionArtwork.set(fallback);haveVersionIcons=true;}
  }else if(!haveVersionIcons&&Object.keys(result.textures).length){localVersionArtwork.set(result);haveVersionIcons=true;}
 }catch{ }
}
