<script lang="ts">
 import UiIcon from './UiIcon.svelte';
 import {localVersionArtwork} from './local_game_art';
 import {resolvePluginIcon} from './plugin_icons';
 let{id,size=24}:{id:string;size?:number}=$props();
 let failedTexture=$state('');
 let icon=$derived(resolvePluginIcon(id,$localVersionArtwork.textures,failedTexture));
</script>

<span class="plugin-icon" style:width={`${size}px`} style:height={`${size}px`} aria-hidden="true">
 {#if icon.texture}
  <img src={icon.texture} alt="" width={size} height={size} onerror={()=>{failedTexture=icon.texture??'';}}/>
 {:else}
  <UiIcon name={icon.vector} {size}/>
 {/if}
</span>

<style>
 .plugin-icon{display:inline-flex;flex:none;align-items:center;justify-content:center;line-height:0;color:currentColor}
 img{display:block;width:100%;height:100%;object-fit:contain;image-rendering:pixelated}
</style>
