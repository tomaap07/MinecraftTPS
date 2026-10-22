<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import SkinModelViewer from "$lib/SkinModelViewer.svelte";

  // Navegación de pestañas
  let activeTab = $state<"jugar" | "instalaciones" | "mods" | "skins" | "ajustes" | "consola">("jugar");

  // Estados del Launcher
  let username = $state("TomasTPS");
  let selectedVersion = $state("1.21.1 - Vanilla");
  let ramGb = $state(4);
  let javaStatus = $state("Verificando Java...");
  let javaOk = $state(false);
  let launchMessage = $state("");
  let isLaunching = $state(false);
  let installingJava = $state(false);
  let progress = $state<{ task: string; percent: number; status: string } | null>(null);
  let rootPath = $state("");
  let mods = $state<string[]>([]);
  let javaInfo = $state<{ installed: boolean; version: string; path: string; is_portable: boolean } | null>(null);
  let showOldVersions = $state(false);
  let versionCatalog = $state<{ id: string; kind: string }[]>([]);
  let installationCategory = $state<"release" | "snapshot" | "old">("release");
  let installedVersions = $state<{ id: string; jar: boolean; json: boolean; size_mb: number }[]>([]);
  let consoleLines = $state<{ timestamp: number; time: string; stream: string; line: string }[]>([]);
  let consoleScrollElement = $state<HTMLDivElement | undefined>();
  let consoleMessage = $state("");
  let javaRuntimes = $state<{ major: number; installed: boolean; enabled: boolean; path: string }[]>([]);
  let iconMessage = $state("");
  let iconPreview = $state("");
  let javaManagerMessage = $state("");
  let recommendedJavaMajor = $state<number | null>(null);
  type MicrosoftProfile = { id: string; name: string; skins: { id: string; state?: string; url?: string; variant?: string }[]; capes: { id: string; alias?: string; url?: string; state?: string }[] };
  let microsoftProfile = $state<MicrosoftProfile | null>(null);
  let loginBusy = $state(false);
  let loginMessage = $state("");
  let skinVariant = $state<"classic" | "slim">("classic");
  let versionSearch = $state("");
  let displayedVersions = $derived(versionCatalog.filter((entry) => entry.id.toLowerCase().includes(versionSearch.toLowerCase())));
  let filteredInstalledVersions = $derived(installedVersions.filter((installed) => {
    const kind = versionCatalog.find((version) => version.id === installed.id)?.kind;
    return installationCategory === "old" ? kind === "old_alpha" || kind === "old_beta" : kind === installationCategory;
  }));
  let progressTimer: ReturnType<typeof setTimeout> | undefined;
  let javaRequirementRequest = 0;
  let skinUsername = $derived(encodeURIComponent(username.trim() || "Steve"));
  let playerAvatarUrl = $derived(microsoftProfile ? `https://mc-heads.net/avatar/${encodeURIComponent(microsoftProfile.id)}/64` : "https://mc-heads.net/avatar/MHF_Steve/64");
  let javaRecommendationStatus = $derived.by(() => {
    if (recommendedJavaMajor === null) return "Consultando requisito…";
    const minimum = Math.max(8, recommendedJavaMajor);
    const compatible = javaRuntimes.filter((runtime) => runtime.installed && runtime.enabled && (minimum === 8 ? runtime.major === 8 : runtime.major >= minimum)).sort((a, b) => a.major - b.major)[0];
    return compatible ? `Java ${compatible.major} disponible` : `Se instalará Java ${minimum} al iniciar`;
  });
  let activeSkinUrl = $derived(microsoftProfile?.skins.find((skin) => skin.state === "ACTIVE")?.url ?? `https://mc-heads.net/skin/${skinUsername}`);
  let activeCapeUrl = $derived(microsoftProfile?.capes.find((cape) => cape.state === "ACTIVE")?.url ?? "");
  let activeSkinModel = $derived(microsoftProfile?.skins.find((skin) => skin.state === "ACTIVE")?.variant?.toLowerCase() === "slim" ? "slim" as const : "auto-detect" as const);
  let consoleText = $derived(consoleLines.map((entry) => `[${entry.time}] [${entry.stream}] ${entry.line}`).join("\n"));
  let recentConsoleLines = $derived(consoleLines.slice(-12));

  // Los 16 Temas de YouTubePersonal / YouTubeTPS
  interface ThemePreset {
    id: string;
    name: string;
    primary: string;
    secondary: string;
    bgBase: string;
  }

  const themes: ThemePreset[] = [
    { id: "amatista", name: "Amatista Real", primary: "#c084fc", secondary: "#9333ea", bgBase: "#0f071e" },
    { id: "nebula", name: "Nébula Violeta", primary: "#e879f9", secondary: "#c026d3", bgBase: "#160824" },
    { id: "indigo", name: "Índigo Profundo", primary: "#818cf8", secondary: "#4f46e5", bgBase: "#090d1f" },
    { id: "carmesi", name: "Cristal Carmesí", primary: "#ff4d6d", secondary: "#e11d48", bgBase: "#1a080c" },
    { id: "cian", name: "Cian Técnico", primary: "#00f2fe", secondary: "#0284c7", bgBase: "#06131f" },
    { id: "esmeralda", name: "Verde Esmeralda", primary: "#34d399", secondary: "#10b981", bgBase: "#071712" },
    { id: "ambar", name: "Ámbar Dorado", primary: "#fbbf24", secondary: "#d97706", bgBase: "#191206" },
    { id: "zafiro", name: "Azul Zafiro", primary: "#38bdf8", secondary: "#2563eb", bgBase: "#071120" },
    { id: "sakura", name: "Cuarzo Sakura", primary: "#f472b6", secondary: "#db2777", bgBase: "#1c0a14" },
    { id: "synthwave", name: "Ocaso Synthwave", primary: "#fb7185", secondary: "#8b5cf6", bgBase: "#14081c" },
    { id: "aurora", name: "Aurora Boreal", primary: "#2dd4bf", secondary: "#0d9488", bgBase: "#061917" },
    { id: "artico", name: "Hielo Ártico", primary: "#7dd3fc", secondary: "#0ea5e9", bgBase: "#08141e" },
    { id: "ororosa", name: "Oro Rosa", primary: "#fda4af", secondary: "#f43f5e", bgBase: "#1a0c10" },
    { id: "matrix", name: "Bosque Matrix", primary: "#4ade80", secondary: "#16a34a", bgBase: "#051408" },
    { id: "obsidiana", name: "Obsidiana Neutro", primary: "#e2e8f0", secondary: "#64748b", bgBase: "#090a0f" },
    { id: "custom", name: "Paleta a Medida", primary: "#10b981", secondary: "#059669", bgBase: "#0b0f19" }
  ];

  // Configuración reactiva de color y transparencias
  let selectedThemeId = $state("esmeralda");
  let primaryColor = $state("#34d399");
  let secondaryColor = $state("#10b981");
  let bgBaseColor = $state("#071712");

  // Opacidades independientes y desenfoque (Glassmorphism)
  let transparencyPreset = $state<"fantasma" | "equilibrado" | "lectura" | "solido">("equilibrado");
  let bgOpacity = $state(0.88);
  let menuOpacity = $state(0.75);
  let barOpacity = $state(0.85);
  let cardOpacity = $state(0.65);
  let blurPx = $state(14);

  // Convierte HEX a RGB para usar en rgba() con transparencias dinámicas
  function hexToRgb(hex: string): string {
    const cleanHex = hex.replace("#", "");
    const num = parseInt(cleanHex, 16);
    const r = (num >> 16) & 255;
    const g = (num >> 8) & 255;
    const b = num & 255;
    return `${r}, ${g}, ${b}`;
  }

  function selectTheme(theme: ThemePreset) {
    selectedThemeId = theme.id;
    if (theme.id !== "custom") {
      primaryColor = theme.primary;
      secondaryColor = theme.secondary;
      bgBaseColor = theme.bgBase;
    }
    saveConfig();
  }

  function handleCustomColorChange() {
    selectedThemeId = "custom";
    saveConfig();
  }

  function applyTransparencyMode(mode: "fantasma" | "equilibrado" | "lectura" | "solido") {
    transparencyPreset = mode;
    if (mode === "fantasma") {
      bgOpacity = 0.55;
      menuOpacity = 0.35;
      barOpacity = 0.45;
      cardOpacity = 0.30;
      blurPx = 22;
    } else if (mode === "equilibrado") {
      bgOpacity = 0.85;
      menuOpacity = 0.72;
      barOpacity = 0.80;
      cardOpacity = 0.60;
      blurPx = 14;
    } else if (mode === "lectura") {
      bgOpacity = 0.94;
      menuOpacity = 0.88;
      barOpacity = 0.92;
      cardOpacity = 0.82;
      blurPx = 8;
    } else if (mode === "solido") {
      bgOpacity = 1.0;
      menuOpacity = 1.0;
      barOpacity = 1.0;
      cardOpacity = 0.98;
      blurPx = 0;
    }
    saveConfig();
  }

  function saveConfig() {
    const config = {
      selectedThemeId,
      primaryColor,
      secondaryColor,
      bgBaseColor,
      transparencyPreset,
      bgOpacity,
      menuOpacity,
      barOpacity,
      cardOpacity,
      blurPx,
      username,
      selectedVersion,
      ramGb
    };
    localStorage.setItem("minecraft_tps_config", JSON.stringify(config));
  }

  function loadConfig() {
    const saved = localStorage.getItem("minecraft_tps_config");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        selectedThemeId = parsed.selectedThemeId ?? "esmeralda";
        primaryColor = parsed.primaryColor ?? "#34d399";
        secondaryColor = parsed.secondaryColor ?? "#10b981";
        bgBaseColor = parsed.bgBaseColor ?? "#071712";
        transparencyPreset = parsed.transparencyPreset ?? "equilibrado";
        bgOpacity = parsed.bgOpacity ?? 0.85;
        menuOpacity = parsed.menuOpacity ?? 0.72;
        barOpacity = parsed.barOpacity ?? 0.80;
        cardOpacity = parsed.cardOpacity ?? 0.60;
        blurPx = parsed.blurPx ?? 14;
        username = parsed.username ?? "TomasTPS";
        selectedVersion = parsed.selectedVersion ?? "1.21.1 - Vanilla";
        ramGb = parsed.ramGb ?? 4;
      } catch (e) {
        console.error("Error cargando configuración:", e);
      }
    }
  }

  onMount(() => {
    let unlisten: (() => void) | undefined;
    let unlistenConsole: (() => void) | undefined;
    void (async () => {
    loadConfig();
    unlistenConsole = await listen<{ timestamp: number; stream: string; line: string }>("launcher-log", (event) => {
      addConsoleLine(event.payload.stream, event.payload.line, event.payload.timestamp);
    });
    try {
      const saved = await invoke<{ timestamp: number; stream: string; line: string }[]>("read_launcher_logs");
      const merged = [...saved.map((entry) => ({ ...entry, time: new Date(entry.timestamp).toLocaleTimeString() })), ...consoleLines];
      consoleLines = [...new Map(merged.map((entry) => [`${entry.timestamp}:${entry.stream}:${entry.line}`, entry])).values()].slice(-10000);
    } catch (err) { addConsoleLine("stderr", `No se pudo restaurar la consola: ${String(err)}`); }
    void loadVersionCatalog(false);
    void refreshInstallations();
    void refreshJavaRuntimes();
    void invoke<string | null>("get_custom_app_icon").then((icon) => { if (icon) iconPreview = icon; }).catch(console.error);
    void invoke<MicrosoftProfile | null>("get_microsoft_profile").then((profile) => { if (profile) { microsoftProfile = profile; username = profile.name; saveConfig(); } }).catch((err) => { loginMessage = String(err); const line = `No se pudo restaurar la cuenta Microsoft: ${String(err)}`; void invoke("record_launcher_log", { stream: "stderr", line }).catch(() => addConsoleLine("stderr", line)); });
    unlisten = await listen<{ task: string; percent: number; status: string }>("download-progress", (event) => {
      progress = event.payload;
      if (event.payload.percent >= 100) {
        clearTimeout(progressTimer);
        progressTimer = setTimeout(() => (progress = null), 3500);
      }
    });
    void invoke<string>("get_root_path").then((p) => (rootPath = p)).catch(console.error);
    void refreshMods();
    try {
      const res = await invoke<{ installed: boolean; version: string; path: string; is_portable: boolean }>("check_java");
      javaInfo = res;
      javaStatus = `${res.version}${res.is_portable ? " · Portable" : " · Sistema"}`;
      javaOk = true;
    } catch (err) {
      javaStatus = String(err);
      javaOk = false;
    }
    void refreshJavaRequirement();
    })();
    return () => { unlisten?.(); unlistenConsole?.(); clearTimeout(progressTimer); };
  });

  $effect(() => {
    consoleLines.length;
    if (activeTab === "consola" && consoleScrollElement) {
      queueMicrotask(() => { if (consoleScrollElement) consoleScrollElement.scrollTop = consoleScrollElement.scrollHeight; });
    }
  });

  async function refreshMods() {
    try { mods = await invoke<string[]>("list_mods"); } catch (err) { launchMessage = String(err); }
  }

  async function refreshInstallations() {
    try { installedVersions = await invoke("list_installed_versions"); } catch (err) { launchMessage = String(err); }
  }

  async function copyConsole() {
    try {
      await navigator.clipboard.writeText(consoleText);
      consoleMessage = "Consola copiada al portapapeles.";
    } catch (err) { consoleMessage = `No se pudo copiar: ${String(err)}`; }
  }

  function clearConsole() {
    consoleLines = [];
    consoleMessage = "Consola limpiada.";
    void invoke("clear_launcher_logs").catch((err) => { consoleMessage = `No se pudo limpiar el registro: ${String(err)}`; });
  }

  function addConsoleLine(stream: string, line: string, timestamp = Date.now()) {
    consoleLines = [...consoleLines, { timestamp, time: new Date(timestamp).toLocaleTimeString(), stream, line }].slice(-10000);
  }

  async function refreshJavaRequirement() {
    const version = selectedVersion.split(" ")[0];
    if (!version) return;
    const request = ++javaRequirementRequest;
    try {
      const major = await invoke<number>("get_version_java_requirement", { version });
      if (request === javaRequirementRequest) recommendedJavaMajor = major;
    } catch {
      if (request === javaRequirementRequest) recommendedJavaMajor = null;
    }
  }

  function openJavaSettings() {
    activeTab = "ajustes";
    setTimeout(() => document.getElementById("java-manager")?.scrollIntoView({ behavior: "smooth", block: "start" }), 80);
  }

  async function refreshJavaRuntimes() {
    try { javaRuntimes = await invoke("list_java_runtimes"); } catch (err) { javaManagerMessage = String(err); }
  }

  async function toggleJava(major: number, enabled: boolean) {
    try { await invoke("set_java_runtime_enabled", { major, enabled }); await refreshJavaRuntimes(); }
    catch (err) { javaManagerMessage = String(err); }
  }

  async function installJavaVersion(major: number) {
    javaManagerMessage = `Instalando Java ${major}…`;
    try { javaManagerMessage = await invoke("install_portable_java", { major }); await refreshJavaRuntimes(); }
    catch (err) { javaManagerMessage = String(err); }
  }

  async function useCustomIcon(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0]; if (!file) return;
    try {
      const bitmap = await createImageBitmap(file);
      const canvas = document.createElement("canvas"); canvas.width = 128; canvas.height = 128;
      const ctx = canvas.getContext("2d"); if (!ctx) throw new Error("No se pudo procesar la imagen");
      const scale = Math.min(128 / bitmap.width, 128 / bitmap.height);
      const w = Math.round(bitmap.width * scale), h = Math.round(bitmap.height * scale);
      ctx.drawImage(bitmap, (128 - w) / 2, (128 - h) / 2, w, h);
      const png = canvas.toDataURL("image/png");
      iconPreview = png;
      iconMessage = await invoke("set_custom_app_icon", { imageBase64: png });
    } catch (err) { iconMessage = `No se pudo cambiar el icono: ${String(err)}`; }
    finally { input.value = ""; }
  }

  async function playInstalled(id: string) {
    selectedVersion = id;
    await handleLaunch();
  }

  async function removeInstalled(id: string) {
    try { await invoke("delete_installed_version", { version: id }); await refreshInstallations(); }
    catch (err) { launchMessage = String(err); }
  }

  async function connectMicrosoft() {
    loginBusy = true; loginMessage = "Abriendo el inicio de sesión seguro de Microsoft…";
    try {
      const profile = await invoke<MicrosoftProfile>("begin_microsoft_login");
      microsoftProfile = profile; username = profile.name; saveConfig();
      loginMessage = `Sesión iniciada como ${profile.name}.`;
    } catch (err) { loginMessage = String(err); const line = `Inicio de sesión Microsoft: ${String(err)}`; void invoke("record_launcher_log", { stream: "stderr", line }).catch(() => addConsoleLine("stderr", line)); }
    finally { loginBusy = false; }
  }

  async function signOutMicrosoft() {
    try { await invoke("logout_microsoft"); microsoftProfile = null; loginMessage = "Sesión cerrada."; }
    catch (err) { loginMessage = String(err); }
  }

  async function useOfflineMode() {
    await signOutMicrosoft();
    loginMessage = "Modo offline activado. Puedes jugar con el nick libre.";
  }

  async function uploadSkin(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0]; if (!file) return;
    try {
      const dataUrl = await new Promise<string>((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(String(reader.result)); reader.onerror = () => reject(reader.error); reader.readAsDataURL(file); });
      microsoftProfile = await invoke<MicrosoftProfile>("upload_minecraft_skin", { skinBase64: dataUrl, variant: skinVariant });
      loginMessage = "Skin actualizada en tu perfil Minecraft.";
    } catch (err) { loginMessage = String(err); }
    finally { input.value = ""; }
  }

  async function activateCape(capeId: string) {
    try { microsoftProfile = await invoke<MicrosoftProfile>("activate_minecraft_cape", { capeId }); loginMessage = "Capa actualizada."; }
    catch (err) { loginMessage = String(err); }
  }

  async function hideCape() {
    try { microsoftProfile = await invoke<MicrosoftProfile>("hide_minecraft_cape"); loginMessage = "Capa oculta."; }
    catch (err) { loginMessage = String(err); }
  }

  async function loadVersionCatalog(includeOld: boolean) {
    try {
      const entries = await invoke<{ id: string; kind: string }[]>("fetch_mojang_versions", { includeOld });
      versionCatalog = entries;
      if (!entries.some((entry) => entry.id === selectedVersion)) selectedVersion = entries.find((entry) => entry.kind === "release")?.id ?? selectedVersion;
      void refreshJavaRequirement();
    } catch (err) { launchMessage = `No se pudo cargar el manifiesto de Mojang: ${String(err)}`; }
  }

  function toggleOldVersions() {
    showOldVersions = !showOldVersions;
    void loadVersionCatalog(showOldVersions);
  }

  async function installJava() {
    installingJava = true;
    try {
      launchMessage = await invoke<string>("install_portable_java", { major: 21 });
      const info = await invoke<{ installed: boolean; version: string; path: string; is_portable: boolean }>("check_java");
      javaInfo = info; javaStatus = `${info.version} · Portable`; javaOk = info.installed;
    } catch (err) { javaStatus = String(err); launchMessage = `Error: ${String(err)}`; }
    finally { installingJava = false; }
  }

  async function handleLaunch() {
    saveConfig();
    isLaunching = true;
    launchMessage = "";
    try {
      const res = await invoke<string>("launch_game", {
        username,
        version: selectedVersion,
        ramGb
      });
      launchMessage = res;
    } catch (err) {
      launchMessage = "Error: " + String(err);
      void invoke("record_launcher_log", { stream: "stderr", line: launchMessage }).catch(() => addConsoleLine("stderr", launchMessage));
    } finally {
      await refreshInstallations();
      setTimeout(() => {
        isLaunching = false;
      }, 1500);
    }
  }
