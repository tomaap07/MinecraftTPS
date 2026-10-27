<script lang="ts">
  import ServiceLogo from './ServiceLogo.svelte';
  import VersionIcon from './VersionIcon.svelte';
  let { loader, size = 24, title, compact = false }: { loader: string; size?:number; title?:string; compact?:boolean } = $props();
</script>
<span title={title} class:compact>{#if loader==='vanilla'}<VersionIcon name="release" {size}/>{:else}<ServiceLogo name={loader} {size}/>{/if}</span>
<style>span{display:inline-flex;align-items:center;justify-content:center;flex-shrink:0;vertical-align:middle;line-height:0}</style>
