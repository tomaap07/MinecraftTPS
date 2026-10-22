<script lang="ts">
  import { onMount } from "svelte";
  import type { PlayerAnimation, SkinViewer } from "skinview3d";

  let { skinUrl, capeUrl, model = "auto-detect" }: {
    skinUrl: string;
    capeUrl?: string;
    model?: "default" | "slim" | "auto-detect";
  } = $props();

  let canvasElement: HTMLCanvasElement;
  let viewer: SkinViewer | undefined;
  let walkingAnimation: (() => PlayerAnimation) | undefined;
  let viewerReady = $state(false);
  let walking = $state(true);
  let zoom = $state(0.85);
  let viewerError = $state("");

  $effect(() => {
    const skin = skinUrl;
    const cape = capeUrl;
    const playerModel = model;
    if (!viewerReady || !viewer) return;
    void Promise.resolve(viewer.loadSkin(skin, { model: playerModel })).catch((error: unknown) => {
      viewerError = `No se pudo cargar la skin: ${String(error)}`;
    });
    if (cape) {
      void Promise.resolve(viewer.loadCape(cape)).catch((error: unknown) => {
        viewerError = `No se pudo cargar la capa: ${String(error)}`;
      });
    } else {
      viewer.resetCape();
    }
  });

  onMount(() => {
    let cancelled = false;
    let resizeObserver: ResizeObserver | undefined;
    void import("skinview3d").then(({ SkinViewer, WalkingAnimation }) => {
      if (cancelled) return;
      viewer = new SkinViewer({
        canvas: canvasElement,
        width: 500,
        height: 560,
        skin: skinUrl,
        cape: capeUrl || undefined,
        model,
        enableControls: true,
        zoom,
        background: "#0b1020",
      });
      walkingAnimation = () => new WalkingAnimation();
      viewer.animation = walkingAnimation();
      viewer.autoRotate = true;
      const parent = canvasElement.parentElement;
      if (parent && typeof ResizeObserver !== "undefined") {
        resizeObserver = new ResizeObserver((entries) => {
          const box = entries[0]?.contentRect;
          if (box && viewer) viewer.setSize(Math.max(280, Math.floor(box.width)), Math.max(360, Math.floor(box.height)));
        });
        resizeObserver.observe(parent);
      }
      viewerReady = true;
    }).catch((error) => {
      viewerError = `El visor 3D no pudo iniciarse: ${String(error)}`;
    });

    return () => {
      cancelled = true;
      resizeObserver?.disconnect();
      viewer?.dispose();
      viewer = undefined;
    };
  });

  function changeZoom(delta: number) {
    zoom = Math.min(1.25, Math.max(0.55, zoom + delta));
    if (viewer) viewer.zoom = zoom;
  }

  function toggleWalking() {
    if (!viewer || !walkingAnimation) return;
    walking = !walking;
    viewer.animation = walking ? walkingAnimation() : null;
  }
</script>

<div class="skin-viewer-wrap">
  <canvas bind:this={canvasElement} aria-label="Modelo 3D de Minecraft. Arrastra para girar y usa la rueda para acercar."></canvas>
  <div class="skin-viewer-controls">
    <button aria-label="Alejar" onclick={() => changeZoom(-0.1)}>−</button>
    <span>{Math.round(zoom * 100)}%</span>
    <button aria-label="Acercar" onclick={() => changeZoom(0.1)}>+</button>
    <button onclick={toggleWalking}>{walking ? "Pausar caminar" : "Animar caminar"}</button>
    <small>Arrastra para girar · rueda para zoom</small>
  </div>
  {#if viewerError}<p class="viewer-error">{viewerError}</p>{/if}
</div>

<style>
  .skin-viewer-wrap { display:flex; flex-direction:column; min-width:0; width:100%; height:510px; overflow:hidden; border-radius:10px; background:#0b1020; }
  canvas { display:block; width:100%; min-height:0; flex:1; touch-action:none; }
  .skin-viewer-controls { display:flex; align-items:center; justify-content:center; flex-wrap:wrap; gap:8px; padding:8px; background:rgba(2,6,23,.9); }
  .skin-viewer-controls button { padding:6px 10px; border:1px solid rgba(255,255,255,.16); border-radius:7px; background:#172033; color:#e2e8f0; cursor:pointer; }
  .skin-viewer-controls span { min-width:42px; text-align:center; color:#cbd5e1; font-variant-numeric:tabular-nums; }
  .skin-viewer-controls small { width:100%; color:#94a3b8; text-align:center; }
  .viewer-error { margin:0; padding:6px 10px; color:#fca5a5; font-size:.8rem; }
</style>
