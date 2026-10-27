<script lang="ts">
  import {observeSize} from "./observe_size";
  import { onMount } from "svelte";
  import type { SkinViewer } from "skinview3d";
  import type { DirectionalLight, Mesh, Group, Material, Quaternion, Vector3, Euler } from "three";
  import { FALLBACK_SKIN, JOINTS, WELCOME_POSE, type Pose } from "./poses_catalog";
  import { createCharacterAccessory, disposeModel, mountHeldItem, type CharacterVariant, type HeldItemId } from "./held_items";
  import { prepareSkinSurface } from "./skin_surface";
  import {defaultPlayerAppearance,isFallbackSkin,loadSkinImage,type DefaultSkinVariant} from "./player_skins";
  import { preparePoserTextures } from "./poser_textures";
  import {localGameArtwork} from './local_game_art';

  let { skinUrl = FALLBACK_SKIN, fallbackSkinUrl = "", capeUrl, model = "auto-detect", pose, breathing = true,
    lightIntensity = 1.8, lightRotation = -35, fov = 42, outerLayer = true, controls = true,
    height = 490, ambient = false, rightItem = 'none', leftItem = 'none', characterVariant = 'none', whiteEyes = false, backEquipment = 'cape' }: { skinUrl?: string; fallbackSkinUrl?:string; capeUrl?: string; model?: "default"|"slim"|"auto-detect";
    pose?: Pose; breathing?: boolean; lightIntensity?: number; lightRotation?: number;
    fov?: number; outerLayer?: boolean; controls?: boolean; height?: number; ambient?: boolean;
    rightItem?:HeldItemId;leftItem?:HeldItemId;characterVariant?:CharacterVariant;whiteEyes?:boolean;backEquipment?:'cape'|'elytra' } = $props();
  let canvasElement: HTMLCanvasElement;
  let viewport: HTMLDivElement;
  let viewer: SkinViewer | undefined;
  let keyLight: DirectionalLight | undefined;
  let groundPlane: Mesh | undefined;
  let animationApi: typeof import("skinview3d") | undefined;
  let threeApi: typeof import("three") | undefined;
  let viewerReady = $state(false);
  let motion = $state<"pose"|"walk"|"run">("pose");
  let idle = $state(true);
  let autoRotate = $state(false);
  let zoom = $state(0.78);
  let viewerError = $state("");
  let skinSequence = 0;
  let previousPose: Pose | undefined;
  let reducedMotion = $state(false);
  let gazeX = 0;
  let gazeY = 0;
  let gazeTargetX = 0;
  let gazeTargetY = 0;
  let homeYaw=0;
  let dragging=$state(false);
  let dragPointer:number|null=null;
  let lastPointerX=0;
  let rigScratch:{torso:Quaternion;point:Vector3;euler:Euler}|undefined;
  let blinkOpen:ImageData|undefined;
  let blinkClosed:ImageData|null=null;
  let eyesClosed=false;
  let nextBlink=4;
  let blinkUntil=0;
  let lastAnimationTime=0;
  let decorations:Group[]=[];
  let originalMaterials=new Map<Mesh,Material|Material[]>();
  let variantMaterial:Material|undefined;
  function clearEquipment(){
    for(const group of decorations)disposeModel(group);decorations=[];
    for(const[mesh,material]of originalMaterials)mesh.material=material;originalMaterials.clear();
    variantMaterial?.dispose();variantMaterial=undefined;
    if(viewer){viewer.playerObject.skin.visible=true;viewer.playerObject.cape.visible=!!capeUrl&&!!viewer.playerObject.cape.map&&backEquipment==='cape';viewer.playerObject.elytra.visible=!!capeUrl&&!!viewer.playerObject.elytra.map&&backEquipment==='elytra';}
  }
  function updateEquipment(){
    if(!viewer||!threeApi)return;
    clearEquipment();const player=viewer.playerObject;const three=threeApi;
    const accessory=createCharacterAccessory(three,characterVariant);
    const mob=(characterVariant==='charged_creeper'||characterVariant==='enderman')&&accessory.children.length>0;
    player.skin.visible=!mob;if(mob){player.cape.visible=false;player.elytra.visible=false;}
    if(characterVariant==='null'||characterVariant==='entity303'){
      variantMaterial=new three.MeshStandardMaterial({color:characterVariant==='null'?'#08090e':'#efefec',roughness:.94});
      player.skin.traverse(object=>{if(object instanceof three.Mesh){originalMaterials.set(object,object.material);object.material=variantMaterial!;}});
    }
    if(['herobrine','null','entity303','crowned_steve'].includes(characterVariant))player.skin.head.add(accessory);
    else player.add(accessory);
    decorations.push(accessory);
    if(!mob){
      decorations.push(mountHeldItem(three,player.skin.rightArm,rightItem,'right',player.skin.modelType==='slim'));
      decorations.push(mountHeldItem(three,player.skin.leftArm,leftItem,'left',player.skin.modelType==='slim'));
    }
    fitAmbientCamera();
  }

  function applyPose(currentPose: Pose, time = 0, breathe = false) {
    if (!viewer) return;
    const player = viewer.playerObject;
    const radians = Math.PI / 180;
    for (const joint of JOINTS) {
      const rotation = currentPose.joints[joint];
      const object = joint === "cape" ? player.cape : player.skin[joint];
      object.rotation.set(rotation[0] * radians, rotation[1] * radians, rotation[2] * radians);
    }
    const breath = breathe ? Math.sin(time * 1.8) * 0.14 : 0;
    player.skin.body.position.y = -6 + breath;
    player.skin.head.position.y = breath;
    player.skin.rightArm.position.y = -2 + breath;
    player.skin.leftArm.position.y = -2 + breath;
    if (ambient && !reducedMotion) {
      gazeX += (gazeTargetX - gazeX) * 0.075;
      gazeY += (gazeTargetY - gazeY) * 0.075;
      player.skin.head.rotation.y=Math.max(-65*radians,Math.min(65*radians,player.skin.head.rotation.y+gazeX*radians*32*Math.cos(homeYaw)));
      player.skin.head.rotation.x=Math.max(-55*radians,Math.min(65*radians,player.skin.head.rotation.x+gazeY*radians*42));
    }


    if(rigScratch){
      const {torso,point,euler}=rigScratch;
      torso.setFromEuler(euler.set(...currentPose.joints.body.map(value=>value*radians) as [number,number,number]));
      const moveUpper=(part:typeof player.skin.head,x:number,y:number,z:number)=>{
        point.set(x,y+12+breath,z).applyQuaternion(torso);
        part.position.set(point.x,point.y-12,point.z);
      };
      moveUpper(player.skin.body,0,-6,0);
      for(const [part,x,y] of [[player.skin.head,0,0],[player.skin.rightArm,-5,-2],[player.skin.leftArm,5,-2]] as const){
        moveUpper(part,x,y,0);part.quaternion.premultiply(torso);
      }
      point.set(0,12+breath,-2).applyQuaternion(torso);
      player.cape.position.set(point.x,point.y-12+player.skin.position.y,point.z);
      const capeAngles=currentPose.joints.cape;
      player.cape.rotation.set(capeAngles[0]*radians,Math.PI+capeAngles[1]*radians,capeAngles[2]*radians);
      player.cape.quaternion.premultiply(torso);
    }
    player.position.set(...currentPose.position);
    player.rotation.set(...currentPose.rotation.map((v) => v * radians) as [number,number,number]);
    if(ambient)player.rotation.y+=homeYaw;
    if(characterVariant==='dinnerbone'||characterVariant==='grumm')player.rotation.z+=Math.PI;
    updateBlink(time);
  }

  function setEyesClosed(closed:boolean){
    if(!viewer||!blinkOpen||!blinkClosed||eyesClosed===closed)return;
    viewer.skinCanvas.getContext('2d')?.putImageData(closed?blinkClosed:blinkOpen,0,0);
    if(viewer.playerObject.skin.map)viewer.playerObject.skin.map.needsUpdate=true;
    eyesClosed=closed;
  }
  function updateBlink(time:number){
    const enabled=ambient&&!reducedMotion&&['none','crowned_steve','dinnerbone','grumm'].includes(characterVariant)&&!!blinkClosed;
    if(!enabled){setEyesClosed(false);return;}
    if(time<lastAnimationTime){nextBlink=time+3+Math.random()*4;blinkUntil=0;}
    lastAnimationTime=time;
    if(time>=nextBlink){blinkUntil=time+.13;nextBlink=time+3.5+Math.random()*4;}
    setEyesClosed(time<blinkUntil);
  }
  function finishSkinLoad(fallback:boolean){
    if(!viewer)return;
    const surface=prepareSkinSurface(viewer.skinCanvas,fallback,whiteEyes);
    blinkOpen=surface.open;blinkClosed=surface.closed;eyesClosed=false;lastAnimationTime=0;
    nextBlink=3+Math.random()*4;blinkUntil=0;


    for(const joint of ['head','body','rightArm','leftArm','rightLeg','leftLeg'] as const){
      for(const [layer,outer] of [[viewer.playerObject.skin[joint].innerLayer,false],[viewer.playerObject.skin[joint].outerLayer,true]] as const){
        layer.traverse(object=>{
          if(!threeApi||!(object instanceof threeApi.Mesh))return;
          for(const entry of Array.isArray(object.material)?object.material:[object.material]){
            entry.transparent=false;entry.opacity=1;entry.depthWrite=true;entry.alphaTest=outer?.5:0;entry.needsUpdate=true;
          }
        });
      }
    }
    if(viewer.playerObject.skin.map)viewer.playerObject.skin.map.needsUpdate=true;
    updateEquipment();
  }

  function startDrag(event:PointerEvent){
    if(!ambient||event.button!==0)return;
    dragPointer=event.pointerId;lastPointerX=event.clientX;dragging=true;
    canvasElement.setPointerCapture(event.pointerId);canvasElement.focus({preventScroll:true});event.preventDefault();
  }
  function moveDrag(event:PointerEvent){
    if(!ambient||event.pointerId!==dragPointer)return;
    const width=Math.max(160,viewport.clientWidth);
    homeYaw+=(event.clientX-lastPointerX)/width*Math.PI*2;
    homeYaw=Math.atan2(Math.sin(homeYaw),Math.cos(homeYaw));lastPointerX=event.clientX;
    if(viewer){applyPose(pose??WELCOME_POSE,lastAnimationTime,breathing&&idle&&!reducedMotion);viewer.render();}
  }
  function endDrag(event:PointerEvent){
    if(event.pointerId!==dragPointer)return;
    dragPointer=null;dragging=false;
    if(canvasElement.hasPointerCapture(event.pointerId))canvasElement.releasePointerCapture(event.pointerId);
  }
  function rotateWithKeyboard(event:KeyboardEvent){
    if(!ambient||!['ArrowLeft','ArrowRight','Home'].includes(event.key))return;
    event.preventDefault();homeYaw=event.key==='Home'?0:homeYaw+(event.key==='ArrowLeft'?-1:1)*Math.PI/12;
    if(viewer){applyPose(pose??WELCOME_POSE,lastAnimationTime,breathing&&idle&&!reducedMotion);viewer.render();}
  }

  function fitAmbientCamera() {
    if (!ambient || !viewer || !threeApi) return;

    const three = threeApi;
    viewer.playerObject.updateMatrixWorld(true);
    const bounds = new three.Box3();
    viewer.playerObject.traverseVisible((object) => {
      if (object instanceof three.Mesh) bounds.expandByObject(object);
    });
    const size = bounds.getSize(new three.Vector3());
    const center = bounds.getCenter(new three.Vector3());
    if (!Number.isFinite(size.y) || size.y === 0) return;
    if (groundPlane) groundPlane.position.y = bounds.min.y - 0.2;
    const verticalFov = viewer.camera.fov * Math.PI / 180;
    const aspect = Math.max(viewer.width / viewer.height, 0.2);
    const halfHeight = Math.max(size.y / 2 + 1.8, (Math.hypot(size.x,size.z) / 2 + 2.0) / aspect);
    const distance = halfHeight / Math.tan(verticalFov / 2) + size.z / 2;
    viewer.camera.position.set(center.x, center.y, center.z + distance);
    viewer.controls.target.copy(center);
    viewer.camera.lookAt(center);
    viewer.camera.updateProjectionMatrix();
  }

  $effect(() => {

    if (pose !== previousPose) { previousPose = pose; motion = "pose"; }
  });

  function imageSource(url: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
      const image = new Image(); image.crossOrigin = "anonymous";
      image.onload = () => resolve(image);
      image.onerror = () => reject(new Error("No se pudo leer la textura"));
      image.src = url;
    });
  }

  async function requestedSkin(source:string,backup:string,playerModel:"default"|"slim"|"auto-detect"){
    for(const url of [...new Set([source,backup].filter(value=>value&&!isFallbackSkin(value)))]){
      try{return{image:await loadSkinImage(url),model:url===backup?"auto-detect" as const:playerModel,generic:false};}catch{ }
    }
    const variant:DefaultSkinVariant=playerModel==='slim'?'alex':'steve';
    try{const appearance=await defaultPlayerAppearance(variant);return{image:await loadSkinImage(appearance.skin_data),model:appearance.model,generic:true};}
    catch{return{image:await loadSkinImage(FALLBACK_SKIN),model:'default' as const,generic:true};}
  }

  $effect(() => {
    const skin = skinUrl || FALLBACK_SKIN,backup=fallbackSkinUrl, cape = capeUrl,playerModel=model,equipment=backEquipment;
    whiteEyes;
    if (!viewerReady || !viewer) return;
    const sequence = ++skinSequence;
    viewerError = "";
    void requestedSkin(skin,backup,playerModel).then(async result=>{
      if(sequence!==skinSequence||!viewer)return;
      viewer.loadSkin(result.image,{model:result.model});
      finishSkinLoad(result.generic);

      if(result.generic||!cape){viewer.resetCape();fitAmbientCamera();return;}
      try{const image=await imageSource(cape);if(sequence===skinSequence&&viewer){viewer.loadCape(image,{backEquipment:equipment});if(characterVariant==='charged_creeper'||characterVariant==='enderman')viewer.playerObject.cape.visible=false;fitAmbientCamera();}}
      catch{if(sequence===skinSequence)viewer?.resetCape();}
    }).catch(()=>{if(sequence===skinSequence&&viewer)viewerError="No se pudo cargar la skin ni su respaldo. Comprueba la conexión.";});
    return()=>{if(sequence===skinSequence)skinSequence++;};
  });

  $effect(()=>{
    rightItem;leftItem;characterVariant;
    if(viewerReady)updateEquipment();
  });

  $effect(()=>{
    const artwork=$localGameArtwork;let cancelled=false;
    if(viewerReady)void preparePoserTextures(artwork.textures,artwork.models).then(()=>{if(!cancelled&&viewer)updateEquipment();});
    return()=>{cancelled=true;};
  });

  $effect(() => {
    const currentPose = pose ?? WELCOME_POSE;
    const breathe = breathing && idle && !reducedMotion;
    const mode = ambient ? "pose" : motion;
    const showOuter = outerLayer;
    const intensity = lightIntensity, direction = lightRotation, fieldOfView = fov;
    if (!viewerReady || !viewer || !animationApi) return;
    viewer.playerObject.skin.setOuterLayerVisible(showOuter);
    viewer.fov = fieldOfView;
    viewer.autoRotate = ambient ? false : autoRotate;
    viewer.controls.enabled = !ambient;
    viewer.controls.enableRotate = !ambient;
    viewer.controls.enableZoom = !ambient;
    viewer.controls.enablePan = false;
    viewer.globalLight.intensity = 1.0;
    viewer.cameraLight.intensity = 0.5;
    if (keyLight) {
      keyLight.intensity = intensity;
      const angle = direction * Math.PI / 180;
      keyLight.position.set(Math.sin(angle) * 55, 55, Math.cos(angle) * 55);
    }
    if (mode === "walk") { viewer.playerObject.resetJoints();viewer.animation = new animationApi.WalkingAnimation(); }
    else if (mode === "run") { viewer.playerObject.resetJoints();viewer.animation = new animationApi.RunningAnimation(); }
    else {
      applyPose(currentPose);
      viewer.animation = new animationApi.FunctionAnimation((_player, time) => applyPose(currentPose, time, breathe));
    }
    fitAmbientCamera();
  });

  onMount(() => {
    let cancelled = false;
    let stopResize: (()=>void) | undefined;
    const motionPreference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const updateMotionPreference = () => { reducedMotion = motionPreference.matches; };
    updateMotionPreference();
    motionPreference.addEventListener("change", updateMotionPreference);
    const lookAtPointer = (event: PointerEvent) => {
      if (!ambient || reducedMotion || !viewport) return;
      const bounds = viewport.getBoundingClientRect();
      let eyeX=bounds.left+bounds.width/2,eyeY=bounds.top+bounds.height*.28;
      if(viewer&&rigScratch){
        viewer.playerObject.updateMatrixWorld(true);
        const head=rigScratch.point.set(0,4,0).applyMatrix4(viewer.playerObject.skin.head.matrixWorld).project(viewer.camera);
        if(Number.isFinite(head.x)&&Number.isFinite(head.y)&&head.z>-1&&head.z<1){
          eyeX=bounds.left+(head.x+1)*bounds.width/2;
          eyeY=bounds.top+(1-head.y)*bounds.height/2;
        }
      }


      gazeTargetX=Math.max(-1,Math.min(1,(event.clientX-eyeX)/Math.max(bounds.width*.7,80)));
      gazeTargetY=Math.max(-1,Math.min(1,(event.clientY-eyeY)/Math.max(bounds.height*.38,80)));
    };
    window.addEventListener("pointermove", lookAtPointer, { passive: true });
    void Promise.all([import("skinview3d"), import("three")]).then(async ([api, three]) => {
      if (cancelled) return;
      animationApi = api;
      threeApi = three;
      rigScratch={torso:new three.Quaternion(),point:new three.Vector3(),euler:new three.Euler()};
      viewer = new api.SkinViewer({canvas:canvasElement,width:440,height,zoom,fov,enableControls:!ambient,preserveDrawingBuffer:true});
      viewer.background = null;
      viewer.renderer.setClearColor(0x000000, 0);
      viewer.renderer.shadowMap.enabled = true;
      viewer.renderer.shadowMap.type = three.PCFSoftShadowMap;
      viewer.controls.enablePan = false;
      viewer.controls.minDistance = ambient ? 1 : 28;
      viewer.controls.maxDistance = ambient ? 512 : 130;
      viewer.playerObject.traverse((object) => { object.castShadow = true; object.receiveShadow = true; });
      keyLight = new three.DirectionalLight(0xffffff, lightIntensity);
      keyLight.castShadow = true;
      keyLight.shadow.mapSize.set(1024,1024);
      keyLight.shadow.camera.left = -28; keyLight.shadow.camera.right = 28;
      keyLight.shadow.camera.top = 35; keyLight.shadow.camera.bottom = -35;
      keyLight.shadow.bias = -0.0003;
      keyLight.shadow.normalBias = 0.05;
      viewer.scene.add(keyLight);
      const floor = new three.Mesh(new three.PlaneGeometry(160,160), new three.ShadowMaterial({color:0x151021,opacity:0.12}));
      groundPlane = floor;
      floor.rotation.x = -Math.PI/2; floor.position.y = -24.5; floor.receiveShadow = true;
      viewer.scene.add(floor);


      const initial=await requestedSkin(skinUrl||FALLBACK_SKIN,fallbackSkinUrl,model);
      if(cancelled||!viewer)return;
      viewer.loadSkin(initial.image,{model:initial.model});
      finishSkinLoad(initial.generic);
      const resize = (width:number, height:number) => {
        if (!viewer) return;
        viewer.setSize(Math.max(160,width),Math.max(220,height));
        fitAmbientCamera();
      };
      stopResize=observeSize(viewport,resize);
      viewerReady = true;

      void preparePoserTextures($localGameArtwork.textures,$localGameArtwork.models).then(()=>{if(!cancelled)updateEquipment();});
    }).catch((error) => { viewerError = `El visor 3D no pudo iniciarse: ${String(error)}`; });
    return () => {
      cancelled = true; skinSequence++; stopResize?.();
      motionPreference.removeEventListener("change", updateMotionPreference);
      window.removeEventListener("pointermove", lookAtPointer);
      clearEquipment();
      groundPlane?.geometry.dispose();
      if (groundPlane) {
        const materials = Array.isArray(groundPlane.material) ? groundPlane.material : [groundPlane.material];
        for (const material of materials) material.dispose();
      }
      viewer?.dispose(); viewer=undefined;
      rigScratch=undefined;blinkOpen=undefined;blinkClosed=null;
    };
  });

  function chooseMotion(next: "pose"|"walk"|"run") { motion = motion === next ? "pose" : next; }
  function backView() { if (viewer) { viewer.resetCameraPose(); viewer.camera.position.z *= -1; viewer.controls.update(); } }
  function resetCamera() { zoom=0.78; if(viewer) { viewer.playerWrapper.rotation.set(0,0,0); viewer.zoom=zoom; viewer.resetCameraPose(); viewer.controls.target.set(0,0,0); viewer.controls.update(); } }

  export async function renderPng(width = 2048, renderHeight = 2048): Promise<Blob> {
    if (!viewer || !viewerReady) throw new Error("Espera a que termine de cargar el modelo 3D.");
    const current = viewer;
    const previous = {width:current.width,height:current.height,ratio:current.pixelRatio,paused:current.renderPaused,groundVisible:groundPlane?.visible};
    current.renderPaused = true;
    if (groundPlane) groundPlane.visible = false;
    try {
      current.pixelRatio=1;
      current.setSize(Math.min(4096, Math.max(256,width)), Math.min(4096,Math.max(256,renderHeight)));
      current.render();
      return await new Promise<Blob>((resolve,reject)=>current.canvas.toBlob((blob)=>blob ? resolve(blob) : reject(new Error("No se pudo exportar el lienzo PNG.")),"image/png"));
    } finally { if(groundPlane) groundPlane.visible=previous.groundVisible??true; current.setSize(previous.width,previous.height); current.pixelRatio=previous.ratio; current.renderPaused=previous.paused; current.render(); }
  }
