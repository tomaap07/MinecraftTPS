<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import ModStudio from "$lib/ModStudio.svelte";
  import ContentNotifications from "$lib/ContentNotifications.svelte";
  import AccountWardrobe from "$lib/AccountWardrobe.svelte";
  import HeroShowcase from "$lib/HeroShowcase.svelte";
  import PoseStudio from "$lib/PoseStudio.svelte";
  import PlayerQuickTools, { type Playtime } from "$lib/PlayerQuickTools.svelte";
  import UiIcon from "$lib/UiIcon.svelte";
  import GameIcon from "$lib/GameIcon.svelte";
  import BrandWordmark from "$lib/BrandWordmark.svelte";
  import { logoEffect, startLogoEffect, stopLogoEffect } from "$lib/brand";
  import PlayerAvatar from "$lib/PlayerAvatar.svelte";
  import SettingsSuite from "$lib/SettingsSuite.svelte";
  import SetupWizard from "$lib/SetupWizard.svelte";
  import ServersPanel from "$lib/ServersPanel.svelte";
  import HostingOptions from "$lib/HostingOptions.svelte";
  import NewsHub from "$lib/NewsHub.svelte";
  import ScreenshotGallery from "$lib/ScreenshotGallery.svelte";
  import AchievementsPanel from "$lib/AchievementsPanel.svelte";
  import WorldSky from "$lib/WorldSky.svelte";
  import CommunityTools from "$lib/CommunityTools.svelte";
  import PluginsPanel from "$lib/PluginsPanel.svelte";
  import LaunchDock from "$lib/LaunchDock.svelte";
  import EndBackdrop from "$lib/EndBackdrop.svelte";
  import LegalNotice from "$lib/LegalNotice.svelte";
  import TermsConditions from "$lib/TermsConditions.svelte";
  import ExportReportDialog from "$lib/ExportReportDialog.svelte";
  import type { ExportResult } from "$lib/export_report";
  import { boundedConsole, consoleCanExpire } from "$lib/console_buffer";
  import SocialProfile from "$lib/SocialProfile.svelte";
  import { gameSession, isGameRunning, setGameSessionState, setGamePreparing, type GameSessionSnapshot } from "$lib/game_session";
  import "$lib/launcher_theme.css";
  import "$lib/launcher_inventory.css";
  import "$lib/launcher_layout.css";
  import LoaderLogo from "$lib/LoaderLogo.svelte";
  import VersionIcon from "$lib/VersionIcon.svelte";
  import { DEFAULT_LAUNCHER_SETTINGS, type LauncherSettings, type AppearanceSettings, type AccountSummary, type SettingsTab, type QuickFolder } from "$lib/settings.types";
  import { setLocale, translateUi } from "$lib/i18n";
  import { FALLBACK_SKIN, FALLBACK_AVATAR } from "$lib/poses_catalog";
  import {defaultPlayerAppearance,crafatarSkinUrl,type PublicPlayerAppearance} from "$lib/player_skins";
  import {refreshLocalGameArtwork} from '$lib/local_game_art';
  import { getWelcomeSession } from "$lib/welcome_session";
  import { getSecretState, subscribeSecretState, onSecretInteraction, startEggClock, type EasterEgg, type SecretTrigger } from "$lib/easter_eggs";


  let activeTab = $state<"jugar" | "instalaciones" | "mods" | "skins" | "poser" | "ajustes" | "consola" | "servers" | "capturas" | "logros" | "comunidad" | "plugins" | "legal">("jugar");
  const sectionNames={jugar:'Jugar',instalaciones:'Instalaciones',mods:'Contenido',skins:'Cuenta y skins',poser:'Estudio 3D',ajustes:'Ajustes',consola:'Consola',servers:'Servidores',capturas:'Capturas',logros:'Logros',comunidad:'Comunidad',plugins:'Plugins del launcher',legal:'Términos y condiciones'};
  let launchDockCollapsed=$state(false);
  let showLaunchDock=$derived(!['jugar','ajustes','comunidad'].includes(activeTab));
  let studioInitialSection = $state<"mod"|"modpack"|"shader"|"resourcepack">("mod");
  let settings = $state<LauncherSettings>({...DEFAULT_LAUNCHER_SETTINGS});
  let settingsTab = $state<SettingsTab>("general");
  let accounts = $state<AccountSummary[]>([]);
  let offlineAllowed=$state(false),offlineMessage=$state("Comprobando la disponibilidad de perfiles locales.");
  let showWizard = $state(false);
  let totalMemoryGb = $state(16);
  let backgroundUrl = $state("");
  let subtitleIndex = $state(0);
  type UpdateInfo = {available:boolean;message:string;version?:string;url?:string;verifiable?:boolean;asset_id?:number;asset_name?:string;download_size?:number};
  let updateInfo=$state<UpdateInfo|null>(null);
  let updateDownload=$state("");
  let updateBusy=$state(false);
  let updateMessage=$state("");
  const heroMessages=["Tu universo de Minecraft sin límites. Elige tu instancia y prepara tu próxima aventura.","Tus mundos, tus versiones y tus amigos. Todo empieza aquí.","Construye, explora y crea. Tu Centro de Mando está listo."];
  let contentPanel = $state<HTMLElement>();
  $effect(() => { activeTab; if(contentPanel) contentPanel.scrollTop=0; });


  let username = $state("Steve");
  const welcomeSession = getWelcomeSession();
  let secrets=$state(getSecretState());
  let visualEgg=$state<EasterEgg|null>(welcomeSession.egg);
  let secretMoment=$state<EasterEgg|null>(welcomeSession.egg?.kind==='appearance'?null:welcomeSession.egg);
  let logoClicks=0,logoLastClick=0;
  let logoSecretActive=$state(false);
  let secretTimeout:ReturnType<typeof setTimeout>|undefined;
  let selectedVersion = $state("1.21.1");
  let ramGb = $state(4);
  let selectedPackId = $state("");
  type StudioPack = { id:string; name:string; game_version:string; loader:string; version_id:string; icon_data:string; icon_url?:string; install_state?:string };
  let studioPacks = $state<StudioPack[]>([]);
  let playtime = $state<Playtime>({ total_seconds:0,session_seconds:0,sessions:0,active_games:0,versions:{} });
  let offlineAppearance = $state<{ name:string;skin_data:string;cape_data:string;model:string;avatar_data:string }|null>(null);
  let defaultAppearance=$state<PublicPlayerAppearance|null>(null);
  let artworkMounted=$state(false);
  let microsoftDefaultAppearance=$state<PublicPlayerAppearance|null>(null);
  let appearancePending = $state(false);
  let appearanceRequest = 0;
  let selectedPack = $derived(studioPacks.find(pack => pack.id === selectedPackId));
  const formatPlaytime = (seconds:number) => `${Math.floor(seconds/3600)} h ${Math.floor(seconds%3600/60)} min`;
  let javaStatus = $state("Verificando Java...");
  let javaOk = $state(false);
  let launchMessage = $state("");
  let compatibilityWarnings = $state<string[]>([]);
  let isLaunching = $state(false);
  let checkingLaunchAccount = $state(false);
  let preparingHomeVersion = $state(false);
  let installingJava = $state(false);
  let progress = $state<{ task: string; percent: number; status: string } | null>(null);
  let rootPath = $state("");
  let mods = $state<string[]>([]);
  let sharedMods = $state<string[]>([]);
  let javaInfo = $state<{ installed: boolean; version: string; path: string; is_portable: boolean } | null>(null);
  let showOldVersions = $state(false);
  type LoaderCategory = "fabric" | "forge" | "neoforge" | "quilt" | "optifine";
  type ExtraLoaderVersion = { id: string; loader_version: string; stable: boolean };
  type OptiFineCandidate = { id: string; name: string; game_version: string; file_name: string; path: string; source: string; profile_id: string; preview: boolean; legacy_patch: boolean };
  let homeVersionCategory = $state<"installed" | "release" | LoaderCategory | "snapshot" | "old">("release");
  let versionCatalog = $state<{ id: string; kind: string }[]>([]);
  let versionCatalogError = $state("");
  let fabricCatalog = $state<{ id: string; stable: boolean }[]>([]);
  let forgeCatalog = $state<{ id: string; forge_version: string; channel: string }[]>([]);
  let neoforgeCatalog = $state<ExtraLoaderVersion[]>([]);
  let quiltCatalog = $state<ExtraLoaderVersion[]>([]);
  let extraLoaderBusy = $state(false);
  let extraLoaderMessage = $state("");
  let optifineCatalog = $state<{ id: string; profile_id: string; game_version: string; name: string; file_name: string; preview: boolean; legacy_patch: boolean }[]>([]);
  let loaderCatalogError = $state("");
  let optifineCandidates = $state<OptiFineCandidate[]>([]);
  let scanningOptifine = $state(false);
  let optifineTarget = $state("");
  let installationCategory = $state<"release" | "snapshot" | "old" | LoaderCategory>("release");
  let installedVersions = $state<{ id: string; jar: boolean; json: boolean; size_mb: number; loader: string; game_version: string }[]>([]);
  let installedFilter = $state<"all" | "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "optifine" | "custom">("all");
  let installedSearch = $state("");
  let installationsRefreshing = $state(false);
  let installationsError = $state("");
  let fabricBusy = $state(false);
  let fabricMessage = $state("");
  let fabricTargetVersion = $state("1.21.9");
  let forgeBusy = $state(false);
  let forgeMessage = $state("");
  let optifineBusy = $state(false);
  let optifineMessage = $state("");
  let modLoader = $state<"fabric" | "forge" | "neoforge" | "quilt">("fabric");
  let modSearch = $state("");
  let modSearchBusy = $state(false);
  let modMessage = $state("");
  let modrinthProjects = $state<{ project_id: string; slug: string; title: string; description: string; icon_url: string; downloads: number }[]>([]);
  let modProjectType = $state<"mod" | "modpack">("mod");
  let modSection = $state<"explore-mods" | "explore-modpacks" | "share-pack">("explore-mods");
  let modGameVersion = $state("all");
  let modPage = $state(1);
  let modTotalHits = $state(0);
  let packBusy = $state(false);
  let packMessage = $state("");
  let exportReport = $state<ExportResult | null>(null);
  let recentVersions = $state<string[]>([]);
  let installingProject = $state("");
  let consoleLines = $state<{ timestamp: number; time: string; stream: string; line: string }[]>([]);
  let consoleScrollElement = $state<HTMLDivElement | undefined>();
  let consoleMessage = $state("");
  let consoleLastActivity = Date.now();
  let consoleSessionStarted = 0;
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
  let filteredInstalledVersions = $derived(installedVersions.filter((installed) => {
    const matchesFilter = installedFilter === "all" || installed.loader === installedFilter;
    const query = installedSearch.trim().toLowerCase();
    const matchesSearch = !query || `${installed.id} ${installed.game_version} ${installed.loader}`.toLowerCase().includes(query);
    return matchesFilter && matchesSearch;
  }));
  let totalInstalledSize = $derived(installedVersions.reduce((total, item) => total + item.size_mb, 0));
  let progressTimer: ReturnType<typeof setTimeout> | undefined;
  let javaRequirementRequest = 0;
  let javaRecommendationStatus = $derived.by(() => {
    if (recommendedJavaMajor === null) return "Consultando requisito…";
    const minimum = Math.max(8, recommendedJavaMajor);
    const compatible = javaRuntimes.filter((runtime) => runtime.installed && runtime.enabled && (minimum === 8 ? runtime.major === 8 : runtime.major >= minimum)).sort((a, b) => a.major - b.major)[0];
    return compatible ? `Java ${compatible.major} disponible` : `Se instalará Java ${minimum} al iniciar`;
  });
  let accountSkinFallback=$derived(microsoftProfile?crafatarSkinUrl(microsoftProfile.id,microsoftProfile.skins.find(skin=>skin.state==='ACTIVE')?.variant?.toLowerCase()==='slim'?'alex':'steve')||'':'');
  let activeSkinUrl = $derived(microsoftProfile ? microsoftProfile.skins.find((skin) => skin.state === "ACTIVE")?.url || microsoftDefaultAppearance?.skin_data || accountSkinFallback || defaultAppearance?.skin_data || FALLBACK_SKIN : offlineAppearance?.skin_data || defaultAppearance?.skin_data || FALLBACK_SKIN);
  let activeCapeUrl = $derived(microsoftProfile ? microsoftProfile.capes.find((cape) => cape.state === "ACTIVE")?.url ?? "" : offlineAppearance?.cape_data ?? "");
  let activeSkinModel = $derived((microsoftProfile ? microsoftProfile.skins.find((skin) => skin.state === "ACTIVE")?.variant ?? microsoftDefaultAppearance?.model : offlineAppearance?.model ?? defaultAppearance?.model)?.toLowerCase() === "slim" ? "slim" as const : "default" as const);
  let consoleText = $derived(consoleLines.map((entry) => `[${entry.time}] [${entry.stream}] ${entry.line}`).join("\n"));
  let recentConsoleLines = $derived(consoleLines.slice(-12));
  let optifineOptions = $derived.by(() => {
    const installed = optifineCatalog.map(item => ({id:item.id, profile_id:item.profile_id, game_version:item.game_version, name:item.name, downloaded:false}));
    const profiles = new Set(installed.map(item => item.profile_id));
    for(const item of installedVersions.filter(item=>item.loader==='optifine')){
      if(profiles.has(item.id))continue;
      profiles.add(item.id);
      installed.push({id:item.id,profile_id:item.id,game_version:item.game_version,name:item.id,downloaded:false});
    }
    const detected=[];
    for(const item of optifineCandidates){
      if(profiles.has(item.profile_id))continue;
      profiles.add(item.profile_id);
      detected.push({id:item.id, profile_id:item.profile_id, game_version:item.game_version, name:item.name, downloaded:true});
    }
    return [...installed,...detected];
  });
  let selectedMinecraftVersion = $derived(installedVersions.find((item) => item.id === selectedVersion)?.game_version ?? optifineOptions.find((item) => item.id === selectedVersion)?.game_version ?? selectedVersion.split(/\s+/)[0]);
  let selectedModLoader = $derived(selectedPack?.loader ?? installedVersions.find((item) => item.id === selectedVersion)?.loader ?? "vanilla");
  let installationCatalogCount = $derived.by(() => {
    if (installationCategory === "fabric") return fabricCatalog.length;
    if (installationCategory === "forge") return forgeCatalog.length;
    if (installationCategory === "neoforge") return neoforgeCatalog.length;
    if (installationCategory === "quilt") return quiltCatalog.length;
    if (installationCategory === "optifine") return optifineOptions.length;
    if (installationCategory === "old") return versionCatalog.filter((item) => item.kind === "old_alpha" || item.kind === "old_beta").length;
    return versionCatalog.filter((item) => item.kind === installationCategory).length;
  });
  let installationCatalogLabel = $derived(installationCategory === "fabric" ? "versiones de juego Fabric" : installationCategory === "forge" ? "builds Forge" : installationCategory === "optifine" ? "versiones OptiFine" : installationCategory === "old" ? "versiones históricas" : `versiones ${installationCategory}`);
  let modPageCount = $derived(Math.max(1, Math.ceil(modTotalHits / 24)));
  let modGameVersions = $derived([...new Set([
    ...versionCatalog.filter((entry) => entry.kind === "release").map((entry) => entry.id),
    ...installedVersions.map((entry) => entry.game_version)
  ])].sort((a, b) => b.localeCompare(a, undefined, { numeric: true })));
  let homeVersionOptions = $derived.by(() => {
    if (selectedPack) return [{ id:selectedPack.version_id,label:`${selectedPack.name} · Minecraft ${selectedPack.game_version} · ${selectedPack.loader}` }];
    if (homeVersionCategory === "installed") {
      return installedVersions.map((item) => ({ id: item.id, label: `${item.id} · ${item.loader} · Minecraft ${item.game_version}` }));
    }
    if (homeVersionCategory === "fabric") {
      return [
        ...fabricCatalog.map((item) => ({ id: item.id, label: `${item.id} · Fabric${item.stable ? " estable" : " snapshot"}` })),
        ...installedVersions.filter((item) => item.loader === "fabric").map((item) => ({ id: item.id, label: `${item.id} · Fabric instalado · Minecraft ${item.game_version}` }))
      ];
    }
    if (homeVersionCategory === "forge") {
      return [
        ...forgeCatalog.map((item) => ({ id: item.id, label: `${item.id} · Forge ${item.forge_version} · ${item.channel}` })),
        ...installedVersions.filter((item) => item.loader === "forge").map((item) => ({ id: item.id, label: `${item.id} · Forge instalado · Minecraft ${item.game_version}` }))
      ];
    }
    if (homeVersionCategory === "neoforge" || homeVersionCategory === "quilt") {
      const loader = homeVersionCategory;
      const catalog = loader === "neoforge" ? neoforgeCatalog : quiltCatalog;
      return [
        ...catalog.map(item => ({id:item.id, label:`${item.id} · ${loader === "neoforge" ? "NeoForge" : "Quilt"} ${item.loader_version}${item.stable ? "" : " · Prueba"}`})),
        ...installedVersions.filter(item => item.loader === loader).map(item => ({id:item.id, label:`${item.id} · Instalado · Minecraft ${item.game_version}`}))
      ];
    }
    if (homeVersionCategory === "optifine") return optifineOptions.map(item => ({id:item.id, label:`Minecraft ${item.game_version} · ${item.name} · ${item.downloaded ? "JAR detectado · Importar al jugar" : "Instalado"}`}));
    const kindMatches = homeVersionCategory === "old"
      ? (entry: { kind: string }) => entry.kind === "old_alpha" || entry.kind === "old_beta"
      : (entry: { kind: string }) => entry.kind === homeVersionCategory;
    return versionCatalog.filter(kindMatches).map((item) => ({ id: item.id, label: `${item.id}${item.kind === "snapshot" ? " · Snapshot" : item.kind === "old_beta" ? " · Beta" : item.kind === "old_alpha" ? " · Alpha" : " · Release"}` }));
  });


  interface ThemePreset {
    id: string;
    name: string;
    primary: string;
    secondary: string;
    bgBase: string;
  }

  const themes: ThemePreset[] = [
    { id:"the-end", name:"The End · TP", primary:"#A855F7",secondary:"#D946EF",bgBase:"#0B0B0F" },
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


  let selectedThemeId = $state("the-end");
  let primaryColor = $state("#A855F7");
  let secondaryColor = $state("#D946EF");
  let bgBaseColor = $state("#0B0B0F");


  let transparencyPreset = $state<"fantasma" | "equilibrado" | "lectura" | "solido">("equilibrado");
  let bgOpacity = $state(0.88);
  let menuOpacity = $state(0.75);
  let barOpacity = $state(0.85);
  let cardOpacity = $state(0.65);
  let blurPx = $state(14);


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
      homeVersionCategory,
      launchDockCollapsed,
      ramGb,
      selectedPackId,
      recentVersions,
      modGameVersion,
      modLoader,
      backgroundUrl: backgroundUrl ? "custom" : ""
    };
    localStorage.setItem("minecraft_tps_config", JSON.stringify(config));
  }

  function loadConfig() {
    const saved = localStorage.getItem("minecraft_tps_config");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        selectedThemeId = parsed.selectedThemeId ?? "the-end";
        primaryColor = parsed.primaryColor ?? "#A855F7";
        secondaryColor = parsed.secondaryColor ?? "#D946EF";
        bgBaseColor = parsed.bgBaseColor ?? "#0B0B0F";
        if(selectedThemeId==='indigo'&&primaryColor==='#818cf8'&&secondaryColor==='#4f46e5'&&bgBaseColor==='#090d1f'){
          selectedThemeId='the-end';primaryColor='#A855F7';secondaryColor='#D946EF';bgBaseColor='#0B0B0F';
        }
        backgroundUrl = typeof parsed.backgroundUrl === "string" && parsed.backgroundUrl !== "custom" ? parsed.backgroundUrl : "";
        transparencyPreset = parsed.transparencyPreset ?? "equilibrado";
        bgOpacity = parsed.bgOpacity ?? 0.85;
        menuOpacity = parsed.menuOpacity ?? 0.72;
        barOpacity = parsed.barOpacity ?? 0.80;
        cardOpacity = parsed.cardOpacity ?? 0.60;
        blurPx = parsed.blurPx ?? 14;
        username = parsed.username ?? "Steve";
        selectedVersion = typeof parsed.selectedVersion === "string" ? parsed.selectedVersion.trim().split(/\s+/)[0] || "1.21.1" : "1.21.1";
        if (["installed","release","fabric","forge","neoforge","quilt","optifine","snapshot","old"].includes(parsed.homeVersionCategory)) homeVersionCategory = parsed.homeVersionCategory;
        launchDockCollapsed=parsed.launchDockCollapsed===true;
        showOldVersions = homeVersionCategory === "old";
        ramGb = parsed.ramGb ?? 4;
        selectedPackId = typeof parsed.selectedPackId === "string" ? parsed.selectedPackId : "";
        recentVersions = Array.isArray(parsed.recentVersions) ? parsed.recentVersions.filter((value: unknown): value is string => typeof value === "string").slice(0, 6) : [];
        modGameVersion = typeof parsed.modGameVersion === "string" ? parsed.modGameVersion : "all";
        if (["fabric", "forge", "neoforge", "quilt"].includes(parsed.modLoader)) modLoader = parsed.modLoader;
      } catch (e) {
        console.error("Error cargando configuración:", e);
      }
    }
  }

  onMount(() => {
    artworkMounted=true;
    const stopBrand=logoEffect.subscribe(effect=>logoSecretActive=effect.active);
    const stopSecrets=subscribeSecretState(state=>secrets=state);
    if(welcomeSession.egg?.kind==='console')revealSecret(welcomeSession.egg);
    let stopped=false,unlistenGame:(()=>void)|undefined;
    void defaultPlayerAppearance('steve').then(value=>{if(!stopped)defaultAppearance=value;}).catch(()=>{});
    void listen<GameSessionSnapshot>('tp-game-session',event=>{
      if(event.payload.running&&!$gameSession.running&&!isLaunching){consoleLines=[];consoleSessionStarted++;consoleLastActivity=Date.now();}
      setGameSessionState(event.payload);playtime={...playtime,active_games:event.payload.active_games};
    }).then(async stop=>{
      if(stopped){stop();return;}unlistenGame=stop;
      const snapshot=await invoke<GameSessionSnapshot>('get_game_session');if(!stopped)setGameSessionState(snapshot);
    }).catch(error=>addConsoleLine('launcher',`Estado de partida: ${String(error)}`));
    let unlisten: (() => void) | undefined;
    let unlistenConsole: (() => void) | undefined;
    void (async () => {
    loadConfig();
    try {
      settings=await invoke<LauncherSettings>("get_launcher_settings");
      setLocale(settings.language);ramGb=settings.ram_gb;
      showWizard=!settings.first_run_completed;
      const memory=await invoke<{total_memory_gb:number;recommended_ram_gb:number}>("get_system_summary");totalMemoryGb=memory.total_memory_gb;
      if(showWizard){ramGb=memory.recommended_ram_gb;settings={...settings,ram_gb:ramGb};}
      if(settings.automatic_updates)void checkUpdates().then(message=>addConsoleLine("launcher",message)).catch(()=>{});
    } catch(error) { launchMessage=`No se pudieron cargar los ajustes: ${String(error)}`; }
    void refreshAccounts();
    unlistenConsole = await listen<{ timestamp: number; stream: string; line: string }>("launcher-log", (event) => {
      addConsoleLine(event.payload.stream, event.payload.line, event.payload.timestamp);
    });
    try {
      const saved = await invoke<{ timestamp: number; stream: string; line: string }[]>("read_launcher_logs");
      const merged = [...saved.map((entry) => ({ ...entry, time: new Date(entry.timestamp).toLocaleTimeString() })), ...consoleLines];
      if(!consoleSessionStarted)consoleLines = boundedConsole([...new Map(merged.map((entry) => [`${entry.timestamp}:${entry.stream}:${entry.line}`, entry])).values()]);
    } catch (err) { addConsoleLine("stderr", `No se pudo restaurar la consola: ${String(err)}`); }
    void loadVersionCatalog(showOldVersions);
    void refreshLoaderCatalogs();
    void refreshInstallations();
    void refreshStudioPacks();
    void refreshJavaRuntimes();
    void invoke<string | null>("get_custom_app_icon").then((icon) => { if (icon) iconPreview = icon; }).catch(console.error);
    void invoke<string>("get_custom_background").then(async image=>{if(image)backgroundUrl=image;else if(backgroundUrl.startsWith("data:image/")){backgroundUrl=await invoke<string>("set_custom_background",{imageBase64:backgroundUrl});saveConfig();}}).catch(error=>addConsoleLine("stderr",String(error)));
    void invoke<MicrosoftProfile | null>("get_microsoft_profile").then((profile) => { if (profile) { microsoftProfile = profile; username = profile.name; saveConfig(); void refreshMods(); void refreshAccounts(); void invoke("record_launcher_log", { stream: "cuenta", line: `Sesión Microsoft recuperada correctamente para ${profile.name}.` }); } }).catch((err) => { loginMessage = String(err); const line = `No se pudo restaurar la cuenta Microsoft: ${String(err)}`; void invoke("record_launcher_log", { stream: "stderr", line }).catch(() => addConsoleLine("stderr", line)); });
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
    const noteConsoleInteraction=(event:Event)=>{if(activeTab==='consola'&&event.target instanceof Element&&event.target.closest('.console-panel'))consoleLastActivity=Date.now();};
    for(const event of ['pointerdown','keydown','wheel'])window.addEventListener(event,noteConsoleInteraction,{passive:true});
    const consoleIdleTimer=setInterval(()=>{
      if(consoleLines.length&&consoleCanExpire(Date.now(),consoleLastActivity,$isGameRunning)){
        consoleLines=[];consoleMessage='Vista vaciada tras 12 minutos sin actividad. El registro de diagnóstico se conserva en el disco.';
      }
    },30000);
    const detectOptifineOnFocus=()=>{if(homeVersionCategory==='optifine'||activeTab==='instalaciones'&&installationCategory==='optifine')void scanOptifine();};
    window.addEventListener('focus',detectOptifineOnFocus);
    const optifineScanTimer=setInterval(()=>{if(document.hasFocus()&&!$isGameRunning)detectOptifineOnFocus();},12000);
    return () => { stopped=true;unlistenGame?.();unlisten?.(); unlistenConsole?.(); clearTimeout(progressTimer);stopSecrets();stopBrand();stopLogoEffect();clearTimeout(secretTimeout);clearInterval(consoleIdleTimer);clearInterval(optifineScanTimer);window.removeEventListener('focus',detectOptifineOnFocus);for(const event of ['pointerdown','keydown','wheel'])window.removeEventListener(event,noteConsoleInteraction); };
  });

  $effect(()=>{
    if($isGameRunning){stopLogoEffect();return;}
    const stopClock=startEggClock(revealSecret);
    const subtitleTimer=setInterval(()=>subtitleIndex=(subtitleIndex+1)%heroMessages.length,9000);
    return()=>{stopClock();clearInterval(subtitleTimer);};
  });

  $effect(() => {
    consoleLines.length;
    if (activeTab === "consola" && consoleScrollElement) {
      queueMicrotask(() => { if (consoleScrollElement) consoleScrollElement.scrollTop = consoleScrollElement.scrollHeight; });
    }
  });

  $effect(()=>{
    const profile=microsoftProfile;const primary=profile?.skins.find(skin=>skin.state==='ACTIVE')?.url;let alive=true;
    microsoftDefaultAppearance=null;
    if(profile&&!primary)void invoke<PublicPlayerAppearance>('get_player_appearance_by_uuid',{uuid:profile.id}).then(value=>{if(alive)microsoftDefaultAppearance=value;}).catch(()=>{});
    return()=>{alive=false;};
  });

  $effect(()=>{
    const version=optifineOptions.find(item=>item.id===selectedVersion)?.game_version??selectedVersion.split(/\s+/)[0];installedVersions;
    if(!artworkMounted||!version)return;
    const timer=setTimeout(()=>void refreshLocalGameArtwork(version,installedVersions.find(item=>item.jar&&item.json)?.id),180);
    return()=>clearTimeout(timer);
  });

  $effect(() => {
    const nick = username.trim();
    const authenticated = !!microsoftProfile;
    const request = ++appearanceRequest;
    offlineAppearance = null;
    if(authenticated||nick.toLowerCase()==='steve'||!/^[a-zA-Z0-9_]{3,16}$/.test(nick)){appearancePending=false;return;}
    if(nick.toLowerCase()==='alex'){appearancePending=true;void defaultPlayerAppearance('alex').then(value=>{if(request===appearanceRequest)offlineAppearance=value;}).catch(()=>{}).finally(()=>{if(request===appearanceRequest)appearancePending=false;});return;}

    appearancePending=true;
    const timer = setTimeout(() => {
      void invoke<typeof offlineAppearance>("resolve_player_appearance",{nick}).then(value => { if(request===appearanceRequest) offlineAppearance=value; }).catch(() => { if(request===appearanceRequest) offlineAppearance=null; }).finally(() => { if(request===appearanceRequest) appearancePending=false; });
    },400);
    return () => clearTimeout(timer);
  });

  async function refreshStudioPacks() {
    try { studioPacks=await invoke<StudioPack[]>("studio_list_packs"); if(selectedPackId && !studioPacks.some(pack=>pack.id===selectedPackId)) selectedPackId=""; }
    catch(error) { addConsoleLine("stderr",`No se pudo leer la biblioteca de modpacks: ${String(error)}`); }
  }

  async function playStudioPack(versionId:string,instanceId:string) {
    if (!studioPacks.some(pack=>pack.id===instanceId)) await refreshStudioPacks();
    selectedVersion=versionId;
    selectedPackId=instanceId;
    activeTab="jugar";
    await refreshJavaRequirement();
    await handleLaunch();
  }

  function updateMicrosoftProfile(profile:MicrosoftProfile) {
    microsoftProfile=profile; username=profile.name; saveConfig();
  }

  function previewLocalSkin(url:string,model:"default"|"slim") {
    if(microsoftProfile) return;
    ++appearanceRequest;
    appearancePending=false;
    offlineAppearance={name:username,skin_data:url,cape_data:"",model,avatar_data:FALLBACK_AVATAR};
    const image=new Image();
    image.onload=()=>{
      if(offlineAppearance?.skin_data!==url) return;
      const canvas=document.createElement("canvas");canvas.width=64;canvas.height=64;
      const context=canvas.getContext("2d");if(!context) return;
      context.imageSmoothingEnabled=false;
      context.drawImage(image,8,8,8,8,0,0,64,64);
      if(image.height===64) context.drawImage(image,40,8,8,8,0,0,64,64);
      offlineAppearance={...offlineAppearance,avatar_data:canvas.toDataURL("image/png")};
    };
    image.src=url;
  }

  async function refreshMods() {
    try {
      const [installed, shared] = await Promise.all([
        invoke<string[]>("list_mods", { username: microsoftProfile?.name ?? (username.trim() || "Steve"), versionId: selectedVersion.split(/\s+/)[0] }),
        invoke<string[]>("list_shared_mods")
      ]);
      mods = installed;
      sharedMods = shared;
    } catch (err) { launchMessage = String(err); }
  }

  async function copySharedModsToInstance() {
    try {
      const copied = await invoke<number>("copy_shared_mods", {
        username: microsoftProfile?.name ?? (username.trim() || "Steve"),
        versionId: selectedVersion.split(/\s+/)[0]
      });
      modMessage = copied ? `Se copiaron ${copied} mods a esta instalación. Los originales se conservaron.` : "No había mods nuevos para copiar.";
      await refreshMods();
    } catch (error) {
      modMessage = String(error);
    }
  }

  async function installFabricForSelectedVersion() {
    const gameVersion = fabricTargetVersion.split(/\s+/)[0];
    if (!gameVersion) return;
    fabricBusy = true;
    fabricMessage = `Instalando Fabric para Minecraft ${gameVersion}…`;
    try {
      const id = await invoke<string>("install_fabric", { gameVersion });
      selectedVersion = id;
      fabricMessage = `Fabric quedó instalado como ${id}. Ya puedes elegir esa instalación y pulsar Jugar.`;
      await refreshInstallations();
      void refreshMods();
      saveConfig();
    } catch (error) {
      fabricMessage = String(error);
    } finally {
      fabricBusy = false;
    }
  }

  async function installForgeForSelectedVersion() {
    const gameVersion = fabricTargetVersion.split(/\s+/)[0];
    if (!gameVersion) return;
    forgeBusy = true;
    forgeMessage = `Consultando e instalando Forge para Minecraft ${gameVersion}…`;
    try {
      const id = await invoke<string>("install_forge", { gameVersion });
      selectedVersion = id;
      forgeMessage = `Forge quedó instalado como ${id}. Ya puedes jugar y buscar mods compatibles.`;
      await refreshInstallations();
      void refreshMods();
      saveConfig();
    } catch (error) {
      forgeMessage = String(error);
    } finally {
      forgeBusy = false;
    }
  }

  function optifineProfileId(version: { profile_id: string }): string {
    return version.profile_id;
  }

  async function installOptifineVersion(versionId:string):Promise<string>{
    const installed=installedVersions.find(item=>item.id===versionId&&item.loader==='optifine'&&item.json);
    if(installed)return installed.id;
    const candidate=optifineCandidates.find(item=>item.id===versionId);
    if(!candidate)throw new Error("Descarga tu versión oficial desde optifine.net; TP Launcher la detectará automáticamente.");
    return invoke<string>('import_discovered_optifine',{candidateId:candidate.id});
  }
  async function openOptifineDownloads(){
    optifineMessage="Descarga tu versión oficial desde optifine.net; TP Launcher la detectará automáticamente.";
    try { const {openUrl}=await import('@tauri-apps/plugin-opener');await openUrl('https://optifine.net/downloads'); }
    catch(error){optifineMessage=`No se pudo abrir la web oficial: ${String(error)}`;}
  }
  async function scanOptifine(){
    if(scanningOptifine)return;
    scanningOptifine=true;
    try {
      const [files,installed]=await Promise.all([
        invoke<OptiFineCandidate[]>('discover_optifine_files'),
        invoke<typeof optifineCatalog>('fetch_optifine_versions')
      ]);
      optifineCandidates=files;optifineCatalog=installed;
      if(!optifineOptions.some(item=>item.id===optifineTarget))optifineTarget=optifineOptions[0]?.id??'';
      if(homeVersionCategory==='optifine'&&!homeVersionOptions.some(item=>item.id===selectedVersion)){
        selectedVersion=homeVersionOptions[0]?.id??'';saveConfig();void refreshJavaRequirement();
      }
    }catch(error){optifineMessage=`No se pudieron detectar los JAR locales: ${String(error)}`;}
    finally{scanningOptifine=false;}
  }
  async function installDetectedOptifine(){
    if(!optifineTarget){await openOptifineDownloads();return;}
    optifineBusy=true;
    try{selectedVersion=await installOptifineVersion(optifineTarget);homeVersionCategory='optifine';await refreshInstallations();await scanOptifine();saveConfig();optifineMessage=`Perfil ${selectedVersion} listo para jugar.`;}
    catch(error){optifineMessage=String(error);}finally{optifineBusy=false;}
  }
  async function installExtraForSelectedVersion(loader:'neoforge'|'quilt'){
    extraLoaderBusy=true;extraLoaderMessage=`Instalando ${loader==='neoforge'?'NeoForge':'Quilt'}…`;
    try{
      const catalog=loader==='neoforge'?neoforgeCatalog:quiltCatalog;
      const target=catalog.find(item=>item.id===fabricTargetVersion);
      if(!target)throw new Error('Selecciona una versión compatible del catálogo oficial.');
      selectedVersion=await invoke<string>('install_extra_loader',{loader,gameVersion:target.id,loaderVersion:target.loader_version});
      homeVersionCategory=loader;saveConfig();await refreshInstallations();extraLoaderMessage=`${selectedVersion} instalado.`;
    }catch(error){extraLoaderMessage=String(error);}finally{extraLoaderBusy=false;}
  }
  async function importOptifineFile(event:Event){
    const input=event.currentTarget as HTMLInputElement,file=input.files?.[0];if(!file)return;optifineBusy=true;optifineMessage="Verificando e importando el JAR local…";
    try{if(file.size>80*1024*1024||!file.name.endsWith('.jar'))throw new Error("Selecciona un JAR original de OptiFine de hasta 80 MB.");
      const data=await new Promise<string>((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(String(reader.result).split(',')[1]??'');reader.onerror=()=>reject(new Error("No se pudo leer el archivo local"));reader.readAsDataURL(file);});
      selectedVersion=await invoke<string>('import_optifine',{fileName:file.name,jarBase64:data});await refreshInstallations();await refreshLoaderCatalogs();saveConfig();optifineMessage=`Perfil ${selectedVersion} importado. Puedes seleccionarlo y jugar.`;
    }catch(error){optifineMessage=String(error);}finally{optifineBusy=false;input.value='';}
  }

  async function launchSelectedVersion(serverAddress?:string) {
    if(!selectedPackId&&homeVersionCategory==='optifine'&&!homeVersionOptions.length){await openOptifineDownloads();return;}
    if(!selectedVersion.trim())return;
    if(selectedPackId) { await handleLaunch(serverAddress); return; }
    if (!["fabric", "forge", "neoforge", "quilt", "optifine"].includes(homeVersionCategory)) {
      await handleLaunch(serverAddress);
      return;
    }
    const requiredLoader = homeVersionCategory;
    const installed = installedVersions.find((item) => item.id === selectedVersion && item.loader === requiredLoader);
    if (installed) {
      await handleLaunch(serverAddress);
      return;
    }
    if(!await checkLaunchAccount())return;
    preparingHomeVersion = true;
    try {
      if (homeVersionCategory === "fabric") {
        const gameVersion = selectedVersion;
        if (!fabricCatalog.some((item) => item.id === gameVersion)) throw new Error("Selecciona una versión compatible con Fabric.");
        launchMessage = `Instalando Fabric para Minecraft ${gameVersion}…`;
        selectedVersion = await invoke<string>("install_fabric", { gameVersion });
      } else if (homeVersionCategory === "forge") {
        const gameVersion = selectedVersion;
        if (!forgeCatalog.some((item) => item.id === gameVersion)) throw new Error("Selecciona una versión compatible con Forge.");
        launchMessage = `Instalando Forge para Minecraft ${gameVersion}…`;
        selectedVersion = await invoke<string>("install_forge", { gameVersion });
      } else if(homeVersionCategory==='neoforge'||homeVersionCategory==='quilt'){
        const catalog=homeVersionCategory==='neoforge'?neoforgeCatalog:quiltCatalog;
        const target=catalog.find(item=>item.id===selectedVersion);
        if(!target)throw new Error('Selecciona una versión compatible del catálogo oficial.');
        selectedVersion=await invoke<string>('install_extra_loader',{loader:homeVersionCategory,gameVersion:target.id,loaderVersion:target.loader_version});
      } else {
        const selectedOptifine = optifineOptions.find((item) => item.id === selectedVersion);
        if (!selectedOptifine) throw new Error("Selecciona una versión disponible en el catálogo OptiFine.");
        launchMessage = `Instalando ${selectedOptifine.name} para Minecraft ${selectedOptifine.game_version}…`;
        selectedVersion = await installOptifineVersion(selectedOptifine.id);
      }
      saveConfig();
      await refreshInstallations();
      await handleLaunch(serverAddress);
    } catch (error) {
      launchMessage = `No se pudo preparar esta versión: ${String(error)}`;
      addConsoleLine("stderr", launchMessage);
    } finally {
      preparingHomeVersion = false;
    }
  }

  async function searchModrinthMods() {
    modSearchBusy = true;
    modMessage = `Buscando ${modProjectType === "mod" ? "mods" : "modpacks"} compatibles en Modrinth…`;
    try {
      const result = await invoke<{ projects: typeof modrinthProjects; total_hits: number; page: number; page_size: number }>("search_modrinth_mods", {
        query: modSearch,
        gameVersion: modGameVersion,
        loader: modLoader,
        projectType: modProjectType,
        page: modPage
      });
      modrinthProjects = result.projects;
      modTotalHits = result.total_hits;
      modMessage = result.total_hits ? `${result.total_hits.toLocaleString()} resultados · página ${result.page} de ${Math.max(1, Math.ceil(result.total_hits / result.page_size))}.` : `No se encontraron ${modProjectType === "mod" ? "mods" : "modpacks"} para esos filtros.`;
    } catch (error) {
      modrinthProjects = [];
      modTotalHits = 0;
      modMessage = String(error);
    } finally {
      modSearchBusy = false;
    }
  }

  async function goToModPage(page: number) {
    modPage = Math.min(modPageCount, Math.max(1, page));
    await searchModrinthMods();
  }

  function getModPageButtons(): number[] {
    if (modPageCount <= 7) return Array.from({ length: modPageCount }, (_, index) => index + 1);
    if (modPage <= 4) return [1, 2, 3, 4, 5, -1, modPageCount];
    if (modPage >= modPageCount - 3) return [1, -1, modPageCount - 4, modPageCount - 3, modPageCount - 2, modPageCount - 1, modPageCount];
    return [1, -1, modPage - 1, modPage, modPage + 1, -1, modPageCount];
  }

  async function exportModPack() {
    packBusy = true;
    packMessage = "Preparando ZIP de la instancia…";
    try {
      const result = await invoke<ExportResult>("export_modpack_zip", {
        username: microsoftProfile?.name ?? (username.trim() || "Steve"),
        versionId: selectedVersion.split(/\s+/)[0]
      });
      exportReport=result;
      packMessage = `Pack creado correctamente: ${result.path}`;
    } catch (error) {
      packMessage = String(error);
    } finally {
      packBusy = false;
    }
  }

  async function importModPack(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    if (file.size > 512 * 1024 * 1024) {
      packMessage = "El ZIP supera el límite de importación de 512 MB.";
      input.value = "";
      return;
    }
    packBusy = true;
    packMessage = "Validando e importando el pack…";
    try {
      const archiveBase64 = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result));
        reader.onerror = () => reject(reader.error);
        reader.readAsDataURL(file);
      });
      const count = await invoke<number>("import_modpack_zip", {
        username: microsoftProfile?.name ?? (username.trim() || "Steve"),
        versionId: selectedVersion.split(/\s+/)[0],
        archiveBase64
      });
      packMessage = `Se importaron ${count} mods. La versión y el loader del pack coinciden con la instalación actual.`;
      await refreshMods();
    } catch (error) {
      packMessage = String(error);
    } finally {
      packBusy = false;
      input.value = "";
    }
  }

  async function installModrinthProject(projectId: string) {
    installingProject = projectId;
    modMessage = "Descargando y verificando el mod…";
    try {
      modMessage = await invoke<string>("install_modrinth_mod", {
        projectId,
        versionId: selectedVersion.split(/\s+/)[0],
        loader: modLoader,
        username: microsoftProfile?.name ?? (username.trim() || "Steve")
      });
      await refreshMods();
    } catch (error) {
      modMessage = String(error);
    } finally {
      installingProject = "";
    }
  }

  async function setInstalledModEnabled(fileName: string, enabled: boolean) {
    try {
      await invoke("set_mod_enabled", {
        username: microsoftProfile?.name ?? (username.trim() || "Steve"),
        versionId: selectedVersion.split(/\s+/)[0],
        fileName,
        enabled
      });
      await refreshMods();
    } catch (error) {
      modMessage = String(error);
    }
  }

  async function removeInstalledMod(fileName: string) {
    try {
      await invoke("remove_mod", {
        username: microsoftProfile?.name ?? (username.trim() || "Steve"),
        versionId: selectedVersion.split(/\s+/)[0],
        fileName
      });
      modMessage = `Se quitó ${fileName}.`;
      await refreshMods();
    } catch (error) {
      modMessage = String(error);
    }
  }

  async function refreshInstallations() {
    installationsRefreshing = true;
    installationsError = "";
    try {
      installedVersions = await invoke<{ id: string; jar: boolean; json: boolean; size_mb: number; loader: string; game_version: string }[]>("list_installed_versions");
      if (modGameVersion === "all") modGameVersion = selectedMinecraftVersion;
    } catch (err) {
      installationsError = `No se pudieron leer las instalaciones locales: ${String(err)}`;
      addConsoleLine("stderr", installationsError);
    } finally {
      installationsRefreshing = false;
    }
  }

  function onGameVersionChanged() {
    selectedPackId = "";
    modGameVersion = selectedMinecraftVersion;
    saveConfig();
    modPage = 1;
    void refreshJavaRequirement();
    void refreshMods();
  }

  async function copyConsole() {
    try {
      await navigator.clipboard.writeText(consoleText);
      consoleMessage = "Consola copiada al portapapeles.";
    } catch (err) { consoleMessage = `No se pudo copiar: ${String(err)}`; }
  }

  function clearConsole() {
    consoleLines = [];
    consoleLastActivity=Date.now();
    consoleMessage = "Vista de consola limpiada. Los registros de diagnóstico se conservan en el disco.";
  }

  function addConsoleLine(stream: string, line: string, timestamp = Date.now()) {
    consoleLastActivity=Date.now();
    consoleLines = boundedConsole([...consoleLines, { timestamp, time: new Date(timestamp).toLocaleTimeString(), stream, line }]);
  }

  async function refreshJavaRequirement() {
    const version = optifineOptions.find(item=>item.id===selectedVersion)?.game_version ?? selectedVersion.split(" ")[0];
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
    settingsTab="java";
  }

  async function refreshJavaRuntimes() {
    javaRuntimes = await invoke("list_java_runtimes");
  }

  async function toggleJava(major: number, enabled: boolean) {
    try { await invoke("set_java_runtime_enabled", { major, enabled }); await refreshJavaRuntimes(); }
    catch (err) { javaManagerMessage = String(err); throw err; }
  }

  async function installJavaVersion(major: number) {
    javaManagerMessage = `Instalando Java ${major}…`;
    try { javaManagerMessage = await invoke("install_portable_java", { major }); await refreshJavaRuntimes(); }
    catch (err) { javaManagerMessage = String(err); throw err; }
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
      bitmap.close();
      const png = canvas.toDataURL("image/png");
      await invoke("set_custom_app_icon", { imageBase64: png });
      iconPreview = png;
      iconMessage = "Logo personalizado guardado.";
    } catch (err) { iconMessage = `No se pudo cambiar el icono: ${String(err)}`; }
    finally { input.value = ""; }
  }

  async function removeCustomIcon() {
    try {
      await invoke("remove_custom_app_icon");
      iconPreview = "";
      iconMessage = "Logo predeterminado de TP LAUNCHER restaurado.";
    } catch (err) {
      iconMessage = `No se pudo quitar el logo: ${String(err)}`;
      throw err;
    }
  }

  async function playInstalled(id: string) {
    selectedPackId = "";
    selectedVersion = id;
    void refreshMods();
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
      await refreshAccounts();
    } catch (err) { loginMessage = String(err); const line = `Inicio de sesión Microsoft: ${String(err)}`; void invoke("record_launcher_log", { stream: "stderr", line }).catch(() => addConsoleLine("stderr", line)); }
    finally { loginBusy = false; }
  }

  async function signOutMicrosoft() {
    try { await invoke("logout_microsoft"); microsoftProfile = null; username = "Steve"; saveConfig(); loginMessage = "Sesión cerrada."; await refreshAccounts(); }
    catch (err) { loginMessage = String(err); }
  }

  async function useOfflineMode() {
    const nick=username.trim()||"Steve";
    if(!/^[A-Za-z0-9_]{3,16}$/.test(nick)){activeTab="ajustes";settingsTab="accounts";loginMessage="Elige un nick local de 3 a 16 letras, números o guiones bajos.";return;}
    loginBusy=true;
    try{await addOfflineAccount(nick);loginMessage=`Perfil local ${nick} seleccionado. Sin autenticación Microsoft.`;launchMessage=loginMessage;activeTab="jugar";}
    catch(error){loginMessage=String(error);}
    finally{loginBusy=false;}
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

  async function loadVersionCatalog(includeOld: boolean, selectForHome?: typeof homeVersionCategory) {
    try {
      const response = await invoke<{ id: string; kind?: string; type?: string }[]>("fetch_mojang_versions", { includeOld });
      const entries = response
        .map((entry) => ({ id: entry.id, kind: entry.kind ?? entry.type ?? "" }))
        .filter((entry) => entry.id && entry.kind);
      versionCatalog = entries;
      versionCatalogError = "";
      const selectionCategory = selectForHome ?? (activeTab === "instalaciones" ? installationCategory : homeVersionCategory);
      const expectedKind = selectionCategory === "old" ? "old" : selectionCategory;
      const matching = selectionCategory ? entries.filter((entry) => expectedKind === "old"
        ? entry.kind === "old_alpha" || entry.kind === "old_beta"
        : entry.kind === expectedKind) : [];
      if (selectionCategory
        && !selectedPackId && matching.length && !matching.some((entry) => entry.id === selectedVersion)) {
        selectedVersion = matching[0].id;
        saveConfig();
      }
      void refreshJavaRequirement();
    } catch (err) {
      versionCatalogError = `No se pudo cargar el catálogo de versiones: ${String(err)}`;
      addConsoleLine("stderr", versionCatalogError);
    }
  }

  function alignLoaderSelection(loader:LoaderCategory){
    if(homeVersionCategory===loader&&!selectedPackId&&!homeVersionOptions.some(item=>item.id===selectedVersion)){
      selectedVersion=homeVersionOptions[0]?.id??'';saveConfig();void refreshJavaRequirement();
    }
    if(loader==='optifine'){
      if(!optifineOptions.some(item=>item.id===optifineTarget))optifineTarget=optifineOptions[0]?.id??'';
    }else if(activeTab==='instalaciones'&&installationCategory===loader){
      const rows=loader==='fabric'?fabricCatalog:loader==='forge'?forgeCatalog:loader==='neoforge'?neoforgeCatalog:quiltCatalog;
      if(!rows.some(item=>item.id===fabricTargetVersion))fabricTargetVersion=rows[0]?.id??'';
    }
  }

  async function refreshLoaderCatalogs(selectCategory?: LoaderCategory) {
    const results = await Promise.allSettled([
      invoke<typeof fabricCatalog>('fetch_fabric_versions').then(value=>{fabricCatalog=value;alignLoaderSelection('fabric');return value;}),
      invoke<typeof forgeCatalog>('fetch_forge_versions').then(value=>{forgeCatalog=value;alignLoaderSelection('forge');return value;}),
      invoke<typeof optifineCatalog>('fetch_optifine_versions').then(value=>{optifineCatalog=value;alignLoaderSelection('optifine');return value;}),
      invoke<ExtraLoaderVersion[]>('fetch_extra_loader_catalog',{loader:'neoforge'}).then(value=>{neoforgeCatalog=value;alignLoaderSelection('neoforge');return value;}),
      invoke<ExtraLoaderVersion[]>('fetch_extra_loader_catalog',{loader:'quilt'}).then(value=>{quiltCatalog=value;alignLoaderSelection('quilt');return value;}),
      invoke<OptiFineCandidate[]>('discover_optifine_files').then(value=>{optifineCandidates=value;alignLoaderSelection('optifine');return value;})
    ]);
    const errors:string[]=[];
    const [fabric,forge,optifine,neoforge,quilt,files]=results;
    if(fabric.status==='fulfilled')fabricCatalog=fabric.value;else errors.push('Fabric: '+String(fabric.reason));
    if(forge.status==='fulfilled')forgeCatalog=forge.value;else errors.push('Forge: '+String(forge.reason));
    if(optifine.status==='fulfilled')optifineCatalog=optifine.value;else errors.push('OptiFine: '+String(optifine.reason));
    if(neoforge.status==='fulfilled')neoforgeCatalog=neoforge.value;else errors.push('NeoForge: '+String(neoforge.reason));
    if(quilt.status==='fulfilled')quiltCatalog=quilt.value;else errors.push('Quilt: '+String(quilt.reason));
    if(files.status==='fulfilled')optifineCandidates=files.value;else errors.push('Detección OptiFine: '+String(files.reason));
    if(!optifineOptions.some(item=>item.id===optifineTarget))optifineTarget=optifineOptions[0]?.id??'';
    loaderCatalogError=errors.join('\n');
    errors.forEach(error=>addConsoleLine('stderr','No se pudo actualizar el catálogo: '+error));
    if(selectCategory&&homeVersionCategory===selectCategory){
      if(!homeVersionOptions.some(item=>item.id===selectedVersion))selectedVersion=homeVersionOptions[0]?.id??'';
      saveConfig();void refreshJavaRequirement();
    }
    if(selectCategory&&activeTab==='instalaciones'&&installationCategory===selectCategory&&selectCategory!=='optifine'){
      const catalog=selectCategory==='fabric'?fabricCatalog:selectCategory==='forge'?forgeCatalog:selectCategory==='neoforge'?neoforgeCatalog:quiltCatalog;
      if(!catalog.some(item=>item.id===fabricTargetVersion))fabricTargetVersion=catalog[0]?.id??'';
    }
  }

  function setHomeVersionCategory(category: typeof homeVersionCategory) {
    selectedPackId='';versionSearch='';homeVersionCategory=category;showOldVersions=category==='old';
    if(category==='old'||category==='release'||category==='snapshot')void loadVersionCatalog(category==='old',category);
    else{
      if(!homeVersionOptions.some(item=>item.id===selectedVersion))selectedVersion=homeVersionOptions[0]?.id??'';
      if(category!=='installed')void refreshLoaderCatalogs(category);
      if(category==='optifine'){
        if(!homeVersionOptions.length)void openOptifineDownloads();
        void scanOptifine();
      }
    }
    saveConfig();void refreshJavaRequirement();
  }

  function setInstallationCategory(category: typeof installationCategory) {
    selectedPackId='';installationCategory=category;
    if(category!=='release'&&category!=='snapshot'&&category!=='old'){
      if(category==='optifine'){
        optifineTarget=optifineOptions[0]?.id??'';
        if(!optifineOptions.length)void openOptifineDownloads();
        void scanOptifine();
      }else{
        const catalog=category==='fabric'?fabricCatalog:category==='forge'?forgeCatalog:category==='neoforge'?neoforgeCatalog:quiltCatalog;
        const current=installedVersions.find(item=>item.id===selectedVersion)?.game_version;
        fabricTargetVersion=(current&&catalog.some(item=>item.id===current)?current:catalog[0]?.id)??'';
      }
      void refreshLoaderCatalogs(category);return;
    }
    homeVersionCategory=category;showOldVersions=category==='old';versionSearch='';
    const matches=versionCatalog.filter(entry=>category==='old'?entry.kind==='old_alpha'||entry.kind==='old_beta':entry.kind===category);
    if(matches.length&&!matches.some(entry=>entry.id===selectedVersion)){selectedVersion=matches[0].id;saveConfig();void refreshJavaRequirement();}
    void loadVersionCatalog(category==='old');
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

  async function checkLaunchAccount():Promise<boolean>{
    if(checkingLaunchAccount)return false;
    checkingLaunchAccount=true;
    try{
      const profile=await invoke<MicrosoftProfile|null>('get_microsoft_profile');
      if(profile){microsoftProfile=profile;username=profile.name;saveConfig();return true;}
      microsoftProfile=null;
      const eligibility=await invoke<{allowed:boolean;message:string}>('get_offline_eligibility');
      offlineAllowed=eligibility.allowed;offlineMessage=eligibility.message;
      if(eligibility.allowed)return true;
      loginMessage=eligibility.message;
      launchMessage='Elige una cuenta en Cuenta antes de preparar la partida.';
    }catch(error){
      loginMessage=String(error);
      launchMessage='No se pudo recuperar la cuenta. Revisa el detalle en Cuenta.';
    }finally{checkingLaunchAccount=false;}
    activeTab='skins';
    return false;
  }

  async function handleLaunch(serverAddress?:string) {
    if(!await checkLaunchAccount())return;
    if(!$gameSession.running){consoleLines=[];consoleSessionStarted++;consoleLastActivity=Date.now();consoleMessage='Nueva sesión. Los registros anteriores se conservan en el disco.';}
    secretInteraction("play");
    saveConfig();
    isLaunching = true;
    setGamePreparing(true);
    launchMessage = "";
    compatibilityWarnings=[];
    try {
      const prepared=await invoke<{installed:string[];skipped:string[];launch_version?:string|null;compatibility_warnings?:string[]}>('prepare_launcher_plugins',{
        username,versionId:selectedVersion.split(/\s+/)[0],instanceId:selectedPackId||null,
        gameVersion:selectedMinecraftVersion,loader:selectedModLoader
      });
      compatibilityWarnings=prepared.compatibility_warnings??[];
      if(prepared.launch_version){selectedVersion=prepared.launch_version;await refreshJavaRequirement();}
      for(const line of [...prepared.installed,...prepared.skipped])addConsoleLine('plugins',line);
      const res = await invoke<string>("launch_game", {
        username,
        version: selectedVersion,
        ramGb,
        instanceId: selectedPackId || null
        ,serverAddress:serverAddress||null
      });
      launchMessage = res;
      recentVersions = [selectedVersion, ...recentVersions.filter((entry) => entry !== selectedVersion)].slice(0, 6);
      saveConfig();
    } catch (err) {
      launchMessage = "Error: " + String(err);
      void invoke("record_launcher_log", { stream: "stderr", line: launchMessage }).catch(() => addConsoleLine("stderr", launchMessage));
    } finally {
      setGamePreparing(false);
      await refreshInstallations();
      setTimeout(() => {
        isLaunching = false;
      }, 1500);
    }
  }

  let appearance=$derived<AppearanceSettings>({selectedThemeId,primaryColor,secondaryColor,bgBaseColor,transparencyPreset,bgOpacity,menuOpacity,barOpacity,cardOpacity,blurPx,backgroundUrl});
  let settingsSaveQueue:Promise<void>=Promise.resolve();
  function changeSettings(patch:Partial<LauncherSettings>):Promise<void>{


    const pending=settingsSaveQueue.then(async()=>{
      settings=await invoke<LauncherSettings>("save_launcher_settings",{settings:{...settings,...patch}});
      setLocale(settings.language);ramGb=settings.ram_gb;saveConfig();
      if(patch.data_root!==undefined){rootPath=await invoke<string>("get_root_path");await refreshInstallations();await refreshStudioPacks();}
    });
    settingsSaveQueue=pending.catch(()=>{});
    return pending;
  }
  async function changeAppearance(patch:Partial<AppearanceSettings>){
    if(patch.selectedThemeId!==undefined)selectedThemeId=patch.selectedThemeId;
    if(patch.primaryColor!==undefined)primaryColor=patch.primaryColor;
    if(patch.secondaryColor!==undefined)secondaryColor=patch.secondaryColor;
    if(patch.bgBaseColor!==undefined)bgBaseColor=patch.bgBaseColor;
    if(patch.transparencyPreset!==undefined)transparencyPreset=patch.transparencyPreset;
    if(patch.bgOpacity!==undefined)bgOpacity=patch.bgOpacity;
    if(patch.menuOpacity!==undefined)menuOpacity=patch.menuOpacity;
    if(patch.barOpacity!==undefined)barOpacity=patch.barOpacity;
    if(patch.cardOpacity!==undefined)cardOpacity=patch.cardOpacity;
    if(patch.blurPx!==undefined)blurPx=patch.blurPx;
    if(patch.backgroundUrl!==undefined)backgroundUrl=await invoke<string>("set_custom_background",{imageBase64:patch.backgroundUrl});
    saveConfig();
  }
  async function refreshAccounts(){try{accounts=await invoke<AccountSummary[]>("list_launcher_accounts");const eligibility=await invoke<{allowed:boolean;message:string}>("get_offline_eligibility");offlineAllowed=eligibility.allowed;offlineMessage=eligibility.message;const offline=accounts.find(a=>a.active&&a.kind==="offline");if(offline&&!microsoftProfile)username=offline.name;}catch(error){addConsoleLine("stderr",String(error));}}
  async function activateAccount(id:string){const result=await invoke<{account:AccountSummary;profile:MicrosoftProfile|null}>("activate_launcher_account",{id});microsoftProfile=result.profile;username=result.account.name;saveConfig();await refreshAccounts();void refreshMods();}
  async function addOfflineAccount(name:string){const account=await invoke<AccountSummary>("add_offline_account",{name});await activateAccount(account.id);}
  async function removeAccount(id:string){const wasActive=accounts.find(account=>account.id===id)?.active;await invoke("remove_launcher_account",{id});if(wasActive){microsoftProfile=null;username="Steve";saveConfig();}await refreshAccounts();}
  async function addMicrosoftAccount(){await connectMicrosoft();await refreshAccounts();}
  async function finishWizard(){await changeSettings({first_run_completed:true,ram_gb:ramGb});showWizard=false;}
  async function checkUpdates(){const result=await invoke<UpdateInfo>("check_launcher_updates");updateInfo=result;if(result.available&&result.verifiable&&settings.automatic_updates)void downloadUpdate();return result.message;}
  async function downloadUpdate(){if(!updateInfo?.asset_id||updateBusy)return;updateBusy=true;updateMessage="Descargando actualización verificada…";try{const result=await invoke<{path:string}>("download_launcher_update",{assetId:updateInfo.asset_id});updateDownload=result.path;updateMessage="Actualización descargada y verificada. Puedes instalarla cuando cierres Minecraft.";}catch(error){updateMessage=String(error);}finally{updateBusy=false;}}
  async function installUpdate(){if(!updateDownload||updateBusy)return;updateBusy=true;try{await invoke("install_launcher_update",{path:updateDownload});}catch(error){updateMessage=String(error);updateBusy=false;}}
  async function openQuickFolder(folder:QuickFolder){await invoke("open_launcher_folder",{folder});}
  async function joinServer(address:string){activeTab="jugar";await launchSelectedVersion(address);}
  function revealSecret(egg:EasterEgg){
    if(egg.kind==='appearance')visualEgg=egg;else secretMoment=egg;
    clearTimeout(secretTimeout);secretTimeout=setTimeout(()=>secretMoment=null,12000);
    if(egg.kind==='console'||egg.kind==='achievement'||egg.kind==='date')void invoke("record_launcher_log",{stream:"secreto",line:egg.text}).catch(()=>{});
  }
  function secretInteraction(trigger:SecretTrigger){const egg=onSecretInteraction(trigger);if(egg)revealSecret(egg);}
  function touchLogo(){const now=Date.now();logoClicks=now-logoLastClick<5000?logoClicks+1:1;logoLastClick=now;if(logoClicks>=5){logoClicks=0;onSecretInteraction("logo");if(logoSecretActive)stopLogoEffect();else startLogoEffect();}}
</script>


<main
  class="launcher"
  data-design="blockcraft"
  use:translateUi
  style:background-image={backgroundUrl ? `linear-gradient(rgba(${hexToRgb(bgBaseColor)},.35),rgba(${hexToRgb(bgBaseColor)},.7)),url("${backgroundUrl}")` : "none"}
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
    --shell-stone: none;
  "
>
  {#if !$isGameRunning}<EndBackdrop/>{/if}
  <div class="world-grid" aria-hidden="true"></div>
  <ContentNotifications />

  <aside class="sidebar">
    <div>
      <button class="logo" onclick={touchLogo} aria-label="TP LAUNCHER">
        <img class="custom-logo" src={iconPreview || '/logos/tp-launcher.svg'} alt={iconPreview ? 'Logo personalizado' : 'Logo original de TP LAUNCHER'} />
        <span class="brand-copy"><span class="brand-title"><BrandWordmark/></span><small>JAVA EDITION</small></span>
      </button>

      <nav class="menu" aria-label="Navegación principal">
        <span class="menu-section-label">TU AVENTURA</span>
        <button class:active={activeTab === "jugar"} onclick={() => (activeTab = "jugar")}>
          <span class="nav-slot" aria-hidden="true"><UiIcon name="play" size={20}/></span><span>Jugar</span>
        </button>
        <button class:active={activeTab === "mods"} onclick={() => { activeTab = "mods"; }}>
          <span class="nav-slot" aria-hidden="true"><UiIcon name="grid" size={20}/></span><span>Mods y modpacks</span>
        </button>
        <button class:active={activeTab === "instalaciones"} onclick={() => { activeTab = "instalaciones"; void refreshInstallations(); }}>
          <span class="nav-slot" aria-hidden="true"><UiIcon name="folder" size={20}/></span><span class="install-nav-label">Instalaciones</span>{#if installedVersions.length}<small class="menu-count" data-no-translate>{installedVersions.length}</small>{/if}
        </button>
        <button class:active={activeTab === "servers"} onclick={() => activeTab="servers"}><span class="nav-slot" aria-hidden="true"><UiIcon name="globe" size={20}/></span><span>Servidores</span></button>
        <button class:active={activeTab === "legal"} onclick={() => activeTab="legal"}><span class="nav-slot" aria-hidden="true"><UiIcon name="shield" size={20}/></span><span>Términos y condiciones</span></button>
        <button class:active={activeTab === "plugins"} onclick={() => activeTab="plugins"}><span class="nav-slot" aria-hidden="true"><UiIcon name="plugins" size={20}/></span><span>Plugins</span><small class="menu-count">HUD</small></button>
        <span class="menu-section-label">TU PERSONAJE</span>
        <button class:active={activeTab === "skins"} onclick={() => {activeTab="skins";secretInteraction("skin");}}>
          <span class="nav-slot" aria-hidden="true"><UiIcon name="user" size={20}/></span><span>Cuenta y skins</span>
        </button>
        <button class:active={activeTab === "poser"} onclick={() => {activeTab="poser";secretInteraction("studio");}}><span class="nav-slot" aria-hidden="true"><UiIcon name="pose" size={20}/></span><span>Estudio 3D</span></button>
        <button class:active={activeTab === "capturas"} onclick={() => activeTab="capturas"}><span class="nav-slot" aria-hidden="true"><UiIcon name="image" size={20}/></span><span>Capturas</span></button>
        <button class:active={activeTab === "logros"} onclick={() => activeTab="logros"}><span class="nav-slot" aria-hidden="true"><UiIcon name="trophy" size={20}/></span><span>Logros</span><small class="menu-count" data-no-translate>{secrets.count}</small></button>
        <span class="menu-section-label">EL LAUNCHER</span>
        <button class:active={activeTab === "comunidad"} onclick={() => activeTab="comunidad"}><span class="nav-slot" aria-hidden="true"><UiIcon name="users" size={20}/></span><span>Comunidad</span></button>
        <button class:active={activeTab === "consola"} onclick={() => {activeTab="consola";secretInteraction("console");}}>
          <span class="nav-slot" aria-hidden="true"><UiIcon name="terminal" /></span><span>Consola</span>
        </button>
        <button class:active={activeTab === "ajustes"} onclick={() => (activeTab = "ajustes")}>
          <span class="nav-slot" aria-hidden="true"><UiIcon name="settings" /></span><span>Ajustes</span>
        </button>
      </nav>
    </div>

    <div class="profile-quick-card">
      <PlayerAvatar skinUrl={activeSkinUrl} name={microsoftProfile?.name ?? (username.trim() || "Steve")} />
      <div class="profile-quick-info"><strong data-no-translate>{microsoftProfile?.name ?? (username.trim() || "Steve")}</strong><small>{microsoftProfile ? "Cuenta Microsoft" : (offlineAppearance ? "Vista previa premium · Offline" : "Modo Offline · Steve")}</small></div>
      <button class="install-java" onclick={()=>{activeTab="ajustes";settingsTab="accounts";void refreshAccounts();}}><UiIcon name="users" size={14}/>Cambiar cuenta</button>
    </div>
  </aside>

  <div class="workspace-shell">

  <section class="content" class:home-content={activeTab === "jugar"} bind:this={contentPanel}>
    <header class="launcher-topbar"><div class="section-location"><span><BrandWordmark/> <UiIcon name="arrow" size={12}/></span><strong>{sectionNames[activeTab]}</strong></div><div class="topbar-actions">{#if !$isGameRunning}<WorldSky compact/>{:else}<span class="playing-indicator"><UiIcon name="play" size={14}/>{$gameSession.running?'Minecraft en ejecución':'Preparando partida…'}</span>{/if}</div></header>
    {#if updateInfo?.available}<aside class="update-banner"><div><strong>Nueva versión de TP LAUNCHER · {updateInfo.version}</strong><small>{updateMessage||updateInfo.message}</small></div>{#if updateDownload}<button onclick={installUpdate} disabled={updateBusy}><UiIcon name="download" size={15}/>Instalar actualización</button>{:else if updateInfo.verifiable}<button onclick={downloadUpdate} disabled={updateBusy}>{updateBusy?"Descargando…":"Descargar actualización"}</button>{/if}</aside>{/if}
    {#if activeTab === "jugar"}
      <div class="hero">
        <div class="hero-edition"><UiIcon name="creeper" size={15}/><span>MINECRAFT · JAVA EDITION</span><span class="edition-line"></span></div>
        <h1>Tu próximo mundo{" "}<span>empieza aquí.</span></h1>
        <p class="hero-message">{heroMessages[subtitleIndex]}</p>
        {#if secretMoment}<p class="secret-moment" data-no-translate>{secretMoment.text}</p>{/if}
      </div>
      <div class="play-layout">
        <div class="background-character">
          {#if !$isGameRunning}<HeroShowcase fallbackSkinUrl={accountSkinFallback} skinUrl={activeSkinUrl} capeUrl={activeCapeUrl} model={activeSkinModel} pose={welcomeSession.pose} equipment={welcomeSession.equipment} easterEgg={visualEgg?.variant==='herobrine'} egg={visualEgg} whiteEyes={logoSecretActive} />{:else}<div class="game-running-placeholder"><PlayerAvatar skinUrl={activeSkinUrl}/><strong>Tu partida está abierta</strong><small>Las animaciones del menú están pausadas.</small></div>{/if}
        </div>
        <div class="launch-panel">

      <section class="bottom-bar" aria-label="Configuración de lanzamiento">
        <div class="launch-block-title"><span class="launch-block-icon"><LoaderLogo loader="vanilla" size={25}/></span><div><small>TU MUNDO TE ESPERA</small><h2>Prepara tu partida</h2></div><span class="java-edition-tag">JAVA</span></div>
        <div class="controls-wrapper">
<div class="control-group">
            <label for="version"><VersionIcon name="dimension"/>Versión</label>
            {#if showOldVersions}<input class="version-search" aria-label="Buscar versión" placeholder="Buscar versión…" bind:value={versionSearch} />{/if}
            <div class="home-version-tabs" aria-label="Filtrar versiones">
              <button class:active={homeVersionCategory === "installed"} onclick={() => setHomeVersionCategory("installed")}><UiIcon name="grid" size={16}/>Instaladas</button>
              <button class:active={homeVersionCategory === "release"} onclick={() => setHomeVersionCategory("release")}><VersionIcon name="release"/>Releases</button>
              <button class:active={homeVersionCategory === "forge"} onclick={() => setHomeVersionCategory("forge")}><LoaderLogo loader="forge" size={16} compact/>Forge</button>
              <button class:active={homeVersionCategory === "fabric"} onclick={() => setHomeVersionCategory("fabric")}><LoaderLogo loader="fabric" size={16}/>Fabric</button>
              <button class:active={homeVersionCategory === "neoforge"} onclick={() => setHomeVersionCategory("neoforge")}><LoaderLogo loader="neoforge" size={16}/>NeoForge</button>
              <button class:active={homeVersionCategory === "quilt"} onclick={() => setHomeVersionCategory("quilt")}><LoaderLogo loader="quilt" size={16}/>Quilt</button>
              <button class:active={homeVersionCategory === "optifine"} onclick={() => setHomeVersionCategory("optifine")}><LoaderLogo loader="optifine" size={16}/>OptiFine</button>
              <button class:active={homeVersionCategory === "snapshot"} onclick={() => setHomeVersionCategory("snapshot")}><VersionIcon name="snapshot"/>Snapshots</button>
              <button class:active={homeVersionCategory === "old"} onclick={() => setHomeVersionCategory("old")}><VersionIcon name="legacy"/>Betas / Alphas</button>
            </div>
            <select id="version" bind:value={selectedVersion} onchange={(event)=>{selectedVersion=event.currentTarget.value;onGameVersionChanged();}}>
              {#each homeVersionOptions.filter((item) => !showOldVersions || item.id.toLowerCase().includes(versionSearch.toLowerCase())) as ver}
                <option value={ver.id}>{ver.label}</option>
              {/each}
            </select>
            <small class="java-recommendation">Java recomendado: {recommendedJavaMajor === null ? "Consultando…" : `Java ${Math.max(8, recommendedJavaMajor)}`} · {javaRecommendationStatus}</small>
            {#if !homeVersionOptions.length}<small>{homeVersionCategory === "installed" ? "No hay instalaciones locales todavía. Descarga una desde Instalaciones." : ["fabric","forge","neoforge","quilt","optifine"].includes(homeVersionCategory) ? homeVersionCategory === "optifine" ? "Descarga tu versión oficial desde optifine.net; TP Launcher la detectará automáticamente." : `El catálogo de ${homeVersionCategory} todavía no ha devuelto versiones.` : "No hay versiones para este filtro."}</small>{/if}
            {#if homeVersionCategory==='optifine'}<div class="optifine-actions"><button onclick={()=>void openOptifineDownloads()}><UiIcon name="external" size={14}/>Descargas oficiales</button><button onclick={()=>void scanOptifine()} disabled={scanningOptifine}><UiIcon name="refresh" size={14}/>{scanningOptifine?'Buscando…':'Detectar JAR descargados'}</button></div>{#if optifineMessage}<small role="status">{optifineMessage}</small>{/if}{/if}
            {#if versionCatalogError && !homeVersionOptions.length}<small class="inline-error">{versionCatalogError}</small>{/if}
            {#if loaderCatalogError && ["fabric", "forge", "neoforge", "quilt", "optifine"].includes(homeVersionCategory)}<small class="inline-error">{loaderCatalogError}</small>{/if}
          </div>
<button class="play-btn" onclick={()=>void launchSelectedVersion()} disabled={isLaunching || preparingHomeVersion || checkingLaunchAccount || (!selectedVersion && homeVersionCategory!=="optifine")}>
          <UiIcon name={isLaunching||preparingHomeVersion||checkingLaunchAccount?'refresh':'play'} size={22}/><span>{checkingLaunchAccount ? "COMPROBANDO CUENTA…" : preparingHomeVersion ? "INSTALANDO CARGADOR…" : isLaunching ? "PREPARANDO..." : "JUGAR AHORA"}</span>
        </button>
<div class="control-group">
            <label for="nick"><UiIcon name="user" size={14}/>Nick del Jugador{microsoftProfile ? " · Cuenta Microsoft" : " · Modo Offline"}</label>
            <input
              id="nick"
              type="text"
              bind:value={username}
              oninput={(event)=>{username=event.currentTarget.value;saveConfig();}}
              placeholder="Tu Nick..."
              disabled={!!microsoftProfile}
            />
            {#if appearancePending}<small class="appearance-status" role="status">Buscando skin del jugador…</small>{/if}
          </div>
<div class="control-group ram-control">
            <label for="ram"><UiIcon name="chip" size={14}/>Memoria RAM: <strong>{ramGb} GB</strong></label>
            <input
              id="ram"
              type="range"
              min="2"
              max={Math.max(2,Math.min(128,totalMemoryGb))}
              step="1"
              bind:value={ramGb}
              oninput={(event)=>{ramGb=Number(event.currentTarget.value);saveConfig();}}
              onchange={()=>void changeSettings({ram_gb:ramGb})}
            />
          </div>
</div>
</section>
      {#if launchMessage}
        <div class="status-log">
          <code>> {launchMessage}</code>
        </div>
      {/if}
      <PlayerQuickTools onexplore={section => { studioInitialSection=section;activeTab="mods"; }} username={microsoftProfile?.name ?? username} currentVersion={selectedVersion.split(/\s+/)[0]} instanceId={selectedPackId} onstats={value => playtime=value} />
      {#if compatibilityWarnings.length}<aside class="compatibility-notice" role="status"><UiIcon name="shield" size={19}/><div><strong>Compatibilidad de esta partida</strong>{#each compatibilityWarnings as warning}<p>{warning}</p>{/each}</div></aside>{/if}
        </div>
      </div>
      <div class="home-overview">
        <div class="quick-cards">
          <div class="info-card">
            <span class="card-label"><UiIcon name="user" size={13}/>PERFIL ACTIVO</span>
            <strong data-no-translate>{username}</strong>
          </div>
          <div class="info-card">
            <span class="card-label"><UiIcon name="cube" size={13}/>VERSIÓN SELECCIONADA</span>
            <strong data-no-translate>{optifineOptions.find(item=>item.id===selectedVersion)?.name??selectedVersion}</strong>
          </div>
          <div class="info-card">
            <span class="card-label"><UiIcon name="chip" size={13}/>MEMORIA ASIGNADA</span>
            <strong>{ramGb} GB</strong>
          </div>
          <div class="info-card"><span class="card-label"><UiIcon name="clock" size={13}/>TIEMPO JUGADO / SESIÓN</span><strong>{formatPlaytime(playtime.total_seconds)}</strong><small>{formatPlaytime(playtime.session_seconds)} esta sesión · {playtime.active_games} juegos abiertos</small></div>
        </div>


      {#if selectedPack}<div class="selected-pack-banner"><div><small>MODPACK ACTIVO</small><strong>{selectedPack.name}</strong><span>Minecraft {selectedPack.game_version} · {selectedPack.loader}</span></div><button onclick={() => {selectedPackId="";saveConfig();}}><UiIcon name="user" size={15}/>Usar perfil normal</button></div>{/if}
      {#if recentVersions.length}
        <section class="recent-versions module-card">
          <div class="module-actions"><h3><GameIcon name="clock" size={18}/>Acceso rápido · jugadas recientemente</h3><button onclick={() => { activeTab = "instalaciones"; void refreshInstallations(); }}><UiIcon name="grid" size={14}/>Ver instalaciones</button></div>
          <div class="recent-version-list">{#each recentVersions as recent}
            <button class="recent-version-card" onclick={() => playInstalled(recent)}><strong data-no-translate>{recent}</strong><span>Jugar de nuevo</span></button>
          {/each}</div>
        </section>
      {/if}




</div>
      <div class="home-secondary">
      {#if !$isGameRunning}<HostingOptions compact/>{/if}
      {#if studioPacks.length}<section class="module-card pack-home"><div class="module-actions"><h3><GameIcon name="chest" size={18}/>Mis modpacks</h3><button onclick={() => activeTab="mods"}><UiIcon name="cube" size={14}/>Abrir estudio</button></div><div class="pack-tiles">{#each studioPacks as pack}<button disabled={!!pack.install_state && pack.install_state!=="ready"} title={pack.install_state && pack.install_state!=="ready"?"Instalación pendiente · Abre Mis modpacks para reintentar":"Jugar modpack"} onclick={() => void playStudioPack(pack.version_id,pack.id)}>{#if pack.icon_data || pack.icon_url}<img src={pack.icon_data || pack.icon_url} alt="" />{:else}<LoaderLogo loader={pack.loader} size={30}/>{/if}<span><strong data-no-translate>{pack.name}</strong><small>{pack.game_version} · {pack.loader}</small></span><UiIcon name="play" /></button>{/each}</div></section>{/if}

      {#if !$isGameRunning}<NewsHub onexplore={section => { studioInitialSection=section;activeTab="mods"; }}/>{/if}
      <section class="mini-console module-card">
        <div class="module-actions"><h3><UiIcon name="terminal" size={18}/>Consola en vivo</h3><button onclick={() => (activeTab = "consola")}><UiIcon name="terminal" size={14}/>Abrir consola completa</button></div>
        <div class="mini-console-output" aria-live="polite">
          {#if recentConsoleLines.length}{#each recentConsoleLines as entry}<div class:error-line={entry.stream === "stderr" || /error|exception|crash/i.test(entry.line)}><span>[{entry.time}] [{entry.stream}]</span> <span data-diagnostic>{entry.line}</span></div>{/each}{:else}<p>Los mensajes de descargas y del proceso de Minecraft aparecerán aquí.</p>{/if}
        </div>
      </section>
      </div>

    {:else if activeTab === "instalaciones"}
      <div class="module-panel">
        <header class="installations-header"><div><h2><GameIcon name="chest" size={26}/>Instalaciones</h2><p>Administra perfiles descargados, revisa su estado y vuelve a iniciar cualquiera desde aquí.</p><code>{rootPath || "TP LAUNCHER"}\versions</code></div><div class="install-total"><strong>{installedVersions.length}</strong><span>perfiles locales</span><small>{totalInstalledSize.toFixed(1)} MB en disco</small></div></header>
        <section class="module-card installed-manager">
          <div class="module-actions installed-manager-heading"><div><h3><GameIcon name="book" size={18}/>Biblioteca local</h3><p>Se leen directamente todas las carpetas de versiones del launcher.</p></div><button class="refresh-button" onclick={refreshInstallations} disabled={installationsRefreshing}>{installationsRefreshing ? "Actualizando…" : "Actualizar lista"}</button></div>
          {#if installationsError}<p class="inline-error" role="alert">{installationsError}</p>{/if}
          <div class="installed-toolbar"><input class="installed-search" type="search" bind:value={installedSearch} placeholder="Buscar perfil o versión de Minecraft" aria-label="Buscar instalaciones"/><div class="installed-filters" aria-label="Filtrar instalaciones">
            <button class:active={installedFilter === "all"} onclick={() => (installedFilter = "all")}>Todas <span>{installedVersions.length}</span></button>
            <button class:active={installedFilter === "vanilla"} onclick={() => (installedFilter = "vanilla")}><LoaderLogo loader="vanilla" size={14}/>Vanilla</button>
            <button class:active={installedFilter === "fabric"} onclick={() => (installedFilter = "fabric")}><LoaderLogo loader="fabric" size={14}/>Fabric</button>
            <button class:active={installedFilter === "forge"} onclick={() => (installedFilter = "forge")}><LoaderLogo loader="forge" size={14}/>Forge</button>
            <button class:active={installedFilter === "neoforge"} onclick={() => (installedFilter = "neoforge")}><LoaderLogo loader="neoforge" size={14}/>NeoForge</button>
            <button class:active={installedFilter === "quilt"} onclick={() => (installedFilter = "quilt")}><LoaderLogo loader="quilt" size={14}/>Quilt</button>
            <button class:active={installedFilter === "optifine"} onclick={() => (installedFilter = "optifine")}><LoaderLogo loader="optifine" size={14}/>OptiFine</button>
            <button class:active={installedFilter === "custom"} onclick={() => (installedFilter = "custom")}><UiIcon name="cube" size={13}/>Otros</button>
          </div></div>
          {#if filteredInstalledVersions.length}
            <div class="installed-grid">{#each filteredInstalledVersions as item}
              <article class="installed-card">
                <div class="installed-card-top"><span class="installed-mark"><LoaderLogo loader={item.loader} size={32}/></span><div class="installed-card-title"><strong data-no-translate>{item.id}</strong><small>Minecraft {item.game_version} · {item.loader === "vanilla" ? "Vanilla" : item.loader}</small></div><span class:complete={item.jar && item.json} class="install-state">{item.jar && item.json ? "Lista" : "Incompleta"}</span></div>
                <div class="installed-card-meta"><span>JAR {item.jar ? "disponible" : "pendiente"}</span><span>JSON {item.json ? "disponible" : "pendiente"}</span><span>{item.size_mb.toFixed(1)} MB</span></div>
                <div class="row-buttons"><button class="primary-action" onclick={() => playInstalled(item.id)}><UiIcon name="play" size={15}/>Jugar</button><button onclick={() => invoke("open_version_folder", { version: item.id })}><UiIcon name="folder" size={15}/>Abrir carpeta</button><button class="danger-button" onclick={() => removeInstalled(item.id)}><UiIcon name="trash" size={15}/>Eliminar</button></div>
              </article>
            {/each}</div>
          {:else if installationsRefreshing}
            <div class="install-empty"><strong>Actualizando biblioteca</strong><span>Buscando instalaciones en la carpeta local…</span></div>
          {:else if installedVersions.length}
            <div class="install-empty"><strong>No hay coincidencias</strong><span>Cambia el filtro o prueba otra búsqueda.</span></div>
          {:else}
            <div class="install-empty"><strong>No se encontraron perfiles</strong><span>Al terminar una descarga, el perfil aparecerá aquí automáticamente. También puedes crearlo desde el panel de instalación nueva.</span></div>
          {/if}
        </section>

        <section class="module-card"><div class="module-actions"><div><h3>Instancias de Modpack Studio</h3><p>Perfiles aislados con sus propios mundos, mods y configuración.</p></div><button onclick={() => { activeTab="mods"; }}>Crear o administrar</button></div>{#if studioPacks.length}<div class="pack-tiles">{#each studioPacks as pack}<button disabled={!!pack.install_state && pack.install_state!=="ready"} title={pack.install_state && pack.install_state!=="ready"?"Instalación pendiente · Abre Mis modpacks para reintentar":"Jugar modpack"} onclick={() => void playStudioPack(pack.version_id,pack.id)}>{#if pack.icon_data || pack.icon_url}<img src={pack.icon_data || pack.icon_url} alt="" />{:else}<LoaderLogo loader={pack.loader} size={30}/>{/if}<span><strong data-no-translate>{pack.name}</strong><small>{pack.game_version} · {pack.loader}</small></span><UiIcon name="play" /></button>{/each}</div>{:else}<p>Crea tu primer modpack desde Mods y modpacks. Sus archivos se guardarán en una instancia independiente.</p>{/if}</section>

        <section class="module-card new-install-manager">
          <div class="module-actions"><div><h3>Nueva instalación</h3><p>Elige el catálogo y la versión que quieres preparar.</p></div><span class="catalog-status">{installationCatalogCount ? `${installationCatalogCount.toLocaleString()} ${installationCatalogLabel}` : "Cargando catálogo…"}</span></div>
          <div class="category-tabs install-source-tabs" aria-label="Catálogo para nueva instalación">
            <button class:active={installationCategory === "release"} onclick={() => setInstallationCategory("release")}><VersionIcon name="release" size={16}/>Releases</button>
            <button class:active={installationCategory === "snapshot"} onclick={() => setInstallationCategory("snapshot")}><VersionIcon name="snapshot" size={16}/>Snapshots</button>
            <button class:active={installationCategory === "old"} onclick={() => setInstallationCategory("old")}><VersionIcon name="legacy" size={16}/>Betas y Alphas</button>
            <button class:active={installationCategory === "fabric"} onclick={() => setInstallationCategory("fabric")}><LoaderLogo loader="fabric" size={14}/>Fabric</button>
            <button class:active={installationCategory === "forge"} onclick={() => setInstallationCategory("forge")}><LoaderLogo loader="forge" size={14}/>Forge</button>
              <button class:active={installationCategory === "neoforge"} onclick={() => setInstallationCategory("neoforge")}><LoaderLogo loader="neoforge" size={14}/>NeoForge</button>
              <button class:active={installationCategory === "quilt"} onclick={() => setInstallationCategory("quilt")}><LoaderLogo loader="quilt" size={14}/>Quilt</button>
            <button class:active={installationCategory === "optifine"} onclick={() => setInstallationCategory("optifine")}><LoaderLogo loader="optifine" size={14}/>OptiFine</button>
          </div>
          {#if versionCatalogError && ["release", "snapshot", "old"].includes(installationCategory)}<p class="inline-error" role="alert">{versionCatalogError}</p>{/if}
          {#if loaderCatalogError && ["fabric", "forge", "neoforge", "quilt", "optifine"].includes(installationCategory)}<p class="inline-error" role="alert">{loaderCatalogError}</p>{/if}
          {#if installationCategory === "fabric"}
            <div class="installer-description"><strong>Fabric Loader</strong><span>Instala el loader estable para una versión release o snapshot compatible.</span></div>
            <div class="new-install-row"><select bind:value={fabricTargetVersion} aria-label="Versión base para Fabric">{#each fabricCatalog as item}<option value={item.id}>{item.id}{item.stable ? " · Estable" : " · Snapshot"}</option>{/each}</select><button class="primary-action" onclick={installFabricForSelectedVersion} disabled={fabricBusy || !fabricCatalog.some((v) => v.id === fabricTargetVersion)}>{fabricBusy ? "Instalando…" : "Instalar Fabric"}</button></div>
            {#if fabricMessage}<p class="inline-message">{fabricMessage}</p>{/if}
          {:else if installationCategory === "forge"}
            <div class="installer-description"><strong>Forge</strong><span>Consulta el catálogo oficial y crea una instalación separada para la versión base.</span></div>
            <div class="new-install-row"><select bind:value={fabricTargetVersion} aria-label="Versión base para Forge">{#each forgeCatalog as item}<option value={item.id}>{item.id} · Forge {item.forge_version} · {item.channel}</option>{/each}</select><button class="primary-action" onclick={installForgeForSelectedVersion} disabled={forgeBusy || !forgeCatalog.some((v) => v.id === fabricTargetVersion)}>{forgeBusy ? "Instalando…" : "Instalar Forge"}</button></div>
            {#if forgeMessage}<p class="inline-message">{forgeMessage}</p>{/if}
          {:else if installationCategory === "neoforge" || installationCategory === "quilt"}
            <div class="installer-description"><strong>{installationCategory==='neoforge'?'NeoForge':'Quilt Loader'}</strong><span>Catálogo oficial verificado con las versiones de Minecraft. Se crea un perfil independiente.</span></div>
            <div class="new-install-row"><select bind:value={fabricTargetVersion} aria-label="Versión base para NeoForge o Quilt">{#each installationCategory==='neoforge'?neoforgeCatalog:quiltCatalog as item}<option value={item.id}>{item.id} · {item.loader_version}{item.stable?'':' · Prueba'}</option>{/each}</select><button class="primary-action" onclick={()=>void installExtraForSelectedVersion(installationCategory as 'neoforge'|'quilt')} disabled={extraLoaderBusy||!(installationCategory==='neoforge'?neoforgeCatalog:quiltCatalog).some(item=>item.id===fabricTargetVersion)}>{extraLoaderBusy?'Instalando…':'Instalar '+(installationCategory==='neoforge'?'NeoForge':'Quilt')}</button></div>
            {#if extraLoaderMessage}<p class="inline-message" role="status">{extraLoaderMessage}</p>{/if}
          {:else if installationCategory === "optifine"}
            <div class="installer-description"><strong>OptiFine · detección local</strong><span>Descarga tu versión oficial desde optifine.net; TP Launcher la detectará automáticamente. Los JAR detectados solo se importan al pulsar Instalar o Jugar.</span></div>
            <div class="new-install-row"><select bind:value={optifineTarget} aria-label="Versión de OptiFine detectada" disabled={!optifineOptions.length}>{#if !optifineOptions.length}<option value="">Descarga un JAR desde la web oficial</option>{/if}{#each optifineOptions as item}<option value={item.id}>Minecraft {item.game_version} · {item.name} · {item.downloaded?'JAR detectado':'Instalado'}</option>{/each}</select><button class="primary-action" disabled={optifineBusy} onclick={()=>void installDetectedOptifine()}>{optifineBusy?'Importando…':optifineTarget?'Instalar / seleccionar':'Descargar OptiFine'}</button></div>
            <div class="optifine-actions"><button onclick={()=>void openOptifineDownloads()}><UiIcon name="external" size={14}/>Web oficial</button><label><UiIcon name="folder" size={14}/>Importar otro JAR<input type="file" accept=".jar,application/java-archive" disabled={optifineBusy} onchange={importOptifineFile}/></label><button onclick={()=>{void refreshInstallations();void scanOptifine();}} disabled={scanningOptifine||optifineBusy}><UiIcon name="refresh" size={14}/>{scanningOptifine?'Buscando…':'Detectar descargas'}</button></div>
            {#if optifineMessage}<p class="inline-message" role="status">{optifineMessage}</p>{/if}
          {:else}
            <div class="installer-description"><strong>{installationCategory === "old" ? "Minecraft histórico" : installationCategory === "snapshot" ? "Snapshot" : "Minecraft Vanilla"}</strong><span>El launcher descargará el cliente, sus bibliotecas, sonidos y recursos al iniciar.</span></div>
            <div class="new-install-row"><select bind:value={selectedVersion} onchange={(event) => {selectedVersion=event.currentTarget.value;homeVersionCategory=installationCategory;showOldVersions=installationCategory==="old";onGameVersionChanged();}} aria-label="Versión para instalar">{#each versionCatalog.filter((v) => installationCategory === "old" ? v.kind === "old_alpha" || v.kind === "old_beta" : v.kind === installationCategory) as item}<option value={item.id}>{item.id}{item.kind === "old_alpha" ? " · Alpha" : item.kind === "old_beta" ? " · Beta" : item.kind === "snapshot" ? " · Snapshot" : " · Release"}</option>{/each}</select><button class="primary-action" onclick={() => playInstalled(selectedVersion)} disabled={!selectedVersion.trim()}>Descargar y jugar</button></div>
          {/if}
        </section>
      </div>
    {:else if activeTab === "legal"}
      <TermsConditions/>
    {:else if activeTab === "ajustes"}
      <SettingsSuite {settings} onchange={changeSettings} {appearance} onappearance={changeAppearance} {themes} runtimes={javaRuntimes} {accounts} {offlineAllowed} {offlineMessage} secretsFound={secrets.count} secretsTotal={secrets.total} initialTab={settingsTab} {totalMemoryGb} onrepeatwizard={()=>{showWizard=true;}} onscanjava={refreshJavaRuntimes} oninstalljava={installJavaVersion} ontogglejava={toggleJava} ontestjava={path=>invoke<string>("test_java_executable",{path})} onaddmicrosoft={addMicrosoftAccount} onaddoffline={addOfflineAccount} onactivateaccount={activateAccount} onremoveaccount={removeAccount} onopenfolder={openQuickFolder} onpickicon={useCustomIcon} onremoveicon={removeCustomIcon} {iconPreview} {iconMessage} oncheckupdates={checkUpdates}/>
    {:else if activeTab === "mods"}
      <ModStudio initialSection={studioInitialSection} {versionCatalog} currentVersion={selectedVersion.split(/\s+/)[0]} currentGameVersion={selectedMinecraftVersion} currentLoader={selectedModLoader} currentPackId={selectedPackId} username={microsoftProfile?.name ?? username} onplay={playStudioPack} oninstalled={() => { void refreshInstallations(); void refreshStudioPacks(); }} />
    {:else if activeTab === "capturas"}
      <ScreenshotGallery />
    {:else if activeTab === "logros"}
      <AchievementsPanel />
    {:else if activeTab === "comunidad"}
      <CommunityTools />
    {:else if activeTab === "plugins"}
      <PluginsPanel />
    {:else if activeTab === "skins"}
      <AccountWardrobe fallbackSkinUrl={accountSkinFallback} profile={microsoftProfile} skinUrl={activeSkinUrl} capeUrl={activeCapeUrl} model={activeSkinModel} {username} {loginBusy} {loginMessage} {offlineAllowed} {offlineMessage} onlogin={() => void connectMicrosoft()} onlogout={() => void signOutMicrosoft()} onoffline={() => void useOfflineMode()} onprofile={updateMicrosoftProfile} onpreview={previewLocalSkin} onposer={() => activeTab="poser"} />
      <SocialProfile fallbackSkinUrl={accountSkinFallback} playerId={microsoftProfile?.id??`offline:${username.toLowerCase()}`} name={username} skinUrl={activeSkinUrl} capeUrl={activeCapeUrl} model={activeSkinModel} paused={$isGameRunning}/>
    {:else if activeTab === "poser"}
      <PoseStudio fallbackSkinUrl={accountSkinFallback} skinUrl={activeSkinUrl} capeUrl={activeCapeUrl} model={activeSkinModel} onclose={() => activeTab="jugar"} />
    {:else if activeTab === "servers"}
      {#if !$isGameRunning}<ServersPanel username={microsoftProfile?.name??username} currentVersion={selectedVersion.split(/\s+/)[0]} instanceId={selectedPackId} currentGameVersion={selectedMinecraftVersion} currentLoader={selectedModLoader} onjoin={joinServer} onconfigure={()=>activeTab="jugar"}/>{:else}<section class="module-card"><h2>Partida en curso</h2><p>El catálogo y sus consultas están pausados mientras juegas. Volverán al cerrar Minecraft.</p></section>{/if}
    {:else if activeTab === "consola"}
      <div class="module-panel console-panel">
        <header><h2><UiIcon name="terminal" size={25}/>Consola de TP LAUNCHER</h2><p>Registro en vivo de descargas, runtime Java y salida estándar/error de Minecraft.</p></header>
        <section class="module-card console-card">
          <div class="module-actions"><h3><UiIcon name="terminal" size={18}/>Salida del launcher y del juego</h3><div class="console-actions"><button onclick={copyConsole}><UiIcon name="copy" size={15}/>Copiar todo</button><button onclick={clearConsole}><UiIcon name="trash" size={15}/>Limpiar consola</button></div></div>
          <div class="console-output" bind:this={consoleScrollElement} aria-live="polite" role="log">
            {#if consoleLines.length}{#each consoleLines as entry}<div class:console-error={entry.stream === "stderr" || /error|exception|crash/i.test(entry.line)}><time>[{entry.time}]</time> <b>[{entry.stream}]</b> <span data-diagnostic>{entry.line}</span></div>{/each}{:else}<p>Consola vacía. Inicia Minecraft para ver el registro.</p>{/if}
          </div>
          {#if consoleMessage}<p class="inline-message">{consoleMessage}</p>{/if}
        </section>
      </div>
    {:else}
      <div class="placeholder-tab">
        <h2>Módulo en Construcción</h2>
        <p>Pronto conectaremos esta sección con el motor de TP LAUNCHER.</p>
      </div>
    {/if}
    <LegalNotice compact onterms={()=>activeTab="legal"}/>
  </section>
  {#if progress}
    <div class="download-progress" role="status"><div class="progress-copy"><strong>{progress.task}</strong><span>{progress.status} · {progress.percent}%</span></div><div class="progress-track"><div class="progress-fill" style="width: {progress.percent}%"></div></div></div>
  {/if}
  {#if showLaunchDock}
    <LaunchDock bind:username bind:selectedVersion bind:ramGb category={homeVersionCategory} options={homeVersionOptions} collapsed={launchDockCollapsed} premium={!!microsoftProfile} skinUrl={activeSkinUrl} maxMemory={totalMemoryGb} recommendedJava={recommendedJavaMajor} javaStatus={javaRecommendationStatus} busy={isLaunching||checkingLaunchAccount} preparing={preparingHomeVersion} packName={selectedPack?.name??''} message={launchMessage} catalogError={!homeVersionOptions.length?(['fabric','forge','neoforge','quilt','optifine'].includes(homeVersionCategory)?loaderCatalogError:versionCatalogError):''} ontoggle={()=>{launchDockCollapsed=!launchDockCollapsed;saveConfig();}} oncategory={setHomeVersionCategory} onversionchange={()=>{versionSearch="";onGameVersionChanged();}} onconfig={saveConfig} onmemorycommit={()=>void changeSettings({ram_gb:ramGb})} onlaunch={()=>void launchSelectedVersion()} onoptifinedownload={()=>void openOptifineDownloads()} onclearpack={()=>{selectedPackId='';saveConfig();void refreshJavaRequirement();}}/>
  {/if}
  </div>
  <ExportReportDialog report={exportReport} onclose={()=>exportReport=null}/>
  {#if showWizard}<SetupWizard {settings} onchange={changeSettings} {appearance} onappearance={changeAppearance} {themes} runtimes={javaRuntimes} onscanjava={refreshJavaRuntimes} oninstalljava={installJavaVersion} onconnectmicrosoft={addMicrosoftAccount} oncomplete={finishWizard} {totalMemoryGb} accountName={microsoftProfile?.name}/>{/if}
</main>

<style>
  .compatibility-notice{display:flex;align-items:flex-start;gap:12px;padding:15px 17px;margin:15px 0;border:1px solid #c6a15c40;border-radius:5px;background:#9f782912;color:#d9c19c}.compatibility-notice>div{min-width:0}.compatibility-notice strong{font-size:.82rem}.compatibility-notice p{font-size:.76rem;line-height:1.6;margin:5px 0 0;color:#bfb4a1}
  .optifine-actions{display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin:12px 0}.optifine-actions>button,.optifine-actions>label{display:inline-flex;align-items:center;gap:7px;border:1px solid #ffffff25;border-radius:4px;background:#ffffff08;color:inherit;padding:8px 11px;font:inherit;font-size:.76rem;cursor:pointer}.optifine-actions input{max-width:180px;font-size:.66rem}
  .playing-indicator{display:inline-flex;align-items:center;gap:7px;color:var(--primary);font-size:.67rem}.game-running-placeholder{height:100%;min-height:320px;display:flex;flex-direction:column;justify-content:center;align-items:center;gap:13px;text-align:center;color:var(--primary)}.game-running-placeholder small{font-size:.7rem;color:#9d91b1}

  @font-face{font-family:'Monocraft';src:url('/fonts/Monocraft.ttf') format('truetype');font-display:swap;font-style:normal;font-weight:400 900}
  :global(html), :global(body) {
    margin: 0;
    font-family: 'Segoe UI', system-ui, sans-serif;
    background: transparent !important;
    background-color: transparent !important;
    color: #f1f5f9;
    user-select: none;
    overflow: hidden;
    color-scheme: dark;
  }

  :global(::-webkit-scrollbar) { width: 8px; height: 8px; }
  :global(::-webkit-scrollbar-track) { background: rgba(15, 23, 42, .4); border-radius: 999px; }
  :global(::-webkit-scrollbar-thumb) { background: rgba(99, 102, 241, .5); border: 1px solid rgba(255,255,255,.06); border-radius: 999px; }
  :global(::-webkit-scrollbar-thumb:hover) { background: rgba(99, 102, 241, .8); }
  :global(*) { scrollbar-color: rgba(99, 102, 241, .6) rgba(15, 23, 42, .4); scrollbar-width: thin; }
  :global(select), :global(option) { color-scheme: dark; color: #f1f5f9; }
  :global(select) { background-color: rgba(15, 23, 42, .88) !important; border-color: rgba(255,255,255,.15); }
  :global(option) { background-color: #111827; color: #f8fafc; }
  :global(input:not([type="range"]):not([type="color"]):not([type="file"])) { color-scheme: dark; }

  .launcher {
    display: flex;
    height: 100vh;
    background: rgba(var(--bg-base-rgb), var(--bg-opacity));
    transition: background 0.25s ease;
  }

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
    border:0;background:transparent;padding:0;cursor:pointer;color:inherit;
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 900;
    font-size: 1.2rem;
    letter-spacing: 1px;
    margin-bottom: 28px;
  }
  .custom-logo { width: 28px; height: 28px; object-fit: contain; border-radius: 6px; }

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
  .profile-quick-info { display: flex; min-width: 0; flex-direction: column; gap: 3px; }
  .profile-quick-info strong { overflow: hidden; text-overflow: ellipsis; }
  .profile-quick-info small, .java-recommendation { color: #aab7ca; font-size: .76rem; }
  .profile-quick-card .install-java { grid-column: 1 / -1; width: 100%; margin-top: 0; }
  .java-recommendation { display: block; margin-top: 7px; }


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

  .recent-versions { margin: 0 0 16px; }
  .recent-version-list { display: flex; gap: 9px; flex-wrap: wrap; margin-top: 12px; }
  .recent-version-card { min-width: 140px; display: flex; flex-direction: column; align-items: flex-start; gap: 4px; padding: 10px 12px; border: 1px solid rgba(var(--primary-rgb),.3); border-radius: 9px; background: rgba(var(--bg-base-rgb),.5); color: #e2e8f0; cursor: pointer; transition: transform .18s ease,border-color .18s ease; }
  .recent-version-card:hover { transform: translateY(-2px); border-color: var(--primary); }
  .recent-version-card span { color: #94a3b8; font-size: .76rem; }

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

  .control-group input[type="range"] {
    accent-color: var(--primary);
    cursor: pointer;
  }
  .home-version-tabs { display: flex; flex-wrap: wrap; gap: 5px; max-height: 78px; overflow: auto; padding: 2px 1px; }
  .home-version-tabs button { padding: 5px 7px; border: 1px solid rgba(var(--primary-rgb),.25); border-radius: 7px; background: rgba(var(--bg-base-rgb),.65); color: #cbd5e1; cursor: pointer; font-size: .68rem; white-space: nowrap; }
  .home-version-tabs button.active { border-color: var(--primary); background: rgba(var(--primary-rgb),.18); color: var(--primary); }
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
  .category-tabs { display: flex; flex-wrap: wrap; gap: 8px; }
  .category-tabs button, .row-buttons button { padding: 8px 11px; border: 1px solid rgba(var(--primary-rgb), .35); background: rgba(var(--primary-rgb), .1); color: #dbeafe; border-radius: 8px; cursor: pointer; transition: all .18s ease; }
  .category-tabs button.active, .category-tabs button:hover, .row-buttons button:hover { background: rgba(var(--primary-rgb), .24); border-color: var(--primary); color: var(--primary); }
  .category-tabs button:disabled { opacity: .45; cursor: not-allowed; }
  .installations-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; padding: 4px 2px 2px; }
  .installations-header h2 { margin: 0 0 6px; color: var(--primary); font-size: 1.65rem; }
  .installations-header p { margin: 0 0 8px; color: #aab7ca; }
  .install-total { min-width: 145px; display: grid; gap: 2px; text-align: right; padding: 10px 14px; border: 1px solid rgba(var(--primary-rgb), .28); border-radius: 12px; background: rgba(var(--primary-rgb), .07); }
  .install-total strong { color: var(--primary); font-size: 1.4rem; line-height: 1.2; }
  .install-total span { color: #dbeafe; font-size: .84rem; }
  .install-total small, .installed-manager-heading p, .new-install-manager .module-actions p { color: #94a3b8; }
  .installed-manager-heading { align-items: flex-start; }
  .installed-manager-heading h3, .new-install-manager .module-actions h3 { margin: 0 0 4px; }
  .installed-manager-heading p, .new-install-manager .module-actions p { margin: 0; font-size: .87rem; }
  .refresh-button { flex: none; padding: 9px 13px; border: 1px solid rgba(var(--primary-rgb), .5); border-radius: 8px; background: rgba(var(--primary-rgb), .15); color: #e9d5ff; cursor: pointer; }
  .refresh-button:disabled { opacity: .6; cursor: wait; }
  .installed-toolbar { display: grid; gap: 12px; margin: 16px 0 12px; }
  .installed-search { width: 100%; box-sizing: border-box; padding: 10px 12px; border: 1px solid rgba(var(--primary-rgb), .3); border-radius: 8px; outline: none; background: rgba(9,13,22,.82); color: #f1f5f9; }
  .installed-filters { display: flex; flex-wrap: wrap; gap: 7px; }
  .installed-filters button { display: inline-flex; align-items: center; gap: 6px; padding: 7px 10px; border: 1px solid rgba(var(--primary-rgb), .24); border-radius: 7px; background: rgba(9,13,22,.45); color: #cbd5e1; cursor: pointer; transition: background .16s ease, border-color .16s ease, color .16s ease; }
  .installed-filters button.active, .installed-filters button:hover { border-color: var(--primary); background: rgba(var(--primary-rgb), .16); color: var(--primary); }
  .installed-filters button span { color: #94a3b8; font-size: .76rem; }
  .installed-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, 340px), 1fr)); gap: 11px; }
  .installed-card { min-width: 0; padding: 13px; border: 1px solid rgba(var(--primary-rgb), .2); border-radius: 10px; background: linear-gradient(145deg, rgba(var(--bg-base-rgb), .74), rgba(var(--primary-rgb), .045)); transition: border-color .18s ease, transform .18s ease, box-shadow .18s ease; }
  .installed-card:hover { transform: translateY(-2px); border-color: rgba(var(--primary-rgb), .55); box-shadow: 0 8px 22px rgba(0,0,0,.18); }
  .installed-card-top { display: flex; align-items: center; gap: 10px; min-width: 0; }
  .installed-mark { flex: none; display: grid; place-items: center; width: 38px; height: 38px; border: 1px solid rgba(var(--primary-rgb), .35); border-radius: 9px; background: rgba(var(--primary-rgb), .14); color: var(--primary); font-size: .88rem; font-weight: 800; }
  .installed-card-title { display: flex; flex: 1; flex-direction: column; gap: 3px; min-width: 0; }
  .installed-card-title strong { overflow: hidden; color: #f1f5f9; text-overflow: ellipsis; white-space: nowrap; }
  .installed-card-title small { color: #94a3b8; }
  .install-state { flex: none; padding: 4px 7px; border: 1px solid rgba(251,191,36,.35); border-radius: 99px; color: #fcd34d; font-size: .72rem; }
  .install-state.complete { border-color: rgba(52,211,153,.35); color: #6ee7b7; }
  .installed-card-meta { display: flex; flex-wrap: wrap; gap: 6px 12px; margin: 12px 0; color: #94a3b8; font-size: .76rem; }
  .installed-card .row-buttons { display: flex; }
  .installed-card .row-buttons button { flex: 1; min-width: 0; padding: 7px 8px; font-size: .78rem; }
  .installed-card .row-buttons .primary-action { font-size: .8rem; }
  .install-empty { display: grid; gap: 6px; place-items: center; min-height: 100px; padding: 14px; border: 1px dashed rgba(var(--primary-rgb), .25); border-radius: 9px; text-align: center; }
  .install-empty strong { color: #e2e8f0; }
  .install-empty span { max-width: 540px; color: #94a3b8; font-size: .86rem; }
  .catalog-status { color: #94a3b8; font-size: .8rem; }
  .install-source-tabs { margin: 16px 0 12px; }
  .installer-description { display: grid; gap: 4px; margin: 10px 0; }
  .installer-description strong { color: #e2e8f0; }
  .installer-description span { color: #94a3b8; font-size: .86rem; }
  .inline-error { padding: 9px 11px; border: 1px solid rgba(248,113,113,.35); border-radius: 8px; background: rgba(127,29,29,.14); color: #fca5a5 !important; overflow-wrap: anywhere; }
  .row-buttons { display: flex; flex-wrap: wrap; gap: 6px; }
  .row-buttons .danger-button { border-color: rgba(248,113,113,.4); color: #fca5a5; }
  .new-install-row { display: flex; flex-wrap: wrap; gap: 10px; }
  .new-install-row select { flex: 1; min-width: 170px; background: rgba(9,13,22,.85); color: white; padding: 9px; border: 1px solid rgba(var(--primary-rgb), .35); border-radius: 8px; }
  .primary-action { padding: 9px 13px; border-radius: 8px; border: 1px solid rgba(var(--primary-rgb), .7); background: linear-gradient(135deg, rgba(var(--primary-rgb),.25), rgba(var(--primary-rgb),.1)); color: var(--primary); font-weight: 750; cursor: pointer; transition: transform .18s ease, box-shadow .18s ease; }
  .primary-action:hover { transform: translateY(-1px); box-shadow: 0 0 18px rgba(var(--primary-rgb),.2); }
  .inline-message { color: var(--primary) !important; overflow-wrap: anywhere; }
  @keyframes launcher-enter { from { opacity: 0; transform: translateY(7px); } to { opacity: 1; transform: translateY(0); } }
  .module-panel, .hero { animation: launcher-enter .28s ease-out both; }
  @media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: .01ms !important; transition-duration: .01ms !important; } }
  .download-progress { position: fixed; left: 260px; right: 32px; bottom: 18px; z-index: 5; padding: 12px 16px; border-radius: 12px; background: rgba(var(--bg-base-rgb), var(--bar-opacity)); backdrop-filter: blur(var(--blur-px)); border: 1px solid rgba(var(--primary-rgb), .4); box-shadow: 0 0 24px rgba(var(--primary-rgb), .18); }
  .progress-copy { display: flex; justify-content: space-between; gap: 12px; margin-bottom: 8px; color: var(--primary); font-size: .84rem; }
  .progress-copy span { color: #cbd5e1; }
  .progress-track { height: 7px; background: rgba(var(--primary-rgb), .15); border-radius: 8px; overflow: hidden; }
  .progress-fill { height: 100%; background: linear-gradient(90deg, var(--primary), var(--secondary)); box-shadow: 0 0 12px rgba(var(--primary-rgb), .7); transition: width .2s ease; }
  @media (max-width: 720px) { .download-progress { left: 12px; right: 12px; bottom: 10px; } }
  :global(*) { box-sizing: border-box; }
  .content { justify-content: flex-start; gap: 18px; }
  .secret-moment { color:var(--primary)!important;font-size:.78rem;letter-spacing:.03em; }
  .update-banner { display:flex;align-items:center;justify-content:space-between;gap:14px;padding:15px 18px;border:1px solid rgba(var(--primary-rgb),.35);border-radius:12px;background:rgba(var(--primary-rgb),.1); }
  .update-banner>div { display:grid;gap:5px; }.update-banner strong{font-size:.84rem}.update-banner small{font-size:.73rem;color:#b4c1d8}.update-banner button{color:#e8eaff;border:1px solid var(--primary);background:rgba(var(--primary-rgb),.15);padding:10px 14px;border-radius:8px;cursor:pointer;flex-shrink:0}
  .menu button { display:flex; align-items:center; gap:10px; }
  .hero h1 { font-size:clamp(1.65rem,2.7vw,2.5rem); }
  .quick-cards { grid-template-columns:repeat(4,minmax(0,1fr)); }
  .info-card strong { overflow-wrap:anywhere; }
  .info-card small { color:#94a3b8;font-size:.7rem; }


  .content.home-content { display:block; }
  .home-content > .update-banner { margin-bottom:22px; }
  .home-secondary { display:grid;grid-template-columns:minmax(0,1fr);gap:24px;margin-top:32px;padding-bottom:24px; }
  .home-secondary .mini-console { margin-top:0; }
  .play-layout { position:relative;isolation:isolate;display:flow-root;padding-right:40%;min-height:540px;height:auto;flex:none; }
  .background-character { position:absolute;right:-12px;top:0;bottom:0;width:42%;height:auto;z-index:0;pointer-events:none;user-select:none; }
  .launch-panel { position:relative;z-index:1;min-width:0;display:flex;flex-direction:column;gap:12px; }
  .appearance-status { color:#94a3b8;font-size:.72rem;letter-spacing:0;text-transform:none; }
  .launch-panel .bottom-bar { display:grid;grid-template-columns:minmax(0,1fr);align-items:stretch;gap:24px;height:auto;max-height:none;min-height:0;flex:none;padding:24px;overflow:visible; }
  .launch-panel .controls-wrapper { display:grid;grid-template-columns:1fr;gap:20px; }
  .launch-panel .play-btn { display:flex;align-items:center;justify-content:center;justify-self:center;position:relative;width:100%;min-height:60px;margin:0;padding:16px 24px;flex:none;line-height:1.3;text-align:center; }




  .selected-pack-banner { display:flex;align-items:center;justify-content:space-between;gap:12px;padding:16px;border:1px solid rgba(var(--primary-rgb),.4);border-radius:12px;background:rgba(var(--primary-rgb),.12); }
  .selected-pack-banner>div { display:grid;gap:5px; }
  .selected-pack-banner small { color:var(--primary);font-size:.65rem;letter-spacing:.12em; }
  .selected-pack-banner span { color:#aab7ca;font-size:.8rem; }
  .selected-pack-banner button { border:1px solid #ffffff25;border-radius:8px;background:#ffffff08;color:#cbd5e1;padding:8px;cursor:pointer; }
  .pack-tiles { display:grid;grid-template-columns:repeat(auto-fill,minmax(240px,1fr));gap:12px;margin-top:14px; }
  .pack-tiles button { display:flex;align-items:center;gap:12px;padding:14px;border:1px solid rgba(var(--primary-rgb),.22);border-radius:12px;background:rgba(var(--bg-base-rgb),.4);color:#e2e8f0;text-align:left;cursor:pointer; }
  .pack-tiles button:hover { border-color:var(--primary); }
  .pack-tiles img { width:42px;height:42px;object-fit:cover;border-radius:9px; }
  .pack-tiles span { flex:1;display:grid;gap:5px;min-width:0; }
  .pack-tiles small { color:#94a3b8; }
  @media(max-width:1100px) { .quick-cards { grid-template-columns:repeat(2,minmax(0,1fr)); }.sidebar { width:180px!important;min-width:180px!important;padding:18px 12px!important; }.content { padding:22px!important; } }
  @media(max-width:1000px) { .play-layout { padding-right:0;padding-top:300px; }.background-character { top:-16px;bottom:auto;right:0;width:100%;height:300px; }.background-character :global(.ambient-character) { height:300px; } }
  @media(max-width:640px) { .sidebar { width:74px!important;min-width:74px!important;padding:12px 8px!important; }.profile-quick-card { display:none!important; }.logo { justify-content:center; }.menu button { flex-direction:column;font-size:.6rem;padding:10px 2px;gap:5px; }.content { padding:16px!important; }.quick-cards { gap:8px; }.info-card { padding:11px; } }



  .launcher{--font-display:'Monocraft','Cascadia Mono',Consolas,monospace;--font-body:'Segoe UI',system-ui,sans-serif;position:relative;isolation:isolate;font-family:var(--font-body);background-size:cover;background-position:center;overflow:hidden}
  .world-grid{position:absolute;inset:0;z-index:-1;pointer-events:none;background-image:linear-gradient(90deg,rgba(var(--primary-rgb),.035) 1px,transparent 1px),linear-gradient(rgba(var(--primary-rgb),.035) 1px,transparent 1px),radial-gradient(ellipse at 85% 0,rgba(var(--primary-rgb),.11),transparent 60%);background-size:64px 64px,64px 64px,100% 100%}
  .sidebar{box-sizing:border-box;width:232px;min-width:232px;padding:22px 14px 16px;background:linear-gradient(160deg,rgba(var(--bg-base-rgb),var(--menu-opacity)),rgba(var(--bg-base-rgb),var(--menu-opacity)));border-right-color:rgba(var(--primary-rgb),.16);gap:20px;z-index:2}
  .sidebar>div:first-child{display:flex;flex-direction:column;min-height:0;flex:1}.logo{flex-shrink:0;gap:11px;padding:4px 7px 18px;margin:0 0 12px;border-bottom:1px solid rgba(var(--primary-rgb),.14);text-align:left}.custom-logo{width:38px;height:38px;flex-shrink:0;border-radius:5px}.brand-copy{display:flex;flex-direction:column;gap:6px;min-width:0}.brand-title{font-family:var(--font-display);font-size:.72rem;letter-spacing:-.04em;white-space:nowrap;color:#ecf0f6}.brand-copy>small{font-size:.48rem;font-weight:650;letter-spacing:.22em;color:#7f92aa}
  .menu{min-height:0;overflow-y:auto;overflow-x:hidden;gap:4px;padding:0 3px 8px;scrollbar-width:thin}.menu-section-label{font-size:.53rem;font-weight:750;letter-spacing:.15em;color:#6f839c;padding:16px 10px 8px}.menu-section-label:first-child{padding-top:7px}.menu button{display:flex;align-items:center;gap:11px;min-height:41px;padding:10px 11px;border-radius:5px;font-size:.8rem;font-weight:550;position:relative;flex-shrink:0;color:#9baec4}.menu button>span{flex:1}.menu button :global(svg){color:#8397b0;flex-shrink:0}.menu button:hover{transform:none;background:#ffffff05;border-color:#ffffff0c;color:#e6eefb}.menu button.active{background:linear-gradient(90deg,rgba(var(--primary-rgb),.18),rgba(var(--primary-rgb),.04));border-color:rgba(var(--primary-rgb),.25);color:#eef4ff;box-shadow:inset 3px 0 var(--primary)}.menu button.active :global(svg){color:var(--primary)}.menu-count{font-family:var(--font-display);font-size:.54rem;color:#a6b7cd;border:1px solid #ffffff12;background:#ffffff04;border-radius:3px;padding:4px 5px;min-width:13px;text-align:center}.menu button.active .menu-count{color:var(--primary);border-color:rgba(var(--primary-rgb),.2)}.profile-quick-card{flex-shrink:0;border-radius:6px;border-color:rgba(var(--primary-rgb),.22);background:linear-gradient(130deg,rgba(var(--primary-rgb),.09),#ffffff02);padding:12px;gap:9px}.profile-quick-info strong{font-size:.79rem;color:#e4edf8}.profile-quick-info small{font-size:.62rem;line-height:1.5}.profile-quick-card .install-java{font-size:.67rem;border-radius:4px;background:#ffffff04;border-color:#ffffff16;padding:8px}
  .content{display:block;justify-content:initial;box-sizing:border-box;padding:0 32px 30px;scroll-padding-top:18px;position:relative;z-index:1}.launcher-topbar{display:flex;align-items:center;justify-content:space-between;gap:20px;min-height:83px;border-bottom:1px solid #ffffff10;margin-bottom:27px;flex-shrink:0}.section-location{display:flex;align-items:center;gap:13px;min-width:0}.section-location>span{display:flex;align-items:center;gap:9px;font-size:.56rem;font-weight:700;letter-spacing:.1em;color:#71859f}.section-location>strong{font-size:.78rem;font-weight:600;color:#c5d3e5;white-space:nowrap}.topbar-actions{display:flex;align-items:center;gap:18px}
  .hero-edition{display:flex;align-items:center;gap:9px;margin-bottom:15px;color:var(--primary);font-size:.57rem;letter-spacing:.13em;font-weight:700}.edition-line{display:block;max-width:70px;flex:1;height:1px;background:linear-gradient(90deg,rgba(var(--primary-rgb),.4),transparent)}.hero h1{font-family:var(--font-display);font-size:clamp(1.45rem,2.1vw,2.05rem);line-height:1.6;letter-spacing:-.04em;font-weight:400;margin:0 0 10px;text-shadow:0 2px #0006}.hero h1 span{color:var(--primary);text-shadow:0 2px #0006}.hero .hero-message{font-size:.82rem;color:#9db0c7;max-width:750px;line-height:1.8;margin:0 0 24px}.hero .secret-moment{font-family:var(--font-display);font-size:.65rem;color:#bdc7db;border-left:2px solid var(--primary);padding:8px 12px;background:rgba(var(--primary-rgb),.055);line-height:1.8;margin-bottom:20px}.quick-cards{grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}.info-card{position:relative;overflow:hidden;border-radius:6px;border-color:#ffffff10;background:linear-gradient(135deg,rgba(var(--bg-base-rgb),var(--card-opacity)),rgba(var(--bg-base-rgb),var(--card-opacity)));padding:17px 16px;gap:9px;box-shadow:inset 0 1px #ffffff05;min-width:0}.info-card:after{content:'';position:absolute;bottom:0;left:16px;width:24px;height:2px;background:var(--primary);opacity:.45}.info-card:hover{transform:none;box-shadow:inset 0 1px #ffffff08;border-color:rgba(var(--primary-rgb),.24)}.card-label{display:flex;align-items:center;gap:7px;font-size:.51rem;letter-spacing:.08em;color:#8b9fb9}.info-card>strong{font-family:var(--font-display);font-size:.79rem;color:#dce6f6;line-height:1.6;overflow-wrap:anywhere}.info-card>small{font-size:.62rem;color:#768da9;line-height:1.6}
  .play-layout{padding-right:39%;min-height:565px}.background-character{width:41%;right:-10px}.launch-panel{gap:14px}.launch-panel .bottom-bar{padding:24px;gap:24px;max-width:none;border:1px solid rgba(var(--primary-rgb),.23);border-radius:8px;background:linear-gradient(145deg,rgba(var(--bg-base-rgb),var(--bar-opacity)),rgba(var(--bg-base-rgb),var(--bar-opacity)));box-shadow:inset 0 2px rgba(var(--primary-rgb),.05),0 16px 40px #0002;backdrop-filter:blur(var(--blur-px))}.launch-block-title{display:flex;align-items:center;gap:12px;padding-bottom:19px;border-bottom:1px solid #ffffff0c}.launch-block-icon{width:38px;height:38px;display:grid;place-items:center;flex-shrink:0;border:1px solid rgba(var(--primary-rgb),.2);border-radius:5px;color:var(--primary);background:rgba(var(--primary-rgb),.08)}.launch-block-title small{display:block;font-size:.47rem;font-weight:700;color:#7187a4;letter-spacing:.13em;margin-bottom:7px}.launch-block-title h2{font-family:var(--font-display);font-weight:400;font-size:.78rem;margin:0;line-height:1.6;color:#dce5f4}.java-edition-tag{margin-left:auto;color:#748aa5;letter-spacing:.1em;border:1px solid #ffffff12;border-radius:3px;font-size:.45rem;padding:5px 6px}.launch-panel .controls-wrapper{gap:22px}.control-group>label{font-size:.59rem;letter-spacing:.08em;color:#94a9c3;margin-bottom:9px}.control-group input[type=text],.control-group select{border-radius:5px;background:rgba(var(--bg-base-rgb),.85);border-color:#ffffff20;min-height:40px;font-size:.81rem;box-shadow:inset 0 2px 4px #0002}.control-group input:focus,.control-group select:focus{outline:2px solid rgba(var(--primary-rgb),.5);outline-offset:2px}.home-version-tabs{gap:5px;margin-bottom:9px}.home-version-tabs button{padding:6px 8px;border-radius:3px;border-color:#ffffff14;background:#ffffff03;color:#8fa4c1;font-size:.62rem}.home-version-tabs button.active{color:var(--primary);border-color:rgba(var(--primary-rgb),.5);background:rgba(var(--primary-rgb),.11);box-shadow:inset 0 -2px rgba(var(--primary-rgb),.35)}.java-recommendation{font-size:.65rem;color:#758da9;line-height:1.7}.launch-panel .play-btn{min-height:62px;gap:13px;border-radius:5px;border:1px solid rgba(var(--primary-rgb),.65);border-bottom:4px solid color-mix(in srgb,var(--primary) 48%,#000);background:linear-gradient(180deg,color-mix(in srgb,var(--primary) 84%,#fff),var(--primary));color:#0a1120;font-family:var(--font-display);font-size:.84rem;font-weight:400;letter-spacing:.03em;box-shadow:inset 0 2px #ffffff35,0 8px 24px rgba(var(--primary-rgb),.16);text-shadow:0 1px #ffffff18;transition:filter .16s,transform .16s,box-shadow .16s}.launch-panel .play-btn:hover:not(:disabled){filter:brightness(1.12);transform:translateY(-1px);box-shadow:inset 0 2px #ffffff40,0 10px 28px rgba(var(--primary-rgb),.24)}.launch-panel .play-btn:active:not(:disabled){transform:translateY(2px);box-shadow:inset 0 1px #ffffff20}.launch-panel .play-btn:disabled{filter:saturate(.55);cursor:wait;opacity:.65}.module-card{border-radius:6px;background:rgba(var(--bg-base-rgb),var(--card-opacity));border-color:#ffffff12;box-shadow:inset 0 1px #ffffff04}.module-card:hover{transform:none;box-shadow:inset 0 1px #ffffff08;border-color:rgba(var(--primary-rgb),.24)}.module-card h3{font-family:var(--font-display);font-size:.75rem;font-weight:400;line-height:1.7}.module-actions button{margin:0;border-radius:4px;font-size:.66rem;background:#ffffff04;border-color:#ffffff1a;color:#b4c5dd}.recent-versions{padding:17px}.recent-version-card{border-radius:4px;padding:10px 12px;min-width:130px;background:#ffffff03;border-color:#ffffff12}.recent-version-card strong{font-size:.72rem;color:#c4d1e4}.recent-version-card span{font-size:.62rem;color:#7188a5}.home-secondary{gap:25px;margin-top:30px}.installations-header h2,.console-panel h2{font-family:var(--font-display);font-weight:400;font-size:1.3rem;line-height:1.7}.installed-card{border-radius:6px}.installed-mark{border-radius:4px}.installed-filters button,.new-install-row select,.category-tabs button,.refresh-button,.primary-action{border-radius:4px}.console-card{padding:20px}.console-output{border-radius:5px;border:1px solid #ffffff10;background:#080d17e0}.download-progress{border-radius:6px;background:rgba(var(--bg-base-rgb),.96);box-shadow:0 -3px 24px #0004}.progress-track,.progress-fill{border-radius:1px}
  @media(max-width:1200px){.sidebar{width:208px;min-width:208px}.brand-title{font-size:.64rem}.custom-logo{width:33px;height:33px}.content{padding:0 24px 24px}.topbar-actions{gap:12px}.hero h1{font-size:1.55rem}.quick-cards{grid-template-columns:repeat(2,minmax(0,1fr))}.play-layout{padding-right:37%}.background-character{width:39%}.launch-panel .bottom-bar{padding:20px}}
  @media(max-width:1000px){.content{padding-top:0!important}.launcher-topbar{min-height:76px;margin-bottom:23px}.section-location>span{display:none}.play-layout{padding-right:0;padding-top:320px}.background-character{top:0;bottom:auto;right:0;width:100%;height:300px}.launch-panel .bottom-bar{padding:22px}.background-character:after{content:'';position:absolute;left:20%;right:20%;bottom:0;height:1px;background:linear-gradient(90deg,transparent,rgba(var(--primary-rgb),.2),transparent);pointer-events:none}}
  @media(max-width:760px){.brand-title{font-size:.56rem}.sidebar{padding-left:10px!important;padding-right:10px!important}.menu button{font-size:.74rem;padding:10px 8px;gap:8px}.menu-count{display:none}.custom-logo{width:29px;height:29px}.hero h1{font-size:1.3rem}.hero .hero-message{font-size:.77rem}.launch-block-title h2{font-size:.73rem}.card-label{font-size:.47rem}}
  @media(max-width:640px){.brand-copy,.menu-section-label{display:none}.logo{padding:3px 0 17px;margin-bottom:10px}.sidebar>div:first-child{width:100%}.menu{padding:0 0 6px;gap:7px}.menu button{font-size:.52rem;line-height:1.6;gap:5px;min-height:53px;text-align:center;justify-content:center}.menu button>span{flex:none}.menu button.active{box-shadow:inset 0 -2px var(--primary)}.menu button :global(svg){width:19px;height:19px}.content{padding-bottom:20px!important}.launcher-topbar{gap:8px;min-height:72px}.section-location>strong{font-size:.65rem}.topbar-actions{gap:0}.hero-edition{font-size:.44rem;letter-spacing:.06em}.hero h1{font-size:1.1rem;line-height:1.8}.quick-cards{gap:8px}.info-card{padding:13px 10px}.info-card>strong{font-size:.65rem}.info-card>small{font-size:.55rem}.card-label{font-size:.43rem}.card-label :global(svg){display:none}.launch-panel .bottom-bar{padding:17px}.launch-panel .play-btn{font-size:.77rem}.java-edition-tag{display:none}}
  @media(prefers-reduced-motion:reduce){.launcher,.menu button,.play-btn,.info-card,.module-card{transition:none!important}.menu button:hover,.launch-panel .play-btn:hover:not(:disabled){transform:none}}

</style>