</script>

<main
  class="launcher"
  style="
    --primary: {primaryColor};
    --primary-rgb: {hexToRgb(primaryColor)};
    --secondary: {secondaryColor};
    --secondary-rgb: {hexToRgb(secondaryColor)};
    --bg-base-rgb: {hexToRgb(bgBaseColor)};
    --bg-opacity: {bgOpacity};
    --menu-opacity: {menuOpacity};
    --bar-opacity: {barOpacity};
    --card-opacity: {cardOpacity};
    --blur-px: {blurPx}px;
  "
>
  <!-- Barra Lateral -->
  <aside class="sidebar">
    <div>
      <div class="logo">
        {#if iconPreview}<img class="custom-logo" src={iconPreview} alt="Logo personalizado" />{/if}
        <span class="neon-text">MINECRAFT</span>
        <span class="badge">TPS</span>
      </div>

      <nav class="menu">
        <button class:active={activeTab === "jugar"} onclick={() => (activeTab = "jugar")}>
          Jugar
        </button>
        <button class:active={activeTab === "mods"} onclick={() => (activeTab = "mods")}>
          Mods
        </button>
        <button class:active={activeTab === "instalaciones"} onclick={() => { activeTab = "instalaciones"; void refreshInstallations(); }}>
          Instalaciones
        </button>
        <button class:active={activeTab === "skins"} onclick={() => (activeTab = "skins")}>
          Skins
        </button>
        <button class:active={activeTab === "consola"} onclick={() => (activeTab = "consola")}>
          Consola
        </button>
        <button class:active={activeTab === "ajustes"} onclick={() => (activeTab = "ajustes")}>
          Ajustes
        </button>
      </nav>
    </div>

    <div class="profile-quick-card">
      <img src={playerAvatarUrl} alt="Avatar de {microsoftProfile?.name ?? 'Steve'}" />
      <div class="profile-quick-info"><strong>{microsoftProfile?.name ?? (username.trim() || "Steve")}</strong><small>{microsoftProfile ? "Cuenta Microsoft" : "Modo Offline · Steve"}</small></div>
      <button class="install-java" onclick={openJavaSettings}>Cambiar versión Java</button>
    </div>
  </aside>

  <!-- Panel Central -->
  <section class="content">
    {#if activeTab === "jugar"}
      <div class="hero">
        <h1>Ecosistema <span>MinecraftTPS</span></h1>
        <p>Rendimiento nativo impulsado por Rust y Tauri v2.</p>

        <div class="quick-cards">
          <div class="info-card">
            <span class="card-label">PERFIL ACTIVO</span>
            <strong>{username}</strong>
          </div>
          <div class="info-card">
            <span class="card-label">VERSIÓN SELECCIONADA</span>
            <strong>{selectedVersion}</strong>
          </div>
          <div class="info-card">
            <span class="card-label">MEMORIA ASIGNADA</span>
            <strong>{ramGb} GB DDR4</strong>
          </div>
        </div>

      </div>

      {#if launchMessage}
        <div class="status-log">
          <code>> {launchMessage}</code>
        </div>
      {/if}

      <!-- Barra Inferior de Control Responsiva -->
      <footer class="bottom-bar">
        <div class="controls-wrapper">
          <div class="control-group">
            <label for="nick">Nick del Jugador{microsoftProfile ? " · Cuenta Microsoft" : " · Modo Offline"}</label>
            <input
              id="nick"
              type="text"
              bind:value={username}
              oninput={saveConfig}
              placeholder="Tu Nick..."
              disabled={!!microsoftProfile}
            />
          </div>

          <div class="control-group">
            <label for="version">Versión</label>
            {#if showOldVersions}<input class="version-search" aria-label="Buscar versión" placeholder="Buscar versión…" bind:value={versionSearch} />{/if}
            <select id="version" bind:value={selectedVersion} onchange={() => { saveConfig(); void refreshJavaRequirement(); }}>
              {#each displayedVersions as ver}
                <option value={ver.id}>{ver.id}{ver.kind === "snapshot" ? " · Snapshot" : ver.kind === "old_beta" ? " · Beta" : ver.kind === "old_alpha" ? " · Alpha" : " · Release"}</option>
              {/each}
            </select>
            <small class="java-recommendation">Java recomendado: {recommendedJavaMajor === null ? "Consultando…" : `Java ${Math.max(8, recommendedJavaMajor)}`} · {javaRecommendationStatus}</small>
            <button class="old-versions-toggle" onclick={toggleOldVersions}>{showOldVersions ? "Ocultar versiones antiguas" : "Mostrar versiones antiguas (Alpha y Beta)"}</button>
          </div>

          <div class="control-group">
            <label for="ram">Memoria RAM: <strong>{ramGb} GB</strong></label>
            <input
              id="ram"
              type="range"
              min="2"
              max="16"
              step="1"
              bind:value={ramGb}
              oninput={saveConfig}
            />
          </div>
        </div>

        <button class="play-btn" onclick={handleLaunch} disabled={isLaunching}>
          {isLaunching ? "PREPARANDO..." : "JUGAR AHORA"}
        </button>
      </footer>

      <section class="mini-console module-card">
        <div class="module-actions"><h3>Consola en vivo</h3><button onclick={() => (activeTab = "consola")}>Abrir consola completa</button></div>
        <div class="mini-console-output" aria-live="polite">
          {#if recentConsoleLines.length}{#each recentConsoleLines as entry}<div class:error-line={entry.stream === "stderr" || /error|exception|crash/i.test(entry.line)}><span>[{entry.time}] [{entry.stream}]</span> {entry.line}</div>{/each}{:else}<p>Los mensajes de descargas y del proceso de Minecraft aparecerán aquí.</p>{/if}
        </div>
      </section>

    {:else if activeTab === "instalaciones"}
      <div class="module-panel">
        <header><h2>Instalaciones de Minecraft</h2><p>Versiones descargadas en <code>{rootPath || "MinecraftTPS"}/versions</code></p></header>
        <div class="category-tabs">
          <button class:active={installationCategory === "release"} onclick={() => { installationCategory = "release"; showOldVersions = false; void loadVersionCatalog(false); }}>Releases</button>
          <button class:active={installationCategory === "snapshot"} onclick={() => { installationCategory = "snapshot"; showOldVersions = false; void loadVersionCatalog(false); }}>Snapshots</button>
          <button class:active={installationCategory === "old"} onclick={() => { installationCategory = "old"; showOldVersions = true; void loadVersionCatalog(true); }}>Betas y Alphas</button>
          <button disabled title="Se habilitarán cuando esté listo el instalador">Fabric</button><button disabled title="Se habilitarán cuando esté listo el instalador">Forge</button><button disabled title="Se habilitarán cuando esté listo el instalador">OptiFine</button>
        </div>
        <section class="module-card">
          <div class="module-actions"><h3>Versiones instaladas</h3><button onclick={refreshInstallations}>Actualizar</button></div>
          {#if filteredInstalledVersions.length}
            <div class="installed-list">{#each filteredInstalledVersions as item}
              <article class="installed-row"><div><strong>{item.id}</strong><small>{item.jar && item.json ? "Instalación completa (JAR + JSON)" : `Instalación incompleta · JAR ${item.jar ? "✓" : "✕"} · JSON ${item.json ? "✓" : "✕"}`} · {item.size_mb.toFixed(1)} MB</small></div><div class="row-buttons"><button onclick={() => playInstalled(item.id)}>Jugar</button><button onclick={() => invoke("open_version_folder", { version: item.id })}>Carpeta</button><button class="danger-button" onclick={() => removeInstalled(item.id)}>Eliminar</button></div></article>
            {/each}</div>
          {:else}<p>Aún no hay versiones descargadas. Selecciona una versión abajo y pulsa Descargar y jugar.</p>{/if}
        </section>
        <section class="module-card"><h3>Nueva instalación Vanilla</h3><p>El primer inicio descarga cliente, bibliotecas, sonidos e idiomas de la versión.</p>
          <div class="new-install-row"><select bind:value={selectedVersion} onchange={() => { saveConfig(); void refreshJavaRequirement(); }}>{#each versionCatalog.filter((v) => installationCategory === "old" ? v.kind === "old_alpha" || v.kind === "old_beta" : v.kind === installationCategory) as item}<option value={item.id}>{item.id}</option>{/each}</select><button class="primary-action" onclick={() => playInstalled(selectedVersion)}>Descargar y jugar</button></div>
        </section>
      </div>
    {:else if activeTab === "ajustes"}
      <!-- Panel de Configuración Estilo YouTubePersonal / YouTubeTPS -->
      <div class="settings-panel">
        <header class="settings-header">
          <h2>Personalización Visual y Transparencias</h2>
          <p>Motor estético unificado del ecosistema TPS (16 temas y Glassmorphism ajustable).</p>
        </header>

        <!-- 1. Selección de los 16 Temas -->
        <div class="settings-section">
          <h3>1. Galería de Temas (YouTubePersonal)</h3>
          <div class="themes-grid">
            {#each themes as theme}
              <button
                class="theme-chip"
                class:selected={selectedThemeId === theme.id}
                onclick={() => selectTheme(theme)}
              >
                <span
                  class="color-dot"
                  style="background: linear-gradient(135deg, {theme.id === 'custom' ? primaryColor : theme.primary}, {theme.id === 'custom' ? secondaryColor : theme.secondary});"
                ></span>
                <span class="theme-name">{theme.name}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- 2. Paleta a Medida (Colores Personalizados) -->
        <div class="settings-section">
          <h3>2. Colores Personalizados (Paleta a Medida)</h3>
          <div class="custom-colors-row">
            <div class="color-picker-item">
              <label for="col-primary">Acento Principal</label>
              <div class="picker-box">
                <input
                  id="col-primary"
                  type="color"
                  bind:value={primaryColor}
                  oninput={handleCustomColorChange}
                />
                <code>{primaryColor}</code>
              </div>
            </div>

            <div class="color-picker-item">
              <label for="col-secondary">Acento Secundario / Brillo</label>
              <div class="picker-box">
                <input
                  id="col-secondary"
                  type="color"
                  bind:value={secondaryColor}
                  oninput={handleCustomColorChange}
                />
                <code>{secondaryColor}</code>
              </div>
            </div>

            <div class="color-picker-item">
              <label for="col-bg">Fondo Base</label>
              <div class="picker-box">
                <input
                  id="col-bg"
                  type="color"
                  bind:value={bgBaseColor}
                  oninput={handleCustomColorChange}
                />
                <code>{bgBaseColor}</code>
              </div>
            </div>
          </div>
        </div>

        <!-- 3. Perfiles de Transparencia en 1 Clic -->
        <div class="settings-section">
          <h3>3. Perfiles Rápidos de Transparencia</h3>
          <div class="preset-modes">
            <button
              class:active={transparencyPreset === "fantasma"}
              onclick={() => applyTransparencyMode("fantasma")}
            >
              Fantasma
            </button>
            <button
              class:active={transparencyPreset === "equilibrado"}
              onclick={() => applyTransparencyMode("equilibrado")}
            >
              Equilibrado
            </button>
            <button
              class:active={transparencyPreset === "lectura"}
              onclick={() => applyTransparencyMode("lectura")}
            >
              Lectura
            </button>
            <button
              class:active={transparencyPreset === "solido"}
              onclick={() => applyTransparencyMode("solido")}
            >
              Sólido
            </button>
          </div>
        </div>

        <!-- 4. Ajustes Finos de Opacidad y Blur -->
        <div class="settings-section">
          <h3>4. Control Independiente de Capas y Glassmorphism</h3>
          <div class="sliders-grid">
            <div class="slider-item">
              <label for="op-bg">Opacidad del Fondo: <strong>{Math.round(bgOpacity * 100)}%</strong></label>
              <input
                id="op-bg"
                type="range"
                min="0.2"
                max="1"
                step="0.02"
                bind:value={bgOpacity}
                oninput={saveConfig}
              />
            </div>

            <div class="slider-item">
              <label for="op-menu">Opacidad de Menús / Lateral: <strong>{Math.round(menuOpacity * 100)}%</strong></label>
              <input
                id="op-menu"
                type="range"
                min="0.15"
                max="1"
                step="0.02"
                bind:value={menuOpacity}
                oninput={saveConfig}
              />
            </div>

            <div class="slider-item">
              <label for="op-bar">Opacidad Barra de Juego: <strong>{Math.round(barOpacity * 100)}%</strong></label>
              <input
                id="op-bar"
                type="range"
                min="0.2"
                max="1"
                step="0.02"
                bind:value={barOpacity}
                oninput={saveConfig}
              />
            </div>

            <div class="slider-item">
              <label for="op-card">Opacidad de Tarjetas: <strong>{Math.round(cardOpacity * 100)}%</strong></label>
              <input
                id="op-card"
                type="range"
                min="0.15"
                max="1"
                step="0.02"
                bind:value={cardOpacity}
                oninput={saveConfig}
              />
            </div>

            <div class="slider-item">
              <label for="blur-range">Desenfoque (Blur): <strong>{blurPx}px</strong></label>
              <input
                id="blur-range"
                type="range"
                min="0"
                max="32"
                step="1"
                bind:value={blurPx}
                oninput={saveConfig}
              />
            </div>
          </div>
        </div>
        <div class="settings-section" id="java-manager">
          <h3>Gestor de motores Java</h3>
          <p>Se detectan runtimes en PATH, JAVA_HOME, carpetas comunes y los runtimes del launcher. La selección respeta el requisito del manifiesto: Java 8 para versiones antiguas y Java 17, 21, 22, 25 o superior para versiones modernas y snapshots.</p>
          <div class="runtime-list">{#each javaRuntimes as runtime}
            <article class="runtime-row"><div><strong>{runtime.major === 0 ? "Java del Sistema" : `Java ${runtime.major}`}</strong><small>{runtime.installed ? runtime.path : "No instalado"}</small></div><label class="switch-label"><input type="checkbox" checked={runtime.enabled} disabled={runtime.major === 0} onchange={(event) => toggleJava(runtime.major, event.currentTarget.checked)} />{runtime.major === 0 ? "Disponible" : runtime.enabled ? "Activado" : "Desactivado"}</label>{#if runtime.major !== 0 && !runtime.installed}<button onclick={() => installJavaVersion(runtime.major)}>Instalar</button>{/if}</article>
          {/each}</div>
          {#if javaManagerMessage}<p class="inline-message">{javaManagerMessage}</p>{/if}
        </div>
        <div class="settings-section">
          <h3>Logo personalizado del launcher</h3>
          <p>Elige un PNG, JPG o ICO. La imagen se ajusta automáticamente a 128 × 128 y se aplica a la ventana del launcher.</p>
          <div class="icon-picker-row">{#if iconPreview}<img src={iconPreview} alt="Vista previa del icono" />{/if}<label class="primary-action file-picker">Elegir imagen<input type="file" accept="image/png,image/jpeg,image/x-icon,.ico" onchange={useCustomIcon} /></label></div>
          {#if iconMessage}<p class="inline-message">{iconMessage}</p>{/if}
          <small>Esto cambia el icono de esta aplicación. El icono incrustado dentro de javaw.exe/Minecraft requiere modificar y reconstruir ese ejecutable.</small>
        </div>
      </div>

    {:else if activeTab === "mods"}
      <div class="module-panel">
        <header><h2>Instalaciones / Mods</h2><p>Directorio del launcher: <code>{rootPath || "Cargando…"}</code></p></header>
        <section class="module-card">
          <div class="module-actions"><h3>Mods instalados</h3><button onclick={refreshMods}>Actualizar lista</button><button onclick={() => invoke("open_mods_folder")}>Abrir carpeta de Mods</button></div>
          {#if mods.length}<ul>{#each mods as mod}<li>{mod}</li>{/each}</ul>{:else}<p>No hay archivos .jar en la carpeta mods.</p>{/if}
        </section>
      </div>
    {:else if activeTab === "skins"}
      <div class="module-panel">
        <header><h2>Skin de {username || "Steve"}</h2><p>Vista previa de avatar y modelo del jugador.</p></header>
        <section class="module-card account-card"><h3>{microsoftProfile ? `Cuenta Premium: ${microsoftProfile.name}` : "Cuenta Microsoft Premium"}</h3>
          {#if microsoftProfile}<p>Autenticado con Microsoft/Xbox. La sesión se guarda cifrada en este dispositivo y se renueva al abrir el launcher.</p><button onclick={signOutMicrosoft}>Cerrar sesión</button>
            <div class="skin-upload-row"><select bind:value={skinVariant}><option value="classic">Modelo Classic</option><option value="slim">Modelo Slim</option></select><label class="primary-action file-picker">Subir skin PNG<input type="file" accept="image/png" onchange={uploadSkin} /></label><small>Formato 64×64 o 64×32.</small></div>
            <div class="cape-list"><h4>Capas de la cuenta</h4>{#if microsoftProfile.capes.length}{#each microsoftProfile.capes as cape}<button class:active={cape.state === "ACTIVE"} onclick={() => activateCape(cape.id)}>{cape.alias ?? "Capa"}{cape.state === "ACTIVE" ? " · Activa" : " · Usar"}</button>{/each}<button onclick={hideCape}>Ocultar capa</button>{:else}<p>Esta cuenta no tiene capas disponibles.</p>{/if}</div>
          {:else}<p>Se abrirá el navegador oficial de Microsoft. Al autorizar la cuenta, MinecraftTPS conectará Xbox y Minecraft Services automáticamente.</p><div class="auth-actions"><button class="primary-action" onclick={connectMicrosoft} disabled={loginBusy}>{loginBusy ? "Esperando a Microsoft…" : "Iniciar sesión con Microsoft"}</button><button onclick={useOfflineMode}>Usar modo offline</button></div>{/if}
          {#if loginMessage}<p class="inline-message">{loginMessage}</p>{/if}
        </section>
        <div class="skins-grid"><article class="module-card"><h3>Modelo 3D · vista 360°</h3><SkinModelViewer skinUrl={activeSkinUrl} capeUrl={activeCapeUrl} model={activeSkinModel} /></article><article class="module-card"><h3>Avatar</h3><img class="skin-avatar" src={playerAvatarUrl} alt="Avatar de {microsoftProfile?.name ?? 'Steve'}" /><p>{microsoftProfile ? "Avatar de tu perfil Premium." : "Vista previa de Steve para el modo Offline."}</p></article></div>
      </div>
    {:else if activeTab === "consola"}
      <div class="module-panel console-panel">
        <header><h2>Consola de MinecraftTPS</h2><p>Registro en vivo de descargas, runtime Java y salida estándar/error de Minecraft.</p></header>
        <section class="module-card console-card">
          <div class="module-actions"><h3>Salida del launcher y del juego</h3><div class="console-actions"><button onclick={copyConsole}>Copiar todo</button><button onclick={clearConsole}>Limpiar consola</button></div></div>
          <div class="console-output" bind:this={consoleScrollElement} aria-live="polite" role="log">
            {#if consoleLines.length}{#each consoleLines as entry}<div class:console-error={entry.stream === "stderr" || /error|exception|crash/i.test(entry.line)}><time>[{entry.time}]</time> <b>[{entry.stream}]</b> {entry.line}</div>{/each}{:else}<p>Consola vacía. Inicia Minecraft para ver el registro.</p>{/if}
          </div>
          {#if consoleMessage}<p class="inline-message">{consoleMessage}</p>{/if}
        </section>
      </div>
    {:else}
      <div class="placeholder-tab">
        <h2>Módulo en Construcción</h2>
        <p>Pronto conectaremos esta sección con el motor en Rust de MinecraftTPS.</p>
      </div>
    {/if}
  </section>
  {#if progress}
    <div class="download-progress" role="status"><div class="progress-copy"><strong>{progress.task}</strong><span>{progress.status} · {progress.percent}%</span></div><div class="progress-track"><div class="progress-fill" style="width: {progress.percent}%"></div></div></div>
  {/if}
</main>

<style>
  :global(html), :global(body) {
    margin: 0;
    font-family: 'Segoe UI', system-ui, sans-serif;
    background: transparent !important;
    background-color: transparent !important;
    color: #f1f5f9;
    user-select: none;
    overflow: hidden;
  }

  .launcher {
    display: flex;
    height: 100vh;
    background: rgba(var(--bg-base-rgb), var(--bg-opacity));
    transition: background 0.25s ease;
  }

  /* Barra Lateral con transparencia dinámica */
  .sidebar {
    width: 220px;
    min-width: 220px;
    background: rgba(var(--bg-base-rgb), var(--menu-opacity));
    backdrop-filter: blur(var(--blur-px));
    border-right: 1px solid rgba(var(--primary-rgb), 0.18);
    display: flex;
    flex-direction: column;
    padding: 24px 16px;
    justify-content: space-between;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 900;
    font-size: 1.2rem;
    letter-spacing: 1px;
    margin-bottom: 28px;
  }
  .custom-logo { width: 28px; height: 28px; object-fit: contain; border-radius: 6px; }

  .neon-text {
    color: var(--primary);
    text-shadow: 0 0 14px rgba(var(--primary-rgb), 0.55);
  }

  .badge {
    background: var(--primary);
    color: #090d16;
    font-size: 0.72rem;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 900;
  }

  .menu {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .menu button {
    background: transparent;
    border: 1px solid transparent;
    color: #94a3b8;
    text-align: left;
    padding: 11px 14px;
    border-radius: 8px;
    font-size: 0.92rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .menu button:hover,
  .menu button.active {
    background: rgba(var(--primary-rgb), 0.16);
    border-color: rgba(var(--primary-rgb), 0.35);
    color: var(--primary);
  }
  .menu button { transition: transform .18s ease, color .18s ease, background .18s ease, border-color .18s ease; }
  .menu button:hover { transform: translateX(3px); }

  .install-java, .module-actions button { margin-top: 9px; background: rgba(var(--primary-rgb), .16); color: var(--primary); border: 1px solid rgba(var(--primary-rgb), .55); border-radius: 7px; padding: 8px 10px; font-weight: 700; cursor: pointer; }
  .install-java:disabled { opacity: .6; cursor: wait; }
  .profile-quick-card { display: grid; grid-template-columns: 42px 1fr; align-items: center; gap: 10px; padding: 12px; border: 1px solid rgba(var(--primary-rgb), .35); border-radius: 10px; background: rgba(var(--primary-rgb), .08); }
  .profile-quick-card > img { width: 42px; height: 42px; image-rendering: pixelated; border-radius: 7px; background: rgba(15, 23, 42, .75); }
  .profile-quick-info { display: flex; min-width: 0; flex-direction: column; gap: 3px; }
  .profile-quick-info strong { overflow: hidden; text-overflow: ellipsis; }
  .profile-quick-info small, .java-recommendation { color: #aab7ca; font-size: .76rem; }
  .profile-quick-card .install-java { grid-column: 1 / -1; width: 100%; margin-top: 0; }
  .java-recommendation { display: block; margin-top: 7px; }
  #java-manager { scroll-margin-top: 18px; }

  /* Contenido Principal */
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 28px 32px;
    overflow-y: auto;
    min-width: 0;
  }

  .hero h1 {
    font-size: 2.3rem;
    margin: 0 0 6px 0;
  }

  .hero h1 span {
    color: var(--primary);
    text-shadow: 0 0 18px rgba(var(--primary-rgb), 0.35);
  }

  .hero p {
    color: #94a3b8;
    font-size: 1rem;
    margin: 0 0 24px 0;
  }

  .quick-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 14px;
  }

  .info-card {
    background: rgba(15, 23, 42, var(--card-opacity));
    backdrop-filter: blur(var(--blur-px));
    border: 1px solid rgba(var(--primary-rgb), 0.22);
    padding: 14px 16px;
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .card-label {
    font-size: 0.7rem;
    color: #94a3b8;
    letter-spacing: 0.6px;
  }

  .info-card strong {
    color: var(--primary);
    font-size: 1rem;
  }

  .status-log {
    background: rgba(0, 0, 0, 0.6);
    border-left: 4px solid var(--primary);
    padding: 12px 16px;
    border-radius: 6px;
    font-family: monospace;
    color: var(--primary);
    margin: 12px 0;
  }

  /* Barra Inferior corregida para que nunca se corte el botón */
  .bottom-bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: rgba(var(--bg-base-rgb), var(--bar-opacity));
    backdrop-filter: blur(var(--blur-px));
    padding: 16px 20px;
    border-radius: 14px;
    border: 1px solid rgba(var(--primary-rgb), 0.25);
  }

  .controls-wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
    flex: 1;
    min-width: 0;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 5px;
    flex: 1;
    min-width: 120px;
  }

  .control-group label {
    font-size: 0.75rem;
    color: #cbd5e1;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .control-group label strong {
    color: var(--primary);
  }

  .control-group input[type="text"],
  .control-group select {
    background: rgba(9, 13, 22, 0.85);
    border: 1px solid rgba(var(--primary-rgb), 0.35);
    color: white;
    padding: 8px 10px;
    border-radius: 8px;
    outline: none;
    font-size: 0.9rem;
  }

  .control-group input[type="range"],
  .slider-item input[type="range"] {
    accent-color: var(--primary);
    cursor: pointer;
  }
  .old-versions-toggle { margin-top: 2px; padding: 6px 8px; border-radius: 7px; border: 1px solid rgba(var(--primary-rgb), .35); background: rgba(var(--primary-rgb), .1); color: var(--primary); cursor: pointer; font-size: .72rem; }
  .version-search { background: rgba(9, 13, 22, .85); color: white; border: 1px solid rgba(var(--primary-rgb), .35); border-radius: 7px; padding: 7px 9px; }

  .play-btn {
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    color: #090d16;
    font-weight: 900;
    font-size: 1rem;
    padding: 14px 26px;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    box-shadow: 0 0 18px rgba(var(--primary-rgb), 0.45);
    transition: transform 0.15s, box-shadow 0.15s;
    white-space: nowrap;
  }

  .play-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 0 26px rgba(var(--primary-rgb), 0.75);
  }

  .play-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  /* Estilos del Panel de Configuración (YouTubePersonal) */
  .settings-panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding-bottom: 12px;
  }

  .settings-header h2 {
    margin: 0 0 4px 0;
    font-size: 1.5rem;
    color: var(--primary);
  }

  .settings-header p {
    margin: 0;
    font-size: 0.88rem;
    color: #94a3b8;
  }

  .settings-section {
    background: rgba(15, 23, 42, var(--card-opacity));
    backdrop-filter: blur(var(--blur-px));
    border: 1px solid rgba(var(--primary-rgb), 0.2);
    border-radius: 12px;
    padding: 14px 18px;
  }

  .settings-section h3 {
    margin: 0 0 12px 0;
    font-size: 0.92rem;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: #e2e8f0;
  }

  .themes-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(145px, 1fr));
    gap: 8px;
  }

  .theme-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(9, 13, 22, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #e2e8f0;
    padding: 8px 10px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
    transition: all 0.18s;
  }

  .theme-chip:hover {
    border-color: rgba(var(--primary-rgb), 0.5);
    transform: translateY(-1px);
  }

  .theme-chip.selected {
    background: rgba(var(--primary-rgb), 0.18);
    border-color: var(--primary);
    color: var(--primary);
    box-shadow: 0 0 12px rgba(var(--primary-rgb), 0.25);
  }

  .color-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .theme-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .custom-colors-row {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
  }

  .color-picker-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    min-width: 140px;
  }

  .color-picker-item label {
    font-size: 0.78rem;
    color: #cbd5e1;
  }

  .picker-box {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(9, 13, 22, 0.8);
    padding: 6px 10px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
  }

  .picker-box input[type="color"] {
    border: none;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    background: transparent;
  }

  .preset-modes {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(115px, 1fr));
    gap: 10px;
  }

  .preset-modes button {
    background: rgba(9, 13, 22, 0.75);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #cbd5e1;
    padding: 9px 12px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.84rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .preset-modes button.active,
  .preset-modes button:hover {
    background: rgba(var(--primary-rgb), 0.2);
    border-color: var(--primary);
                                                              |    color: var(--primary);
  }

  .sliders-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
    gap: 12px 18px;
  }

  .slider-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .slider-item label {
    font-size: 0.78rem;
    color: #cbd5e1;
  }

  .slider-item label strong {
    color: var(--primary);
  }

  .placeholder-tab {
    margin: auto;
    text-align: center;
    color: #94a3b8;
  }
  .module-panel { display: flex; flex-direction: column; gap: 18px; }
  .module-panel header h2 { color: var(--primary); margin: 0 0 6px; }
  .module-panel header p, .module-card p { color: #94a3b8; }
  .module-panel code { color: var(--primary); overflow-wrap: anywhere; }
  .module-card { background: rgba(15, 23, 42, var(--card-opacity)); backdrop-filter: blur(var(--blur-px)); border: 1px solid rgba(var(--primary-rgb), .22); border-radius: 12px; padding: 18px; }
  .module-card, .info-card { transition: transform .22s ease, border-color .22s ease, box-shadow .22s ease; }
  .module-card:hover, .info-card:hover { transform: translateY(-2px); border-color: rgba(var(--primary-rgb), .5); box-shadow: 0 12px 30px rgba(var(--primary-rgb), .1); }
  .module-card h3 { margin-top: 0; color: var(--primary); }
  .module-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
  .module-actions h3 { flex: 1; margin: 0; }
  .module-actions button { margin: 0; }
  .mini-console { margin-top: 16px; }
  .mini-console-output { max-height: 190px; overflow: auto; margin-top: 12px; padding: 10px; border-radius: 8px; background: rgba(2,6,23,.88); color: #cbd5e1; font: .76rem/1.5 Consolas, "Cascadia Mono", monospace; overflow-wrap: anywhere; }
  .mini-console-output p { margin: 0; }
  .mini-console-output span { color: #94a3b8; }
  .mini-console-output .error-line { color: #fca5a5; }
  .console-panel { min-height: 0; flex: 1; }
  .console-card { min-height: 0; flex: 1; display: flex; flex-direction: column; }
  .console-actions { display: flex; gap: 8px; flex-wrap: wrap; }
  .console-output { min-height: 260px; max-height: calc(100vh - 240px); flex: 1; overflow: auto; margin-top: 14px; padding: 14px; border-radius: 9px; background: rgba(2,6,23,.92); color: #cbd5e1; font: .78rem/1.55 Consolas, "Cascadia Mono", monospace; white-space: pre-wrap; overflow-wrap: anywhere; }
  .console-output p { color: #94a3b8; }
  .console-output time, .console-output b { color: #94a3b8; }
  .console-output .console-error { color: #fca5a5; }
  .control-group input:disabled { opacity: .75; color: #94a3b8; cursor: not-allowed; }
  .module-card ul { padding-left: 20px; }
  .module-card li { padding: 5px 0; }
  .skins-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; }
  .account-card button { margin-right: 8px; margin-top: 8px; padding: 9px 12px; border-radius: 8px; border: 1px solid rgba(var(--primary-rgb), .25); background: rgba(var(--primary-rgb), .1); color: #94a3b8; }
  .account-card button:disabled { cursor: not-allowed; opacity: .62; }
  .auth-actions { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; }
  .auth-actions > button:not(.primary-action) { padding: 9px 12px; border: 1px solid rgba(255,255,255,.14); border-radius: 8px; background: rgba(255,255,255,.06); color: #cbd5e1; cursor: pointer; }
  .cape-list { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; margin-top: 12px; }
  .cape-list h4 { width: 100%; margin: 4px 0; color: #cbd5e1; }
  .cape-list button { margin: 0; }
  .cape-list button.active { border-color: var(--primary); color: var(--primary); }
  .skin-avatar { width: 96px; height: 96px; image-rendering: pixelated; border: 1px solid rgba(var(--primary-rgb), .35); border-radius: 12px; }
  .category-tabs { display: flex; flex-wrap: wrap; gap: 8px; }
  .category-tabs button, .row-buttons button, .runtime-row button { padding: 8px 11px; border: 1px solid rgba(var(--primary-rgb), .35); background: rgba(var(--primary-rgb), .1); color: #dbeafe; border-radius: 8px; cursor: pointer; transition: all .18s ease; }
  .category-tabs button.active, .category-tabs button:hover, .row-buttons button:hover, .runtime-row button:hover { background: rgba(var(--primary-rgb), .24); border-color: var(--primary); color: var(--primary); }
  .category-tabs button:disabled { opacity: .45; cursor: not-allowed; }
  .installed-list, .runtime-list { display: flex; flex-direction: column; gap: 8px; margin-top: 12px; }
  .installed-row, .runtime-row { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px; padding: 11px 12px; border: 1px solid rgba(255,255,255,.08); border-radius: 9px; background: rgba(var(--bg-base-rgb), .55); }
  .installed-row > div:first-child, .runtime-row > div:first-child { display: flex; flex-direction: column; gap: 4px; }
  .installed-row small, .runtime-row small { color: #94a3b8; overflow-wrap: anywhere; }
  .row-buttons { display: flex; flex-wrap: wrap; gap: 6px; }
  .row-buttons .danger-button { border-color: rgba(248,113,113,.4); color: #fca5a5; }
  .new-install-row { display: flex; flex-wrap: wrap; gap: 10px; }
  .new-install-row select { flex: 1; min-width: 170px; background: rgba(9,13,22,.85); color: white; padding: 9px; border: 1px solid rgba(var(--primary-rgb), .35); border-radius: 8px; }
  .primary-action { padding: 9px 13px; border-radius: 8px; border: 1px solid rgba(var(--primary-rgb), .7); background: linear-gradient(135deg, rgba(var(--primary-rgb),.25), rgba(var(--primary-rgb),.1)); color: var(--primary); font-weight: 750; cursor: pointer; transition: transform .18s ease, box-shadow .18s ease; }
  .primary-action:hover { transform: translateY(-1px); box-shadow: 0 0 18px rgba(var(--primary-rgb),.2); }
  .switch-label { display: flex; align-items: center; gap: 8px; color: #cbd5e1; cursor: pointer; }
  .switch-label input { appearance: none; width: 38px; height: 21px; border-radius: 99px; background: #334155; position: relative; transition: background .2s ease; cursor: pointer; }
  .switch-label input::after { content: ""; position: absolute; width: 15px; height: 15px; top: 3px; left: 3px; border-radius: 50%; background: white; transition: transform .2s ease; }
  .switch-label input:checked { background: var(--primary); }
  .switch-label input:checked::after { transform: translateX(17px); background: #08120e; }
  .icon-picker-row { display: flex; align-items: center; gap: 14px; }
  .icon-picker-row img { width: 64px; height: 64px; object-fit: contain; border-radius: 12px; background: rgba(var(--primary-rgb),.1); }
  .file-picker { display: inline-flex; position: relative; overflow: hidden; }
  .file-picker input { position: absolute; inset: 0; opacity: 0; cursor: pointer; }
  .inline-message { color: var(--primary) !important; overflow-wrap: anywhere; }
  .settings-section > small { display: block; color: #94a3b8; margin-top: 8px; }
  @keyframes launcher-enter { from { opacity: 0; transform: translateY(7px); } to { opacity: 1; transform: translateY(0); } }
  .module-panel, .hero, .settings-panel { animation: launcher-enter .28s ease-out both; }
  @media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: .01ms !important; transition-duration: .01ms !important; } }
  .download-progress { position: fixed; left: 260px; right: 32px; bottom: 18px; z-index: 5; padding: 12px 16px; border-radius: 12px; background: rgba(var(--bg-base-rgb), var(--bar-opacity)); backdrop-filter: blur(var(--blur-px)); border: 1px solid rgba(var(--primary-rgb), .4); box-shadow: 0 0 24px rgba(var(--primary-rgb), .18); }
  .progress-copy { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 8px; color: var(--primary); font-size: .84rem; }
  .progress-copy span { color: #cbd5e1; }
  .progress-track { height: 7px; background: rgba(var(--primary-rgb), .15); border-radius: 8px; overflow: hidden; }
  .progress-fill { height: 100%; background: linear-gradient(90deg, var(--primary), var(--secondary)); box-shadow: 0 0 12px rgba(var(--primary-rgb), .7); transition: width .2s ease; }
  @media (max-width: 720px) { .download-progress { left: 12px; right: 12px; bottom: 10px; } }
</style>
