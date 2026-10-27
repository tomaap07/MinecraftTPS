<script lang="ts">
  import UiIcon from './UiIcon.svelte';
  import TpPixelIcon from './TpPixelIcon.svelte';
  import { brandAssets, serviceSymbols } from './brand_assets';

  let { name, size = 24, remote = false }: { name: string; size?: number; remote?:boolean } = $props();
  let failed = $state(false);
  const assets = brandAssets;
  let key = $derived(name.toLowerCase().replace(/[^a-z0-9]/g, ''));
  let asset = $derived(assets[({microsoftpremium:'microsoft',minecraft:'vanilla',wiki:'minecraftwiki'} as Record<string,string>)[key] ?? key]);
  let label = $derived(asset?.label ?? name);
  const domains:Record<string,string>={namemc:'namemc.com',skinmc:'skinmc.net',blockbench:'blockbench.net',mcstacker:'mcstacker.net',chunkbase:'chunkbase.com',planetminecraft:'planetminecraft.com',minecraftwiki:'minecraft.wiki',modrinth:'modrinth.com',curseforge:'curseforge.com'};
  let source=$derived(remote&&domains[key]?`https://www.google.com/s2/favicons?domain=${domains[key]}&sz=128`:asset?.src);
  $effect(() => { name; remote; failed = false; });
</script>

<span class="service-logo" class:unavailable={failed || !asset}
  style:--icon-size={`${size}px`} style:width={`${size}px`} style:height={`${size}px`} role="img" aria-label={label}
  title={label}>
  {#if source && !failed}
    <img src={source} alt="" width={size} height={size} loading="lazy" decoding="async" draggable="false" referrerpolicy="no-referrer" onerror={() => failed = true}/>
  {:else}
    {#if ['vanilla','minecraft'].includes(key)}<TpPixelIcon name="dimension" {size}/>{:else}<UiIcon name={serviceSymbols[key] ?? 'external'} {size}/>{/if}
  {/if}
</span>

<style>
  .service-logo{display:inline-flex;justify-content:center;align-items:center;flex-shrink:0;vertical-align:middle;line-height:0}
  .service-logo img{width:100%;height:100%;object-fit:contain;pointer-events:none}
  .service-logo.unavailable{color:var(--text-secondary,#a7b6b2)}
</style>
