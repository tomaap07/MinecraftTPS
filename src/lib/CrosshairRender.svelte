<script lang="ts">
import{crosshairPath,type CrosshairSettings}from './crosshair';
let{settings,spread=0,color='',cooldown=null,hit=false}:{settings:CrosshairSettings;spread?:number;color?:string;cooldown?:number|null;hit?:boolean}=$props();
let path=$derived(crosshairPath(settings,spread)),ink=$derived(color||settings.color),radius=$derived(settings.size+settings.gap+spread+5);
</script>
<svg class:chroma={settings.chroma&&!color} style:--ink={ink} width="100" height="100" viewBox="-50 -50 100 100" fill="none" aria-label="Mira personalizada" style:opacity={settings.opacity}>
 {#if settings.shape==='pixels'}<g shape-rendering="crispEdges" fill="var(--ink)" stroke="#08080C" stroke-width={settings.outline*.25}>{#each settings.pixels as value,i}{#if value}<rect x={(i%settings.grid-settings.grid/2)*2} y={(Math.floor(i/settings.grid)-settings.grid/2)*2} width="2" height="2"/>{/if}{/each}</g>{:else}
 {#if settings.outline>0}<path d={path} stroke="#08080C" stroke-width={settings.thickness+settings.outline*2} stroke-linecap="square"/>{/if}<path d={path} stroke="var(--ink)" stroke-width={settings.thickness} stroke-linecap="square"/>
 {#if settings.shape==='dot'||settings.shape==='cross-dot'}<circle r={settings.dot} fill="var(--ink)" stroke="#08080C" stroke-width={settings.outline}/>{/if}{/if}
 {#if cooldown!=null&&settings.cooldown==='ring'}<circle r={radius} stroke="#08080C" stroke-width="3"/><circle r={radius} stroke="var(--ink)" stroke-width="1.5" pathLength="100" stroke-dasharray={`${Math.max(0,Math.min(1,cooldown))*100} 100`} transform="rotate(-90)"/>{:else if cooldown!=null&&settings.cooldown==='bar'}<path d={`M -12 ${radius+2} H 12`} stroke="#08080C" stroke-width="4"/><path d={`M -12 ${radius+2} h ${24*Math.max(0,Math.min(1,cooldown))}`} stroke="var(--ink)" stroke-width="2"/>{/if}
 {#if hit}<path d="M -13 -13 l 5 5 M 13 -13 l -5 5 M -13 13 l 5 -5 M 13 13 l -5 -5" stroke={settings.hit_color} stroke-width="2"/>{/if}
</svg>
<style>svg{overflow:visible;color:var(--ink)}.chroma{animation:crosshair-chroma 7s linear infinite}@keyframes crosshair-chroma{from{filter:hue-rotate(0deg)}to{filter:hue-rotate(360deg)}}</style>
