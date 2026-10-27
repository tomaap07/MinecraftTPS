<script lang="ts">
  import { isGameRunning } from './game_session';
  import { invoke } from '@tauri-apps/api/core';
  import UiIcon from './UiIcon.svelte';
  export type Playtime = { total_seconds: number; session_seconds: number; sessions: number; active_games: number; versions: Record<string,{seconds:number;sessions:number}> };
  let { username, currentVersion, instanceId = '', onstats, onexplore }: { username:string; currentVersion:string; instanceId?:string; onexplore?:(section:"shader"|"resourcepack")=>void; onstats?:(value:Playtime)=>void } = $props();
  type Shot = { id:string; name:string; instance:string; modified:number; thumbnail:string };
  let stats = $state<Playtime>({total_seconds:0,session_seconds:0,sessions:0,active_games:0,versions:{}});
  let shots = $state<Shot[]>([]);
  let message = $state('');
  let busy = $state(false);
  let dialog = $state<HTMLDialogElement>();
  let tab = $state<'screenshots'|'stats'>('screenshots');
  const formatTime = (seconds:number) => `${Math.floor(seconds/3600)} h ${Math.floor(seconds%3600/60)} min`;
  async function refreshStats() { try { stats = await invoke<Playtime>('get_playtime',{username}); onstats?.(stats); } catch(error) { message = String(error); } }
  $effect(() => { username;if($isGameRunning)return;void refreshStats();const timer=setInterval(()=>void refreshStats(),10000);return()=>clearInterval(timer); });
  async function openGallery(section:'screenshots'|'stats') { tab = section; message = ''; dialog?.showModal(); await refreshStats(); if(section === 'screenshots') await refreshScreenshots(); }
  async function refreshScreenshots() { busy=true; try { shots=await invoke<Shot[]>('list_screenshots'); } catch(error) { message=String(error); } finally { busy=false; } }
  async function openFolder(folder:string) { try { await invoke('open_player_folder',{username,versionId:currentVersion,instanceId:instanceId||null,folder}); } catch(error) { message=String(error); } }
  async function openShot(id:string) { try { await invoke('open_screenshot',{id}); } catch(error) { message=String(error); } }
  async function copyShot(id:string) {
    try {

      const image = invoke<string>('screenshot_data',{id}).then(async data => (await fetch(data)).blob());
      await navigator.clipboard.write([new ClipboardItem({'image/png':image})]);
      message='Imagen copiada al portapapeles.';
    } catch(error) { message=`No se pudo copiar la captura: ${String(error)}`; }
  }
</script>

<div class="quick-tools">
  <button onclick={() => void openGallery('screenshots')}><UiIcon name="image" size={19}/>Capturas</button>
  <button onclick={() => void openFolder('saves')}><UiIcon name="map" size={19}/>Mundos</button>
  <button onclick={() => void openFolder('instance')}><UiIcon name="folder" size={19}/>Carpeta del juego</button>
  <button onclick={() => onexplore?.('shader')}><UiIcon name="sun" size={19}/>Explorar shaders</button>
  <button onclick={() => onexplore?.('resourcepack')}><UiIcon name="layers" size={19}/>Explorar recursos</button>
  <button onclick={() => void openGallery('stats')}><UiIcon name="clock" size={19}/>Tiempo por versión</button>
