<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { t } from './i18n';
  import UiIcon from './UiIcon.svelte';
  const url = 'https://www.minecraft.net/es-es/store/minecraft-deluxe-collection-pc?tabs=%7B%22details%22%3A0%7D';
  let message = $state('');
  async function openStore(event: MouseEvent) {
    event.preventDefault(); message = '';
    try { await openUrl(url); }
    catch { message = $t('No se pudo abrir la tienda oficial.'); }
  }
</script>

<aside class="official-game-store">
  <p>{$t('¿No posees Minecraft con licencia aún o necesitas adquirirlo?')}</p>
  <a href={url} target="_blank" rel="noopener noreferrer" onclick={openStore}>{$t('Comprar Minecraft en la tienda oficial')}<UiIcon name="external" size={16}/></a>
  {#if message}<small role="status">{message}</small>{/if}
</aside>

<style>
  .official-game-store{display:flex;flex-direction:column;align-items:flex-start;gap:10px;margin:19px 0;padding:17px 19px;border:1px solid #5ad47e36;border-radius:10px;background:linear-gradient(115deg,#15332470,#16102175);color:#c8d9cf}.official-game-store p{font:inherit;font-size:.78rem;line-height:1.65;margin:0;color:#c2d1ca}.official-game-store a{display:inline-flex;align-items:center;justify-content:center;gap:10px;max-width:100%;box-sizing:border-box;padding:10px 13px;border:1px solid #69ed9257;border-radius:7px;color:#86f4a7;background:#19432a70;font:inherit;font-size:.78rem;font-weight:700;text-decoration:none;line-height:1.5}.official-game-store a:hover{color:#b5ffcc;border-color:#8afdac;background:#205c3770}.official-game-store a:focus-visible{outline:2px solid #9affb8;outline-offset:3px}.official-game-store :global(svg){flex-shrink:0}.official-game-store small{font-size:.72rem;color:#efb3b3;line-height:1.5}
</style>
