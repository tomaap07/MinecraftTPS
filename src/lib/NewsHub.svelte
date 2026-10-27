<script lang="ts">
  import BrandWordmark from "./BrandWordmark.svelte";
  import GameIcon from "./GameIcon.svelte";
  import ServiceLogo from "./ServiceLogo.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import UiIcon from "./UiIcon.svelte";
  import NewsArtwork from "./NewsArtwork.svelte";
  let { onexplore }: { onexplore?:(section:"shader"|"resourcepack")=>void } = $props();
  type News = {title:string;summary:string;image:string;image_fallback?:string;url:string;date:string;category:string};
  let tab=$state("news"),news=$state<News[]>([]),error=$state(""),loading=$state(true);
  const links=[
    ["Minecraft","https://www.minecraft.net/","minecraft"],["Minecraft Wiki","https://minecraft.wiki/","wiki"],
    ["Estado de los servicios","https://help.minecraft.net/hc/en-us/articles/360047118272","minecraft"],
    ["Comunidad Minecraft · Discord","https://discord.com/invite/minecraft","discord"],
    ["Código y contribuciones","https://github.com/tomaap07/MinecraftTPS","github"],
  ];
  const changes=[
    {title:"Tu Centro de Mando",text:"TP LAUNCHER reúne tus versiones, biblioteca de contenido y herramientas de creación en un mismo espacio."},
    {title:"Multijugador a tu manera",text:"Gestiona servers.dat, consulta el estado de los servidores y comparte mundos LAN desde la instancia que elijas."},
    {title:"Un estudio para tu personaje",text:"Poses, iluminación y objetos equipables para preparar renders transparentes y tu próxima miniatura."},
    {title:"Ajustes que se adaptan a ti",text:"Perfiles de cuenta, Java por versión, idiomas y un asistente que prepara tu primera sesión."},
  ];
  onMount(()=>{void load();});
  async function load(){loading=true;error="";try{news=await invoke<News[]>("launcher_news");}catch(e){error=String(e);}finally{loading=false;}}
  async function open(url:string){try{await openUrl(url);}catch(e){error=String(e);}}
</script>

