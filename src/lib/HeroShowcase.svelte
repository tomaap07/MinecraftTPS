<script lang="ts">
  import SkinModelViewer from "./SkinModelViewer.svelte";
  import { FALLBACK_SKIN, type Pose } from "./poses_catalog";
  import { itemsForPose, type EquippedItems } from "./held_items";
  import type { EasterEgg } from "./easter_eggs";

  let {
    skinUrl = FALLBACK_SKIN,
    fallbackSkinUrl = "",
    capeUrl,
    model = "auto-detect",
    pose,
    equipment,
    easterEgg = false,
    whiteEyes = false,
    egg = null,
    height = 520,
  }: {
    skinUrl?: string;
    fallbackSkinUrl?:string;
    capeUrl?: string;
    model?: "default" | "slim" | "auto-detect";
    pose: Pose;
    equipment?:EquippedItems;
    easterEgg?: boolean;
    whiteEyes?: boolean;
    egg?:EasterEgg|null;
    height?: number;
  } = $props();
  let expired=$state(false);
  let variant=$derived(whiteEyes||expired?'none':egg?.variant??(easterEgg?'herobrine':'none'));
  let automaticItems=$derived(equipment??itemsForPose(pose));
  let chosenSkin=$derived(variant==='herobrine'?'/skins/tp-spectral.png':['entity303','crowned_steve'].includes(variant)?FALLBACK_SKIN:skinUrl);
  let chosenCape=$derived(variant==='none'||variant==='dinnerbone'||variant==='grumm'?capeUrl:'');
  $effect(()=>{
    egg?.id;easterEgg;expired=false;
    if(egg?.durationMs&&egg.variant){const timer=setTimeout(()=>expired=true,egg.durationMs);return()=>clearTimeout(timer);}
  });
</script>

<div class="ambient-character" style:--character-height="{height}px">
  <SkinModelViewer
    skinUrl={chosenSkin}
    fallbackSkinUrl={variant==='none'||variant==='dinnerbone'||variant==='grumm'?fallbackSkinUrl:''}
    capeUrl={chosenCape}
    model={['herobrine','entity303','crowned_steve'].includes(variant)?'default':model}
    {pose}
    {height}
    ambient
    controls={false}
    breathing
    lightIntensity={easterEgg ? 1.65 : 1.8}
    rightItem={egg?.item??automaticItems.right}
    leftItem={egg?.leftItem??automaticItems.left}
    characterVariant={variant}
    {whiteEyes}
  />
</div>

<style>
  .ambient-character {
    width: 100%;
    height: 100%;
    min-width: 0;
    pointer-events: none;
    user-select: none;
    background: transparent;
  }

</style>
