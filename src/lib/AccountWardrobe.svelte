<script lang="ts">
  import GameIcon from "./GameIcon.svelte";
  import ServiceLogo from "./ServiceLogo.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import UiIcon from "./UiIcon.svelte";
  import SkinModelViewer from "./SkinModelViewer.svelte";
  import CapePreview from "./CapePreview.svelte";

  type Profile = { id: string; name: string; skins: { id: string; state?: string; url?: string; variant?: string }[]; capes: { id: string; alias?: string; url?: string; state?: string }[] };
  type SavedSkin = { id: string; name: string; model: "classic" | "slim"; created_at: number; file_name: string; data_url: string; error?: string };
  let { profile, skinUrl, fallbackSkinUrl="", capeUrl = "", model = "auto-detect", username, loginBusy = false, loginMessage = "",offlineAllowed=false,offlineMessage="", onlogin, onlogout, onoffline, onprofile, onposer, onpreview }: {
    profile: Profile | null; skinUrl: string; fallbackSkinUrl?:string; capeUrl?: string; model?: "default" | "slim" | "auto-detect"; username: string;
    loginBusy?: boolean; loginMessage?: string;offlineAllowed?:boolean;offlineMessage?:string; onlogin: () => void; onlogout: () => void; onoffline: () => void; onprofile: (value: Profile) => void; onposer: () => void; onpreview?: (url: string, model: "default" | "slim") => void;
  } = $props();
  let skins = $state<SavedSkin[]>([]);
  let busy = $state(false);
  let loading = $state(true);
  let message = $state("");
  let error = $state(false);
  let variant = $state<"classic" | "slim">("classic");
  let skinName = $state("");
  let importNick = $state("");
  let selectedSkin = $state<SavedSkin | null>(null);
  let renaming = $state("");
  let renameName = $state("");
  let dragActive = $state(false);
  let previewUrl = $derived(selectedSkin?.data_url || skinUrl);
  let previewModel = $derived(selectedSkin ? (selectedSkin.model === "slim" ? "slim" as const : "default" as const) : model);
  let previewName = $derived(selectedSkin?.name ?? profile?.name ?? (username || "Steve"));

  $effect(() => { variant = model === "slim" ? "slim" : "classic"; });
  onMount(() => { void refresh(); });

  async function exploreSkins() {
    try { await openUrl("https://namemc.com/minecraft-skins"); }
    catch { error = true; message = "No se pudo abrir NameMC en tu navegador. Puedes visitar namemc.com/minecraft-skins."; }
  }

  async function refresh() {
    loading = true;
    try { skins = await invoke<SavedSkin[]>("list_saved_skins"); }
    catch (cause) { message = String(cause); error = true; }
    finally { loading = false; }
  }

  async function perform(action: () => Promise<void>) {
    if (busy) return;
    busy = true; error = false; message = "";
    try { await action(); } catch (cause) { error = true; message = String(cause); }
    finally { busy = false; }
  }

  function readFile(file: File): Promise<string> {
    if (!file.name.toLowerCase().endsWith(".png") || file.size > 2 * 1024 * 1024) return Promise.reject("Elige un PNG de menos de 2 MB.");
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result));
      reader.onerror = () => reject("No se pudo leer el archivo PNG.");
      reader.readAsDataURL(file);
    });
  }

  async function addFile(file: File) {
    await perform(async () => {
      const skinBase64 = await readFile(file);
      const saved = await invoke<SavedSkin>("save_library_skin", { name: skinName.trim() || file.name.replace(/\.png$/i, ""), skinBase64, model: variant });
      selectedSkin = saved;
      await refresh();
      message = `«${saved.name}» se guardó en tu armario. Puedes previsualizarla o sincronizarla con Microsoft.`;
    });
  }

  function fileChanged(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (file) void addFile(file);
    input.value = "";
  }

  function dropFile(event: DragEvent) {
    event.preventDefault(); dragActive = false;
    const file = event.dataTransfer?.files[0];
    if (file) void addFile(file);
  }

  async function saveCurrent() {
    await perform(async () => {
      const active = profile?.skins.find(skin => skin.state === "ACTIVE");
      let source = active?.url ?? skinUrl;
      if (source.startsWith("/") || source.startsWith("blob:") || source.startsWith("https://crafatar.com/")) {
        const response = await fetch(source);
        if (!response.ok) throw new Error("No se pudo leer la skin local del visor.");
        source = await readFile(new File([await response.blob()], "skin.png", { type: "image/png" }));
      }
      const saved = await invoke<SavedSkin>("save_current_skin", {
        name: skinName.trim() || `${profile?.name ?? username ?? "Steve"} · ${new Date().toLocaleDateString()}`,
        skinUrl: source, model: active?.variant?.toLowerCase() === "slim" ? "slim" : variant,
      });
      selectedSkin = saved; await refresh(); message = "Tu skin actual quedó guardada en el armario local.";
    });
  }

  async function importByNick() {
    await perform(async () => {
      const saved = await invoke<SavedSkin>("import_library_skin", { nick: importNick.trim(), name: skinName.trim() });
      selectedSkin = saved; await refresh(); message = `Skin de ${importNick.trim()} importada como «${saved.name}».`;
    });
  }

  async function useSkin(skin: SavedSkin) {
    if (skin.error) { error = true; message = skin.error; return; }
    selectedSkin = skin;
    if (!profile) { onpreview?.(skin.data_url, skin.model === "slim" ? "slim" : "default"); error = false; message = "Vista previa actualizada. Inicia sesión con Microsoft para sincronizar esta skin dentro del juego."; return; }
    await perform(async () => {
      const updated = await invoke<Profile>("apply_library_skin", { id: skin.id });
      onprofile(updated); selectedSkin = null; message = `«${skin.name}» ya está activa en tu cuenta de Minecraft.`;
    });
  }

  async function renameSkin(skin: SavedSkin) {
    await perform(async () => {
      await invoke("rename_library_skin", { id: skin.id, name: renameName });
      if (selectedSkin?.id === skin.id) selectedSkin = { ...skin, name: renameName.trim() };
      renaming = ""; await refresh(); message = "Nombre actualizado.";
    });
  }

  async function deleteSkin(skin: SavedSkin) {
    await perform(async () => {
      await invoke("delete_library_skin", { id: skin.id });
      if (selectedSkin?.id === skin.id) selectedSkin = null;
      await refresh(); message = `«${skin.name}» se eliminó del armario.`;
    });
  }

  async function changeCape(id?: string) {
    await perform(async () => {
      await invoke("get_microsoft_profile");
      const updated = await invoke<Profile>(id ? "activate_minecraft_cape" : "hide_minecraft_cape", id ? { capeId: id } : {});
      onprofile(updated); message = id ? "Capa equipada en tu perfil oficial." : "Capa ocultada en tu perfil oficial.";
    });
  }

  function portrait(canvas: HTMLCanvasElement, skin: SavedSkin) {
    let request = 0;
    function draw(value: SavedSkin) {
      const version = ++request;
      const image = new Image();
      image.onload = () => {
        if (version !== request) return;
        canvas.width = 160; canvas.height = 256;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;
        ctx.imageSmoothingEnabled = false; ctx.clearRect(0, 0, 160, 256);
        const p = (sx: number, sy: number, sw: number, sh: number, dx: number, dy: number) => ctx.drawImage(image, sx, sy, sw, sh, dx * 8, dy * 8, sw * 8, sh * 8);
        const slim = value.model === "slim" && image.height === 64;
        const arm = slim ? 3 : 4;
        p(8, 8, 8, 8, 6, 0);
        let drawHat = true;
        if (image.height === 32) {
          const source = document.createElement("canvas"); source.width = 64; source.height = 32;
          const sourceContext = source.getContext("2d");
          if (sourceContext) {
            sourceContext.drawImage(image, 0, 0);
            const pixels = sourceContext.getImageData(32, 0, 32, 32).data;
            drawHat = false;
            for (let index = 3; index < pixels.length; index += 4) { if (pixels[index] < 128) { drawHat = true; break; } }
          }
        }
        if (drawHat) p(40, 8, 8, 8, 6, 0);
        p(20, 20, 8, 12, 6, 8);
        p(44, 20, arm, 12, 6 - arm, 8);
        if (image.height === 64) {
          p(36, 52, arm, 12, 14, 8); p(20, 52, 4, 12, 10, 20);
          p(20, 36, 8, 12, 6, 8); p(44, 36, arm, 12, 6 - arm, 8); p(52, 52, arm, 12, 14, 8);
          p(4, 20, 4, 12, 6, 20); p(4, 36, 4, 12, 6, 20); p(4, 52, 4, 12, 10, 20);
        } else {
          ctx.save(); ctx.translate(144, 0); ctx.scale(-1, 1); ctx.drawImage(image, 44, 20, 4, 12, 0, 64, 32, 96); ctx.restore();
          p(4, 20, 4, 12, 6, 20); p(4, 20, 4, 12, 10, 20);
        }
      };
      image.src = value.data_url;
    }
    draw(skin);
    return { update: draw, destroy() { request++; } };
  }
