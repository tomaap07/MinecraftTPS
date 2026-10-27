<script lang="ts">
  import DynamicText from "./DynamicText.svelte";
  import {onMount} from 'svelte';
  import {invoke} from '@tauri-apps/api/core';
  import UiIcon from './UiIcon.svelte';
  import ProviderLogo from './ProviderLogo.svelte';
  import LoaderLogo from './LoaderLogo.svelte';
  import ContentProgress from './ContentProgress.svelte';
  import {compatibleTargets,newestVersions,installRequest,type InstallProject,type ContentVersion,type InstallTarget} from './content_install';
  let {project,username,preferredTarget='',requestedFile='',game='all',loader='all',onclose,oninstalled}:{project:InstallProject;username:string;preferredTarget?:string;requestedFile?:string;game?:string;loader?:string;onclose:()=>void;oninstalled:(message:string)=>void}=$props();
  let dialog=$state<HTMLDialogElement>();
  let versions=$state<ContentVersion[]>([]),targets=$state<InstallTarget[]>([]),selected=$state('');
  let loading=$state(true),installing=$state(false),message=$state(''),error=$state(''),artFailed=$state(false);
  let alive=true;
  const isPack=$derived(project.project_type==='modpack');
  const choices=$derived(compatibleTargets(versions,targets,project.project_type,requestedFile,game,isPack||project.project_type!=='mod'?'all':loader));
  const choice=$derived(choices.find(row=>row.target.id===selected));
  $effect(()=>{if(dialog&&!dialog.open)dialog.showModal();});
  onMount(()=>{
    void load();
    return()=>{alive=false;};
  });
  async function load(){
    loading=true;error='';
    try{
      const [detail,destinations]=await Promise.all([
        invoke<{project:InstallProject;versions:ContentVersion[]}>('studio_project',{provider:project.provider,projectId:project.id}),
        isPack?Promise.resolve([]):invoke<InstallTarget[]>('studio_install_targets',{username})
      ]);
      if(!alive)return;
      if(detail.project.project_type!==project.project_type)throw new Error('El tipo del proyecto cambió. Actualiza el catálogo antes de instalar.');
      versions=newestVersions(detail.versions);targets=destinations;
      const available=compatibleTargets(versions,targets,project.project_type,requestedFile,game,project.project_type==='mod'?loader:'all');
      selected=available.find(row=>row.target.id===preferredTarget)?.target.id??available[0]?.target.id??'';
      if(isPack){
        const newest=versions.find(version=>!requestedFile||version.id===requestedFile);
        if(!newest)throw new Error('El proyecto no tiene archivos disponibles para instalar.');
        await install(newest);
      }
    }catch(reason){if(alive)error=String(reason);}finally{if(alive)loading=false;}
  }
  async function install(version=choice?.version){
    if(!version||installing||(!isPack&&!choice))return;
    installing=true;error='';
    const target=choice?.target;
    try{
      const result=await invoke<string>('studio_install_version',installRequest(project,version,target,username));
      if(alive){message=result;oninstalled(result);}
    }catch(reason){if(alive){error=String(reason);oninstalled('');}}
    finally{if(alive)installing=false;}
  }
  function close(){if(!installing){dialog?.close();onclose();}}
</script>

