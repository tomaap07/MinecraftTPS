import seed from './data/server_catalog.json';
import type { MotdSpan } from './server_live';

export type DirectoryEntry = {
  id:string;name:string;address:string;description:string;version:string;tags:string[];
  online:boolean|null;players_online:number|null;players_max:number|null;
  checked_at:string;java_compatible:boolean;category?:string;region?:string;language?:string;featured?:boolean;icon?:string;
  motd?:MotdSpan[];from_cloud?:boolean;sponsor_url?:string;featured_until?:string;
};
export type DirectoryPage = {entries:DirectoryEntry[];total:number;page:number;pages:number;source:string;stale:boolean;message:string};
export const DIRECTORY_MODES = [
  ['', 'Todas las modalidades'], ['survival','Survival'], ['smp','SMP'], ['minigames','Minijuegos'],
  ['pvp','PvP'], ['skyblock','SkyBlock'], ['anarchy','Anarquía'], ['prison','Prison'],
  ['factions','Factions'], ['parkour','Parkour'], ['mmorpg','RPG / MMORPG'], ['pixelmon','Pixelmon'],
  ['economy','Economía'], ['hardcore','Hardcore'], ['lifesteal','Lifesteal'], ['towny','Towny'],
] as const;
export function addressKey(address:string):string {
  return address.trim().toLowerCase().replace(/\.$/,'').replace(/:25565$/,'');
}
export const normalizeSearch = (value:string) => value.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase();
export function directoryTimestamp(value:string):number {
  if(value.startsWith('unix:'))return Number(value.slice(5))*1000;
  const parsed=Date.parse(value);return Number.isFinite(parsed)?parsed:0;
}
export const COMMUNITY_SERVERS: readonly DirectoryEntry[] = seed.map(row => {
  const category=normalizeSearch(row.category);
  const tags=DIRECTORY_MODES.filter(([id,label])=>id&&(category.includes(id)||category.includes(normalizeSearch(label).split(' / ')[0])))
    .map(([id])=>id as string);
  if(category.includes('rpg')&&!tags.includes('mmorpg'))tags.push('mmorpg');
  if(category.includes('multimodo'))tags.push('minigames');
  return {id:`community:${row.id}`,name:row.name,address:row.ip,category:row.category,region:row.region,language:row.lang,
    featured:row.featured,description:row.category,version:'',tags,online:null,players_online:null,players_max:null,
    checked_at:'',java_compatible:true};
});


export function directoryPages(current:number,total:number):number[] {
  return [...new Set([1,...Array.from({length:5},(_,i)=>current+i-2).filter(n=>n>1&&n<total),total])].sort((a,b)=>a-b);
}
