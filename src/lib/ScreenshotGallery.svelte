<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import UiIcon from "./UiIcon.svelte";

  type Shot = { id:string; name:string; instance:string; modified:number; thumbnail:string };
  type ShotPage = { items:Shot[]; total:number; page:number; pages:number };
  let shots = $state<Shot[]>([]);
  let total = $state(0);
  let page = $state(1);
  let pages = $state(1);
  let loading = $state(true);
  let refreshing = $state(false);
  let error = $state("");
  let selected = $state<Shot|null>(null);
  let preview = $state("");
  let previewLoading = $state(false);
  let previewError = $state("");
  let dialog = $state<HTMLDialogElement>();
  let previewRequest = 0;
  let disposed = false;

  const date = (value:number) => new Date(value*1000).toLocaleString();
  async function refresh() {
    if(refreshing)return;
    refreshing=true;
    try {
      const result=await invoke<ShotPage>("screenshots_page",{page});
      if(disposed)return;
      shots=result.items;total=result.total;page=result.page;pages=result.pages;error="";
    }catch(reason){if(!disposed)error=String(reason);}
    finally{if(!disposed){loading=false;refreshing=false;}}
  }
  async function showPreview(shot:Shot) {
    const request=++previewRequest;
    selected=shot;preview="";previewError="";previewLoading=true;
    dialog?.showModal();
    try {
      const image=await invoke<string>("screenshot_data",{id:shot.id});
      if(!disposed&&request===previewRequest)preview=image;
    }catch(reason){if(!disposed&&request===previewRequest)previewError=String(reason);}
    finally{if(!disposed&&request===previewRequest)previewLoading=false;}
  }
  function closePreview(){dialog?.close();previewRequest++;selected=null;preview="";previewLoading=false;}
  async function openOriginal(){if(!selected)return;try{await invoke("open_screenshot",{id:selected.id});}catch(reason){previewError=String(reason);}}
  onMount(()=>{
    void refresh();
    const timer=setInterval(()=>{if(!document.hidden)void refresh();},15000);
    return()=>{disposed=true;previewRequest++;clearInterval(timer);};
  });
</script>

