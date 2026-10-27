<script lang="ts">
  import { contentOperation } from './content_operations';
  import UiIcon from './UiIcon.svelte';
  import { t } from './i18n';
  const size = (n: number) => `${(n / 1024 / 1024).toFixed(1)} MB`;
</script>

{#if $contentOperation}
  <section class="content-progress" class:failed={$contentOperation.status === 'error'} class:completed={$contentOperation.status === 'success'} aria-label={$t('Progreso de instalación')}>
    <div class="progress-heading"><UiIcon name={$contentOperation.status === 'running' ? 'download' : $contentOperation.status === 'error' ? 'terminal' : 'cube'} /><strong>{$t($contentOperation.stage)}</strong><span>{Math.floor($contentOperation.percent)}%</span></div>
    <progress max="100" value={$contentOperation.percent} aria-label={$t('Progreso de instalación')}></progress>
    <div class="progress-detail"><span data-no-translate>{$contentOperation.detail}</span><small>{#if $contentOperation.total > 0}{$contentOperation.current} / {$contentOperation.total} {$t('archivos')}{/if}{#if $contentOperation.bytes_total > 0} · {size($contentOperation.bytes_done)} / {size($contentOperation.bytes_total)}{/if}</small></div>
  </section>
{/if}

<style>
  .content-progress{border:1px solid #918afa45;border-radius:12px;background:linear-gradient(125deg,#26233ff2,#121a2df7);padding:16px 18px;margin:14px 0;color:#edf0ff;box-shadow:0 8px 28px #0002}.progress-heading{display:flex;align-items:center;gap:9px;font-size:.78rem}.progress-heading span{margin-left:auto;color:#c3bcff;font-variant-numeric:tabular-nums}.progress-heading strong{font-weight:600}progress{display:block;width:100%;height:7px;margin:12px 0 10px;border:0;border-radius:10px;overflow:hidden;background:#8c85c51b;accent-color:var(--primary,#9185ff)}progress::-webkit-progress-bar{background:#8c85c51b;border-radius:10px}progress::-webkit-progress-value{background:linear-gradient(90deg,#8577ef,var(--primary,#a7a2ff));border-radius:10px;transition:width .2s}.progress-detail{display:flex;flex-wrap:wrap;gap:5px 16px;justify-content:space-between;font-size:.71rem;color:#adb8d1;line-height:1.5}.progress-detail>span{overflow-wrap:anywhere;min-width:0;flex:1}.progress-detail small{white-space:nowrap;font-variant-numeric:tabular-nums}.failed{border-color:#ef899d65}.completed{border-color:#87d9ac55}
</style>
