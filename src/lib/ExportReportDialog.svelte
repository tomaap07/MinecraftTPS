<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import UiIcon from './UiIcon.svelte';
  import type { ExportResult } from './export_report';
  let { report, onclose }: { report: ExportResult | null; onclose: () => void } = $props();
  let error = $state('');
  function showDialog(node: HTMLDialogElement) { node.showModal(); return { destroy: () => node.close() }; }
  async function openProject(url: string) {
    try {
      const parsed = new URL(url);
      if (parsed.protocol !== 'https:') throw new Error('Se requiere un enlace HTTPS.');
      await openUrl(parsed.href);
    } catch (e) { error = String(e); }
  }
</script>

{#if report}
  <div class="export-scrim">
    <dialog use:showDialog aria-labelledby="export-report-title" oncancel={event => {event.preventDefault();onclose();}}>
      <header><UiIcon name="zip" size={25}/><div><small>EXPORTACIÓN TERMINADA</small><h2 id="export-report-title">Tu pack está listo</h2></div><button onclick={onclose} aria-label="Cerrar informe"><UiIcon name="close"/></button></header>
      <p>{report.included_files} archivos incluidos · {report.referenced_files} descargas referenciadas y verificadas.</p>
      <code data-no-translate>{report.path}</code>
      {#if report.omitted.length}
        <h3>Archivos que deben obtenerse por separado</h3>
        <p>Estos binarios no se redistribuyen dentro del pack. El informe también está guardado en el archivo exportado.</p>
        <ul>{#each report.omitted as item}<li><div><strong data-no-translate>{item.name}</strong><span data-no-translate>Versión: {item.version || 'No declarada en el archivo'}</span><small data-no-translate>{item.filename}</small></div><p>{item.reason}</p>{#if item.project_url}<button onclick={() => void openProject(item.project_url!)}><UiIcon name="external" size={14}/>Sitio del proyecto</button>{/if}</li>{/each}</ul>
      {:else}<p class="success">No se omitieron archivos.</p>{/if}
      {#if error}<p role="alert">{error}</p>{/if}
      <footer><button class="accept" onclick={onclose}>Entendido</button></footer>
    </dialog>
  </div>
{/if}

<style>
  .export-scrim{position:fixed;inset:0;z-index:1000;background:#000b;backdrop-filter:blur(8px);display:grid;place-items:center;padding:24px}dialog{width:min(660px,100%);max-height:85vh;overflow:auto;background:#111017;border:1px solid #a855f766;border-radius:12px;padding:25px;box-shadow:0 24px 80px #0009;color:#e4e1ed}dialog::backdrop{background:#0009}header{display:flex;align-items:center;gap:14px}header>div{flex:1}h2{margin:4px 0;font-size:1.3rem}header small{font-size:.6rem;letter-spacing:.15em;color:#c4a0e4}button{display:inline-flex;align-items:center;justify-content:center;gap:7px;padding:9px 13px;background:#ffffff08;border:1px solid #ffffff24;color:inherit;border-radius:5px;font:inherit;cursor:pointer}header>button{padding:7px}p{color:#aaa5b9;line-height:1.6;font-size:.82rem}code{display:block;overflow-wrap:anywhere;background:#0005;padding:12px;border-radius:5px;font-size:.76rem}h3{font-size:1rem;margin-top:24px}ul{list-style:none;padding:0}li{padding:15px 0;border-top:1px solid #ffffff16}li div{display:grid;gap:4px}li span{font-size:.8rem}li small{font-size:.7rem;color:#a39cb3}li p{margin:8px 0}.success{color:#9dc9a5}footer{display:flex;justify-content:flex-end;padding-top:14px}.accept{background:#a855f725;border-color:#a855f770}button:focus-visible{outline:2px solid #d946ef;outline-offset:3px}
</style>
