<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import UiIcon from './UiIcon.svelte';
  import ServerDirectory from './ServerDirectory.svelte';
  import CommunitySpotlight from './CommunitySpotlight.svelte';
  import HostingAdvisor from './HostingAdvisor.svelte';
  import HostingOptions from './HostingOptions.svelte';
  import { isGameRunning } from './game_session';
  import { createLiveServerMonitor, type LiveServerStatus } from './server_live';
  import { addressKey, type DirectoryEntry } from './server_directory';

  let { username, currentVersion, instanceId = '', currentGameVersion = '', currentLoader = 'vanilla', onjoin, onconfigure }:
    {username:string;currentVersion:string;instanceId?:string;currentGameVersion?:string;currentLoader?:string;onjoin:(address:string)=>void|Promise<void>;onconfigure?:()=>void} = $props();
  type Entry = {id:string;name:string;address:string;icon:string};
  type ServerList = {entries:Entry[];revision:string;path:string};
  type Lan = {port:number|null;listening:boolean;local_addresses:string[];public_address:string|null;e4mc_installed:boolean;message:string};
  type Context = {username:string;versionId:string;instanceId:string|null};
  let list = $state<ServerList>({entries:[],revision:'',path:''});
  let statuses = $state<Record<string,LiveServerStatus>>({});
  let pinging = $state<Record<string,boolean>>({});
  let liveEnabled = $state(false);
  const monitor=createLiveServerMonitor(snapshot=>{statuses=snapshot.statuses;pinging=snapshot.pinging;liveEnabled=snapshot.enabled;});
  let loading = $state(false);
  let saving = $state(false);
  let installing = $state(false);
  let error = $state('');
  let notice = $state('');
  let name = $state('');
  let address = $state('');
  let editingId = $state<string|null>(null);
  let page = $state(1);
  let view = $state<'discover'|'mine'>('discover');
  let filter = $state('');
  let lan = $state<Lan|null>(null);
  let manualPort = $state('');
  let confirmDelete = $state<Entry|null>(null);
  let deleteDialog = $state<HTMLDialogElement>();
  let listGeneration = 0;
  let lanGeneration = 0;
  let disposed = false;
  const pageSize = 12;
  const context = ():Context => ({username,versionId:currentVersion,instanceId:instanceId||null});
  let filtered = $derived(list.entries.filter(entry => `${entry.name} ${entry.address}`.toLocaleLowerCase().includes(filter.toLocaleLowerCase())));
  let totalPages = $derived(Math.max(1,Math.ceil(filtered.length/pageSize)));
  let visible = $derived(filtered.slice((Math.min(page,totalPages)-1)*pageSize,Math.min(page,totalPages)*pageSize));
  let compatibleLoader = $derived(['fabric','forge','neoforge','quilt'].includes(currentLoader.toLowerCase()));
  let currentTarget = $derived(instanceId ? `Instancia · ${instanceId}` : `Perfil · ${currentVersion || 'Sin seleccionar'}`);

  $effect(() => {
    const selected=context();
    untrack(() => { void load(selected); void refreshLan(selected); });
  });
  $effect(() => {
    const entries=view==='mine'&&!loading&&!$isGameRunning?visible:[];
    untrack(() => monitor.setTargets(entries));
  });
  async function load(selected=context()) {
    const generation=++listGeneration;
    loading=true;saving=false;error='';notice='';statuses={};pinging={};page=1;editingId=null;
    try {
      const data=await invoke<ServerList>('servers_list',selected);
      if(generation!==listGeneration||disposed)return;
      list=data;
    } catch(problem) { if(generation===listGeneration&&!disposed){error=String(problem);list={entries:[],revision:'',path:''};} }
    finally {if(generation===listGeneration&&!disposed)loading=false;}
  }
  async function saveEntries(entries:Entry[],success:string) {
    if(saving||loading||!list.revision)return false;
    const generation=listGeneration;const selected=context();
    saving=true;error='';notice='';
    try {
      const saved=await invoke<ServerList>('servers_save',{...selected,entries,revision:list.revision});
      if(generation!==listGeneration||disposed)return false;
      list=saved;statuses={};pinging={};notice=success;return true;
    } catch(problem){if(generation===listGeneration&&!disposed)error=String(problem);return false;}
    finally{if(generation===listGeneration&&!disposed)saving=false;}
  }
  async function submitServer(event:SubmitEvent){
    event.preventDefault();if(!name.trim()||!address.trim())return;
    const entries=list.entries.map(entry=>({...entry}));
    if(editingId){const index=entries.findIndex(entry=>entry.id===editingId);if(index<0)return;entries[index]={...entries[index],name:name.trim(),address:address.trim()};}
    else entries.push({id:'',name:name.trim(),address:address.trim(),icon:''});
    if(await saveEntries(entries,editingId?'Servidor actualizado en Minecraft.':'Servidor añadido a la lista de Minecraft.')){name='';address='';editingId=null;page=totalPages;}
  }
  async function addFromDirectory(entry:DirectoryEntry):Promise<boolean>{
    if(list.entries.some(saved=>addressKey(saved.address)===addressKey(entry.address))){notice='Este servidor ya está en tu lista.';return false;}
    return saveEntries([...list.entries,{id:'',name:entry.name,address:entry.address,icon:entry.icon||''}],`${entry.name} se añadió a Mis servidores.`);
  }
  function edit(entry:Entry){editingId=entry.id;name=entry.name;address=entry.address;notice='';}
  function cancelEdit(){editingId=null;name='';address='';}
  async function move(entry:Entry,direction:number){
    const entries=[...list.entries];const from=entries.findIndex(item=>item.id===entry.id);const to=from+direction;
    if(from<0||to<0||to>=entries.length)return;[entries[from],entries[to]]=[entries[to],entries[from]];
    await saveEntries(entries,'Orden guardado en Minecraft.');
  }
  function requestDelete(entry:Entry){confirmDelete=entry;deleteDialog?.showModal();}
  async function remove(){
    if(!confirmDelete)return;const id=confirmDelete.id;
    if(await saveEntries(list.entries.filter(entry=>entry.id!==id),'Servidor eliminado de la lista.')){confirmDelete=null;deleteDialog?.close();if(page>totalPages)page=totalPages;}
  }
  async function join(entry:Entry){error='';try{await onjoin(entry.address);}catch(problem){error=String(problem);}}
  async function copy(value:string){try{await navigator.clipboard.writeText(value);notice='Dirección copiada. Lista para compartir.';}catch(problem){error=`No se pudo copiar: ${String(problem)}`;}}
  async function refreshLan(selected=context()) {
    const generation=++lanGeneration;
    const numericPort=manualPort.trim()?Number(manualPort):null;
    if(numericPort!==null&&(!Number.isInteger(numericPort)||numericPort<1||numericPort>65535))return;
    try {
      const status=await invoke<Lan>('lan_status',{...selected,manualPort:numericPort});
      if(generation===lanGeneration&&!disposed)lan=status;
    }catch(problem){if(generation===lanGeneration&&!disposed)error=String(problem);}
  }
  async function applyPort(){
    const port=Number(manualPort);
    if(manualPort.trim()&&(!Number.isInteger(port)||port<1||port>65535)){error='El puerto debe estar entre 1 y 65535.';return;}
    error='';await refreshLan();
  }
  async function installTunnel(){
    if(installing||!compatibleLoader)return;installing=true;error='';notice='';
    const selected=context();const generation=listGeneration;
    try {
      const message=await invoke<string>('install_e4mc',{...selected,gameVersion:currentGameVersion||currentVersion,loader:currentLoader.toLowerCase()});
      if(generation===listGeneration&&!disposed){notice=message;await refreshLan();}
    }catch(problem){if(generation===listGeneration&&!disposed)error=String(problem);}
    finally{if(!disposed)installing=false;}
  }
  onMount(()=>{
    monitor.start();
    let unlisten:(()=>void)|undefined;let debounce:ReturnType<typeof setTimeout>|undefined;
    void listen<{line:string}>('launcher-log',event=>{
      if(/e4mc|local game hosted|partida local alojada|started serving on/i.test(event.payload.line)){
        if(debounce)clearTimeout(debounce);debounce=setTimeout(()=>void refreshLan(),350);
      }
    }).then(stop=>{if(disposed)stop();else unlisten=stop;});
    const lanTimer=setInterval(()=>{if(!$isGameRunning)void refreshLan();},6000);
    return()=>{disposed=true;listGeneration++;lanGeneration++;unlisten?.();if(debounce)clearTimeout(debounce);clearInterval(lanTimer);monitor.stop();};
  });
