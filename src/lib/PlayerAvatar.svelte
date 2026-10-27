<script lang="ts">
  import { FALLBACK_SKIN } from "./poses_catalog";
  import {defaultPlayerAppearance,isFallbackSkin,loadSkinImage} from './player_skins';

  let { skinUrl = FALLBACK_SKIN,fallbackSkinUrl='', name = "Steve", size = 42,model='auto-detect' }: { skinUrl?: string;fallbackSkinUrl?:string; name?: string; size?: number;model?:'default'|'slim'|'auto-detect' } = $props();
  let canvas: HTMLCanvasElement;

  $effect(() => {
    const url = skinUrl || FALLBACK_SKIN,backup=fallbackSkinUrl,variant=model==='slim'?'alex':'steve';
    if (!canvas) return;
    let disposed = false;
    const draw = (image:HTMLImageElement) => {
      if(disposed)return;const context=canvas.getContext('2d');if(!context)return;
      const scale=image.width/64,scratch=document.createElement('canvas');scratch.width=16*scale;scratch.height=8*scale;
      const crop=scratch.getContext('2d');if(!crop)return;
      crop.drawImage(image,8*scale,8*scale,8*scale,8*scale,0,0,8*scale,8*scale);
      crop.drawImage(image,40*scale,8*scale,8*scale,8*scale,8*scale,0,8*scale,8*scale);
      const pixels=crop.getImageData(0,0,scratch.width,scratch.height),half=scratch.width/2;
      let opaqueHat=false;
      if(image.height===image.width/2){const legacy=document.createElement('canvas');legacy.width=legacy.height=32*scale;const legacyContext=legacy.getContext('2d');if(legacyContext){legacyContext.drawImage(image,32*scale,0,32*scale,32*scale,0,0,legacy.width,legacy.height);const hat=legacyContext.getImageData(0,0,legacy.width,legacy.height).data;opaqueHat=true;for(let i=3;i<hat.length;i+=4)if(hat[i]<128){opaqueHat=false;break;}}}
      for(let y=0;y<scratch.height;y++)for(let x=0;x<scratch.width;x++){const i=(y*scratch.width+x)*4+3;pixels.data[i]=x<half?255:(image.height===image.width/2&&opaqueHat?0:pixels.data[i]>=128?255:0);}
      crop.putImageData(pixels,0,0);context.clearRect(0,0,64,64);context.imageSmoothingEnabled=false;
      context.drawImage(scratch,0,0,half,scratch.height,0,0,64,64);context.drawImage(scratch,half,0,half,scratch.height,0,0,64,64);
    };
    void(async()=>{
      for(const source of [...new Set([url,backup].filter(value=>value&&!isFallbackSkin(value)))]){try{draw(await loadSkinImage(source));return;}catch{if(disposed)return;}}
      try{const appearance=await defaultPlayerAppearance(variant);if(!disposed)draw(await loadSkinImage(appearance.skin_data));}
      catch{if(!disposed)try{draw(await loadSkinImage(FALLBACK_SKIN));}catch{ }}
    })();
    return () => { disposed = true; };
  });
</script>

<canvas bind:this={canvas} width="64" height="64" style:width="{size}px" style:height="{size}px" aria-label="Avatar de {name}"></canvas>

<style>
  canvas { display: block; flex-shrink: 0; border-radius: 8px; image-rendering: pixelated; background: #141722; }
</style>
