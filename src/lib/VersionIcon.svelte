<script lang="ts">
 import UiIcon from './UiIcon.svelte';
 import {localVersionArtwork} from './local_game_art';
 let{name,size=16}:{name:'dimension'|'release'|'snapshot'|'legacy';size?:number}=$props();
 const ids={dimension:'stone_block',release:'grass_block',snapshot:'redstone',legacy:'cobblestone_block'};
 let id=$derived(ids[name]);
 let model=$derived($localVersionArtwork.models[id]);
 let texture=$derived($localVersionArtwork.textures[id]);
 function faces(direction:'up'|'south'|'east'){
  const layers=model?.elements?.flatMap(element=>{const info=element.faces[direction];return info&&model?.textures[info.texture]?[{src:model.textures[info.texture],tinted:info.tintindex!=null}]:[];})??[];
  return layers.length?layers:[{src:texture,tinted:false}];
 }
 let top=$derived(faces('up')),left=$derived(faces('south')),right=$derived(faces('east'));
 let tint=$derived(model?.tint_color??'#91bd59');
 let red=$derived(parseInt(tint.slice(1,3),16)/255),green=$derived(parseInt(tint.slice(3,5),16)/255),blue=$derived(parseInt(tint.slice(5,7),16)/255);
 const componentId=$props.id();
 const filterId='local-grass-'+componentId;
</script>

<span class="version-icon" style:width={`${size}px`} style:height={`${size}px`} aria-hidden="true" title={texture?`${$localVersionArtwork.source}`:'Icono TP · aún no hay un recurso local disponible'}>
 {#if texture&&name==='snapshot'}<img src={texture} alt="" width={size} height={size}/>
 {:else if texture}
  <svg width={size} height={size} viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
   <defs><filter id={filterId} color-interpolation-filters="sRGB"><feColorMatrix type="matrix" values={`${red} 0 0 0 0 0 ${green} 0 0 0 0 0 ${blue} 0 0 0 0 0 1 0`}/></filter></defs>
   {#each top as face}<image href={face.src} width="16" height="16" transform="matrix(.875 .5 -.875 .5 16 1)" filter={face.tinted?`url(#${filterId})`:undefined}/>{/each}
   {#each left as face}<image href={face.src} width="16" height="16" transform="matrix(.875 .5 0 .875 2 9)" filter={face.tinted?`url(#${filterId})`:undefined}/>{/each}
   {#each right as face}<image href={face.src} width="16" height="16" transform="matrix(.875 -.5 0 .875 16 17)" filter={face.tinted?`url(#${filterId})`:undefined}/>{/each}
   <path d="M2 9 16 17v14L2 23Z" fill="#000" opacity=".12"/><path d="m16 17 14-8v14l-14 8Z" fill="#000" opacity=".3"/>
  </svg>
 {:else}<UiIcon name={name==='snapshot'?'flask':name==='legacy'?'layers':'cube'} {size}/>{/if}
</span>

<style>.version-icon{display:inline-flex;align-items:center;justify-content:center;flex:0 0 auto;vertical-align:middle;line-height:0}.version-icon img,.version-icon svg{display:block;image-rendering:pixelated;object-fit:contain}</style>
