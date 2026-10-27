<script lang="ts">
  import type { SecretIconName } from './achievement_catalog';
  import SecretIcon from './SecretIcon.svelte';
  import UiIcon from './UiIcon.svelte';
  import TpPixelIcon from './TpPixelIcon.svelte';
  import { interfaceIcon } from './interface_icons';
  import {localVersionArtwork} from './local_game_art';

  let { name, size = 20, fallback }: { name: string; size?: number; fallback?: SecretIconName } = $props();

  let icon = $derived(interfaceIcon(name));
  const localAliases:Record<string,string>={sword:'diamond_sword',grass_side:'grass_block',stone:'stone_block',block:'stone_block',cobblestone:'cobblestone_block',command_block_front:'command_block',music_disc_11:'music_disc_13',filled_map:'map'};
  let localTexture=$derived($localVersionArtwork.textures[localAliases[name]??name]);
</script>

<span class="game-icon" style:--icon-size={`${size}px`} style:width={`${size}px`} style:height={`${size}px`} aria-hidden="true">
  {#if localTexture}<img src={localTexture} alt="" width={size} height={size}/>
  {:else if icon.pixel}<TpPixelIcon name={icon.pixel} {size}/>
  {:else if fallback}<SecretIcon name={fallback} {size}/>
  {:else}<UiIcon name={icon.ui} {size}/>{/if}
</span>

<style>
  .game-icon{display:inline-flex;align-items:center;justify-content:center;flex:0 0 auto;vertical-align:middle;line-height:0}
  .game-icon :global(svg){flex-shrink:0;max-width:100%;max-height:100%}
  .game-icon img{width:100%;height:100%;object-fit:contain;image-rendering:pixelated}
</style>
