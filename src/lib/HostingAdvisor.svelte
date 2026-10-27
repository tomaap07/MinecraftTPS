<script lang="ts">
  import { untrack, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { isGameRunning } from './game_session';
  import UiIcon from './UiIcon.svelte';
  import GameIcon from './GameIcon.svelte';
  let {username,currentVersion,instanceId=''}:{username:string;currentVersion:string;instanceId?:string}=$props();
  type Recommendation={enabled_mods:number;recommended_gb:number;estimated:boolean;explanation:string};
  let value=$state<Recommendation|null>(null),loading=$state(false),error=$state('');
  let generation=0,disposed=false;
  $effect(()=>{const context={username,versionId:currentVersion,instanceId:instanceId||null};if(!$isGameRunning)untrack(()=>void refresh(context));});
  async function refresh(context={username,versionId:currentVersion,instanceId:instanceId||null}){
    if($isGameRunning)return;const request=++generation;loading=true;error='';
    try{const result=await invoke<Recommendation>('get_hosting_recommendation',context);if(request===generation&&!disposed&&!$isGameRunning)value=result;}
    catch(problem){if(request===generation&&!disposed)error=String(problem);}
    finally{if(request===generation&&!disposed)loading=false;}
  }
  onDestroy(()=>{disposed=true;generation++;});
</script>

{#if !$isGameRunning}<section class="hosting-advisor"><span class="advisor-icon"><GameIcon name="redstone" size={28}/></span><div><small>JUGAR CON AMIGOS · ESTIMACIÓN DE MEMORIA</small><h3>{value?value.enabled_mods+' mods activos → punto de partida: '+value.recommended_gb+' GB de RAM':'Prepara un servidor para tu instalación'}</h3><p>{error||value?.explanation||'Se cuentan los archivos .jar activos de la instalación seleccionada.'}</p></div><button disabled={loading} onclick={()=>void refresh()}><UiIcon name="refresh" size={15}/>{loading?'Calculando…':'Recalcular'}</button></section>{/if}

<style>
  .hosting-advisor{display:flex;align-items:center;gap:14px;padding:17px 19px;border:1px solid rgba(var(--primary-rgb,168,85,247),.22);background:rgba(var(--bg-base-rgb,11,11,15),.7);color:#e8e1f5}.advisor-icon{width:46px;height:46px;display:grid;place-items:center;background:#0003;border:1px solid #ffffff13;flex-shrink:0}.hosting-advisor>div{flex:1;min-width:0}small{color:var(--primary,#A855F7);font-size:.53rem;letter-spacing:.07em}h3{font-size:.88rem;line-height:1.6;margin:7px 0}p{color:#aaa0bd;font-size:.69rem;line-height:1.7;margin:0}button{display:flex;gap:7px;align-items:center;color:#d7cce9;background:#ffffff05;border:1px solid #ffffff22;padding:10px 12px;font:inherit;font-size:.7rem;cursor:pointer}button:disabled{opacity:.5}button:focus-visible{outline:2px solid var(--primary,#A855F7);outline-offset:3px}@media(max-width:800px){.hosting-advisor{flex-wrap:wrap}.hosting-advisor>button{margin-left:60px}}
</style>
