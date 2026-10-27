<script lang="ts">
  let { name, size = 16, label }: {
    name: 'dimension' | 'release' | 'snapshot' | 'legacy';
    size?: number;
    label?: string;
  } = $props();
</script>


<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 16 16"
  shape-rendering="crispEdges" fill="none" class="tp-pixel-icon"
  role={label ? 'img' : undefined} aria-label={label} aria-hidden={label ? undefined : true} focusable="false">
  {#if name === 'dimension'}
    <path fill="#A855F7" d="M7 1h2v1h2v1h2v1h2v8h-2v1h-2v1H9v1H7v-1H5v-1H3v-1H1V4h2V3h2V2h2z"/>
    <path fill="#27273A" d="M7 3h2v1h2v1h2v1h-2v1H9v1H7V7H5V6H3V5h2V4h2z"/>
    <path fill="#16161E" d="M3 7h2v1h2v1h1v4H7v-1H5v-1H3z"/>
    <path fill="#0B0B0F" d="M13 7v4h-2v1H9V9h2V8h1V7z"/>
    <path fill="#D946EF" d="M7 4h2v1h2v1H9v1H7V6H5V5h2z"/>
    <path fill="#A855F7" d="M7 9h2v4H7z"/>
  {:else if name === 'release'}
    <path fill="#E2E8F0" d="M7 1h2v1h2v1h2v1h2v8h-2v1h-2v1H9v1H7v-1H5v-1H3v-1H1V4h2V3h2V2h2z"/>
    <path fill="#62518B" d="M2 6h2v1h2v1h2v6H7v-1H5v-1H3v-1H2z"/>
    <path fill="#33284C" d="M14 6v5h-1v1h-2v1H9V8h2V7h2V6z"/>
    <path fill="#A855F7" d="M7 2h2v1h2v1h2v1h1v1h-3v1H9v1H7V7H5V6H2V5h1V4h2V3h2z"/>
    <path fill="#D946EF" d="M7 3h2v1h2v1H9v1H6V5H4V4h3zM8 8h1v6H8z"/>
    <path fill="#F5E8FF" d="M7 3h2v1H7zM11 5h2v1h-2zM2 6h2v1H2z"/>
    <path fill="#D8B4FE" d="M3 8h1v2H3zM6 11h1v1H6zM11 9h1v2h-1z"/>
  {:else if name === 'snapshot'}
    <path fill="#A855F7" d="M5 1h6v2h-1v3h1v2h1v1h1v2h1v3h-1v1H3v-1H2v-3h1V9h1V8h1V6h1V3H5z"/>
    <path fill="#16161E" d="M7 3h2v4h1v2h1v2h1v2H4v-2h1V9h1V7h1z"/>
    <path fill="#D946EF" d="M5 10h6v1h1v2H4v-2h1z"/>
    <path fill="#D8B4FE" d="M6 11h2v1H6zM8 8h1v1H8z"/>
    <path fill="#D946EF" d="M13 3h1v1h1v1h-1v1h-1V5h-1V4h1zM2 5h1v1H2z"/>
  {:else}
    <path fill="#B4A8D8" d="M7 1h2v1h2v1h2v1h2v8h-2v1h-2v1H9v1H7v-1H5v-1H3v-1H1V4h2V3h2V2h2z"/>
    <path fill="#A6A0B5" d="M7 3h2v1h2v1h2v1h-2v1H9v1H7V7H5V6H3V5h2V4h2z"/>
    <path fill="#645D7B" d="M2 7h2v1h2v1h2v5H7v-1H5v-1H3v-1H2z"/>
    <path fill="#363047" d="M14 7v4h-1v1h-2v1H9V9h2V8h2V7z"/>
    <path fill="#0B0B0F" d="M7 4h1v2h1v2H8v2H7V8H6V7h1zM4 9h1v1h1v2H5v-1H4zM11 10h2v1h-1v2h-1z"/>
    <path fill="#E2E8F0" d="M7 2h2v1H7zM3 5h2v1H3zM2 7h1v3H2z"/>
    <path fill="#A855F7" d="M4 5h1v1H4zM9 5h2v1H9zM5 11h1v1H5zM13 7h1v3h-1z"/>
    <path fill="#D946EF" d="M12 8h1v1h-1zM8 12h1v2H8z"/>
  {/if}
</svg>

<style>
  .tp-pixel-icon{display:inline-block;flex-shrink:0;vertical-align:middle}
</style>