</div>
{#if message && !dialog?.open}<p class="tool-message" role="status">{message}</p>{/if}
<dialog bind:this={dialog} class="gallery-dialog">
  <header><div><small>TU BIBLIOTECA</small><h2>{tab==='screenshots'?'Capturas de pantalla':'Tu tiempo de juego'}</h2></div><button aria-label="Cerrar" onclick={() => dialog?.close()}><UiIcon name="close" /></button></header>
  <nav><button class:active={tab==='screenshots'} onclick={() => {tab='screenshots';void refreshScreenshots();}}><UiIcon name="image" size={16}/>Capturas</button><button class:active={tab==='stats'} onclick={() => {tab='stats';void refreshStats();}}><UiIcon name="clock" size={16}/>Estadísticas</button><button onclick={() => tab==='screenshots'?void refreshScreenshots():void refreshStats()}><UiIcon name="refresh" />Actualizar</button></nav>
  {#if message}<p class="tool-message" role="status">{message}</p>{/if}
  {#if tab==='screenshots'}
    <p>Las últimas 48 capturas de todas tus instancias. Las imágenes se conservan en sus carpetas originales.</p>
    {#if busy}<p>Preparando galería…</p>{:else if !shots.length}<div class="empty"><UiIcon name="image" size={32}/><h3>Tus aventuras aparecerán aquí</h3><p>Presiona F2 dentro de Minecraft para tomar tu primera captura.</p><button onclick={() => void openFolder('screenshots')}><UiIcon name="folder" size={16}/>Abrir carpeta de capturas</button></div>{:else}<div class="shots">{#each shots as shot}<article><button class="preview" onclick={() => void openShot(shot.id)}><img src={shot.thumbnail} alt={shot.name} /></button><strong>{shot.name}</strong><small>{shot.instance} · {new Date(shot.modified*1000).toLocaleDateString()}</small><div><button onclick={() => void openShot(shot.id)}><UiIcon name="external" size={15}/>Abrir</button><button onclick={() => void copyShot(shot.id)}><UiIcon name="copy"/>Copiar imagen</button></div></article>{/each}</div>{/if}
  {:else}
    <div class="stat-total"><strong>{formatTime(stats.total_seconds)}</strong><span>{stats.sessions} sesiones · {stats.active_games} juegos abiertos</span><small>Sesión actual: {formatTime(stats.session_seconds)}</small></div>
    {#if Object.keys(stats.versions).length}<div class="stats-table">{#each Object.entries(stats.versions).sort((a,b) => b[1].seconds-a[1].seconds) as [version,record]}<div><strong>{version}</strong><span>{formatTime(record.seconds)}</span><small>{record.sessions} sesiones</small><progress value={record.seconds} max={Math.max(1,stats.total_seconds)}></progress></div>{/each}</div>{:else}<p>El registro comenzará cuando abras Minecraft desde este launcher.</p>{/if}
  {/if}
</dialog>

<style>
  .quick-tools{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}.quick-tools button,dialog button{display:inline-flex;align-items:center;justify-content:center;gap:7px;border:1px solid rgba(255,255,255,.14);background:rgba(20,23,43,.65);color:#dbe2f5;padding:9px 12px;border-radius:10px;cursor:pointer;transition:.18s}.quick-tools button:hover,dialog button:hover{border-color:var(--primary,#a78bfa);background:rgba(124,90,216,.2)}.gallery-dialog{background:rgba(13,15,29,.98);color:#e9ecf7;border:1px solid rgba(167,139,250,.35);border-radius:20px;width:min(1060px,90vw);max-height:85vh;padding:24px;box-shadow:0 30px 100px #0009}.gallery-dialog::backdrop{background:#03050bbd;backdrop-filter:blur(8px)}header{display:flex;justify-content:space-between;align-items:center;gap:20px}h2{margin:6px 0 18px}header small{color:#a78bfa;letter-spacing:.12em}nav{display:flex;gap:8px;flex-wrap:wrap}nav .active{border-color:#a78bfa;background:#a78bfa25}p,small{color:#a5afc8}.shots{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:16px}.shots article{border:1px solid #ffffff18;background:#ffffff04;border-radius:14px;padding:10px;display:grid;gap:9px;min-width:0}.shots strong,.shots small{overflow-wrap:anywhere}.preview{padding:0;overflow:hidden;aspect-ratio:16/9}.preview img{width:100%;height:100%;object-fit:cover}.shots article>div{display:flex;gap:6px}.empty{padding:50px 15px;text-align:center;border:1px dashed #ffffff20;border-radius:15px;margin-top:20px}.stat-total{display:grid;gap:10px;margin:25px 0}.stat-total>strong{font-size:2rem;color:#c4b5fd}.stats-table>div{display:grid;grid-template-columns:1fr auto auto;gap:12px;padding:16px 0;border-bottom:1px solid #ffffff12}.stats-table progress{grid-column:1/-1;width:100%;height:5px;accent-color:#a78bfa}.tool-message{color:#c4b5fd;font-size:.85rem}button:focus-visible{outline:2px solid #c4b5fd;outline-offset:3px}
</style>