<dialog bind:this={dialog} class="install-dialog" aria-labelledby="content-install-title" oncancel={event=>{event.preventDefault();close();}}>
  <header><span><UiIcon name="download" size={18}/>{isPack?'Instalar modpack':'Elegir instalación'}</span><button class="icon-button" aria-label="Cerrar" disabled={installing} onclick={close}><UiIcon name="close" size={20}/></button></header>
  <section class="project"><div class="art">{#if project.icon_url&&!artFailed}<img src={project.icon_url} alt={`Imagen de ${project.title}`} onerror={()=>artFailed=true}/>{:else}<UiIcon name="layers" size={40}/>{/if}</div><div><div class="provider"><ProviderLogo provider={project.provider} size={17}/>{project.provider==='curseforge'?'CurseForge':'Modrinth'}</div><h2 id="content-install-title">{project.title}</h2><p><DynamicText text={project.description}/></p></div></section>
  {#if error}<p class="notice error" role="alert">{error}</p>{/if}
  {#if installing}<ContentProgress/><p class="status" role="status">Instalando el contenido y comprobando sus dependencias…</p>
  {:else if loading}<p class="status" role="status"><UiIcon name="refresh" size={18}/>Comprobando archivos e instalaciones compatibles…</p>
  {:else if message}<p class="notice success" role="status"><UiIcon name="check" size={19}/>{message}</p>
  {:else if !isPack}
    <div class="destination"><h3>¿Dónde quieres instalarlo?</h3><p>Solo aparecen instalaciones y modpacks descargados compatibles con este contenido. Se elegirá el archivo más reciente para cada destino.</p>
    {#if choices.length}<label>Instalación compatible<select bind:value={selected}>{#each choices as row}<option value={row.target.id}>{row.target.name} · Minecraft {row.target.game_version} · {row.target.loader}{row.target.pack_id?' · Modpack':''}</option>{/each}</select></label>
    {#if choice}<div class="selected-file"><LoaderLogo loader={choice.target.loader} size={24}/><span><strong>{choice.version.name||choice.version.number}</strong><small>{choice.version.filename}</small><small>Minecraft {choice.target.game_version} · {choice.version.release_type}</small></span></div>{/if}
    {:else}<div class="empty"><UiIcon name="info" size={25}/><strong>No hay instalaciones compatibles</strong><p>{project.project_type==='shader'?'Instala Iris, Oculus u OptiFine compatible en una de tus instalaciones antes de añadir este shader.':'Instala una versión de Minecraft y el cargador indicado por el autor. Los modpacks pendientes no aparecen hasta terminar su instalación.'}</p></div>{/if}</div>
  {/if}
  <footer><button disabled={installing} onclick={close}>{message?'Listo':'Cerrar'}</button>{#if !isPack&&!message}<button class="primary" disabled={loading||installing||!choice} onclick={()=>void install()}><UiIcon name="download" size={17}/>Instalar</button>{:else if isPack&&!loading&&!installing&&error}<button class="primary" onclick={()=>void load()}><UiIcon name="refresh" size={17}/>Reintentar</button>{/if}</footer>
</dialog>

<style>
  .install-dialog{box-sizing:border-box;width:min(680px,94vw);max-height:90vh;overflow:auto;padding:0;border:1px solid #ffffff26;border-radius:14px;background:#121019;color:#edf0f5;box-shadow:0 30px 110px #000b;font:inherit;color-scheme:dark}.install-dialog::backdrop{background:#030208b8;backdrop-filter:blur(7px)}header,footer{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:18px 24px;border-bottom:1px solid #ffffff14}header>span,.provider,.status,.success{display:flex;align-items:center;gap:9px}header>span{font-size:.83rem;font-weight:700;color:#c5cbd6}footer{border-top:1px solid #ffffff14;border-bottom:0;justify-content:flex-end}.project{display:flex;align-items:flex-start;gap:18px;padding:26px 24px}.project>div:last-child{min-width:0}.art{display:grid;place-items:center;flex:0 0 80px;width:80px;height:80px;overflow:hidden;border-radius:12px;background:#ffffff08;border:1px solid #ffffff16;color:#b7bfce}.art img{width:100%;height:100%;object-fit:contain}.provider{font-size:.7rem;color:#aeb8ca}h2{font-size:1.4rem;line-height:1.3;margin:9px 0;overflow-wrap:anywhere}p{font-size:.8rem;line-height:1.6;color:#aeb6c5;margin:0}.destination{padding:0 24px 24px}h3{font-size:1rem;margin:0 0 8px}label{display:grid;gap:8px;font-size:.75rem;color:#c2cbda;margin:20px 0 14px}select{width:100%;box-sizing:border-box;font:inherit;padding:12px;border-radius:7px;border:1px solid #ffffff28;color:#f4f6fb;background:#1b1824;min-width:0}.selected-file{display:flex;align-items:center;gap:14px;background:#ffffff05;padding:15px;border:1px solid #ffffff12;border-radius:8px}.selected-file>span{display:grid;gap:5px;min-width:0}.selected-file strong{font-size:.8rem}.selected-file small{font-size:.7rem;color:#9da9bd;overflow-wrap:anywhere}.empty{display:grid;gap:12px;justify-items:center;text-align:center;padding:28px 20px;margin-top:20px;border:1px dashed #ffffff24;border-radius:8px}.empty strong{font-size:.88rem}.notice,.status{margin:0 24px 22px;padding:14px 16px;font-size:.8rem;border-radius:8px}.error{color:#ffc0c6;background:#db4d6814;border:1px solid #db4d6845;overflow-wrap:anywhere}.success{color:#b9e7ce;background:#58b98a12;border:1px solid #58b98a40}.status{background:#ffffff05;color:#b4bfce}button{display:inline-flex;align-items:center;justify-content:center;gap:8px;padding:10px 16px;font:inherit;font-size:.8rem;border:1px solid #ffffff25;border-radius:7px;background:#ffffff07;color:#e8edf5;cursor:pointer}button:hover:enabled{background:#ffffff12}button.primary{background:var(--primary,#a855f7);color:#10091d;font-weight:750;border-color:transparent}button:disabled{opacity:.45;cursor:default}.icon-button{padding:5px;border:0;background:none}button:focus-visible,select:focus-visible{outline:2px solid var(--primary,#a855f7);outline-offset:3px}@media(max-width:480px){header,footer,.project{padding:17px}.destination{padding:0 17px 17px}.art{width:58px;height:58px;flex-basis:58px}.project{gap:12px}h2{font-size:1.12rem}.notice,.status{margin-left:17px;margin-right:17px}}
</style>
