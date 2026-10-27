<script lang="ts">
 import TpPixelIcon from './TpPixelIcon.svelte';
 let{image='',fallback='',category=''}:{image?:string;fallback?:string;category?:string}=$props();
 let failed=$state<string[]>([]);
 let sources=$derived([...new Set([image,fallback].filter(Boolean))]);
 let current=$derived(sources.find(url=>!failed.includes(url)));
 $effect(()=>{image;fallback;failed=[];});
</script>
<figure class="news-art" aria-hidden="true">
 {#if current}<img src={current} alt="" loading="lazy" decoding="async" referrerpolicy="no-referrer" onerror={()=>{if(current)failed=[...failed,current];}}/>
 {:else}<TpPixelIcon name={/snapshot|preview|beta/i.test(category)?'snapshot':'dimension'} size={58}/><i></i>{/if}
</figure>
<style>.news-art{margin:0;position:relative;height:137px;flex-shrink:0;display:grid;place-items:center;overflow:hidden;background:radial-gradient(ellipse at center,#A855F723,transparent 65%),repeating-linear-gradient(90deg,transparent 0 27px,#A855F70B 27px 28px),#111019}.news-art img{display:block;width:100%;height:100%;object-fit:cover}.news-art i{position:absolute;inset:15px;border:1px solid #A855F720;pointer-events:none}</style>
