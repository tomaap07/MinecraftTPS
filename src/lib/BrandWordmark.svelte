<script lang="ts">
  import { BRAND_NAME, logoEffect } from './brand';
</script>

<span class="wordmark" class:glitched={$logoEffect.active} style:--glitch-hue={$logoEffect.hue} aria-label={BRAND_NAME} data-no-translate>
  <span aria-hidden="true">{#if $logoEffect.active}{$logoEffect.text}{:else}<b>TP</b>{' '}LAUNCHER{/if}</span>
</span>

<style>
  .wordmark{display:inline-block;white-space:nowrap;line-height:inherit;font:inherit;font-weight:750;letter-spacing:.055em;color:#f0ebfa}
  b{color:var(--primary,#A855F7);font-weight:inherit}
  .glitched{color:hsl(var(--glitch-hue) 85% 75%);text-shadow:2px 0 hsl(calc(var(--glitch-hue) + 100) 85% 60%),-2px 0 hsl(calc(var(--glitch-hue) + 210) 90% 65%)}
  @media(prefers-reduced-motion:reduce){.glitched{color:#d8b4fe;text-shadow:none}}
</style>
