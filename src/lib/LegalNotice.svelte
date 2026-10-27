<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  let {compact=false,onterms}:{compact?:boolean;onterms?:()=>void}=$props();
  let notices = $state('');
  let loading = false;
  async function readNotices(event: Event) {
    if (!(event.currentTarget as HTMLDetailsElement).open || notices || loading) return;
    loading = true;
    try {
      const response = await fetch('/legal/THIRD-PARTY-NOTICES.txt');
      if (!response.ok) throw new Error('Unavailable');
      notices = await response.text();
    } catch { notices = 'Los avisos se incluyen en legal/THIRD-PARTY-NOTICES.txt de esta distribución.'; }
    finally { loading = false; }
  }
  function external(event: MouseEvent, url: string) {
    event.preventDefault(); void openUrl(url).catch(() => {});
  }
</script>

<div class="legal-footer" class:compact><p class="legal-notice">TP LAUNCHER es un proyecto independiente de TP LABS. No es un producto oficial de Minecraft ni está aprobado o asociado con Mojang, Microsoft o las comunidades enlazadas.</p>{#if onterms}<button type="button" class="terms-link" onclick={onterms}>Términos y condiciones<span aria-hidden="true">↗</span></button>{/if}</div>
<details class="credits">
  <summary>Créditos y licencias de terceros</summary>
  <p>Fabric, Forge, OptiFine, Modrinth y CurseForge identifican los proyectos y proveedores integrados. Los logos y marcas pertenecen a sus respectivos titulares; no forman parte del arte original de TP. NeoForge: equipo NeoForged · CC BY 4.0. Quilt: QuiltMC/art · CC0 1.0. Monocraft: Idrees Hassan y colaboradores · SIL OFL 1.1. Visor 3D: skinview3d y Three.js · MIT.</p>
  <p>Blockbench: JannisX11; logo usado como crédito y enlace conforme a su guía pública. Microsoft, GitHub y Discord identifican sus servicios. Sus marcas pertenecen a sus titulares; su presencia no implica patrocinio ni afiliación.</p>
  <p>Las licencias CC BY 4.0 y CC0 aquí citadas se refieren a derechos de autor del material indicado y excluyen derechos de marca. Los créditos no sustituyen autorizaciones específicas que correspondan.</p>
  <nav aria-label="Licencias y fuentes">
    {#each [
      ['Fabric', 'https://fabricmc.net/'],
      ['Blockbench · guía de marca', 'https://blockbench.net/wiki/blockbench/logos/'],
      ['NeoForge · CC BY 4.0', 'https://github.com/neoforged/websites/blob/main/LICENSE.txt'],
      ['Quilt · CC0', 'https://github.com/QuiltMC/art/blob/master/brand/LICENSE'],
      ['Minecraft · condiciones de uso', 'https://www.minecraft.net/en-us/usage-guidelines'],
    ] as [label, url]}<a href={url} onclick={(event) => external(event, url)}>{label}</a>{/each}
  </nav>
  <details ontoggle={readNotices}><summary>Leer avisos completos</summary><pre>{notices || 'Cargando avisos…'}</pre></details>
</details>

<style>
  .legal-footer{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:16px 0 5px;border-top:1px solid var(--block-line,#27273A)}.legal-notice{margin:0;color:var(--block-faint,#817B95);font-size:.67rem;line-height:1.8;text-wrap:pretty}.compact{padding-top:11px}.compact .legal-notice{font-size:.61rem}.terms-link{display:inline-flex;align-items:center;gap:8px;flex-shrink:0;min-height:34px;padding:7px 11px;border:1px solid #a855f740;background:#a855f70b;color:#cdb0ea;font:inherit;font-size:.65rem;cursor:pointer}.terms-link:hover{border-color:#a855f7;color:#e9d5ff}.terms-link:focus-visible{outline:2px solid #d946ef;outline-offset:3px}@media(max-width:700px){.legal-footer{align-items:flex-start;flex-direction:column;gap:9px}}
  .credits{color:var(--text-secondary,#a69bb5);font-size:.66rem;line-height:1.7;padding-bottom:8px}.credits summary{cursor:pointer;width:fit-content}.credits p{max-width:950px}.credits nav{display:flex;gap:10px;flex-wrap:wrap;margin-bottom:8px}.credits a{color:var(--primary,#A855F7)}.credits pre{white-space:pre-wrap;overflow:auto;max-height:280px;padding:12px;background:#0b0b0f;border:1px solid #27273a;font:inherit;line-height:1.5}
</style>
