<script lang="ts">
  import {onMount} from 'svelte';
  import {getCurrentWindow} from '@tauri-apps/api/window';
  import UiIcon from './UiIcon.svelte';
  import {t} from './i18n';
  let maximized=$state(false),error=$state('');
  let window:ReturnType<typeof getCurrentWindow>|null=null;
  onMount(()=>{
    if(!('__TAURI_INTERNALS__' in globalThis))return;
    window=getCurrentWindow();
    let disposed=false,unlisten:(()=>void)|undefined;
    const refresh=()=>void window?.isMaximized().then(value=>{if(!disposed)maximized=value;}).catch(()=>{});
    refresh();
    void window.onResized(refresh).then(stop=>{if(disposed)stop();else unlisten=stop;});
    return()=>{disposed=true;unlisten?.();};
  });
  async function action(kind:'minimize'|'maximize'|'close'){
    try{if(kind==='close')await window?.close();else if(kind==='minimize')await window?.minimize();else await window?.toggleMaximize();}
    catch{error=$t('No se pudo cambiar el estado de la ventana.');}
  }
</script>
<header class="window-chrome" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region><img src="/logos/tp-launcher.svg" alt="" draggable="false" data-tauri-drag-region/><span data-tauri-drag-region>TP LAUNCHER</span></div>
  {#if error}<span class="window-error" role="status">{error}</span>{/if}
  <nav aria-label={$t('Controles de ventana')}><button aria-label={$t('Minimizar')} title={$t('Minimizar')} onclick={()=>void action('minimize')}><UiIcon name="minus" size={16}/></button><button aria-label={$t(maximized?'Restaurar ventana':'Maximizar')} title={$t(maximized?'Restaurar ventana':'Maximizar')} onclick={()=>void action('maximize')}><UiIcon name={maximized?'restore':'maximize'} size={15}/></button><button class="close" aria-label={$t('Cerrar')} title={$t('Cerrar')} onclick={()=>void action('close')}><UiIcon name="close" size={17}/></button></nav>
</header>
<style>.window-chrome{box-sizing:border-box;position:fixed;inset:0 0 auto;height:38px;z-index:30000;display:flex;align-items:center;justify-content:space-between;padding-left:14px;background:#100c19;border-bottom:1px solid #a855f72b;color:#d3c6e7;user-select:none}.brand{display:flex;align-items:center;gap:9px;font:700 10px/1.1 'Segoe UI',sans-serif;letter-spacing:.09em;height:100%;flex:1}.brand img{height:21px;width:21px}.window-chrome nav{display:flex;height:100%;align-items:stretch}.window-chrome button{display:grid;place-items:center;width:47px;border:0;background:transparent;color:#b6a8ce;padding:0;cursor:pointer}.window-chrome button:hover{background:#a855f724;color:#f0e5ff}.window-chrome button.close:hover{background:#8e285e;color:#fff}.window-chrome button:focus-visible{outline:2px solid #c084fc;outline-offset:-3px}.window-error{font:11px 'Segoe UI',sans-serif;padding:0 10px;max-width:40%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}</style>
