<script lang="ts">
 import type{EquipmentKind}from './hud_equipment';
 let{kind='tool',size=24}:{kind?:EquipmentKind;size?:number}=$props();
</script>

<svg width={size} height={size} viewBox="0 0 16 16" shape-rendering="crispEdges" fill="none" aria-hidden="true">
 <g fill="#d8b4fe" stroke="#281737" stroke-width="1">
 {#if kind==='head'}<path d="M3 3h10v9h-3V8H6v4H3z"/><path d="M5 4h6v2H5z" fill="#f3e8ff" stroke="none"/>
 {:else if kind==='chest'}<path d="M4 2h3v2h2V2h3l3 3-3 3v6H4V8L1 5z"/><path d="M6 6h4v6H6z" fill="#a855f7" stroke="none"/>
 {:else if kind==='legs'}<path d="M4 2h8v12H9V8H7v6H4z"/><path d="M5 3h6v2H5z" fill="#f3e8ff" stroke="none"/>
 {:else if kind==='feet'}<path d="M3 3h3v8h1v3H1v-3h2zm7 0h3v8h2v3H9v-3h1z"/>
 {:else if kind==='shield'}<path d="M3 2h10v8l-5 4-5-4z"/><path d="M7 3h2v9H7z" fill="#a855f7" stroke="none"/>
 {:else if kind==='bow'}<path d="M5 1h3v2h2v2h2v6h-2v2H8v2H5v-2h2v-2h2V5H7V3H5z"/><path d="M5 2v12" stroke="#f3e8ff"/>
 {:else if kind==='sword'}<path d="M11 1h4v4l-6 6-4-4z"/><path d="m3 6 7 7-2 2-7-7z" fill="#a855f7"/><path d="m4 10 2 2-3 3-2-2z" fill="#6b3e86"/>
 {:else if kind==='pickaxe'}<path d="m2 2 8-1 5 5-2 2-4-4-6 1z"/><path d="m9 5 2 2-8 8-2-2z" fill="#8d60ad"/>
 {:else if kind==='axe'}<path d="m6 1 5 1 3 5-4 4-2-4H4V4z"/><path d="m8 6 2 2-6 7-2-2z" fill="#8d60ad"/>
 {:else if kind==='shovel'}<path d="m10 1 5 4-4 5-5-4z"/><path d="m7 7 2 2-6 6-2-2z" fill="#8d60ad"/>
 {:else}<path d="M3 2h3v4h4V2h3v6l-3 2v4H6v-4L3 8z"/>
 {/if}
 </g>
</svg>
