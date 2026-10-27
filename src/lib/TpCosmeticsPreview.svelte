<script lang="ts">
  import { onMount } from 'svelte';
  import SkinModelViewer from './SkinModelViewer.svelte';
  import { FALLBACK_SKIN, WELCOME_POSE } from './poses_catalog';
  import type { TpCosmetics } from './launcher_extensions';
  let {skinUrl=FALLBACK_SKIN,fallbackSkinUrl="",capeUrl,model='auto-detect',cosmetics,paused=false}:
    {skinUrl?:string;fallbackSkinUrl?:string;capeUrl?:string;model?:'default'|'slim'|'auto-detect';cosmetics:TpCosmetics;paused?:boolean}=$props();
  let tpCape=$state('');
  onMount(()=>{

    const canvas=document.createElement('canvas');canvas.width=64;canvas.height=32;
    const ctx=canvas.getContext('2d');if(!ctx)return;
    ctx.fillStyle='#16161e';ctx.fillRect(0,0,64,32);
    ctx.fillStyle='#27273a';ctx.fillRect(1,1,10,16);ctx.fillRect(12,1,10,16);
    for(const x of [1,12]){
      ctx.fillStyle='#a855f7';ctx.fillRect(x,1,10,1);ctx.fillRect(x,16,10,1);
      ctx.fillStyle='#0b0b0f';ctx.fillRect(x+1,3,8,11);

      for(let row=0;row<9;row++){
        const half=4-Math.abs(4-row);ctx.fillStyle=row<4?'#D946EF':'#A855F7';
        ctx.fillRect(x+5-half,4+row,Math.max(1,half*2),1);
      }
    }
    tpCape=canvas.toDataURL('image/png');
  });
  let displayedCape=$derived((cosmetics.cape==='end'||cosmetics.wings)&&tpCape?tpCape:capeUrl);
</script>

<div class="cosmetic-scene" class:aura={cosmetics.aura&&!paused}>
  {#if !paused}<SkinModelViewer {fallbackSkinUrl} {skinUrl} capeUrl={displayedCape} {model} pose={WELCOME_POSE} backEquipment={cosmetics.wings?'elytra':'cape'} height={330} controls breathing/>{:else}<p class="paused">Vista previa pausada durante la partida.</p>{/if}
  {#if cosmetics.aura&&!paused}<div class="aura-particles" aria-hidden="true">{#each Array(14) as _,index}<i style={`--x:${10+(index*23)%80}%;--y:${15+(index*17)%70}%;--delay:${index*-.7}s;--duration:${4+index%4}s`}></i>{/each}</div>{/if}
  <span class="preview-label">VISTA PREVIA LOCAL · COSMÉTICOS TP</span>
</div>

<style>
  .cosmetic-scene{min-width:0;position:relative;overflow:hidden;background:radial-gradient(ellipse at 50% 90%,#a855f71a,transparent 65%),#0b0b0f80;border:1px solid #ffffff12;min-height:330px}.preview-label{position:absolute;bottom:10px;left:12px;font-size:.49rem;color:#a087bf;letter-spacing:.08em;pointer-events:none}.aura-particles{position:absolute;inset:0;pointer-events:none}.aura-particles i{position:absolute;left:var(--x);top:var(--y);width:3px;height:3px;background:#d946ef;box-shadow:0 0 9px #a855f7;animation:drift var(--duration) ease-in-out infinite;animation-delay:var(--delay)}.paused{color:#8f829e;font-size:.8rem;padding:90px 25px;text-align:center}@keyframes drift{0%,100%{transform:translateY(20px);opacity:0}35%,70%{opacity:.7}85%{transform:translateY(-65px)}}@media(prefers-reduced-motion:reduce){.aura-particles i{animation:none;opacity:.35}}
</style>