</script>

<div class="account-workspace">
  <header class="page-heading"><div><span class="eyebrow">TU IDENTIDAD EN MINECRAFT</span><h2><GameIcon name="name_tag" size={28}/>Cuenta y armario</h2><p>Tu personaje, tus capas y tus skins favoritas en un mismo lugar.</p></div><button class="outline" onclick={onposer}><GameIcon name="armor_stand" size={18}/>Abrir Estudio 3D <UiIcon name="external" size={14}/></button></header>
  <section class="account-summary glass">
    <div class="account-identity"><div class="account-symbol" aria-hidden="true">{#if profile}<ServiceLogo name="microsoft" size={28}/>{:else}<GameIcon name="name_tag" size={28}/>{/if}</div><div><h3>{profile ? `Cuenta Premium: ${profile.name}` : `Modo offline · ${username || "Steve"}`}</h3><p>{profile ? "Sesión Microsoft protegida y renovada automáticamente en este dispositivo." : "El armario local está disponible. Conecta tu cuenta para aplicar skins y capas oficiales."}</p></div></div>
    {#if profile}<button class="outline" onclick={onlogout} disabled={busy}><UiIcon name="close" size={16}/>Cerrar sesión</button>{:else}<div class="auth-actions"><button class="primary" onclick={onlogin} disabled={loginBusy}><ServiceLogo name="microsoft" size={18}/>{loginBusy ? "Esperando a Microsoft…" : "Iniciar sesión con Microsoft"}</button><button class="outline" disabled={loginBusy||!offlineAllowed} title={offlineMessage} onclick={onoffline}><UiIcon name="user" size={16}/>Modo offline</button></div>{/if}
    {#if loginMessage}<p class="account-note">{loginMessage}</p>{/if}
  </section>

  <section class="discovery-banner" aria-labelledby="skin-discovery-title">
    <div class="discovery-icon" aria-hidden="true"><ServiceLogo name="namemc" size={38}/></div>
    <div class="discovery-copy"><span class="eyebrow">UN NUEVO ASPECTO PARA TU AVENTURA</span><h3 id="skin-discovery-title">¿Todavía no tienes skin?</h3><p>Encuentra tu próximo personaje en el catálogo externo de NameMC.</p><ol><li>Busca un diseño que te guste.</li><li>Descarga su archivo PNG.</li><li>Impórtalo aquí y equípalo en tu cuenta.</li></ol></div>
    <button class="primary discovery-action" onclick={exploreSkins}><ServiceLogo name="namemc" size={18}/>Explorar skins en NameMC <UiIcon name="external" size={14}/></button>
  </section>

  <div class="identity-grid">
    <section class="glass viewer-card"><div class="section-heading"><div><span class="eyebrow">VISTA 360°</span><h3>{previewName}</h3></div>{#if selectedSkin}<button class="subtle" onclick={() => selectedSkin = null}><UiIcon name="user" size={15}/>Volver a mi perfil</button>{/if}</div><SkinModelViewer fallbackSkinUrl={selectedSkin?'':fallbackSkinUrl} skinUrl={previewUrl} {capeUrl} model={previewModel}/><p class="hint">Arrastra para girar y usa la rueda para ajustar la cámara.</p><p class="skin-providers">Perfiles y texturas de Mojang · Respaldo de <button onclick={()=>void openUrl("https://crafatar.com/").catch(()=>{error=true;message="No se pudo abrir Crafatar.";})}>Crafatar</button></p></section>
    <section class="glass import-card"><div><span class="eyebrow">PERSONALIZA TU PERSONAJE</span><h3><UiIcon name="skin" size={19}/>Importar una skin</h3><p>Guarda tu diseño localmente y sincronízalo con tu cuenta cuando quieras.</p></div>
      <label class="field">Nombre de la skin<input bind:value={skinName} placeholder="Mi próxima aventura" maxlength="80"/></label>
      <div class="field"><span>Modelo del personaje</span><div class="segmented" aria-label="Modelo de skin"><button class:active={variant === "classic"} aria-pressed={variant === "classic"} onclick={() => variant = "classic"}><UiIcon name="skin" size={15}/>Clásico <small>4 px</small></button><button class:active={variant === "slim"} aria-pressed={variant === "slim"} onclick={() => variant = "slim"}><UiIcon name="skin" size={15}/>Slim <small>3 px</small></button></div></div>
      <label class:dragging={dragActive} class="drop-zone" ondragover={(event) => { event.preventDefault(); dragActive = true; }} ondragleave={() => dragActive = false} ondrop={dropFile}>
        <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><path d="M12 16V3m-5 5 5-5 5 5M4 14v6a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-6"/></svg><strong>{busy ? "Procesando skin…" : "Subir archivo PNG"}</strong><span>Arrástralo aquí o haz clic para elegir</span><small>64 × 64 o 64 × 32 · hasta 2 MB</small><input type="file" accept="image/png,.png" onchange={fileChanged} disabled={busy}/>
      </label>
      <div class="import-nick"><label class="field">Importar desde un nick premium<input bind:value={importNick} maxlength="16" placeholder="Nombre de Minecraft" onkeydown={(event) => { if (event.key === "Enter" && importNick.trim() && !busy) void importByNick(); }}/></label><button class="outline" onclick={importByNick} disabled={busy || !importNick.trim()}><UiIcon name="download" size={15}/>Importar</button></div>
      <button class="outline save-current" onclick={saveCurrent} disabled={busy || !skinUrl}><UiIcon name="save" size={16}/>Guardar skin actual en el armario</button>
    </section>
  </div>

  {#if message}<div class:error class="notice" role="status">{message}</div>{/if}

  <section class="glass wardrobe-section"><div class="section-heading"><div><span class="eyebrow">BIBLIOTECA PERSONAL</span><h3>Armario de skins guardadas <span class="count">{skins.length}</span></h3><p>Guardadas en este dispositivo. Selecciona una para verla a tamaño completo.</p></div><button class="outline" onclick={refresh} disabled={busy || loading}><UiIcon name="refresh" size={16}/>{loading ? "Cargando…" : "Actualizar"}</button></div>
    {#if loading && !skins.length}<div class="empty-state">Cargando tu armario…</div>{:else if !skins.length}<div class="empty-state"><strong>Tu colección empieza aquí</strong><p>Importa un PNG, busca un nick premium o guarda tu skin actual.</p></div>{:else}<div class="wardrobe-grid">{#each skins as skin (skin.id)}<article class:selected={selectedSkin?.id === skin.id} class="skin-card"><button class="skin-portrait" aria-label={`Ver ${skin.name}`} onclick={() => selectedSkin = skin}><canvas use:portrait={skin} aria-label={`Vista de cuerpo entero de ${skin.name}`}></canvas><span>{skin.model === "slim" ? "Slim · 3 px" : "Clásico · 4 px"}</span></button><div class="skin-details">{#if renaming === skin.id}<form class="rename-form" onsubmit={(event) => { event.preventDefault(); void renameSkin(skin); }}><input bind:value={renameName} aria-label="Nuevo nombre" maxlength="80"/><div><button class="subtle" type="submit" disabled={busy || !renameName.trim()}><UiIcon name="save" size={14}/>Guardar</button><button class="subtle" type="button" onclick={() => renaming = ""}><UiIcon name="close" size={14}/>Cancelar</button></div></form>{:else}<h4 title={skin.name}>{skin.name}</h4><small>{new Date(skin.created_at * 1000).toLocaleDateString()}</small>{#if skin.error}<p class="hint">{skin.error}</p>{/if}{/if}<button class="primary use-skin" onclick={() => useSkin(skin)} disabled={busy || !!skin.error}><UiIcon name={profile?"check":"eye"} size={16}/>{profile ? "Usar y sincronizar" : "Ver skin"}</button><div class="card-actions"><button class="subtle" onclick={() => { renaming = skin.id; renameName = skin.name; }} disabled={busy}><UiIcon name="sliders" size={14}/>Renombrar</button><button class="subtle danger" onclick={() => deleteSkin(skin)} disabled={busy}><UiIcon name="trash" size={14}/>Eliminar</button></div></div></article>{/each}</div>{/if}
  </section>

  <section class="glass capes-section"><div class="section-heading"><div><span class="eyebrow">COLECCIÓN OFICIAL</span><h3>Capas y élitros</h3><p>Vista del diseño recortada desde la textura original.</p></div>{#if profile?.capes.some(cape => cape.state === "ACTIVE")}<button class="outline" onclick={() => changeCape()} disabled={busy}><UiIcon name="eye-off" size={16}/>Ocultar capa</button>{/if}</div>
    {#if profile?.capes.length}<div class="capes-grid">{#each profile.capes as cape (cape.id)}<article class:equipped={cape.state === "ACTIVE"} class="cape-card">{#if cape.url}<CapePreview url={cape.url} name={cape.alias ?? "Capa oficial"}/>{:else}<div class="empty-cape">Sin textura disponible</div>{/if}<div class="cape-label"><strong>{cape.alias ?? "Capa oficial"}</strong><small>{cape.state === "ACTIVE" ? "Equipada" : "Disponible en tu cuenta"}</small></div><button class={cape.state === "ACTIVE" ? "outline" : "primary"} onclick={() => changeCape(cape.id)} disabled={busy || cape.state === "ACTIVE"}><UiIcon name={cape.state==="ACTIVE"?"check":"shield"} size={16}/>{cape.state === "ACTIVE" ? "Capa activa" : "Equipar capa"}</button></article>{/each}</div>{:else}<div class="empty-state"><strong>{profile ? "Esta cuenta no tiene capas oficiales" : "Conecta tu cuenta para ver tus capas"}</strong><p>Las capas disponibles se consultan directamente desde tu perfil de Minecraft.</p></div>{/if}
  </section>
</div>

<style>
.skin-providers{display:flex;align-items:center;justify-content:center;flex-wrap:wrap;gap:4px;color:#a99ab7;font-size:10px;line-height:1.6}.skin-providers button{padding:0;border:0;background:none;color:#cda1f4;font:inherit;text-decoration:underline;text-underline-offset:2px;cursor:pointer}

  .account-workspace { display: grid; gap: 24px; min-width: 0; padding-bottom: 24px; color: #e6eceb; }
  .page-heading, .section-heading { display: flex; align-items: center; justify-content: space-between; gap: 18px; flex-wrap: wrap; }
  .page-heading h2 { font: 800 clamp(1.65rem,3vw,2.25rem)/1.15 var(--font-display,'Consolas',monospace); margin: 11px 0; letter-spacing: -.045em; color: #f1f6f4; }
  .eyebrow { font: 700 .61rem var(--font-display,'Consolas',monospace); letter-spacing: .15em; color: var(--primary,#7fdbac); }
  .account-workspace p { line-height: 1.65; color: #a5b3b4; font-size: .81rem; margin: 8px 0 0; }
  .glass { position: relative; min-width: 0; padding: 24px; border: 1px solid #ffffff19; border-radius: 6px; background: rgba(var(--bg-base-rgb,12,20,20),.83); backdrop-filter: blur(var(--blur-px,12px)); box-shadow: inset 0 1px #ffffff05,0 8px 25px #0001; }
  .account-summary { display: flex; align-items: center; justify-content: space-between; gap: 22px; flex-wrap: wrap; border-left: 3px solid var(--primary,#7fdbac); background: linear-gradient(105deg,rgba(var(--primary-rgb,77,205,142),.09),transparent),rgba(var(--bg-base-rgb,12,20,20),.86); }
  .account-identity { display: flex; gap: 16px; align-items: center; flex: 1; min-width: 230px; }
  .account-symbol { flex-shrink: 0; display: grid; place-items: center; width: 53px; height: 53px; color: var(--primary,#7fdbac); border: 1px solid rgba(var(--primary-rgb,77,205,142),.28); border-radius: 4px; background: rgba(var(--primary-rgb,77,205,142),.09); box-shadow: inset 2px 2px #ffffff07,inset -2px -2px #0004; }
  .account-workspace h3 { font: 700 1.04rem/1.45 var(--font-display,'Consolas',monospace); margin: 0; color: #eaf2ee; }
  .account-workspace button, .account-workspace input { font-family: inherit; }
  .account-workspace button { cursor: pointer; transition: background .18s,border-color .18s,box-shadow .18s; }
  .account-workspace button:disabled { opacity: .45; cursor: default; }
  .account-workspace button:focus-visible, .account-workspace input:focus-visible, .drop-zone:focus-within { outline: 2px solid var(--primary,#7fdbac); outline-offset: 3px; }
  .primary, .outline { padding: 11px 15px; border-radius: 4px; font-size: .76rem; font-weight: 650; line-height: 1.45; }
  .primary { border: 1px solid rgba(var(--primary-rgb,77,205,142),.55); background: rgba(var(--primary-rgb,77,205,142),.18); color: #ecf5ef; box-shadow: inset 0 2px #ffffff0b,inset 0 -2px #0004; }
  .primary:hover:enabled { background: rgba(var(--primary-rgb,77,205,142),.28); border-color: var(--primary,#7fdbac); }
  .outline { color: #c2ccca; background: #ffffff04; border: 1px solid #ffffff25; box-shadow: inset 0 1px #ffffff05; }
  .outline:hover:enabled { border-color: rgba(var(--primary-rgb,77,205,142),.6); background: rgba(var(--primary-rgb,77,205,142),.07); }
  .auth-actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .account-note { flex-basis: 100%; border-top: 1px solid #ffffff12; padding-top: 13px; }
  .discovery-banner { position: relative; display: flex; align-items: center; gap: 21px; padding: 25px; border: 1px solid rgba(var(--primary-rgb,77,205,142),.3); border-radius: 6px; overflow: hidden; background: linear-gradient(110deg,rgba(var(--primary-rgb,77,205,142),.13),transparent),rgba(var(--bg-base-rgb,12,20,20),.82); }
  .discovery-banner:after { content: ''; position: absolute; right: 45px; bottom: -90px; width: 155px; height: 155px; border: 20px solid rgba(var(--primary-rgb,77,205,142),.05); pointer-events: none; }
  .discovery-icon { display: grid; place-items: center; flex-shrink: 0; width: 68px; height: 68px; border: 1px solid rgba(var(--primary-rgb,77,205,142),.28); color: var(--primary,#7fdbac); background: rgba(var(--primary-rgb,77,205,142),.1); box-shadow: inset 3px 3px #ffffff07,inset -3px -3px #0004; }
  .discovery-copy { flex: 1; min-width: 0; }
  .discovery-copy h3 { margin-top: 8px; font-size: 1.12rem; }
  .discovery-copy p { font-size: .77rem; }
  .discovery-copy ol { display: flex; flex-wrap: wrap; gap: 8px 23px; counter-reset: skin-steps; padding: 0; margin: 17px 0 0; list-style: none; }
  .discovery-copy li { display: flex; align-items: center; gap: 7px; font-size: .68rem; color: #9cacaa; }
  .discovery-copy li:before { counter-increment: skin-steps; content: counter(skin-steps); display: grid; place-items: center; width: 18px; height: 18px; border: 1px solid rgba(var(--primary-rgb,77,205,142),.25); background: #ffffff05; color: var(--primary,#7fdbac); font: 650 .61rem var(--font-display,'Consolas',monospace); }
  .discovery-action { position: relative; z-index: 1; display: flex; justify-content: center; gap: 13px; align-items: center; flex-shrink: 0; }
  .identity-grid { display: grid; grid-template-columns: minmax(260px,1fr) minmax(300px,1fr); gap: 24px; }
  .viewer-card { overflow: hidden; background: linear-gradient(180deg,rgba(var(--primary-rgb,77,205,142),.04),transparent 45%),rgba(var(--bg-base-rgb,12,20,20),.82); }
  .viewer-card .section-heading { border-bottom: 1px solid #ffffff10; padding-bottom: 15px; }
  .viewer-card :global(.skin-viewer-wrap) { height: 450px; }
  .viewer-card .hint { font-size: .7rem; text-align: center; border-top: 1px solid #ffffff0d; padding-top: 13px; }
  .import-card { display: grid; align-content: start; gap: 19px; }
  .import-card>div:first-child { padding-bottom: 3px; }
  .import-card>div:first-child h3 { margin-top: 9px; }
  .field { display: grid; gap: 9px; color: #bdc9c7; font-size: .73rem; font-weight: 600; }
  .field input, .rename-form input { width: 100%; box-sizing: border-box; min-width: 0; padding: 12px; border: 1px solid #ffffff23; border-radius: 4px; background: rgba(var(--bg-base-rgb,12,20,20),.7); color: #e6efeb; font-size: .8rem; }
  .field input::placeholder { color: #81938e; }
  .segmented { display: flex; padding: 4px; gap: 4px; border: 1px solid #ffffff17; border-radius: 4px; background: #0002; }
  .segmented button { flex: 1; padding: 11px; border: 1px solid transparent; border-radius: 3px; color: #95a6a0; background: transparent; font-size: .76rem; }
  .segmented button.active { background: rgba(var(--primary-rgb,77,205,142),.13); border-color: rgba(var(--primary-rgb,77,205,142),.45); color: var(--primary,#7fdbac); box-shadow: inset 0 -2px rgba(var(--primary-rgb,77,205,142),.4); }
  .segmented small { font-size: .64rem; opacity: .7; margin-left: 6px; }
  .drop-zone { display: flex; flex-direction: column; align-items: center; justify-content: center; position: relative; gap: 9px; min-height: 145px; padding: 22px 12px; border: 1px dashed rgba(var(--primary-rgb,77,205,142),.4); border-radius: 4px; background: repeating-linear-gradient(45deg,#ffffff01 0 8px,transparent 8px 16px),rgba(var(--primary-rgb,77,205,142),.04); color: var(--primary,#7fdbac); cursor: pointer; text-align: center; transition: background .18s,border-color .18s; }
  .drop-zone.dragging, .drop-zone:hover { border-color: var(--primary,#7fdbac); background: rgba(var(--primary-rgb,77,205,142),.11); }
  .drop-zone strong { font: 650 .84rem var(--font-display,'Consolas',monospace); color: #dbe9e3; }
  .drop-zone span { font-size: .72rem; color: #a2b1ac; }
  .drop-zone small { font-size: .65rem; color: #81978d; }
  .drop-zone input { position: absolute; width: 1px; height: 1px; opacity: 0; overflow: hidden; }
  .import-nick { display: flex; align-items: end; gap: 10px; }
  .import-nick .field { flex: 1; min-width: 0; }
  .import-nick button { padding: 12px 13px; }
  .save-current { width: 100%; }
  .notice { padding: 16px 18px; border-radius: 4px; border: 1px solid rgba(var(--primary-rgb,77,205,142),.3); background: rgba(var(--primary-rgb,77,205,142),.08); color: #c0e5d3; font-size: .8rem; line-height: 1.6; }
  .notice.error { border-color: #fda4af45; background: #c52d4e16; color: #f7b3c0; }
  .count { display: inline-flex; justify-content: center; min-width: 20px; font: 650 .67rem var(--font-display,'Consolas',monospace); padding: 3px 6px; background: rgba(var(--primary-rgb,77,205,142),.12); border: 1px solid rgba(var(--primary-rgb,77,205,142),.2); color: var(--primary,#7fdbac); border-radius: 3px; margin-left: 8px; vertical-align: middle; }
  .wardrobe-grid { display: grid; grid-template-columns: repeat(auto-fill,minmax(170px,1fr)); gap: 16px; margin-top: 23px; }
  .skin-card { min-width: 0; overflow: hidden; border: 1px solid #ffffff18; border-radius: 5px; background: rgba(var(--bg-base-rgb,12,20,20),.65); transition: border-color .18s; }
  .skin-card:hover { border-color: rgba(var(--primary-rgb,77,205,142),.35); }
  .skin-card.selected { border-color: var(--primary,#7fdbac); box-shadow: inset 0 3px var(--primary,#7fdbac); }
  .skin-portrait { position: relative; height: 222px; width: 100%; display: grid; place-items: center; border: none; padding: 25px 0; background: linear-gradient(180deg,transparent,rgba(var(--primary-rgb,77,205,142),.075)),repeating-linear-gradient(0deg,transparent 0 23px,#ffffff02 23px 24px),repeating-linear-gradient(90deg,transparent 0 23px,#ffffff02 23px 24px); }
  .skin-portrait canvas { height: 170px; width: auto; image-rendering: pixelated; filter: drop-shadow(0 8px 4px #0007); }
  .skin-portrait span { position: absolute; right: 9px; top: 10px; color: #9eafa9; font-size: .58rem; padding: 5px 6px; border-radius: 3px; background: rgba(var(--bg-base-rgb,12,20,20),.85); border: 1px solid #ffffff15; }
  .skin-details { padding: 14px; border-top: 1px solid #ffffff11; }
  .skin-details h4 { font: 650 .78rem var(--font-display,'Consolas',monospace); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin: 0 0 7px; color: #deeae3; }
  .skin-details>small { font-size: .65rem; color: #879d91; }
  .use-skin { width: 100%; margin-top: 14px; padding: 9px 7px; font-size: .71rem; }
  .card-actions { display: flex; justify-content: space-between; margin-top: 9px; gap: 6px; }
  .subtle { border: 0; background: transparent; color: #a0afa9; font-size: .68rem; padding: 6px 0; cursor: pointer; }
  .subtle:hover:enabled { color: var(--primary,#7fdbac); }
  .subtle.danger { color: #d28e92; }
  .rename-form input { font-size: .74rem; padding: 8px; }
  .rename-form div { display: flex; gap: 13px; }
  .empty-state { text-align: center; padding: 37px 18px; color: #bccdc3; font-size: .83rem; border: 1px dashed #ffffff22; border-radius: 4px; margin-top: 23px; }
  .empty-state strong { font-family: var(--font-display,'Consolas',monospace); }
  .empty-state p { font-size: .76rem; }
  .capes-grid { display: grid; grid-template-columns: repeat(auto-fill,minmax(180px,1fr)); gap: 16px; margin-top: 23px; }
  .cape-card { border: 1px solid #ffffff18; padding: 14px; border-radius: 5px; background: rgba(var(--bg-base-rgb,12,20,20),.65); display: grid; gap: 14px; }
  .cape-card.equipped { border-color: rgba(var(--primary-rgb,77,205,142),.6); background: rgba(var(--primary-rgb,77,205,142),.065); box-shadow: inset 0 3px var(--primary,#7fdbac); }
  .cape-label { display: grid; gap: 6px; text-align: center; padding-top: 3px; }
  .cape-label strong { font: 650 .8rem var(--font-display,'Consolas',monospace); }
  .cape-label small { color: #91a79b; font-size: .66rem; }
  .cape-card button { width: 100%; font-size: .72rem; padding: 10px; }
  .empty-cape { height: 182px; display: grid; place-items: center; font-size: .74rem; color: #8ba093; }
  .section-heading p { font-size: .75rem; }
  .wardrobe-section, .capes-section { padding: 24px; }
  .page-heading>.outline { align-self: center; font-family: var(--font-display,'Consolas',monospace); }
  @media(max-width:1080px) { .discovery-banner { flex-wrap: wrap; } .discovery-action { margin-left: 89px; } }
  @media(max-width:950px) { .identity-grid { grid-template-columns: 1fr; } .viewer-card :global(.skin-viewer-wrap) { height: 400px; } .glass,.wardrobe-section,.capes-section { padding: 20px; } }
  @media(max-width:600px) { .discovery-banner { padding: 20px; gap: 15px; } .discovery-icon { width: 46px; height: 46px; } .discovery-action { margin-left: 0; width: 100%; } .discovery-copy { min-width: 180px; } .discovery-copy h3 { font-size: .98rem; } .discovery-copy ol { display: grid; } .account-identity { min-width: 0; align-items: start; } .account-symbol { width: 42px; height: 42px; } .account-identity h3 { font-size: .9rem; } .import-nick { flex-wrap: wrap; } .import-nick .field { min-width: 180px; } }
  @media(prefers-reduced-motion:reduce) { .account-workspace button,.drop-zone,.skin-card { transition: none; } }
</style>