</script>

<div class="skin-viewer-wrap" class:ambient style:min-height={ambient ? undefined : `${height}px`} style:height={ambient ? "100%" : undefined}>
  <div class="viewer-viewport" bind:this={viewport} style:height={ambient ? "100%" : `${height}px`}>
    {#if !ambient}<div class="viewer-halo" aria-hidden="true"></div>{/if}
    <canvas bind:this={canvasElement} class:dragging role={ambient?'application':'img'} tabindex={ambient?0:undefined} onpointerdown={startDrag} onpointermove={moveDrag} onpointerup={endDrag} onpointercancel={endDrag} onlostpointercapture={endDrag} onkeydown={rotateWithKeyboard} aria-label={ambient ? "Personaje de Minecraft. Mantén pulsado y arrastra para girarlo 360 grados. Usa las flechas para girar o Inicio para restablecer." : "Personaje de Minecraft en 3D. Arrastra para girar y usa la rueda para acercar."}></canvas>
    {#if !ambient && !viewerReady && !viewerError}<span class="viewer-loading">Preparando personaje…</span>{/if}
  </div>
  {#if controls && !ambient}<div class="skin-viewer-controls">
    <button class:active={motion === "walk"} onclick={() => chooseMotion("walk")}>Caminar</button>
    <button class:active={motion === "run"} onclick={() => chooseMotion("run")}>Correr</button>
    <button onclick={backView}>Ver espalda / capa</button>
    <button onclick={resetCamera}>Reset cámara</button>
    <button class:active={idle} onclick={() => (idle=!idle)} aria-pressed={idle}>Respiración {idle ? "activa" : "pausada"}</button>
    <button class:active={autoRotate} onclick={() => (autoRotate=!autoRotate)} aria-pressed={autoRotate}>Giro automático</button>
    <small>Arrastra para girar · rueda para zoom · fondo transparente</small>
  </div>{/if}
  {#if viewerError && !ambient}<p class="viewer-error" role="status">{viewerError}</p>{/if}
</div>

<style>
  .skin-viewer-wrap{display:flex;flex-direction:column;width:100%;min-width:0;border-radius:18px;overflow:hidden;background:transparent}
  .skin-viewer-wrap.ambient{border-radius:0;overflow:visible;pointer-events:none;user-select:none}
  .ambient .viewer-viewport{flex:1;min-height:0;overflow:visible}
  .ambient canvas{touch-action:pan-y;pointer-events:auto;cursor:grab}.ambient canvas.dragging{cursor:grabbing}.ambient canvas:focus-visible{outline:2px solid var(--primary);outline-offset:-3px;border-radius:12px}
  .viewer-viewport{position:relative;min-width:0;isolation:isolate;overflow:hidden}
  canvas{display:block;width:100%;height:100%;position:relative;z-index:1;touch-action:none;outline:none}
  .viewer-halo{position:absolute;inset:18% 14%;border-radius:50%;background:radial-gradient(ellipse,rgba(148,95,230,.10),transparent 67%);pointer-events:none}
  .viewer-loading{position:absolute;inset:45% 0 auto;text-align:center;color:#c3b2da;font-size:.8rem}
  .skin-viewer-controls{display:flex;align-items:center;justify-content:center;flex-wrap:wrap;gap:7px;padding:12px;border-top:1px solid rgba(180,160,220,.1);background:rgba(16,14,29,.24)}
  .skin-viewer-controls button{padding:7px 10px;border:1px solid rgba(185,159,225,.20);border-radius:9px;background:rgba(39,31,59,.5);color:#d6cbe8;cursor:pointer;font-size:.75rem;transition:.18s}
  .skin-viewer-controls button:hover,.skin-viewer-controls button.active{border-color:rgba(186,139,255,.6);color:#e9d6ff;background:rgba(130,73,195,.22)}
  .skin-viewer-controls small{width:100%;font-size:.66rem;color:#8c819e;text-align:center}
  .viewer-error{margin:0;padding:8px 12px;color:#f6aaa7;font-size:.8rem}
  @media(prefers-reduced-motion:reduce){.skin-viewer-controls button{transition:none}}
</style>
