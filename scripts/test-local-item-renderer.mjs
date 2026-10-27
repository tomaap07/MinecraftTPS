import assert from 'node:assert/strict';
import{readFileSync}from'node:fs';import{resolve}from'node:path';
import ts from'typescript';import*as THREE from'three';import{SkinObject}from'skinview3d';
const root=resolve(import.meta.dirname,'..');
const code=path=>ts.transpileModule(readFileSync(resolve(root,path),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText;
const uri=text=>'data:text/javascript;base64,'+Buffer.from(text).toString('base64');
const modelUrl=uri(code('src/lib/item_models.ts'));
const textureCode=code('src/lib/poser_textures.ts').replace("'./item_models'",JSON.stringify(modelUrl));
const textureUrl=uri(textureCode),{preparePoserTextures,voxelItem}=await import(textureUrl);
const heldCode=code('src/lib/held_items.ts').replace("'./item_models'",JSON.stringify(modelUrl)).replace("'./poser_textures'",JSON.stringify(textureUrl));
assert.doesNotMatch(heldCode+textureCode,/original_item_sprites|originalItemSprite|fetch\(/,'No fabricated or remote object fallback is permitted');
const{mountHeldModel,mountHeldItem,disposeModel}=await import(uri(heldCode));
const{buildLocalItemModel,displayForHand}=await import(modelUrl);
const{POSES}=await import(uri(code('src/lib/poses_catalog.ts')));
const pixels=(w,h,color)=>{const data=new Uint8ClampedArray(w*h*4);for(let i=0;i<data.length;i+=4)data.set(color,i);return{width:w,height:h,data};};
const transform=(rotation=[0,0,0],translation=[0,0,0],scale=[1,1,1])=>({rotation,translation,scale});
const base={textures:{},elements:[],display:{right:transform(),left:transform()},source_model:'fixture'};
await preparePoserTextures({},{});assert.equal(voxelItem(THREE,'diamond_sword'),null,'An absent installed resource must leave an empty hand');assert.equal(mountHeldItem(THREE,new THREE.Group(),'shield','left').children.length,0);
const image=pixels(16,16,[0,0,0,0]);for(const[x,y,a]of[[3,13,255],[4,12,255],[5,11,127]])image.data.set([60,220,230,a],(y*16+x)*4);
const sword={...base,kind:'sprite',display:{right:transform([0,-90,55],[0,4,.5],[.85,.85,.85]),left:transform([0,90,-55],[0,4,.5],[.85,.85,.85])}};
const swordImages=new Map([['layer0',image]]);const sprite=buildLocalItemModel(THREE,sword,swordImages);assert.equal(sprite.children[0].count,2);assert.equal(sprite.children[0].geometry.parameters.depth,1);assert.equal(sprite.userData.localArtwork,true);disposeModel(sprite);
const faces=Object.fromEntries(['north','south','east','west','up','down'].map(name=>[name,{texture:name,uv:[0,0,16,16],rotation:0}]));
const cube={...base,kind:'model',elements:[{from:[0,0,0],to:[16,16,16],faces}]};const cubeImages=new Map(Object.keys(faces).map((name,i)=>[name,pixels(16,16,[30+i*20,100,180,255])]));
const model=buildLocalItemModel(THREE,cube,cubeImages);const axis={north:[0,0,-1],south:[0,0,1],east:[1,0,0],west:[-1,0,0],up:[0,1,0],down:[0,-1,0]};
for(const face of model.children[0].children){const n=new THREE.Vector3().fromBufferAttribute(face.geometry.getAttribute('normal'),0);assert.ok(n.dot(new THREE.Vector3(...axis[face.name]))>.999,face.name+' face winding is inverted');assert.equal(face.material.map.image.width,16);assert.equal(face.material.alphaTest,.5);}
assert.deepEqual(new THREE.Box3().setFromObject(model).getSize(new THREE.Vector3()).toArray(),[16,16,16]);disposeModel(model);
const rotatedCube={...cube,elements:[{...cube.elements[0],faces:{north:{texture:'north',uv:[0,0,16,16],rotation:90}}}]};
const rotated=buildLocalItemModel(THREE,rotatedCube,cubeImages);assert.deepEqual([...rotated.children[0].children[0].geometry.getAttribute('uv').array],[0,0,1,0,1,1,0,1],'Face rotation must rotate its texture coordinates without changing geometry');disposeModel(rotated);
assert.equal(buildLocalItemModel(THREE,cube,new Map()),null,'Unresolved model faces must not create a misleading empty prop');
const shield={...base,kind:'shield',display:{right:transform([0,90,0],[10,6,-4]),left:transform([0,90,0],[10,6,12])}};
const trident={...base,kind:'trident',display:{right:transform([0,60,0],[11,17,-2]),left:transform([0,60,0],[3,17,12])}};
assert.deepEqual(displayForHand(shield,'left').translation,[-10,6,12]);assert.deepEqual(shield.display.left.translation,[10,6,12],'Mirroring must not mutate the descriptor');
let contacts=0;
for(const slim of[false,true])for(const hand of['right','left']){
 const skin=new SkinObject();skin.modelType=slim?'slim':'default';const arm=hand==='left'?skin.leftArm:skin.rightArm;
 for(const[item,id,images]of[[sword,'diamond_sword',swordImages],[shield,'shield',new Map([['entity',pixels(64,64,[180,120,50,255])]])],[trident,'trident',new Map([['entity',pixels(64,32,[30,160,170,255])]])]]){
  const visual=buildLocalItemModel(THREE,item,images),mount=mountHeldModel(THREE,arm,id,hand,slim,visual,item);
  const grip=item.kind==='sprite'?new THREE.Vector3(-4.5,-5.5,0):new THREE.Vector3(...visual.userData.grip);
  for(const pose of POSES){arm.rotation.set(...pose.joints[hand==='left'?'leftArm':'rightArm'].map(v=>v*Math.PI/180));skin.position.set(...pose.position);skin.rotation.set(...pose.rotation.map(v=>v*Math.PI/180));skin.updateMatrixWorld(true);
   const world=visual.localToWorld(grip.clone()),local=arm.worldToLocal(world.clone()),palmX=(hand==='left'?1:-1)*(slim?.5:1);
   assert.ok(world.toArray().every(Number.isFinite));
   if(item.kind==='sprite'){assert.ok(Math.abs(local.x-palmX)<1.6&&local.y>=-10.5&&local.y<=-6.5&&Math.abs(local.z)<2.5,'Actual sword hilt left the palm');}
   else assert.ok(local.distanceTo(new THREE.Vector3(palmX,-8.5,0))<1e-5,'Entity handle left the palm');contacts++;
  }
  mount.updateMatrixWorld(true);mount.traverse(object=>{if(object.geometry){for(const attr of Object.values(object.geometry.attributes))assert.ok([...attr.array].every(Number.isFinite));}});disposeModel(mount);
 }
 skin.traverse(object=>{if(object.geometry)object.geometry.dispose();});
}
console.log(`${POSES.length} poses: ${contacts} classic/slim hand contacts, real-display mirroring, entity grips, six outward cube faces, sprite alpha extrusion, finite UVs and empty missing resources verified.`);
