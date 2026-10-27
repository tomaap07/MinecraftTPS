<script lang="ts">
  import type { MotdSpan } from './server_live';
  let {spans,stale=false}:{spans:readonly MotdSpan[];stale?:boolean}=$props();
</script>

<div class="server-motd" class:stale data-no-translate title={stale?'Último mensaje recibido del servidor':'Mensaje del servidor'}>{#each spans as span}<span style:color={span.color} style:font-weight={span.bold?'700':'400'} style:font-style={span.italic?'italic':'normal'} style:text-decoration={`${span.underlined?'underline ':''}${span.strikethrough?'line-through':''}`}>{span.text}</span>{/each}</div>

<style>
  .server-motd{font-family:var(--font-display,monospace);font-size:.64rem;line-height:1.9;white-space:pre-wrap;overflow-wrap:anywhere;max-height:8em;overflow-y:auto;min-height:2.4em;padding:9px 10px;background:#0003;border:1px solid #ffffff0a;color:#dce5ef;text-shadow:1px 1px #0009;scrollbar-width:thin;scrollbar-color:#ffffff30 transparent}.stale{opacity:.7}
</style>
