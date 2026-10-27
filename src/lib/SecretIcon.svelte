<script lang="ts">
  import type { SecretIconName } from './achievement_catalog';
  let { name = 'trophy', size = 24 }: { name?: SecretIconName; size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
  {#if name === 'trophy'}
    <path d="M10 5h12v9a6 6 0 0 1-12 0V5ZM10 8H5v4a5 5 0 0 0 5 5m12-9h5v4a5 5 0 0 1-5 5M16 20v6m-6 1h12"/>
  {:else if name === 'lock'}
    <rect x="7" y="14" width="18" height="14" rx="2"/><path d="M11 14V9a5 5 0 0 1 10 0v5m-5 6v3"/>
  {:else if name === 'eyes'}
    <path d="m16 3 10 13-10 13L6 16zM16 10l4 6-4 6-4-6zM1 16h3m24 0h3"/>
  {:else if name === 'sword'}
    <path d="m8 24 14-14 5-5v7L13 26M18 10l4 4M7 18l7 7M6 27l4-4"/><path d="m8 19-3 3 5 5 3-3"/>
  {:else if name === 'block'}
    <path d="m16 3 12 7v13l-12 7-12-7V10L16 3ZM4 10l12 7 12-7M16 17v13M10 7l12 7"/>
  {:else if name === 'music'}
    <path d="M13 23V8l13-3v15M13 12l13-3"/><ellipse cx="9" cy="24" rx="4" ry="3"/><ellipse cx="22" cy="21" rx="4" ry="3"/>
  {:else if name === 'disc'}
    <circle cx="16" cy="16" r="12"/><circle cx="16" cy="16" r="4"/><path d="M8 16a8 8 0 0 1 8-8m8 8a8 8 0 0 1-8 8"/>
  {:else if name === 'portal'}
    <path d="M6 28V5h20v23H6Zm5 0V10h10v18M14 14h4m-4 5h4m-4 5h4"/><path d="M3 10H1m30 12h-2M16 1v2"/>
  {:else if name === 'skull'}
    <path d="M7 21V10l4-5h10l4 5v11l-4 2v5H11v-5l-4-2ZM14 27v-4m4 4v-4"/><path d="M10 13h4v5h-4zm8 0h4v5h-4z" fill="currentColor" opacity=".4"/><path d="m14 21 2-3 2 3"/>
  {:else if name === 'crown'}
    <path d="M5 10 10 16l6-10 6 10 5-6-3 16H8L5 10ZM8 21h16"/><circle cx="4" cy="8" r="2"/><circle cx="16" cy="4" r="2"/><circle cx="28" cy="8" r="2"/>
  {:else if name === 'creeper'}
    <rect x="8" y="5" width="16" height="22" rx="2"/><path d="M12 11h8v10h-8zM3 10h5m-5 7h5m16-7h5m-5 7h5M12 1v4m8 22v4"/>
  {:else if name === 'enderman'}
    <path d="m16 2 11 7v14l-11 7-11-7V9zM16 8l6 4v8l-6 4-6-4v-8zM2 3h3m22 25h3"/>
  {:else if name === 'tree'}
    <path d="M13 22v7h6v-7M9 6h14v4h5v12H4V10h5V6ZM12 2h8v4M4 17h24"/>
  {:else if name === 'compass'}
    <circle cx="16" cy="16" r="12"/><path d="m21 11-3 7-7 3 3-7 7-3ZM16 4v3m12 9h-3m-9 12v-3M4 16h3"/>
  {:else if name === 'heart'}
    <path d="M16 28 4 16V8h4V4h5l3 4 3-4h5v4h4v8L16 28Z"/>
  {:else if name === 'moon'}
    <path d="M23 24A12 12 0 0 1 10 4a12 12 0 1 0 18 16l-5 4ZM23 5v6m-3-3h6"/>
  {:else if name === 'sun'}
    <circle cx="16" cy="16" r="6"/><path d="M16 2v4m0 20v4M2 16h4m20 0h4M6 6l3 3m14 14 3 3M6 26l3-3M23 9l3-3"/>
  {:else if name === 'calendar'}
    <rect x="5" y="6" width="22" height="23" rx="2"/><path d="M5 13h22M11 3v6m10-6v6m-11 9h3m6 0h3m-12 6h3m6 0h3"/>
  {:else if name === 'terminal'}
    <rect x="3" y="6" width="26" height="21" rx="2"/><path d="m8 12 5 5-5 5m9 0h7M3 10h26"/>
  {:else if name === 'palette'}
    <path d="M28 16A12 12 0 1 0 16 28h2a3 3 0 0 0 2-5l-1-1a2 2 0 0 1 2-3h4a3 3 0 0 0 3-3Z"/><circle cx="10" cy="11" r="1.5"/><circle cx="17" cy="8" r="1.5"/><circle cx="23" cy="12" r="1.5"/><circle cx="8" cy="18" r="1.5"/>
  {:else if name === 'person'}
    <path d="M11 3h10v10H11zM9 17h14v12H9zM5 17H2v10h3m22-10h3v10h-3M16 23v6"/>
  {:else if name === 'play'}
    <path d="m11 5 17 11-17 11V5Z"/><path d="M4 5v22"/>
  {:else if name === 'logo'}
    <path d="M4 7h24v6H4zM13 13v14h6V13M7 22h3m12 0h3"/>
  {:else if name === 'book'}
    <path d="M16 8C11 4 7 4 3 5v22c5-1 9 0 13 2 4-2 8-3 13-2V5c-4-1-8-1-13 3v21M7 11l5 1m-5 5 5 1m8-6 5-1m-5 7 5-1"/>
  {:else if name === 'redstone'}
    <path d="m16 4 8 10-8 8-8-8 8-10ZM16 22v7M8 14H3m21 0h5M4 23l4-4m16 0 4 4"/><path d="m12 14 4-5 4 5-4 4-4-4Z" fill="currentColor" opacity=".25"/>
  {:else if name === 'feather'}
    <path d="M28 3C12 3 5 10 7 23l5 2C23 23 29 15 28 3ZM4 29 23 10m-13 6 1 5 6 1m-2-12 1 5 6 1"/>
  {:else if name === 'potion'}
    <path d="M11 3h10v5h-2v7l8 9v5H5v-5l8-9V8h-2V3ZM9 21h14M13 8h6"/><path d="M10 25h12"/>
  {:else if name === 'map'}
    <path d="m3 7 9-4 8 4 9-4v22l-9 4-8-4-9 4V7ZM12 3v22m8-18v22M7 13l2 2m15 2 2 2m-2 0 2-2"/>
  {:else if name === 'chest'}
    <path d="M4 10h24v18H4V10Zm0 0 3-5h18l3 5M4 17h24M4 23h24"/><path d="M13 14h6v8h-6z" fill="currentColor" opacity=".25"/>
  {:else}
    <path d="m16 3 4 9 9 4-9 4-4 9-4-9-9-4 9-4 4-9Z"/>
  {/if}
</svg>