<section class="capture-gallery">
  <header class="gallery-heading"><div><span class="eyebrow">TUS AVENTURAS EN IMÁGENES</span><h2>Capturas</h2><p>Las imágenes que tomas con <kbd>F2</kbd> en Minecraft aparecen aquí automáticamente.</p></div><button onclick={()=>void refresh()} disabled={refreshing}><UiIcon name="refresh" size={17}/>{refreshing?"Actualizando…":"Actualizar"}</button></header>
  <div class="gallery-meta"><span>{total} capturas · Todas tus instalaciones</span><small>Los archivos originales se conservan en la carpeta screenshots de cada instalación.</small></div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if loading}<div class="empty" role="status"><UiIcon name="image" size={36}/><p>Preparando galería…</p></div>
  {:else if !total}<div class="empty"><UiIcon name="image" size={44}/><h3>Tu próxima aventura merece una captura</h3><p>Presiona F2 dentro del juego. Aquí verás su imagen, fecha e instalación.</p></div>
  {:else}
    <div class="capture-grid">{#each shots as shot (shot.id)}<button class="capture-card" onclick={()=>void showPreview(shot)} aria-label={`Ver captura ${shot.name}`}><div class="thumbnail"><img src={shot.thumbnail} alt={shot.name} loading="lazy"/><span><UiIcon name="image" size={15}/>Vista previa</span></div><div class="caption"><strong data-no-translate>{shot.name}</strong><time>{date(shot.modified)}</time><small data-no-translate>{shot.instance||"Carpeta principal"}</small></div></button>{/each}</div>
    {#if pages>1}<nav class="pagination" aria-label="Páginas de capturas"><button disabled={page<=1||refreshing} onclick={()=>{page--;void refresh();}}><UiIcon name="left" size={15}/>Anterior</button><span>Página {page} de {pages}</span><button disabled={page>=pages||refreshing} onclick={()=>{page++;void refresh();}}>Siguiente<UiIcon name="arrow" size={15}/></button></nav>{/if}
  {/if}
</section>

<dialog bind:this={dialog} class="preview-dialog" oncancel={closePreview} onclose={()=>{if(selected)closePreview();}} aria-label="Vista previa de la captura">
  <header><div><h3 data-no-translate>{selected?.name}</h3>{#if selected}<small>{date(selected.modified)}</small>{/if}</div><button onclick={closePreview} aria-label="Cerrar vista previa"><UiIcon name="close"/></button></header>
  <div class="full-preview">{#if previewLoading}<p role="status">Cargando imagen original…</p>{:else if preview}<img src={preview} alt={selected?.name||"Captura de Minecraft"}/>{/if}</div>
  {#if previewError}<p class="error" role="alert">{previewError}</p>{/if}
  <footer><small data-no-translate>{selected?.instance}</small><button onclick={()=>void openOriginal()}><UiIcon name="image" size={16}/>Abrir imagen original</button></footer>
</dialog>

<style>
  .capture-gallery{color:#edf0ff;min-width:0}.gallery-heading{display:flex;align-items:center;justify-content:space-between;gap:20px;margin-bottom:24px}.eyebrow{font-size:.6rem;font-weight:700;letter-spacing:.15em;color:var(--primary);display:block;margin-bottom:9px}h2{font-size:2rem;letter-spacing:-.035em;margin:0 0 10px}p{color:#a9b5cb;font-size:.84rem;line-height:1.7;margin:0}kbd{font:inherit;color:#e4e7fb;border:1px solid #ffffff25;border-radius:4px;padding:2px 6px;background:#ffffff09}button{font:inherit;cursor:pointer;border:1px solid #ffffff20;color:#e1e5f6;background:#171c31b3;border-radius:10px;padding:10px 14px;display:inline-flex;align-items:center;justify-content:center;gap:8px}button:hover{border-color:var(--primary);background:rgba(var(--primary-rgb),.12)}button:disabled{opacity:.5;cursor:default}button:focus-visible{outline:2px solid var(--primary);outline-offset:3px}.gallery-heading>button{flex-shrink:0}.gallery-meta{display:flex;justify-content:space-between;gap:15px;flex-wrap:wrap;padding:17px 0;border-top:1px solid #ffffff15;margin-bottom:12px}.gallery-meta>span{font-size:.79rem;color:#c6cde0}small,time{font-size:.69rem;color:#95a2bd}.capture-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(260px,100%),1fr));gap:19px}.capture-card{padding:0;display:flex;flex-direction:column;overflow:hidden;text-align:left;background:rgba(var(--bg-base-rgb),.65);transition:border-color .18s}.thumbnail{position:relative;width:100%;aspect-ratio:16/9;overflow:hidden;background:#101522}.thumbnail img{width:100%;height:100%;object-fit:cover;display:block}.thumbnail>span{display:inline-flex;align-items:center;gap:6px;position:absolute;right:10px;bottom:10px;background:#0a0f22c9;color:#e5e9fa;border:1px solid #ffffff25;border-radius:6px;padding:6px 8px;font-size:.61rem}.caption{align-self:stretch;padding:15px;display:grid;gap:8px}.caption strong{font-size:.79rem;overflow-wrap:anywhere}.caption small{overflow-wrap:anywhere;color:#7f90aa}.empty{display:grid;place-items:center;gap:17px;text-align:center;padding:80px 20px;border:1px dashed #ffffff24;border-radius:16px;background:#10162966}.empty h3{font-size:1.1rem;margin:0}.empty :global(svg){color:var(--primary)}.error{padding:12px 16px;background:#f07f8b0d;border:1px solid #f07f8b30;color:#f0b2b9;border-radius:9px;overflow-wrap:anywhere;margin:14px 0}.pagination{display:flex;align-items:center;justify-content:center;gap:20px;margin-top:26px}.pagination span{font-size:.76rem;color:#a6b3cd}.preview-dialog{box-sizing:border-box;width:min(1200px,94vw);max-height:94vh;overflow:auto;border:1px solid rgba(var(--primary-rgb),.35);background:#0c1120;color:#edf0ff;border-radius:18px;padding:20px;box-shadow:0 30px 100px #0009}.preview-dialog::backdrop{background:#03060cd9;backdrop-filter:blur(8px)}.preview-dialog header,.preview-dialog footer{display:flex;align-items:center;justify-content:space-between;gap:15px}.preview-dialog h3{font-size:.95rem;margin:0 0 5px;overflow-wrap:anywhere}.preview-dialog header>div{min-width:0}.preview-dialog header>button{flex-shrink:0}.full-preview{display:grid;place-items:center;min-height:160px;margin:18px 0;background:#070c16;border-radius:9px;overflow:hidden}.full-preview img{display:block;max-width:100%;max-height:67vh;object-fit:contain}.preview-dialog footer{flex-wrap:wrap}.preview-dialog footer small{overflow-wrap:anywhere}@media(max-width:650px){.gallery-heading{align-items:flex-start;flex-direction:column}h2{font-size:1.6rem}.preview-dialog{padding:14px}}
</style>
