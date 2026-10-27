<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import UiIcon from './UiIcon.svelte';
  import GameIcon from './GameIcon.svelte';
  import { t } from './i18n';
  import type { LauncherSettings, Result } from './settings.types';

  type DisplayInfo={width:number;height:number;work_width:number;work_height:number;scale_factor:number;maximized:boolean};
  let {settings,onchange}:{settings:LauncherSettings;onchange:(patch:Partial<LauncherSettings>)=>Result}=$props();
  let scale=$state(100),windowWidth=$state(1280),windowHeight=$state(820);
  let info=$state<DisplayInfo|null>(null),desktop=$state(true),savingScale=$state(false),windowBusy=$state(false);
  let message=$state(''),failed=$state(false);
  let pendingScale:number|null=null;
  let timer:ReturnType<typeof setTimeout>|undefined;
  let resizeTimer:ReturnType<typeof setTimeout>|undefined;
  let savedWidth:number|undefined,savedHeight:number|undefined;
  let infoSequence=0;
  let disposed=false;
  const scalePresets=[100,115,125,150,175,200];
  const windowPresets=[{width:1100,height:720},{width:1280,height:820},{width:1440,height:900},{width:1600,height:1000}];
  const boundedScale=(value:number)=>Math.max(80,Math.min(200,Math.round(value/5)*5));

  $effect(()=>{
    const saved=settings.ui_scale_percent??100;
    untrack(()=>{if(!savingScale&&pendingScale===null)scale=saved;});
  });
  $effect(()=>{
    const width=settings.launcher_width??1280,height=settings.launcher_height??820;
    untrack(()=>{if(width!==savedWidth)windowWidth=width;if(height!==savedHeight)windowHeight=height;savedWidth=width;savedHeight=height;});
  });

  function chooseScale(value:number,immediate=false){
    if(!Number.isFinite(value))return;
    scale=boundedScale(value);pendingScale=scale;clearTimeout(timer);failed=false;message='';
    if(immediate)void flushScale();else timer=setTimeout(()=>void flushScale(),180);
  }
  async function flushScale(){
    clearTimeout(timer);
    if(savingScale||pendingScale===null)return;
    savingScale=true;
    while(pendingScale!==null){
      const percent=pendingScale;pendingScale=null;
      const saved=settings.ui_scale_percent??100;
      try{
        if(!isTauri())throw new Error($t('Abre el launcher de escritorio para ajustar su interfaz.'));
        await invoke('preview_launcher_scale',{percent});
        await onchange({ui_scale_percent:percent});
        if(!disposed&&pendingScale===null){failed=false;message=$t('Tamaño de interfaz guardado.');}
      }catch(problem){

        if(pendingScale===null){
          if(isTauri())try{await invoke('preview_launcher_scale',{percent:settings.ui_scale_percent??saved});}catch{ }
          if(!disposed){scale=settings.ui_scale_percent??saved;failed=true;message=String(problem);}
        }
      }
    }
    if(!disposed)savingScale=false;
  }
  async function readDisplay(){
    if(!isTauri()){desktop=false;return null;}
    const sequence=++infoSequence;
    const next=await invoke<DisplayInfo>('get_launcher_display_info');
    if(!disposed&&sequence===infoSequence)info=next;
    return next;
  }
  async function applyWindow(width:number,height:number,maximized=false){
    if(windowBusy)return;
    const requestedWidth=Math.round(Number(width)),requestedHeight=Math.round(Number(height));
    if(!Number.isFinite(requestedWidth)||!Number.isFinite(requestedHeight)||requestedWidth<320||requestedHeight<240||requestedWidth>7680||requestedHeight>4320){
      failed=true;message=$t('El ancho debe estar entre 320 y 7680 y el alto entre 240 y 4320.');return;
    }
    windowBusy=true;failed=false;message='';
    try{
      if(!isTauri())throw new Error($t('El tamaño de ventana se ajusta desde la aplicación de escritorio.'));
      const next=await invoke<DisplayInfo>('apply_launcher_window',{width:requestedWidth,height:requestedHeight,maximized});
      await onchange({launcher_width:maximized?(settings.launcher_width??1280):next.width,
        launcher_height:maximized?(settings.launcher_height??820):next.height,launcher_maximized:maximized});
      if(!disposed){infoSequence++;info=next;windowWidth=settings.launcher_width;windowHeight=settings.launcher_height;
        message=maximized?$t('El launcher se abrirá maximizado.'):$t('Tamaño de ventana guardado.');}
    }catch(problem){if(!disposed){failed=true;message=String(problem);}}
    finally{if(!disposed)windowBusy=false;}
  }
  async function fitWindow(){
    if(windowBusy)return;
    windowBusy=true;
    try{const current=await readDisplay();windowBusy=false;if(current)await applyWindow(Math.min(7680,current.work_width),Math.min(4320,current.work_height));}
    catch(problem){if(!disposed){failed=true;message=String(problem);}}
    finally{if(!disposed)windowBusy=false;}
  }
  onMount(()=>{
    void readDisplay().catch(()=>{if(!disposed){failed=true;message=$t('No se pudo consultar el tamaño de la ventana.');}});
    const resized=()=>{clearTimeout(resizeTimer);resizeTimer=setTimeout(()=>void readDisplay().catch(()=>{}),250);};
    window.addEventListener('resize',resized);
    return()=>{window.removeEventListener('resize',resized);clearTimeout(resizeTimer);};
  });
  onDestroy(()=>{disposed=true;infoSequence++;clearTimeout(timer);clearTimeout(resizeTimer);void flushScale();});