</script>

<section class="server-hub">
  <header class="page-heading"><div><span class="eyebrow">MULTIJUGADOR</span><h1>Servidores y mundos compartidos</h1><p>Tu lista de Minecraft, conectada a cada instalación. Reúne a tu equipo y continúa la aventura.</p></div><div class="target-badge"><UiIcon name="cube"/><div><strong>{currentTarget}</strong><small>{currentGameVersion||currentVersion} · {currentLoader}</small></div>{#if onconfigure}<button class="icon-button" title="Cambiar instalación en Jugar" onclick={onconfigure}><UiIcon name="settings"/></button>{/if}</div></header>
  {#if error}<div class="notice error" role="alert"><UiIcon name="terminal"/><span>{error}</span><button class="icon-button" onclick={()=>error=''} aria-label="Cerrar error"><UiIcon name="close"/></button></div>{/if}
  {#if notice}<div class="notice" role="status"><UiIcon name="copy"/><span>{notice}</span><button class="icon-button" onclick={()=>notice=''} aria-label="Cerrar aviso"><UiIcon name="close"/></button></div>{/if}
  <nav class="server-sections" aria-label="Secciones de servidores"><button class:active={view==='discover'} aria-pressed={view==='discover'} onclick={()=>view='discover'}><UiIcon name="globe" size={17}/>Descubrir servidores</button><button class:active={view==='mine'} aria-pressed={view==='mine'} onclick={()=>view='mine'}><UiIcon name="server" size={17}/>Mis servidores <span>{list.entries.length}</span></button></nav>
  <HostingAdvisor {username} {currentVersion} {instanceId}/>
  {#if view==='discover'}
    <ServerDirectory savedAddresses={list.entries.map(entry=>entry.address)} busy={loading||saving} {currentVersion} onadd={addFromDirectory} onjoin={onjoin}/>
  {:else}
  <div class="hub-grid">
    <section class="glass servers-section" aria-labelledby="servers-title">
      <header class="section-heading"><div><h2 id="servers-title">Tu lista multijugador <span class="count">{list.entries.length}</span></h2><p>Se guarda en el mismo <code>servers.dat</code> que utiliza Minecraft.</p><small class="live-status">{liveEnabled?'Iconos, mensajes y jugadores · actualización cada 15 s':'Actualización automática pausada'}</small></div><button disabled={loading||saving} onclick={()=>void load()}><UiIcon name="refresh"/>{loading?'Actualizando…':'Actualizar lista'}</button></header>
      <div class="list-tools"><label class="search"><UiIcon name="grid"/><input aria-label="Buscar servidor" placeholder="Buscar por nombre o dirección…" bind:value={filter} oninput={()=>page=1}/></label><button disabled={loading||saving||!visible.length||!liveEnabled||Object.values(pinging).some(Boolean)} onclick={()=>monitor.refresh()}><UiIcon name="refresh"/>Consultar estado</button></div>
      {#if loading}<div class="empty"><span class="spinner"></span><h3>Leyendo tu lista de Minecraft…</h3></div>
      {:else if !list.entries.length}<div class="empty"><UiIcon name="cube" size={38}/><h3>El próximo mundo empieza con una dirección</h3><p>Añade tu primer servidor. Aparecerá también en la pantalla Multijugador de esta instalación.</p></div>
      {:else if !visible.length}<div class="empty"><h3>No hay servidores con esa búsqueda</h3><button onclick={()=>filter=''}><UiIcon name="grid" size={15}/>Mostrar todos</button></div>
      {:else}<div class="server-list">{#each visible as entry (entry.id)}{@const status=statuses[entry.id]}{@const measured=status?.measurement_available===false?undefined:status}{@const icon=status?.icon||entry.icon}<article class="server-card">
        <div class="server-icon">{#if icon}<img src={icon} alt="Icono del servidor" loading="lazy"/>{:else}<UiIcon name="cube" size={30}/>{/if}</div>
        <div class="server-info"><div class="server-title"><h3>{entry.name}</h3><span class="connection" class:online={measured?.online&&!status?.refresh_error} class:pending={pinging[entry.id]}>{pinging[entry.id]?'Consultando…':status?.refresh_error?'Sin actualización':measured?.online?`${measured.latency_ms} ms`:measured?'Sin respuesta':'Sin consultar'}</span></div><button class="address-copy" title="Copiar dirección" onclick={()=>void copy(entry.address)}>{entry.address}<UiIcon name="copy" size={13}/></button>
          {#if status?.motd?.length}<div class="motd" class:stale-motd={!measured?.online||!!status.refresh_error} data-no-translate>{#each status.motd as span}<span style:color={span.color} style:font-weight={span.bold?'700':'400'} style:font-style={span.italic?'italic':'normal'} style:text-decoration={`${span.underlined?'underline ':''}${span.strikethrough?'line-through':''}`}>{span.text}</span>{/each}</div>{:else if pinging[entry.id]}<p class="ping-error">Recibiendo el mensaje del servidor…</p>{/if}
          {#if measured?.online}<div class="server-meta"><strong>{measured.players_online.toLocaleString()} / {measured.players_max.toLocaleString()} jugadores</strong><span>{measured.version}</span></div>{:else if measured}<p class="ping-error">{measured.error}</p>{/if}
          {#if measured}<small class="measurement-time">{measured.refresh_error?'Último dato · reintentando':'Consulta directa'} · {new Date(measured.checked_at).toLocaleTimeString()}</small>{:else if status?.refresh_error}<small class="measurement-time">{status.refresh_error}</small>{/if}
        </div>
        <div class="server-actions"><button class="primary small" disabled={saving} onclick={()=>void join(entry)}><UiIcon name="play" size={15}/>Entrar directo</button><div class="row-actions"><button class="icon-button" title="Subir servidor" aria-label={`Subir ${entry.name}`} disabled={saving||list.entries[0]?.id===entry.id} onclick={()=>void move(entry,-1)}><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="m6 15 6-6 6 6"/></svg></button><button class="icon-button" title="Bajar servidor" aria-label={`Bajar ${entry.name}`} disabled={saving||list.entries.at(-1)?.id===entry.id} onclick={()=>void move(entry,1)}><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg></button><button class="icon-button" title="Editar servidor" aria-label={`Editar ${entry.name}`} disabled={saving} onclick={()=>edit(entry)}><UiIcon name="settings" size={15}/></button><button class="icon-button danger" title="Eliminar servidor" aria-label={`Eliminar ${entry.name}`} disabled={saving} onclick={()=>requestDelete(entry)}><svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M3 6h18M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7m4-7v7"/></svg></button></div></div>
      </article>{/each}</div>{/if}
      {#if totalPages>1}<nav class="pagination" aria-label="Páginas de servidores"><button disabled={page<=1} onclick={()=>page--}><UiIcon name="left" size={15}/>Anterior</button><span>{Math.min(page,totalPages)} / {totalPages}</span><button disabled={page>=totalPages} onclick={()=>page++}>Siguiente<UiIcon name="arrow" size={15}/></button></nav>{/if}
      <form onsubmit={submitServer} class="server-form"><h3>{editingId?'Editar servidor':'Añadir servidor'}</h3><div class="form-fields"><label>Nombre<input bind:value={name} maxlength="128" placeholder="El mundo de mi equipo" required disabled={saving||loading}/></label><label>Dirección<input bind:value={address} maxlength="320" placeholder="play.ejemplo.net o [::1]:25565" required disabled={saving||loading}/></label></div><div class="form-bottom"><small>Los dominios sin puerto conservan su detección SRV. Los cambios incluyen una copia <code>servers.dat_old</code>.</small><div>{#if editingId}<button type="button" onclick={cancelEdit}><UiIcon name="close" size={15}/>Cancelar</button>{/if}<button class="primary" type="submit" disabled={saving||loading||!list.revision||!name.trim()||!address.trim()}><UiIcon name={editingId?"save":"plus"} size={16}/>{saving?'Guardando…':editingId?'Guardar cambios':'Añadir servidor'}</button></div></div></form>
      {#if list.path}<details class="storage-path"><summary>Archivo sincronizado de esta instalación</summary><code>{list.path}</code><p>Si Minecraft tiene abierta su lista Multijugador, vuelve al menú y ábrela de nuevo para que lea los cambios.</p></details>{/if}
    </section>
    <aside class="lan-column">
      <section class="glass lan-section"><span class="eyebrow">JUGAR CON AMIGOS</span><header class="section-heading"><div><h2>Alojar un mundo LAN</h2><p>Tu mundo, tu equipo, una dirección compartida.</p></div><span class="status-dot" class:online={lan?.listening} title={lan?.listening?'Puerto LAN abierto':'Esperando un mundo LAN'}></span></header>
        <ol class="steps"><li><span>1</span><div><strong>Abre tu mundo</strong><small>Inicia la instalación que aparece en este panel.</small></div></li><li><span>2</span><div><strong>Selecciona Abrir en LAN</strong><small>En el menú de pausa de Minecraft, publica la partida.</small></div></li><li><span>3</span><div><strong>Comparte la dirección</strong><small>Red local para quienes estén cerca; e4mc para tus amigos en internet.</small></div></li></ol>
        <p class="lan-state" role="status">{lan?.message||'Buscando una partida LAN de esta instalación…'}</p>
        <div class="manual-port"><label>Puerto manual <span>Opcional</span><input inputmode="numeric" pattern="[0-9]*" maxlength="5" bind:value={manualPort} placeholder={lan?.port?String(lan.port):'Detección automática'}/></label><button onclick={()=>void applyPort()}><UiIcon name="refresh" size={16}/>Detectar</button></div>
        {#if lan?.local_addresses.length}<div class="share-group"><h3>En tu red local</h3>{#each lan.local_addresses as local}<button class="share-address" onclick={()=>void copy(local)}><code>{local}</code><UiIcon name="copy" size={16}/></button>{/each}<small>Comparte esta dirección con jugadores conectados a tu misma red Wi-Fi o Ethernet.</small></div>{/if}
        {#if lan?.public_address}<div class="share-group public"><span class="eyebrow">DIRECCIÓN PÚBLICA DETECTADA</span><button class="share-address" onclick={()=>void copy(lan!.public_address!)}><code>{lan.public_address}</code><UiIcon name="copy" size={18}/></button><small>Detectada en el registro de e4mc. El túnel depende de la conexión del juego con su servicio.</small></div>{/if}
      </section>
      <section class="glass tunnel-section"><div class="tunnel-title"><span class="tunnel-symbol"><UiIcon name="cube" size={24}/></span><div><h2>Conexión pública con e4mc</h2><span class="tag">{lan?.e4mc_installed?'Instalado en esta instancia':'Mod del anfitrión'}</span></div></div><p>Instala la edición compatible desde Modrinth, vuelve a abrir Minecraft y publica tu mundo en LAN. e4mc asignará un dominio temporal para compartirlo.</p><button class="primary install" disabled={installing||!compatibleLoader} onclick={()=>void installTunnel()}><UiIcon name="download"/>{installing?'Preparando e4mc y sus dependencias…':lan?.e4mc_installed?'Actualizar e4mc compatible':'Instalar e4mc para esta instancia'}</button>
        {#if !compatibleLoader}<p class="compatibility">Esta instalación usa {currentLoader}. Selecciona una instalación Fabric, Forge, NeoForge o Quilt compatible en Jugar.</p>{:else}<small class="compatibility">Minecraft {currentGameVersion||currentVersion} · {currentLoader}. Solo se instalarán archivos que declaren esta compatibilidad.</small>{/if}
        <button class="external-link" onclick={()=>void openUrl('https://e4mc.link')}>Conoce e4mc y a sus creadores <UiIcon name="arrow" size={14}/></button><p class="footnote">El mundo debe permanecer abierto para recibir conexiones. Tus amigos necesitan una versión y un conjunto de mods compatibles con tu partida.</p>
      </section>
    </aside>
  </div>
  {/if}
  <CommunitySpotlight placement="servers"/>
  <HostingOptions/>
</section>
<dialog bind:this={deleteDialog} class="delete-dialog"><h2>Eliminar servidor</h2><p>Se quitará <strong>{confirmDelete?.name}</strong> de <code>servers.dat</code> de esta instalación.</p><div><button onclick={()=>deleteDialog?.close()} disabled={saving}><UiIcon name="close" size={15}/>Cancelar</button><button class="danger" onclick={()=>void remove()} disabled={saving}><UiIcon name="trash" size={15}/>{saving?'Eliminando…':'Eliminar de la lista'}</button></div></dialog>

<style>
  .live-status{display:block;margin-top:7px;font-size:.63rem;color:#88bba7}.measurement-time{display:block;font-size:.59rem;margin-top:6px;color:#8f9eb6}.stale-motd{opacity:.65}.server-meta strong{font-size:.73rem;color:#dae7f4;font-weight:600}
  .server-sections{display:flex;gap:9px;flex-wrap:wrap;border-bottom:1px solid #ffffff15;padding-bottom:14px}.server-sections button{padding:12px 17px}.server-sections .active{border-color:var(--primary,#c084fc);background:rgba(var(--primary-rgb),.13);color:var(--primary,#c084fc)}.server-sections span{padding:2px 6px;background:#ffffff09;font-size:.62rem}
  .server-hub{display:grid;gap:22px;color:var(--text,#e6eafa);min-width:0}.page-heading,.section-heading{display:flex;justify-content:space-between;align-items:center;gap:18px}.page-heading{align-items:flex-end}.eyebrow{font-size:.65rem;font-weight:800;letter-spacing:.16em;color:var(--primary,#9396ff)}h1{font-size:clamp(1.65rem,2.2vw,2.1rem);letter-spacing:-.035em;margin:8px 0 10px}h2{font-size:1.08rem;letter-spacing:-.02em;margin:0 0 7px}h3{font-size:.92rem;margin:0}p{color:var(--text-muted,#9ca7be);line-height:1.6;font-size:.85rem;margin:0}small{color:var(--text-muted,#9ca7be);font-size:.72rem;line-height:1.5}.target-badge{display:flex;align-items:center;gap:12px;border:1px solid color-mix(in srgb,var(--primary,#9396ff) 27%,transparent);border-radius:13px;padding:12px;background:#12182c55;max-width:330px}.target-badge>div{display:grid;gap:3px;min-width:0}.target-badge strong{font-size:.78rem;overflow-wrap:anywhere}.hub-grid{display:grid;grid-template-columns:minmax(0,1.7fr) minmax(290px,.85fr);align-items:start;gap:22px}.glass{border:1px solid color-mix(in srgb,var(--primary,#9396ff) 22%,transparent);background:color-mix(in srgb,var(--panel-bg,#11172c) 86%,transparent);border-radius:19px;padding:22px;box-shadow:0 15px 40px #05081725;min-width:0}.servers-section{display:grid;gap:19px}.count{display:inline-flex;align-items:center;justify-content:center;min-width:23px;padding:3px 6px;border-radius:7px;background:color-mix(in srgb,var(--primary,#9396ff) 17%,transparent);color:var(--primary,#a5a8ff);font-size:.7rem;margin-left:5px}button{display:inline-flex;align-items:center;justify-content:center;gap:7px;background:#ffffff06;border:1px solid #ffffff18;border-radius:9px;padding:9px 12px;color:inherit;font:inherit;font-size:.75rem;cursor:pointer;transition:background .17s,border-color .17s;flex-shrink:0}button:hover:not(:disabled){border-color:var(--primary,#9396ff);background:color-mix(in srgb,var(--primary,#9396ff) 13%,transparent)}button:disabled{opacity:.4;cursor:default}button:focus-visible,input:focus-visible{outline:2px solid var(--primary,#9396ff);outline-offset:3px}.primary{background:linear-gradient(125deg,color-mix(in srgb,var(--primary,#8589ff) 85%,#11162a),color-mix(in srgb,var(--primary,#8589ff) 65%,#3434bd));border-color:color-mix(in srgb,var(--primary,#8589ff) 60%,transparent);color:#fff;font-weight:700}.icon-button{width:29px;height:29px;padding:5px;border-color:transparent;background:transparent}.danger{color:#ff9aab}.danger:hover:not(:disabled){background:#f04e6f20;border-color:#f04e6f70}.list-tools{display:flex;gap:9px}.search{display:flex;align-items:center;gap:9px;flex:1;min-width:0;background:#070d1c72;border:1px solid #ffffff16;border-radius:10px;padding:0 11px;color:#8491ae}input{width:100%;box-sizing:border-box;border:1px solid #ffffff1c;border-radius:9px;background:#090e20a8;color:inherit;font:inherit;font-size:.8rem;padding:11px 12px;min-width:0}.search input{border:0;background:transparent;padding:11px 0;outline:none}.server-list{display:grid;gap:9px}.server-card{display:flex;align-items:flex-start;gap:13px;padding:15px;border-radius:13px;background:#ffffff03;border:1px solid #ffffff0e;min-width:0}.server-icon{width:55px;height:55px;flex-shrink:0;display:grid;place-items:center;border-radius:11px;background:linear-gradient(140deg,#ffffff0b,#ffffff02);color:#7f8ca9;border:1px solid #ffffff0a}.server-icon img{width:100%;height:100%;object-fit:contain;border-radius:10px;image-rendering:pixelated}.server-info{min-width:0;flex:1}.server-title{display:flex;align-items:center;gap:9px;flex-wrap:wrap}.server-title h3{overflow-wrap:anywhere}.connection{font-size:.6rem;color:#ab7c83;white-space:nowrap}.connection.online{color:#71d5b5}.connection.pending{color:#acb3d6}.address-copy{font-size:.71rem;padding:4px 0;background:none;border:0;max-width:100%;word-break:break-all;text-align:left;justify-content:flex-start;color:#8c9ab7}.motd{white-space:pre-wrap;overflow-wrap:anywhere;line-height:1.45;font-size:.76rem;margin:5px 0;max-height:100px;overflow:auto}.server-meta{display:flex;flex-wrap:wrap;gap:12px;font-size:.65rem;color:#8796b3;margin-top:6px}.ping-error{font-size:.7rem;margin-top:6px;max-height:64px;overflow:hidden;overflow-wrap:anywhere;color:#bd93a1}.server-actions{display:grid;gap:7px;justify-items:end}.row-actions{display:flex;gap:1px}.small{font-size:.69rem;padding:8px 10px}.empty{min-height:190px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:12px;text-align:center;padding:25px;color:#8b97b4;border:1px dashed #ffffff16;border-radius:14px}.empty p{max-width:390px;font-size:.8rem}.server-form{border-top:1px solid #ffffff12;padding-top:19px;display:grid;gap:13px}.form-fields{display:grid;grid-template-columns:1fr 1.3fr;gap:12px}label{display:grid;gap:7px;font-size:.73rem;color:#b8c2d9}.form-bottom{display:flex;align-items:center;justify-content:space-between;gap:14px}.form-bottom>small{max-width:340px;font-size:.66rem}.form-bottom>div{display:flex;gap:7px}.storage-path{font-size:.68rem;color:#8999b8;overflow-wrap:anywhere}.storage-path summary{cursor:pointer;padding:5px 0}.storage-path code{display:block;padding-top:6px;font-size:.65rem}.storage-path p{font-size:.68rem;padding-top:7px}.lan-column{display:grid;gap:18px}.lan-section{display:grid;gap:17px}.status-dot{height:9px;width:9px;border-radius:50%;background:#63708b;box-shadow:0 0 0 5px #63708b15;flex-shrink:0}.status-dot.online{background:#6bd3ad;box-shadow:0 0 0 5px #6bd3ad16}.steps{padding:0;margin:0;display:grid;gap:16px;list-style:none}.steps li{display:flex;gap:11px;align-items:flex-start}.steps li>span{height:23px;min-width:23px;display:grid;place-items:center;border:1px solid color-mix(in srgb,var(--primary,#9396ff) 28%,transparent);background:color-mix(in srgb,var(--primary,#9396ff) 9%,transparent);color:var(--primary,#a5a8ff);border-radius:7px;font-size:.7rem;font-weight:800}.steps li>div{display:grid;gap:4px}.steps strong{font-size:.77rem}.steps small{font-size:.69rem}.lan-state{font-size:.74rem;border:1px solid #ffffff0a;border-radius:10px;background:#070e1d55;padding:11px}.manual-port{display:flex;gap:8px;align-items:flex-end}.manual-port label{flex:1;min-width:0;display:flex;flex-wrap:wrap;justify-content:space-between;font-size:.69rem}.manual-port label>span{color:#7887a3;font-size:.63rem}.manual-port input{flex-basis:100%;padding:9px}.manual-port>button{padding:9px}.share-group{display:grid;gap:9px;border-top:1px solid #ffffff12;padding-top:16px}.share-group h3{font-size:.77rem}.share-address{display:flex;justify-content:space-between;text-align:left;background:#080f1e99;border-color:#8d98d72f;padding:11px;min-width:0}.share-address code{font-size:.73rem;overflow-wrap:anywhere;white-space:normal}.share-address :global(svg){flex-shrink:0}.public .share-address{border-color:#67d0ab66;background:#58b99b0c;color:#8fe6c5}.share-group>small{font-size:.66rem}.tunnel-section{display:grid;gap:13px}.tunnel-title{display:flex;align-items:center;gap:11px}.tunnel-title h2{font-size:.94rem}.tunnel-symbol{width:43px;height:43px;display:grid;place-items:center;border:1px solid #77c7b040;border-radius:12px;color:#8adbba;background:#77c7b00a}.tag{display:inline-block;font-size:.59rem;letter-spacing:.02em;padding:3px 6px;color:#a9c4ba;background:#88c2ab13;border:1px solid #88c2ab25;border-radius:5px}.tunnel-section>p{font-size:.76rem}.install{white-space:normal;width:100%;font-size:.74rem;padding:12px}.compatibility{font-size:.68rem!important;color:#93a0be}.external-link{justify-content:flex-start;padding:4px 0;background:transparent;border:0;font-size:.72rem;color:var(--primary,#acafff)}.footnote{border-top:1px solid #ffffff12;padding-top:12px;font-size:.65rem!important}.notice{display:flex;align-items:center;gap:11px;font-size:.78rem;line-height:1.5;color:#b4cded;padding:13px 15px;background:#517eaf12;border:1px solid #71a6dc35;border-radius:11px}.notice span{flex:1;overflow-wrap:anywhere}.notice.error{background:#c4496110;border-color:#e36d8740;color:#eaa7b8}.notice :global(svg){flex-shrink:0}.pagination{display:flex;align-items:center;justify-content:center;gap:13px;font-size:.75rem;color:#a4aec7}.delete-dialog{background:#11172cf5;color:#e6eafa;border:1px solid #8e96cc44;border-radius:17px;padding:25px;max-width:450px}.delete-dialog::backdrop{background:#050818b5;backdrop-filter:blur(6px)}.delete-dialog p{padding:13px 0 22px}.delete-dialog>div{display:flex;justify-content:flex-end;gap:9px}.spinner{width:23px;height:23px;border:2px solid #ffffff14;border-top-color:var(--primary,#9396ff);border-radius:50%;animation:spin 1s linear infinite}@keyframes spin{to{transform:rotate(360deg)}}
  @media(max-width:1250px){.hub-grid{grid-template-columns:minmax(0,1fr)}.lan-column{grid-template-columns:repeat(2,minmax(0,1fr))}.page-heading{align-items:flex-start}.target-badge{max-width:240px}.server-card{padding:13px}}@media(max-width:800px){.page-heading{flex-direction:column}.target-badge{max-width:none;width:100%;box-sizing:border-box}.lan-column{grid-template-columns:1fr}.glass{padding:16px}.section-heading{align-items:flex-start;flex-wrap:wrap}.form-fields{grid-template-columns:1fr}.form-bottom{flex-direction:column;align-items:stretch}.form-bottom>small{max-width:none}.form-bottom>div{justify-content:flex-end}.server-card{gap:10px;flex-wrap:wrap}.server-icon{width:43px;height:43px}.server-actions{width:100%;grid-template-columns:1fr auto;align-items:center;justify-items:start;padding-left:53px;box-sizing:border-box}.list-tools{flex-wrap:wrap}.list-tools>.search{flex-basis:100%}.server-actions .primary{width:100%}}@media(prefers-reduced-motion:reduce){.spinner{animation:none}button{transition:none}}
</style>
