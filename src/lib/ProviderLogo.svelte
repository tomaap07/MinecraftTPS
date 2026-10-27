<script lang="ts">
  import ServiceLogo from './ServiceLogo.svelte';
  let { provider, size = 24, title, compact = false }: { provider: string; size?:number; title?:string; compact?:boolean } = $props();
</script>
<span title={title} class:compact><ServiceLogo name={provider} {size}/></span>
<style>span{display:inline-flex;align-items:center;justify-content:center;flex-shrink:0;vertical-align:middle;line-height:0}</style>
