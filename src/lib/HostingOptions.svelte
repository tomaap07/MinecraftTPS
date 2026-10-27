<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { isGameRunning } from './game_session';
  import UiIcon from './UiIcon.svelte';
  import configuredProviders from './data/hosting_providers.json';
  let { compact=false }:{compact?:boolean}=$props();
  let message=$state('');
  function safeUrl(value:string){try{const url=new URL(value);return url.protocol==='https:'&&!url.username&&!url.password?url.href:'';}catch{return '';}}

  const providers=configuredProviders.filter(provider=>safeUrl(provider.url)).map(provider=>{
    const affiliate=provider.affiliateActive&&!!safeUrl(provider.affiliateUrl);
    return {...provider,affiliate,target:affiliate?safeUrl(provider.affiliateUrl):safeUrl(provider.url)};
  });
  const hasAffiliate=providers.some(provider=>provider.affiliate);
  async function visit(url:string){try{await openUrl(url);}catch{message='No se pudo abrir el navegador.';}}
</script>

{#if !$isGameRunning}
  <section class:compact class="hosting-options" aria-label="Opciones de hosting">
    <header><span class="section-icon"><UiIcon name="server" size={21}/></span><div><small>OPCIONES DE HOSTING</small><h3>Un espacio para jugar con amigos</h3></div></header>
    <div class="providers">{#each providers as provider}<button onclick={()=>void visit(provider.target)}><UiIcon name="server" size={18}/><span><strong>{provider.name}</strong><small>{provider.affiliate?'Enlace de afiliado':compact?'Sitio del proveedor':provider.detail}</small>{#if provider.affiliate&&provider.offer}<small>{provider.offer}</small>{/if}</span><UiIcon name="external" size={14}/></button>{/each}</div>
    <footer><p>{hasAffiliate?'Los enlaces identificados como afiliados pueden generar una comisión para TP LABS. Revisa el precio y las condiciones finales con el proveedor.':'Enlaces informativos a proveedores externos. No hay afiliaciones ni descuentos de TP LAUNCHER activos.'}</p>{#if !compact}<button class="feature" onclick={()=>void visit('https://github.com/tomaap07/MinecraftTPS/issues/new?title=Solicitud%20de%20servidor%20destacado&body=Nombre%20del%20servidor%3A%0AIP%3A%0ASitio%20web%3A')}><UiIcon name="trophy" size={16}/>Solicitar un destacado<UiIcon name="external" size={13}/></button>{/if}</footer>
    {#if message}<p role="status">{message}</p>{/if}
  </section>
{/if}

<style>
  .hosting-options{padding:19px;border:1px solid var(--border,#27273a);background:rgba(var(--bg-base-rgb,11,11,15),.8);color:#e8e1f5}header{display:flex;align-items:center;gap:12px;margin-bottom:15px}.section-icon{padding:10px;background:#a855f712;border:1px solid #a855f72b;color:var(--primary,#a855f7)}h3{margin:5px 0 0;font-size:.85rem}small{font-size:.58rem;color:#aaa0bd;line-height:1.6}header small{letter-spacing:.1em;color:var(--primary,#a855f7)}.providers{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:9px}button{font:inherit;display:flex;align-items:center;gap:10px;text-align:left;background:#ffffff03;color:#dcd2ef;border:1px solid #ffffff17;padding:12px;cursor:pointer;min-width:0}button>span{display:grid;gap:5px;flex:1;min-width:0}strong{font-size:.73rem}button:hover{border-color:var(--primary,#a855f7);background:#a855f70b}button:focus-visible{outline:2px solid var(--primary,#a855f7);outline-offset:3px}button :global(svg){flex-shrink:0}footer{display:flex;justify-content:space-between;align-items:center;gap:15px;margin-top:12px}p{font-size:.61rem;line-height:1.7;color:#9b90ad;margin:0}.feature{font-size:.65rem;white-space:nowrap;padding:8px}.compact{padding:15px}.compact header{margin-bottom:12px}.compact .providers button{padding:10px}.compact footer{margin-top:10px}@media(max-width:800px){.providers{grid-template-columns:1fr}footer{align-items:start;flex-direction:column}}
</style>
