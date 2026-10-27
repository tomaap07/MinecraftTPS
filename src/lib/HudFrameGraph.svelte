<script lang="ts">
import{untrack}from 'svelte';
let{value,time=0,preview=false}:{value:number|null|undefined;time?:number;preview?:boolean}=$props();
let samples=$state<number[]>([]),lastTime=0;
$effect(()=>{const n=value,stamp=time;if(!preview&&stamp!==lastTime&&n!=null&&Number.isFinite(n)&&n>=0){lastTime=stamp;untrack(()=>samples=[...samples.slice(-79),n]);}});
let points=$derived(preview?[7,8,7,7,11,8,7,7,9,8,7,7,7,10,8,7]:samples),ceiling=$derived(Math.max(20,...points));
let path=$derived(points.map((n,i)=>`${i? 'L':'M'}${points.length>1?i*156/(points.length-1):0},${30-Math.min(30,n/ceiling*28)}`).join(' '));
</script>
<svg width="160" height="34" viewBox="0 0 160 34" role="img" aria-label={preview?'Gráfica de ejemplo':'Historial de tiempo entre presentaciones reales'}><path d="M0 30H160 M0 15H160" stroke="currentColor" stroke-opacity=".15"/><path d={path} fill="none" stroke="var(--hud-accent,#a855f7)" stroke-width="1.5"/></svg>
