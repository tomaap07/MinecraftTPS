<script lang="ts">let{major}:{major:number}=$props();</script>

<span class="java-runtime" role="img" aria-label={major?`Runtime Java ${major}`:'Java del sistema'}>
<svg width="32" height="32" viewBox="0 0 16 16" shape-rendering="crispEdges" fill="none" aria-hidden="true">
 <path fill="#F89820" d="M5 1h2v2H6v1H4V2h1zM9 0h2v2h-1v2H8V2h1z"/>
 <path fill="#E2E8F0" d="M2 6h10v6h-1v1H3v-1H2zM12 7h3v4h-3v-1h2V8h-2z"/>
 <path fill="#5382A1" d="M3 7h8v5H4v-1H3zM1 14h12v1H1z"/>
 <path fill="#89BED7" d="M4 7h2v4H4z"/><path fill="#A855F7" d="M7 8h3v3H7zM3 13h8v1H3z"/>
 <path fill="#D946EF" d="M8 9h1v1H8z"/>
</svg><strong>{major||'SYS'}</strong>
</span>
<style>.java-runtime{display:inline-flex;align-items:center;justify-content:center;gap:6px;flex-shrink:0;color:#e2e8f0;line-height:1}.java-runtime svg{flex-shrink:0}.java-runtime strong{display:grid;place-items:center;font:700 .7rem/1.2 var(--font-display,monospace);min-width:22px;min-height:22px;padding:3px;border:1px solid #a855f780;background:#21152f;color:#ead5ff;box-sizing:border-box}</style>
