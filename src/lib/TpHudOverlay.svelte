<script lang="ts">
 import {onMount} from 'svelte';
 import {invoke} from '@tauri-apps/api/core';
 import {listen} from '@tauri-apps/api/event';
 import HudTitleMenu from './HudTitleMenu.svelte';
 import HudWidgets from './HudWidgets.svelte';
 import CrosshairOverlay from './CrosshairOverlay.svelte';
 import HudInGameMenu from './HudInGameMenu.svelte';
 import {defaultHudProfile,normalizeHudProfile,type HudProfile,type PluginWorkspace} from './launcher_plugins';
 import type {OverlaySnapshot} from './launcher_extensions';
 let profile=$state<HudProfile>(defaultHudProfile()),frame=$state<OverlaySnapshot|null>(null),message=$state(''),saving=$state(false),dirty=$state(false);
 let layoutMode=$state(false);
 let timer:ReturnType<typeof setTimeout>|undefined,revision=0,alive=true;
 function changed(){dirty=true;revision++;if(timer)clearTimeout(timer);timer=setTimeout(()=>void persist(),180);}
 async function persist(){const current=revision;saving=true;message='';try{const next=await invoke<HudProfile>('save_launcher_plugins',{profile:$state.snapshot(profile)});if(alive&&current===revision){profile=normalizeHudProfile(next);dirty=false;}}catch{if(alive)message='No se pudo guardar el diseño. Reintenta antes de salir.';}finally{if(alive)saving=false;}}
 async function close(){if(timer)clearTimeout(timer);if(dirty){await persist();if(dirty)return;}try{await invoke('set_hud_interactive',{editing:false});}catch(error){message=String(error);}}
 onMount(()=>{alive=true;const stops:(()=>void)[]=[];let disposed=false;
  function watch<T>(name:string,handler:(payload:T)=>void){void listen<T>(name,event=>{if(alive)handler(event.payload);}).then(stop=>{if(disposed)stop();else stops.push(stop);}).catch(()=>{if(alive)message='No se pudo conectar el HUD. Vuelve a activarlo desde Plugins.';});}
  watch<OverlaySnapshot>('tp-hud-frame',value=>{if(!value.editing)layoutMode=false;frame=value;});watch<HudProfile>('tp-hud-profile',value=>{if(!dirty)profile=normalizeHudProfile(value);});
  void invoke<PluginWorkspace>('get_launcher_plugins').then(value=>{if(alive&&!dirty)profile=normalizeHudProfile(value.profile);}).catch(()=>{if(alive)message='No se pudo leer el perfil del HUD.';});void invoke<OverlaySnapshot>('get_overlay_snapshot').then(value=>{if(alive)frame=value;}).catch(()=>{});
  return()=>{alive=false;disposed=true;for(const stop of stops)stop();if(timer)clearTimeout(timer);};
 });
</script>
<svelte:head><title>TP LAUNCHER · HUD universal</title></svelte:head>
<div class="overlay-root">
 {#if frame?.focused&&profile.title_theme&&frame.telemetry?.labels?.screen==='title'&&!frame.editing}<HudTitleMenu bind:profile {frame} onchange={changed}/>{/if}
 {#if frame?.focused}<CrosshairOverlay {profile} {frame}/><HudWidgets {profile} {frame} editing={!!frame?.editing&&layoutMode} onchange={changed}/>{/if}
 {#if frame?.editing&&layoutMode}<div class="layout-tools"><button onclick={()=>layoutMode=false}>Volver al menú</button><span>Arrastra las cajas · Flechas para afinar · Se guarda al soltar</span><button onclick={()=>void close()}>Volver al juego</button></div>{/if}
 {#if frame?.editing&&!layoutMode}<div class="editor-layer"><HudInGameMenu bind:profile onchange={changed} onlayout={()=>layoutMode=true} onclose={()=>void close()} {saving} {message} fpsStatus={frame.fps_source} {frame}/></div>{/if}
</div>
<style>
.layout-tools{position:absolute;top:14px;left:50%;transform:translateX(-50%);display:flex;gap:12px;align-items:center;padding:10px;background:#0b0b0fd9;border:1px solid #a855f755;color:#e6d8f2;font:12px Segoe UI;pointer-events:auto;z-index:4;white-space:nowrap}.layout-tools button{background:#a855f726;color:inherit;border:1px solid #a855f755;padding:9px;cursor:pointer}

 :global(html),:global(body){margin:0!important;background:transparent!important;overflow:hidden!important}:global(body){min-width:0!important}.overlay-root{position:fixed;inset:0;background:transparent;pointer-events:none}.editor-layer{position:absolute;inset:0;display:grid;place-items:center;pointer-events:auto;background:transparent}
</style>
