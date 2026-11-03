<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SkinModelViewer from './SkinModelViewer.svelte';
  import UiIcon from './UiIcon.svelte';
  import { isGameRunning } from './game_session';
  let { skinUrl, fallbackSkinUrl='', username, model='auto-detect' }: { skinUrl:string; fallbackSkinUrl?:string; username:string; model?:'default'|'slim'|'auto-detect' } = $props();
  type Cape={id:string;name:string;original:boolean;data_url:string};
  type Target={id:string;name:string;game_version:string;loader:string;version_id:string;pack_id:string|null};
  let capes=$state<Cape[]>([]), targets=$state<Target[]>([]), selected=$state(''), target=$state('');
  let canvas:HTMLCanvasElement;
  let name=$state('Mi capa'), color=$state('#a855f7'), tool=$state<'paint'|'erase'|'fill'>('paint');
  let message=$state(''), failed=$state(false), busy=$state(false), wings=$state(false), crown=$state(false);
  let revision=$state(0), preview=$state('');
  let past:ImageData[]=[], future:ImageData[]=[];
  let stroke=false, changed=false;
  let undoCount=$state(0), redoCount=$state(0);
  const context=()=>canvas?.getContext('2d',{willReadFrequently:true});
  function snapshot(){const ctx=context();if(ctx){past.push(ctx.getImageData(0,0,64,32));if(past.length>40)past.shift();future=[];undoCount=past.length;redoCount=0;}}
  function update(){revision++;preview=canvas.toDataURL('image/png');}
  function undo(){const ctx=context(),image=past.pop();if(!ctx||!image)return;future.push(ctx.getImageData(0,0,64,32));ctx.putImageData(image,0,0);undoCount=past.length;redoCount=future.length;selected='';update();}
  function redo(){const ctx=context(),image=future.pop();if(!ctx||!image)return;past.push(ctx.getImageData(0,0,64,32));ctx.putImageData(image,0,0);undoCount=past.length;redoCount=future.length;selected='';update();}
  function flood(x:number,y:number){
    const ctx=context();if(!ctx)return;const image=ctx.getImageData(0,0,64,32),data=image.data,index=(y*64+x)*4;
    const old=Array.from(data.slice(index,index+4)),rgba=[parseInt(color.slice(1,3),16),parseInt(color.slice(3,5),16),parseInt(color.slice(5,7),16),255];
    if(old.every((v,i)=>v===rgba[i]))return;
    const queue=[y*64+x],seen=new Uint8Array(2048);while(queue.length){const p=queue.pop()!;if(seen[p])continue;seen[p]=1;const i=p*4;if(!old.every((v,k)=>data[i+k]===v))continue;data.set(rgba,i);const px=p%64,py=Math.floor(p/64);if(px>0)queue.push(p-1);if(px<63)queue.push(p+1);if(py>0)queue.push(p-64);if(py<31)queue.push(p+64);}
    ctx.putImageData(image,0,0);
  }
  function paint(event:PointerEvent){if(!stroke)return;const ctx=context();if(!ctx)return;const rect=canvas.getBoundingClientRect(),x=Math.floor((event.clientX-rect.left)/rect.width*64),y=Math.floor((event.clientY-rect.top)/rect.height*32);if(x<0||y<0||x>=64||y>=32)return;
    if(tool==='erase')ctx.clearRect(x,y,1,1);else if(tool==='fill'){flood(x,y);stroke=false;}else{ctx.fillStyle=color;ctx.fillRect(x,y,1,1);}changed=true;selected='';
  }
  function begin(event:PointerEvent){if(event.button!==0)return;snapshot();stroke=true;changed=false;canvas.setPointerCapture(event.pointerId);paint(event);}
  function finish(){if(stroke||changed){stroke=false;changed=false;update();}}
  async function perform(action:()=>Promise<void>){if(busy)return;busy=true;message='';failed=false;try{await action();}catch(error){failed=true;message=String(error);}finally{busy=false;}}
  async function load(cape:Cape){const img=new Image();img.onload=()=>{snapshot();const ctx=context();ctx?.clearRect(0,0,64,32);ctx?.drawImage(img,0,0);selected=cape.id;name=cape.name;update();};img.onerror=()=>{failed=true;message='No se pudo abrir la capa';};img.src=cape.data_url;}
  async function refresh(){capes=await invoke<Cape[]>('list_tp_capes');targets=(await invoke<Target[]>('studio_install_targets',{username})).filter(t=>t.loader==='fabric'&&t.game_version==='1.21.1');if(!target&&targets.length)target=targets[0].id;}
  onMount(()=>{void perform(async()=>{await refresh();if(capes[0])await load(capes[0]);});});
  async function save(){await perform(async()=>{const cape=await invoke<Cape>('save_tp_cape',{name,dataUrl:canvas.toDataURL('image/png')});selected=cape.id;await refresh();message='Capa guardada localmente. No se ha publicado ni enviado a terceros.';});}
  async function importFile(event:Event){const input=event.currentTarget as HTMLInputElement,file=input.files?.[0];input.value='';if(!file)return;await perform(async()=>{if(file.size>256*1024)throw new Error('El PNG debe ocupar hasta 256 KiB');const dataUrl=await new Promise<string>((resolve,reject)=>{const r=new FileReader();r.onload=()=>resolve(String(r.result));r.onerror=()=>reject(new Error('No se pudo leer el PNG'));r.readAsDataURL(file);});const cape=await invoke<Cape>('save_tp_cape',{name:file.name.replace(/\.png$/i,''),dataUrl});await refresh();await load(cape);message='Capa importada de forma local.';});}
  function download(){const link=document.createElement('a');link.href=canvas.toDataURL('image/png');link.download=`${name.replace(/[^a-zA-Z0-9_-]/g,'_')||'capa-tp'}.png`;link.click();}
  async function equip(){await perform(async()=>{const installation=targets.find(t=>t.id===target);if(!installation)throw new Error('Selecciona una instancia Fabric 1.21.1 instalada');message=await invoke<string>('equip_tp_cosmetics',{username,versionId:installation.version_id,packId:installation.pack_id,capeId:selected||null,wings,crown});});}
