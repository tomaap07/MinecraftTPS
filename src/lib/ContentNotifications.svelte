<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
  import { contentOperation, type ContentOperation } from './content_operations';
  import ContentProgress from './ContentProgress.svelte';
  import UiIcon from './UiIcon.svelte';
  import { translate } from './i18n';
  let dismissed = $state('');
  $effect(() => {
    const operation = $contentOperation;
    if(operation?.status !== 'success')return;
    const timer=setTimeout(()=>dismissed=operation.id,9000);
    return ()=>clearTimeout(timer);
  });
  let hydrated = false;
  const notified = new Set<string>();
  async function receive(operation: ContentOperation, historical = false) {
    contentOperation.set(operation);
    if (historical || operation.status !== 'success' || notified.has(operation.id)) return;
    notified.add(operation.id);
    try {
      let allowed = await isPermissionGranted();
      if (!allowed) allowed = await requestPermission() === 'granted';
      if (allowed) sendNotification({ title: `TP LAUNCHER · ${translate('Instalación completada')}`, body: operation.detail });
    } catch {   }
  }
  onMount(() => {
    let disposed = false, off: UnlistenFn | undefined;
    void (async () => {
      off = await listen<ContentOperation>('studio-progress', event => { hydrated = true; void receive(event.payload); });
      if (disposed) { off(); return; }
      const current = await invoke<ContentOperation | null>('studio_current_operation');
      if (!disposed && !hydrated && current) void receive(current, true);
    })().catch(() => {});
    return () => { disposed = true; off?.(); };
  });
</script>

{#if $contentOperation && dismissed !== $contentOperation.id}
  <aside class="operation-toast" role={$contentOperation.status === 'error' ? 'alert' : 'status'} aria-live="polite">
    <button class="dismiss-operation" aria-label={translate('Cerrar')} onclick={() => dismissed = $contentOperation?.id ?? ''}><UiIcon name="close" size={16}/></button>
    <ContentProgress />
  </aside>
{/if}

<style>
  .operation-toast{position:fixed;z-index:8000;right:24px;top:24px;width:min(480px,calc(100vw - 48px));filter:drop-shadow(0 12px 35px #0008);animation:notice-in .2s ease}.dismiss-operation{position:absolute;right:7px;top:-1px;display:grid;place-items:center;width:24px;height:24px;border:1px solid #9f98c840;border-radius:7px;background:#1b2039;color:#bfc6e0;cursor:pointer}@keyframes notice-in{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}@media(prefers-reduced-motion:reduce){.operation-toast{animation:none}}
</style>