<section class="news-hub" class:news-mode={tab==='news'} aria-label="Noticias y novedades">
  <header>
    <div><small>{tab==='news' ? 'AL DÍA CON TU UNIVERSO' : 'TP LAUNCHER / DIARIO DEL BLOQUE'}</small><h2>{tab==='news' ? 'Noticias y novedades' : 'Notas de versión · TP LAUNCHER'}</h2></div>
    <button class="refresh" onclick={load} disabled={loading} aria-label="Actualizar noticias"><UiIcon name="refresh" size={16}/></button>
  </header>
  <div class="news-layout">
    <div class="editorial">
      <nav aria-label="Noticias y notas de versión">
        <button class:active={tab==='news'} onclick={()=>tab='news'}><ServiceLogo name="minecraft" size={17}/>Noticias de Minecraft</button>
        <button class:active={tab==='changes'} onclick={()=>tab='changes'}><UiIcon name="book" size={17}/>Notas de versión · TP LAUNCHER</button>
      </nav>
      {#if error}<p class="error" role="status">{error}</p>{/if}
      <div class="article-scroll" class:news-content={tab==='news'} aria-busy={loading && tab==='news'}>
        {#if tab==='news'}
          {#if loading}<p class="muted" role="status">Consultando el feed oficial de Minecraft…</p>
          {:else if news.length}
            <div class="news-grid">
              {#each news as item}
                <a class="news-card" href={item.url} onclick={(event)=>{event.preventDefault();void open(item.url);}}>
                  <NewsArtwork image={item.image} fallback={item.image_fallback} category={item.category}/>
                  <div><small>{item.category} · {item.date}</small><h3>{item.title}</h3><p>{item.summary}</p><span>Leer en Minecraft.net <UiIcon name="arrow" size={14}/></span></div>
                </a>
              {/each}
            </div>
          {:else}<p class="muted">El feed oficial todavía no tiene noticias disponibles.</p>{/if}
        {:else}
          {#each changes as item}<article class="news-article"><div class="article-meta"><span>TP LAUNCHER</span></div><h3>{item.title}</h3><p>{item.text}</p></article>{/each}
        {/if}
      </div>
      {#if tab==='news'}<p class="source-note">Fuente: feed oficial de Minecraft. Se muestra la fecha de cada publicación.</p>{/if}
    </div>
    {#if tab==='changes'}
    <aside aria-label="Enlaces oficiales y comunidad">
      <section class="link-section">
        <h3>Enlaces oficiales</h3>
        <a href="https://www.minecraft.net/" onclick={(event)=>{event.preventDefault();void open('https://www.minecraft.net/');}}><ServiceLogo name="minecraft" size={16}/>Minecraft.net <UiIcon name="arrow" size={13}/></a>
        <a href="https://help.minecraft.net/" onclick={(event)=>{event.preventDefault();void open('https://help.minecraft.net/');}}><UiIcon name="shield" size={16}/>Ayuda y soporte <UiIcon name="arrow" size={13}/></a>
      </section>
      <section class="link-section"><h3>Comunidad</h3>{#each links.slice(1) as [title,url,logo]}<a href={url} onclick={(event)=>{event.preventDefault();void open(url);}}><ServiceLogo name={logo} size={17}/><span>{title}</span><UiIcon name="arrow" size={13}/></a>{/each}</section>
      <a class="archive-link" href="https://www.minecraft.net/en-us/articles" onclick={(event)=>{event.preventDefault();void open('https://www.minecraft.net/en-us/articles');}}><UiIcon name="grid" size={20}/><span>Todas las noticias<small>Minecraft.net</small></span><UiIcon name="arrow" size={15}/></a>
      <div class="edition-mark"><span><BrandWordmark/></span><small>TU PRÓXIMA AVENTURA EMPIEZA AQUÍ</small></div>
    </aside>
    {/if}
  </div>
  {#if tab==='news'}<footer>{#each links as [title,url,logo]}<a href={url} onclick={(event)=>{event.preventDefault();void open(url);}}><ServiceLogo name={logo} size={17}/><span>{title}</span><UiIcon name="arrow" size={13}/></a>{/each}</footer>{/if}
  {#if onexplore}<div class="content-shortcuts" aria-label="Explorar shaders y recursos"><button onclick={()=>onexplore?.('shader')}><GameIcon name="ender_eye" size={18}/>Explorar shaders<UiIcon name="arrow" size={13}/></button><button onclick={()=>onexplore?.('resourcepack')}><GameIcon name="painting" size={18}/>Explorar recursos<UiIcon name="arrow" size={13}/></button></div>{/if}
</section>

<style>



  .content-shortcuts{display:flex;flex-wrap:wrap;gap:9px;padding:17px 30px;border-top:1px solid #ffffff12}.news-mode .content-shortcuts{padding:17px 0 0;margin-top:17px}.content-shortcuts button{display:inline-flex;align-items:center;gap:9px;border:1px solid rgba(var(--primary-rgb),.3);background:rgba(var(--primary-rgb),.09);color:#dbe3fa;border-radius:9px;padding:10px 14px;font-size:.74rem}.content-shortcuts button:hover{border-color:var(--primary);color:var(--primary)}
  .news-hub{position:relative;min-width:0;overflow:hidden;border:1px solid #ffffff18;border-top:3px solid var(--primary);border-radius:10px;background-color:#15171b;background-image:linear-gradient(135deg,#ffffff03 25%,transparent 25%,transparent 75%,#ffffff03 75%),linear-gradient(135deg,#ffffff03 25%,transparent 25%,transparent 75%,#ffffff03 75%);background-size:8px 8px;background-position:0 0,4px 4px;box-shadow:0 16px 40px #0003;color:#e4e5e9}
  header{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:27px 30px 23px;border-bottom:1px solid #ffffff10;background:#0b0e1266}header small{font-size:.58rem;letter-spacing:.18em;color:#939aa9;font-weight:700}h2{font-size:clamp(1.3rem,2.2vw,1.8rem);letter-spacing:-.035em;margin:7px 0 0;line-height:1.2}button{font:inherit;cursor:pointer}.refresh{display:grid;place-items:center;width:35px;height:35px;border:1px solid #ffffff20;border-radius:5px;background:#ffffff05;color:#bec5d4}.refresh:hover{color:var(--primary);border-color:var(--primary)}button:disabled{opacity:.5;cursor:wait}
  .news-layout{display:grid;grid-template-columns:minmax(0,1fr) 225px}.editorial{min-width:0;padding:0 30px}nav{display:flex;gap:22px;border-bottom:1px solid #ffffff12;flex-wrap:wrap}nav button{border:0;border-bottom:2px solid transparent;border-radius:0;padding:17px 0 13px;background:none;color:#999fae;font-size:.74rem;text-align:left}nav button:hover,nav button.active{color:#ececf4}nav button.active{border-bottom-color:var(--primary)}
  .article-scroll{max-height:450px;overflow-y:auto;scrollbar-width:thin;scrollbar-color:#555966 #191b21;padding-right:14px;overscroll-behavior:contain}.news-article{padding:25px 0 26px;border-bottom:1px solid #ffffff10}.news-article:last-child{border-bottom:0}.article-meta{display:flex;align-items:center;gap:12px;flex-wrap:wrap;font-size:.62rem;color:#9299a6;margin-bottom:10px}.article-meta>span{color:#a6adb9;text-transform:uppercase;font-size:.58rem;letter-spacing:.1em}.news-article h3{font-size:1.12rem;line-height:1.4;margin:0 0 11px;font-weight:700}.news-article p{font-size:.81rem;color:#b7bdc8;line-height:1.85;margin:0;overflow-wrap:anywhere}.source-note,.muted{font-size:.65rem;color:#858e9e;line-height:1.7}.muted{padding:20px 0}.source-note{padding:14px 0;margin:0;border-top:1px solid #ffffff10}.error{color:#f3b0b5;font-size:.78rem;overflow-wrap:anywhere}
  aside{border-left:1px solid #ffffff12;background:#0a0d124d;padding:23px 23px 20px;display:flex;flex-direction:column;gap:25px;min-width:0}.link-section h3{font-size:.86rem;margin:0 0 13px;color:#dadde4}.link-section a{display:flex;align-items:center;justify-content:space-between;gap:8px;color:#acb0d6;font-size:.72rem;line-height:1.5;margin-top:10px;text-underline-offset:3px}.link-section a:hover{color:#e0dfff}.link-section :global(svg){flex-shrink:0;opacity:.65}.archive-link{display:flex;align-items:center;gap:10px;padding:13px 11px;background:#ffffff04;border:1px solid #ffffff14;border-radius:4px;text-decoration:none;color:#bfc3df;font-size:.71rem}.archive-link>span{flex:1}.archive-link small{display:block;color:#818998;font-size:.6rem;margin-top:4px}.archive-link:hover{border-color:var(--primary)}.edition-mark{padding-top:20px;margin-top:auto;border-top:1px solid #ffffff12}.edition-mark>span{font-size:1.1rem;font-weight:900;letter-spacing:.03em;color:#a0a6b2}.edition-mark>small{display:block;font-size:.48rem;letter-spacing:.1em;color:#6f7888;margin-top:7px}a:focus-visible,button:focus-visible{outline:2px solid var(--primary);outline-offset:4px}
  .news-hub.news-mode{border:1px solid rgba(var(--primary-rgb),.2);border-radius:18px;padding:22px;background:rgba(var(--bg-base-rgb),.52);backdrop-filter:blur(12px);box-shadow:none}
  .news-mode header{padding:0;margin-bottom:17px;border:0;background:none}.news-mode header small{color:var(--primary)}.news-mode h2{font-size:1.2rem;letter-spacing:normal}.news-mode .news-layout{display:block}.news-mode .editorial{padding:0}.news-mode nav{gap:7px;border:0;margin-bottom:18px}.news-mode nav button{border:1px solid #abb5dd24;border-radius:9px;padding:9px 12px;background:#98a6e10b;color:#dbe3fa}.news-mode nav button.active{border-color:var(--primary);color:var(--primary)}.news-mode .news-content{max-height:none;overflow:visible;padding:0}
  .news-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(230px,100%),1fr));gap:14px;max-height:490px;overflow:auto;scrollbar-width:thin;scrollbar-color:#555966 transparent}.news-card{border:1px solid #abb5dd24;border-radius:9px;background:#98a6e10b;color:#dbe3fa;overflow:hidden;text-align:left;display:flex;flex-direction:column;min-width:0;text-decoration:none}.news-card:hover{border-color:var(--primary);background:rgba(var(--primary-rgb),.1)}.news-card>div{padding:15px;display:flex;flex-direction:column;gap:9px;flex:1}.news-card small{font-size:.62rem;color:#a7b5d1}.news-card h3{font-size:.95rem;margin:0;line-height:1.4}.news-card p{font-size:.76rem;line-height:1.55;color:#b1bfd5;margin:0;display:-webkit-box;-webkit-line-clamp:3;line-clamp:3;-webkit-box-orient:vertical;overflow:hidden}.news-card span{display:flex;align-items:center;gap:5px;font-size:.68rem;color:var(--primary);margin-top:auto}.news-mode .source-note{border:0;padding:12px 0 0}footer{display:flex;gap:7px;flex-wrap:wrap;padding-top:18px;margin-top:18px;border-top:1px solid #a3aee01c}footer a{display:flex;align-items:center;gap:6px;font-size:.67rem;text-decoration:none;border:1px solid #abb5dd24;background:#98a6e10b;color:#dbe3fa;border-radius:9px;padding:9px 12px}footer a:hover{border-color:var(--primary);color:var(--primary)}
  @media(max-width:850px){.news-layout{grid-template-columns:minmax(0,1fr) 190px}.editorial{padding:0 20px}aside{padding:22px 17px}header{padding:23px 20px}}
  @media(max-width:600px){.news-hub.news-mode{padding:16px}.news-layout{grid-template-columns:1fr}aside{border-left:0;border-top:1px solid #ffffff12;display:grid;grid-template-columns:1fr 1fr;gap:22px}.edition-mark{margin:0}.article-scroll{max-height:440px}nav{gap:14px}nav button{font-size:.68rem}.news-article h3{font-size:1rem}}
  @media(prefers-reduced-motion:no-preference){a,button{transition:color .15s,border-color .15s,background .15s}}
</style>
