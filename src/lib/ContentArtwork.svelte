<script lang="ts">
  let {src='',alternate='',name='',kind='mod',size=42}:{src?:string;alternate?:string;name?:string;kind?:'mod'|'pack';size?:number}=$props();
  let failed=$state<string[]>([]);
  let current=$derived([src,alternate].find(value=>value&&!failed.includes(value))??'');
  $effect(()=>{src;alternate;failed=[];});
  const colors=['#A855F7','#38BDF8','#34D399','#F59E0B','#F472B6'];
  let accent=$derived(colors[Array.from(name).reduce((sum,char)=>(sum+char.charCodeAt(0))%colors.length,0)]);
</script>
<span class="artwork" style:width={`${size}px`} style:height={`${size}px`} aria-hidden="true">
  {#if current}<img src={current} alt="" loading="lazy" onerror={()=>{if(current)failed=[...failed,current];}}/>
  {:else}<svg viewBox="0 0 16 16" fill="none" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">
    <path fill="#0D1022" d="M1 0h14v1h1v14h-1v1H1v-1H0V1h1z"/>
    {#if kind==='pack'}
      <path fill="#D7C9F2" d="M3 3h9v2H3zm0 4h10v2H3zm0 4h8v2H3z"/>
      <path fill={accent} d="M2 2h10v1H3v2h10v1H2zm0 4h11v1H3v2h11v1H2zm0 4h9v1H3v2h9v1H2z"/>
      <path fill="#756389" d="M12 2h2v4h-2zm1 4h2v4h-2zm-2 4h2v4h-2z"/>
      <path fill="#FDE68A" d="M10 3h1v2h-1zM8 7h1v2H8zm-2 4h1v2H6z"/>
    {:else}
      <path fill={accent} d="M3 3h10v10H3z"/><path fill="#E2E8F0" d="M3 3h7v1H4v6H3z"/>
      <path fill="#FFFFFF" fill-opacity=".25" d="M4 4h8v2H4z"/><path fill="#15182D" d="M6 6h4v4H6z"/>
      <path fill="#E2E8F0" d="M7 7h2v2H7zM5 1h1v2H5zm5 0h1v2h-1zM5 13h1v2H5zm5 0h1v2h-1zM1 5h2v1H1zm0 5h2v1H1zm12-5h2v1h-2zm0 5h2v1h-2z"/>
      <path fill="#FDE68A" d="M11 11h2v2h-2z"/>
    {/if}
  </svg>{/if}
</span>
<style>
.artwork{display:inline-grid;place-items:center;flex-shrink:0;overflow:hidden;border-radius:8px;background:#121322}.artwork img,.artwork svg{display:block;width:100%;height:100%;object-fit:contain}.artwork svg{image-rendering:pixelated}
</style>