</script>

<div class="launcher-display-settings">
  <section class="surface interface-size" aria-labelledby="launcher-scale-title">
    <header class="display-heading"><span class="display-icon"><UiIcon name="sliders" size={24}/></span><div><span class="eyebrow">TP LAUNCHER · {$t('PANTALLA')}</span><h3 id="launcher-scale-title">{$t('Tamaño de la interfaz')}</h3><p>{$t('Aumenta los textos, botones, iconos y menús de todo el launcher.')}</p></div><output class="scale-value" for="launcher-ui-scale">{scale}%</output></header>
    <div class="scale-controls"><button class="square-control" aria-label={$t('Reducir tamaño de interfaz')} disabled={scale<=80} onclick={()=>chooseScale(scale-5,true)}><UiIcon name="minus" size={18}/></button><label class="range-control" for="launcher-ui-scale"><span class="sr-only">{$t('Tamaño de la interfaz')}</span><input id="launcher-ui-scale" type="range" min="80" max="200" step="5" value={scale} oninput={event=>chooseScale(Number(event.currentTarget.value))} onchange={()=>void flushScale()}/><span class="range-labels"><small>80%</small><small>{$t('Más grande')}</small><small>200%</small></span></label><button class="square-control" aria-label={$t('Aumentar tamaño de interfaz')} disabled={scale>=200} onclick={()=>chooseScale(scale+5,true)}><UiIcon name="plus" size={18}/></button></div>
    <div class="scale-presets" aria-label={$t('Tamaños de interfaz')}>
      {#each scalePresets as percent}<button class:selected={scale===percent} aria-pressed={scale===percent} onclick={()=>chooseScale(percent,true)}>{percent}%</button>{/each}
      <button class="restore" onclick={()=>chooseScale(100,true)}><UiIcon name="refresh" size={14}/>{$t('Restablecer 100%')}</button>
    </div>
    <div class="scale-help"><UiIcon name="info" size={16}/><p>{$t('Se aplica al instante y se conserva al volver a abrir el launcher.')}</p><small>{savingScale?$t('Guardando…'):$t('Sin reiniciar')}</small></div>
    {#if info}<p class="system-scaling">{$t('Escala de Windows detectada')}: {Math.round(info.scale_factor*100)}%. {$t('Este ajuste se aplica sobre la escala de tu sistema.')}</p>{/if}
  </section>

  <section class="surface launcher-window" aria-labelledby="launcher-window-title">
    <header class="display-heading"><span class="display-icon"><UiIcon name="image" size={23}/></span><div><h3 id="launcher-window-title">{$t('Tamaño de la ventana del launcher')}</h3><p>{$t('Elige cuánto espacio ocupa TP LAUNCHER en tu monitor.')}</p></div></header>
    {#if !desktop}<p class="browser-notice">{$t('Estas opciones están disponibles en la aplicación de escritorio.')}</p>{/if}
    <div class="window-presets" aria-label={$t('Resoluciones del launcher')}>{#each windowPresets as preset}<button class:selected={!settings.launcher_maximized&&settings.launcher_width===preset.width&&settings.launcher_height===preset.height} disabled={windowBusy||!desktop} onclick={()=>void applyWindow(preset.width,preset.height)}>{preset.width} × {preset.height}</button>{/each}</div>
    <form class="custom-size" onsubmit={event=>{event.preventDefault();void applyWindow(windowWidth,windowHeight);}}><label for="launcher-window-width">{$t('Ancho')}<div><input id="launcher-window-width" type="number" min="320" max="7680" step="1" bind:value={windowWidth} disabled={windowBusy||!desktop} required/><span>px</span></div></label><span class="dimensions-separator" aria-hidden="true">×</span><label for="launcher-window-height">{$t('Alto')}<div><input id="launcher-window-height" type="number" min="240" max="4320" step="1" bind:value={windowHeight} disabled={windowBusy||!desktop} required/><span>px</span></div></label><button class="primary" disabled={windowBusy||!desktop}><UiIcon name="check" size={16}/>{windowBusy?$t('Aplicando…'):$t('Aplicar tamaño')}</button></form>
    <div class="window-actions"><button disabled={windowBusy||!desktop} onclick={()=>void fitWindow()}><UiIcon name="grid" size={15}/>{$t('Ajustar a la pantalla')}</button><button class:selected={settings.launcher_maximized} aria-pressed={settings.launcher_maximized} disabled={windowBusy||!desktop} onclick={()=>void applyWindow(settings.launcher_width,settings.launcher_height,true)}><UiIcon name="plus" size={15}/>{$t('Maximizar')}</button></div>
    {#if info}<div class="monitor-info"><span><UiIcon name="image" size={14}/>{$t('Ventana actual')}: <strong>{info.width} × {info.height}</strong>{#if info.maximized}<small>{$t('Maximizada')}</small>{/if}</span><span>{$t('Espacio disponible')}: {info.work_width} × {info.work_height}</span></div>{/if}
    <p class="game-separation"><GameIcon name="grass_block" size={17}/>{$t('La resolución de Minecraft se configura por separado en la pestaña Minecraft.')}</p>
  </section>
  {#if message}<p class="display-status" class:failed role="status">{message}</p>{/if}
</div>

<style>
  .launcher-display-settings{display:grid;gap:17px;min-width:0;color:#e5ecf8}.surface{padding:23px;border:1px solid rgba(var(--primary-rgb,129,140,248),.24);background:rgba(var(--bg-base-rgb,9,14,30),.72);min-width:0;box-shadow:inset 0 2px #ffffff06,0 3px #0003}.display-heading{display:flex;align-items:center;gap:14px;margin-bottom:22px}.display-icon{width:45px;height:45px;display:grid;place-items:center;flex-shrink:0;background:rgba(var(--primary-rgb,129,140,248),.1);border:1px solid rgba(var(--primary-rgb,129,140,248),.25);color:var(--primary,#a5b4fc)}.display-heading>div{flex:1;min-width:0}.eyebrow{font-size:.54rem;letter-spacing:.12em;font-weight:700;color:var(--primary,#a5b4fc)}h3{font-size:1rem;margin:5px 0 8px;line-height:1.6}p{font-size:.77rem;line-height:1.75;color:#a2b2c9;margin:0}.scale-value{font-size:1.6rem;color:var(--primary,#c084fc);font-weight:700;line-height:1.3;white-space:nowrap;margin-left:auto}button{display:inline-flex;align-items:center;justify-content:center;gap:7px;border:1px solid rgba(var(--primary-rgb,129,140,248),.28);background:#ffffff05;color:#dce5f6;font:inherit;font-size:.73rem;padding:10px 13px;cursor:pointer;line-height:1.4}button:hover:not(:disabled){background:rgba(var(--primary-rgb,129,140,248),.13);border-color:var(--primary,#a5b4fc)}button:disabled{opacity:.4;cursor:default}button.selected,.primary{border-color:var(--primary,#a5b4fc);color:var(--primary,#c084fc);background:rgba(var(--primary-rgb,129,140,248),.16)}button:focus-visible,input:focus-visible{outline:2px solid var(--primary,#c084fc);outline-offset:3px}.scale-controls{display:flex;align-items:start;gap:13px}.square-control{width:38px;height:38px;padding:8px;flex-shrink:0}.range-control{flex:1;min-width:0;display:grid;gap:8px;padding-top:4px}input[type=range]{width:100%;accent-color:var(--primary,#c084fc);cursor:pointer;margin:0;min-height:27px}.range-labels{display:flex;align-items:center;justify-content:space-between;color:#8b9fb9;font-size:.65rem}.range-labels small{font-size:inherit}.scale-presets{display:flex;align-items:center;flex-wrap:wrap;gap:7px;margin-top:18px}.scale-presets>button:not(.restore){min-width:58px}.restore{margin-left:auto;font-size:.68rem}.scale-help{display:flex;align-items:start;gap:9px;padding-top:18px;margin-top:18px;border-top:1px solid #ffffff0d}.scale-help :global(svg){color:var(--primary,#c084fc);flex-shrink:0;margin-top:3px}.scale-help p{flex:1;font-size:.7rem}.scale-help>small{font-size:.59rem;white-space:nowrap;color:#93b8a8}.system-scaling{margin-top:12px;font-size:.66rem;color:#8196b2}.window-presets,.window-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap}.window-presets button{font-size:.7rem}.custom-size{display:flex;align-items:end;gap:12px;margin:20px 0;flex-wrap:wrap}.custom-size label{flex:1;min-width:100px;font-size:.7rem;display:grid;gap:8px;color:#aabbd2}.custom-size label>div{display:flex;align-items:center;border:1px solid #ffffff24;background:#090f1f}.custom-size input{width:100%;min-width:0;box-sizing:border-box;font:inherit;font-size:.82rem;border:0;background:none;color:#e4ecf8;padding:11px 12px;color-scheme:dark}.custom-size label>div>span{font-size:.63rem;color:#7f93b0;margin-right:12px}.dimensions-separator{color:#7c8da7;padding-bottom:11px}.custom-size button{min-height:40px}.window-actions button{font-size:.7rem}.monitor-info{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap;margin-top:19px;padding-top:15px;border-top:1px solid #ffffff10;font-size:.64rem;color:#8fa3be}.monitor-info>span:first-child{display:flex;align-items:center;gap:7px;flex-wrap:wrap}.monitor-info strong{font-weight:500;color:#c2d1e6}.monitor-info small{font-size:.58rem;color:var(--primary,#c084fc)}.game-separation{display:flex;align-items:center;gap:8px;padding:12px;margin-top:17px;background:#0002;border:1px solid #ffffff0b;font-size:.68rem}.browser-notice{padding:12px;border:1px solid #ffffff15;margin-bottom:15px}.display-status{font-size:.74rem;border:1px solid #7dbba742;background:#7dbba70b;padding:13px 15px;color:#abd8c9}.display-status.failed{border-color:#cfaa7050;background:#cfaa700a;color:#d5bf99}.sr-only{position:absolute;width:1px;height:1px;padding:0;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0}
  @media(max-width:800px){.surface{padding:17px}.display-heading{gap:10px;flex-wrap:wrap}.display-heading>div{flex-basis:calc(100% - 60px)}.scale-value{margin-left:55px;font-size:1.4rem}.restore{margin-left:0}.scale-help{flex-wrap:wrap}.scale-help>small{margin-left:25px}.custom-size button{width:100%}.monitor-info{display:grid;gap:9px}.window-presets button{flex:1}.scale-presets button{flex:1}.scale-presets>.restore{flex:1 1 100%}.game-separation{align-items:start}}
</style>
