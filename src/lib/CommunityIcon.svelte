<script lang="ts">
 let{name,domain,size=32,remote=true}:{name:string;domain:string;size?:number;remote?:boolean}=$props();
 let failed=$state(false);
 let key=$derived(name.toLowerCase().replace(/[^a-z0-9]/g,''));
 let safeDomain=$derived(/^[a-z0-9.-]+\.[a-z]{2,}$/i.test(domain)?domain.toLowerCase():'');
 let source=$derived(remote&&safeDomain?`https://www.google.com/s2/favicons?domain=${encodeURIComponent(safeDomain)}&sz=128`:'');
 $effect(()=>{source;failed=false;});
</script>
<span class="community-icon" role="img" aria-label={name} title={name} style:width={`${size}px`} style:height={`${size}px`}>
 {#if source&&!failed}<img src={source} alt="" width={size} height={size} loading="lazy" decoding="async" referrerpolicy="no-referrer" draggable="false" onerror={()=>failed=true}/>
 {:else}

 <svg width={size} height={size} viewBox="0 0 16 16" shape-rendering="crispEdges" aria-hidden="true" fill="none">
 {#if key==='namemc'||key==='skinmc'}
  <path fill="#E2E8F0" d="M6 1h5v1h2v1h2v9h-2v1h-2v1H5v-1H3V4h1V2h2z"/>
  <path fill={key==='skinmc'?'#CA6A42':'#6544A7'} d="M6 2h5v1h2v1h1v6h-2V7H5V5H4V4h1V3h1z"/>
  <path fill="#F6BD88" d="M4 6h6v1h2v6h-1v1H5v-1H4z"/><path fill="#B87661" d="M12 7h2v4h-1v1h-1z"/>
  <path fill="#34253F" d="M4 6h2v2H4zM9 7h3v1H9zM6 11h1v1h3v1H6z"/>
  <path fill="#FFF6ED" d="M5 9h2v1H5zM9 9h2v1H9z"/><path fill="#40DCC1" d="M6 9h1v1H6zM10 9h1v1h-1z"/>
  <path fill="#F4C1FF" d="M6 3h3v1H6z"/>
 {:else if key==='mcstacker'}
  <path fill="#D8B4FE" d="M1 3h14v10H1z"/><path fill="#25193C" d="M2 4h12v8H2z"/><path fill="#A855F7" d="M2 4h12v2H2z"/>
  <path fill="#72E6A5" d="M3 7h1v1h1v1H4v1H3V9h1V8H3zM7 10h4v1H7z"/><path fill="#D946EF" d="M5 14h6v1H5zM7 13h2v1H7z"/><path fill="#FDE68A" d="M12 4h1v1h-1z"/>
 {:else if key==='blockbench'}
  <path fill="#E2E8F0" d="M4 1h4v1h2v4h-1v1H4V6H2V2h2zM10 8h3v1h2v4h-2v1h-3v-1H8V9h2zM2 8h3v1h2v4H5v1H2v-1H1V9h1z"/>
  <path fill="#38BDF8" d="M4 2h4v1h1v2H7v1H4V5H3V3h1z"/><path fill="#0369A1" d="M6 4h3v2H6z"/>
  <path fill="#F89820" d="M2 9h3v1h1v2H4v1H2V9z"/><path fill="#D946EF" d="M10 9h3v1h1v2h-2v1h-2z"/>
 {:else if key==='chunkbase'}
  <path fill="#C4B5FD" d="M7 1h2v1h2v1h2v1h2v8h-2v1h-2v1H9v1H7v-1H5v-1H3v-1H1V4h2V3h2V2h2z"/>
  <path fill="#53C99C" d="M7 2h2v1h2v1h2v1h1v1h-3v1H9v1H7V7H5V6H2V5h1V4h2V3h2z"/>
  <path fill="#B7F0B4" d="M7 3h2v1H7zM4 5h2v1H4zM10 5h3v1h-3z"/><path fill="#7A6390" d="M2 7h3v1h2v1h1v5H7v-1H5v-1H3v-1H2z"/>
  <path fill="#3E395C" d="M14 7v4h-1v1h-2v1H9V9h2V8h2V7z"/><path fill="#EBC78B" d="M3 8h2v2H3zM6 11h1v1H6zM11 10h2v1h-2z"/>
 {:else if key==='planetminecraft'}
  <path fill="#C4B5FD" d="M5 1h6v1h2v2h1v2h1v5h-1v2h-2v1h-1v1H5v-1H3v-1H2v-2H1V6h1V4h1V2h2z"/>
  <path fill="#2876CF" d="M5 2h6v1h2v3h1v5h-1v2h-2v1H5v-1H3v-2H2V6h1V4h2z"/>
  <path fill="#5BDAAC" d="M5 3h3v2H6v2H3V5h2zM9 6h4v3h-2v2h-1v2H8v-3H7V8h2zM4 10h2v2H4z"/>
  <path fill="#BAE6FD" d="M7 2h3v1H7zM3 7h1v2H3zM12 11h1v1h-1z"/>
 {:else if key==='minecraftwiki'}
  <path fill="#E9BD62" d="M1 3h5v1h4V3h5v10h-5v1H6v-1H1z"/><path fill="#E2E8F0" d="M2 2h4v1h1v1h2V3h1V2h4v9h-4v1H6v-1H2z"/>
  <path fill="#9B5AE8" d="M3 3h3v1h1v7H6v-1H3z"/><path fill="#6334AC" d="M10 3h3v7h-3v1H9V4h1z"/>
  <path fill="#FDE68A" d="M4 5h2v1H4zM4 8h2v1H4zM10 5h2v1h-2zM10 8h2v1h-2zM7 3h2v1H7z"/><path fill="#D946EF" d="M7 6h2v6H7z"/>
 {:else}
  <path fill="#DDD6FE" d="M3 2h10v1h2v10h-2v1H3v-1H1V3h2z"/><path fill={key==='curseforge'?'#A7582D':'#365D6B'} d="M2 4h12v8H2z"/>
  <path fill={key==='curseforge'?'#FFAB69':'#71E4BA'} d="M6 4h4v3h2v1h-1v1h-1v1H6V9H5V8H4V7h2z"/>
  <path fill="#E2E8F0" d="M4 11h8v1H4z"/>
 {/if}
 </svg>{/if}
</span>
<style>.community-icon{display:inline-flex;align-items:center;justify-content:center;flex-shrink:0;vertical-align:middle;line-height:0}.community-icon img{width:100%;height:100%;object-fit:contain}.community-icon svg{flex-shrink:0}</style>