</script>

<section class="cape-studio" aria-label="Editor de capas TP">
  <header><div><small>TP CLIENT · ARTE ORIGINAL</small><h2>Tu capa, tu diseño</h2><p>Editor local de capas PNG. Tus capas oficiales de Minecraft se gestionan por separado.</p></div><span class="free">Gratis · sin publicaciones automáticas</span></header>
  <div class="gifts"><article><UiIcon name="image"/><strong>Nebulosa Infinita</strong><span>Capa original incluida</span></article><article><UiIcon name="pose"/><strong>Serafín Celestial</strong><span>Alas cosméticas · no conceden vuelo</span></article><article><UiIcon name="trophy"/><strong>Eclipse Elemental</strong><span>Corona original · solo visual</span></article></div>
  <div class="workspace">
    <div class="editor"><div class="tools"><label>Nombre<input bind:value={name} maxlength="60" /></label><label>Color<input type="color" bind:value={color}/></label><label>Herramienta<select bind:value={tool}><option value="paint">Pincel</option><option value="erase">Borrador</option><option value="fill">Relleno</option></select></label></div>
      <div class="canvas-wrap"><canvas width="64" height="32" bind:this={canvas} onpointerdown={begin} onpointermove={paint} onpointerup={finish} onpointercancel={finish} onlostpointercapture={finish} aria-label="Lienzo de capa de 64 por 32 píxeles"></canvas></div>
      <p class="hint">Textura completa de 64 × 32. La vista 3D permite revisar frente, reverso y bordes.</p>
      <div class="actions"><button onclick={undo} disabled={!undoCount||busy}>Deshacer</button><button onclick={redo} disabled={!redoCount||busy}>Rehacer</button><label class="button">Importar PNG<input class="file" type="file" accept="image/png" onchange={importFile} disabled={busy}/></label><button onclick={download} disabled={!revision}>Exportar PNG</button><button class="primary" onclick={save} disabled={busy||!revision}>Guardar capa</button></div>
    </div>
    <div class="preview"><h3>Vista previa local</h3>{#if preview&&!$isGameRunning}<SkinModelViewer {skinUrl} {fallbackSkinUrl} {model} capeUrl={preview} controls height={320}/>{:else}<p>La vista 3D se pausa mientras juegas.</p>{/if}<small>No reclama propiedad de capas oficiales. Otros jugadores necesitan una integración compatible para ver cosméticos TP.</small></div>
  </div>
  <div class="library"><h3>Armario de capas locales</h3><div class="cape-grid">{#each capes as cape}<button class:selected={selected===cape.id} onclick={()=>load(cape)} disabled={busy}><img src={cape.data_url} alt=""/><strong>{cape.name}</strong><span>{cape.original?'Original TP':'PNG local del usuario'}</span></button>{/each}</div></div>
  <div class="equip"><div><h3>Equipar en TP Client</h3><p>Instancias compatibles: Fabric 1.21.1. TP Client debe estar instalado; activa Cosméticos en su menú. Solo se copian archivos propios a la instancia elegida.</p></div><label>Instancia<select aria-label="Instancia" bind:value={target} disabled={busy}><option value="">Selecciona una instalación</option>{#each targets as installation}<option value={installation.id}>{installation.name}</option>{/each}</select></label><div class="actions"><label><input type="checkbox" bind:checked={wings}/> Alas Serafín Celestial</label><label><input type="checkbox" bind:checked={crown}/> Corona Eclipse Elemental</label><button onclick={equip} disabled={busy||!target||$isGameRunning||(!selected&&revision>0)}>Aplicar equipamiento</button><button onclick={()=>perform(refresh)} disabled={busy}>Actualizar instancias</button></div><p class="hint">Para quitar la capa, desequipa desde TP Client. Guarda los cambios del lienzo antes de equiparlo.</p></div>
  {#if message}<p class:error={failed} class="notice" role="status">{message}</p>{/if}
</section>

<style>
  .cape-studio{margin-top:28px;border:1px solid #ffffff18;background:#0b0b12d9;padding:24px;color:#eee9f6;border-radius:14px}.cape-studio header{display:flex;gap:16px;justify-content:space-between;align-items:center}.cape-studio h2{margin:7px 0;font-size:1.5rem}.cape-studio p{color:#b3acbf;line-height:1.6;font-size:.82rem}.cape-studio small{color:#baa4d0}.free{border:1px solid #83d7b740;color:#9de5c4;padding:8px;font-size:.7rem}.gifts{display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin:20px 0}.gifts article{display:grid;grid-template-columns:28px 1fr;gap:7px;padding:14px;background:#ffffff04;border:1px solid #ffffff0f}.gifts span{grid-column:2;color:#a69db6;font-size:.7rem}.workspace{display:grid;grid-template-columns:minmax(0,1.6fr) minmax(240px,1fr);gap:18px}.tools,.actions{display:flex;gap:10px;align-items:end;flex-wrap:wrap}.tools label{display:grid;gap:6px;font-size:.75rem;flex:1}.cape-studio input:not([type=checkbox]):not([type=color]),.cape-studio select{background:#161421;color:#ece7f4;border:1px solid #ffffff26;padding:9px;min-width:0;width:100%;box-sizing:border-box}.cape-studio input[type=color]{height:38px;border:0;width:60px;background:transparent}.canvas-wrap{margin-top:16px;overflow:hidden;background-color:#292631;background-image:linear-gradient(45deg,#201d28 25%,transparent 25%),linear-gradient(-45deg,#201d28 25%,transparent 25%),linear-gradient(45deg,transparent 75%,#201d28 75%),linear-gradient(-45deg,transparent 75%,#201d28 75%);background-size:20px 20px;background-position:0 0,0 10px,10px -10px,-10px 0}.canvas-wrap canvas{width:100%;aspect-ratio:2;display:block;image-rendering:pixelated;touch-action:none;cursor:crosshair}.hint{font-size:.72rem!important}.cape-studio button,.button{border:1px solid #ffffff24;background:#ffffff08;color:#e9e3f0;padding:9px 13px;font:inherit;font-size:.77rem;cursor:pointer;border-radius:6px}.cape-studio button:disabled{opacity:.45;cursor:default}.cape-studio button.primary,.cape-studio button.selected{background:#a855f728;border-color:#b77bf3}.file{display:none}.preview{padding:14px;background:#ffffff03;border:1px solid #ffffff0f}.preview small{display:block;line-height:1.5;font-size:.7rem}.library{margin-top:20px}.cape-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(125px,1fr));gap:10px}.cape-grid button{display:grid;gap:6px;text-align:left}.cape-grid img{width:100%;image-rendering:pixelated;background:#ffffff06}.cape-grid span{color:#a69db6;font-size:.67rem}.equip{margin-top:20px;border-top:1px solid #ffffff18;padding-top:16px}.equip>label{display:grid;gap:7px;margin:12px 0;font-size:.8rem}.equip .actions{align-items:center}.equip .actions label{font-size:.78rem}.notice{border:1px solid #5bc39760;padding:12px;color:#b9e8d1!important}.notice.error{border-color:#cf7c83;color:#f3bcc4!important}.cape-studio :focus-visible{outline:2px solid #c084fc;outline-offset:3px}@media(max-width:900px){.workspace{grid-template-columns:1fr}.gifts{grid-template-columns:1fr}.cape-studio header{align-items:start;flex-direction:column}}
</style>
