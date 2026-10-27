<script lang="ts">
  let { url, name = "Capa oficial" }: { url: string; name?: string } = $props();
  let canvas: HTMLCanvasElement;
  let mode = $state<"cape" | "elytra">("cape");
  let error = $state("");

  $effect(() => {
    const target = canvas;
    const source = url;
    const currentMode = mode;
    if (!target || !source) return;
    let disposed = false;
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.onload = () => {
      if (disposed) return;
      const scale = image.width / 64;
      const width = 10;
      const height = currentMode === "cape" ? 16 : 20;
      target.width = width * 12;
      target.height = height * 12;
      const context = target.getContext("2d");
      if (!context) return;
      context.imageSmoothingEnabled = false;
      context.clearRect(0, 0, target.width, target.height);
      context.drawImage(image, (currentMode === "cape" ? 1 : 36) * scale, (currentMode === "cape" ? 1 : 2) * scale, width * scale, height * scale, 0, 0, target.width, target.height);
      error = "";
    };
    image.onerror = () => { if (!disposed) error = "No se pudo cargar la textura"; };
    image.src = source;
    return () => { disposed = true; image.onload = null; image.onerror = null; };
  });
</script>

<div class="cape-art">
  <div class="cape-stage"><canvas bind:this={canvas} aria-label={`${name} · ${mode === "cape" ? "capa" : "élitros"}`}></canvas>{#if error}<small>{error}</small>{/if}</div>
  <div class="cape-toggle" aria-label={`Vista de ${name}`}>
    <button class:active={mode === "cape"} aria-pressed={mode === "cape"} onclick={() => mode = "cape"}>Capa</button>
    <button class:active={mode === "elytra"} aria-pressed={mode === "elytra"} onclick={() => mode = "elytra"}>Élitros</button>
  </div>
</div>

<style>
  .cape-art {display:grid;gap:12px;width:100%}.cape-stage{height:182px;display:grid;place-items:center;border-radius:14px;background:radial-gradient(ellipse at 50% 60%,rgba(var(--primary-rgb,168,85,247),.12),transparent 72%);overflow:hidden}.cape-stage canvas{display:block;width:auto;height:158px;max-width:90%;image-rendering:pixelated;filter:drop-shadow(0 10px 9px #0006);border-radius:3px}.cape-stage small{color:#d4b0bc}.cape-toggle{display:flex;padding:3px;border:1px solid #ffffff16;background:#ffffff05;border-radius:9px;gap:3px}.cape-toggle button{flex:1;color:#a5aec1;padding:6px 8px;font:inherit;font-size:.74rem;cursor:pointer;border:0;background:transparent;border-radius:6px}.cape-toggle button.active{background:rgba(var(--primary-rgb,168,85,247),.19);color:#edddff}.cape-toggle button:focus-visible{outline:2px solid var(--primary,#c084fc);outline-offset:1px}
</style>
