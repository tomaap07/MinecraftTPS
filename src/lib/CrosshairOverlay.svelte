<script lang="ts">
import{onDestroy}from 'svelte';
import type{HudProfile}from './launcher_plugins';
import type{OverlaySnapshot}from './launcher_extensions';
import CrosshairRender from './CrosshairRender.svelte';
let{profile,frame=null,preview=false}:{profile:HudProfile;frame?:OverlaySnapshot|null;preview?:boolean}=$props();
let lastSequence:number|null=null,hit=$state(false),timer:ReturnType<typeof setTimeout>|undefined;
$effect(()=>{const sequence=frame?.telemetry?.hit_sequence;if(sequence==null)return;if(lastSequence!=null&&sequence>lastSequence&&profile.crosshair.hitmarker){hit=true;clearTimeout(timer);timer=setTimeout(()=>hit=false,profile.crosshair.hit_duration);}lastSequence=sequence;});
onDestroy(()=>clearTimeout(timer));
let data=$derived(frame?.telemetry),settings=$derived(profile.crosshair);
let color=$derived(!settings.target_color?'':data?.target_distance!=null&&data.target_distance<3?settings.range_color:data?.target_kind==='hostile'?settings.hostile_color:data?.target_kind==='passive'?settings.passive_color:'');
let spread=$derived(settings.dynamic?Math.min(5,(data?.speed??data?.world_metrics?.speed??(frame?.keys.w||frame?.keys.a||frame?.keys.s||frame?.keys.d?4:0))*.35+(frame?.keys.lmb?2:0)):0);
</script>
{#if settings.enabled&&(preview||data?.has_world)}<div class="crosshair"><CrosshairRender {settings} {spread} {color} cooldown={data?.attack_cooldown??null} {hit}/></div>{/if}
<style>.crosshair{position:absolute;left:50%;top:50%;width:100px;height:100px;transform:translate(-50%,-50%);pointer-events:none}</style>
