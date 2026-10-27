<script lang="ts">
 import UiIcon from './UiIcon.svelte';
 import {brandAssets,serviceSymbols} from './brand_assets';
 let{name,domain,size=32,remote=true}:{name:string;domain:string;size?:number;remote?:boolean}=$props();
 let failed=$state(false);
 let key=$derived(name.toLowerCase().replace(/[^a-z0-9]/g,''));
 const symbols:Record<string,string>={misode:'sliders',minecrafttools:'plugins',gamergeeks:'terminal',modpackindex:'filter',mcsrvstat:'server'};
 const existingSites=new Set(['namemc','skinmc','mcstacker','chunkbase','planetminecraft','minecraftwiki']);
 let safeDomain=$derived(/^[a-z0-9.-]+\.[a-z]{2,}$/i.test(domain)?domain.toLowerCase():'');
 let source=$derived(brandAssets[key]?.src??(remote&&safeDomain&&existingSites.has(key)?`https://www.google.com/s2/favicons?domain=${encodeURIComponent(safeDomain)}&sz=128`:''));
 $effect(()=>{source;failed=false;});
</script>
<span class="community-icon" role="img" aria-label={name} title={name} style:width={`${size}px`} style:height={`${size}px`}>
 {#if source&&!failed}<img src={source} alt="" width={size} height={size} loading="lazy" decoding="async" referrerpolicy="no-referrer" draggable="false" onerror={()=>failed=true}/>
 {:else}<UiIcon name={symbols[key]??serviceSymbols[key]??'external'} {size}/>{/if}
</span>
<style>.community-icon{display:inline-flex;align-items:center;justify-content:center;flex-shrink:0;vertical-align:middle;line-height:0;color:#c3ccd8}.community-icon img{width:100%;height:100%;object-fit:contain}.community-icon :global(svg){flex-shrink:0}</style>
