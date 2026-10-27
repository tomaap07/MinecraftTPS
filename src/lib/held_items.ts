
import type * as THREE from 'three';
import type { Pose } from './poses_catalog';
import {voxelItem,localItemModel} from './poser_textures';
import {displayForHand,type LocalItemModel} from './item_models';

export const HELD_ITEMS = [
  {id:'none',name:'Ninguno'},
  {id:'diamond_sword',name:'Espada de diamante'},
  {id:'netherite_sword',name:'Espada de netherite'},
  {id:'iron_sword',name:'Espada de hierro'},
  {id:'wooden_sword',name:'Espada de madera'},
  {id:'diamond_pickaxe',name:'Pico de diamante'},
  {id:'diamond_axe',name:'Hacha de diamante'},
  {id:'netherite_axe',name:'Hacha de netherite'},
  {id:'mace',name:'Maza'},
  {id:'totem',name:'Tótem de la Inmortalidad'},
  {id:'shield',name:'Escudo'},
  {id:'trident',name:'Tridente'},
  {id:'bow',name:'Arco'},
  {id:'bow_pulling_2',name:'Arco tensado'},
  {id:'crossbow',name:'Ballesta'},
  {id:'crossbow_arrow',name:'Ballesta cargada'},
  {id:'arrow',name:'Flecha'},
  {id:'fishing_rod',name:'Caña de pescar'},
  {id:'enchanted_golden_apple',name:'Manzana dorada encantada'},
  {id:'torch',name:'Antorcha'},
  {id:'lantern',name:'Linterna'},
  {id:'ender_pearl',name:'Perla de Ender'},
  {id:'grass_block',name:'Bloque de césped'},
  {id:'stone_block',name:'Bloque de piedra'},
  {id:'cobblestone_block',name:'Bloque de adoquín'},
  {id:'oak_planks_block',name:'Tablones de roble'},
  {id:'diamond_block',name:'Bloque de diamante'},
  {id:'tnt_block',name:'TNT'},
  {id:'command_block',name:'Bloque de comandos · Raro'},
] as const;
export type HeldItemId = typeof HELD_ITEMS[number]['id'];
export type CharacterVariant = 'none'|'herobrine'|'entity303'|'null'|'crowned_steve'|'dinnerbone'|'grumm'|'charged_creeper'|'enderman';
export type EquippedItems = {right:HeldItemId;left:HeldItemId};
export type ItemDisplayTransform = {rotation:[number,number,number];translation:[number,number,number];scale:[number,number,number]};
export type ItemDisplays = Record<string,{right:ItemDisplayTransform;left:ItemDisplayTransform}>;

type EquipmentPose = Pick<Pose,'name'|'category'> & {equipment?:EquippedItems};

export function itemsForPose(pose: EquipmentPose):EquippedItems {


  if(pose.equipment)return{...pose.equipment};
  const name=pose.name.toLocaleLowerCase();
  if(/dos espadas/.test(name))return{right:'diamond_sword',left:'netherite_sword'};
  if(/escudo|bloqueo/.test(name))return{right:'diamond_sword',left:'shield'};
  if(/ballesta/.test(name))return{right:'crossbow_arrow',left:'none'};
  if(/arco/.test(name))return{right:'bow_pulling_2',left:'none'};
  if(/hacha/.test(name))return{right:'diamond_axe',left:'none'};
  if(/maza/.test(name))return{right:'mace',left:'none'};
  if(/pesca|pescando/.test(name))return{right:'fishing_rod',left:'none'};
  if(/tridente/.test(name))return{right:'trident',left:'none'};
  if(/picando|minero|mina|pico/.test(name))return{right:'diamond_pickaxe',left:'torch'};
  if(/antorcha|explorador|cueva/.test(name))return{right:'torch',left:'none'};
  if(/bloque|constru|colocando/.test(name))return{right:'grass_block',left:'none'};
  if(/perla|ender/.test(name))return{right:'ender_pearl',left:'none'};
  if(/espada|crítico|guardia|espadazo|golpe final/.test(name))return{right:'diamond_sword',left:'none'};
  if(/héroe|invencible|victoria pvp/.test(name))return{right:'netherite_sword',left:'totem'};
  return{right:'none',left:'none'};
}



export function equipmentForWelcome(pose:EquipmentPose,sample=.5):EquippedItems {
  const base=itemsForPose(pose);
  if(pose.equipment)return base;
  const fraction=Number.isFinite(sample)?Math.max(0,Math.min(.999999999,sample)):.5;
  const pick=(items:readonly HeldItemId[])=>items[Math.floor(fraction*items.length)];
  if(base.right==='diamond_sword'||base.right==='netherite_sword')return{...base,right:pick(['diamond_sword','netherite_sword','iron_sword','wooden_sword','diamond_axe','netherite_axe','mace'])};
  if(base.right==='grass_block')return{...base,right:pick(['grass_block','stone_block','cobblestone_block','oak_planks_block','diamond_block','tnt_block'])};
  if(base.right==='bow_pulling_2')return{...base,right:pick(['bow_pulling_2','crossbow_arrow'])};
  return base;
}

type ThreeApi=typeof import('three');
function box(three:ThreeApi,root:THREE.Group,color:string,size:[number,number,number],position:[number,number,number],emissive=false):THREE.Mesh{
  const material=new three.MeshStandardMaterial({color,roughness:.85,emissive:emissive?color:'#000000',emissiveIntensity:emissive?.65:0});
  const mesh=new three.Mesh(new three.BoxGeometry(...size),material);mesh.position.set(...position);mesh.castShadow=true;mesh.receiveShadow=true;root.add(mesh);return mesh;
}

export function createHeldItem(three:ThreeApi,id:HeldItemId):THREE.Group {
  const visual=voxelItem(three,id);if(visual)return visual;
  const empty=new three.Group();empty.name=`unavailable-${id}`;return empty;
}

export function mountHeldModel(three:ThreeApi,arm:THREE.Object3D,id:HeldItemId,hand:'right'|'left',slim:boolean,visual:THREE.Group,model:LocalItemModel):THREE.Group{
  const mount=new three.Group();mount.name=`hand-${hand}`;mount.userData.item=id;
  const sign=hand==='left'?-1:1;



  mount.position.set(-sign*(slim?.5:1),-10,2);mount.rotation.set(Math.PI/2,Math.PI,0);
  const display=displayForHand(model,hand),item=new three.Group();item.name='thirdperson-display';
  item.position.set(...display.translation);item.rotation.set(...display.rotation.map(v=>v*Math.PI/180) as [number,number,number]);item.scale.set(...display.scale);item.add(visual);item.updateMatrix();mount.updateMatrix();
  if(model.kind==='shield'||model.kind==='trident'){



    const grip=new three.Vector3(...visual.userData.grip as [number,number,number]).applyMatrix4(item.matrix);
    const palm=new three.Vector3(-sign*(slim?.5:1),-8.5,0).applyMatrix4(mount.matrix.clone().invert());
    item.position.add(palm.sub(grip));
  }
  mount.add(item);arm.add(mount);return mount;
}
export function mountHeldItem(three:ThreeApi,arm:THREE.Object3D,id:HeldItemId,hand:'right'|'left',slim=false):THREE.Group{
  const model=localItemModel(id),visual=model&&voxelItem(three,id);
  if(!model||!visual){const empty=new three.Group();empty.name=`unavailable-${id}`;return empty;}
  return mountHeldModel(three,arm,id,hand,slim,visual,model);
}
export function disposeModel(root:THREE.Object3D){
  root.removeFromParent();const geometries=new Set<THREE.BufferGeometry>(),materials=new Set<THREE.Material>();
  root.traverse(object=>{if((object as THREE.InstancedMesh).isInstancedMesh)(object as THREE.InstancedMesh).dispose();if('geometry'in object)geometries.add((object as THREE.Mesh).geometry);if('material'in object){const material=(object as THREE.Mesh).material;for(const entry of Array.isArray(material)?material:[material])materials.add(entry);}});
  const textures=new Set<THREE.Texture>();for(const material of materials){const mapped=material as THREE.MeshStandardMaterial;if(mapped.map)textures.add(mapped.map);}for(const texture of textures)texture.dispose();
  for(const geometry of geometries)geometry.dispose();for(const material of materials)material.dispose();
}
export function createCharacterAccessory(three:ThreeApi,variant:CharacterVariant):THREE.Group {
  if(variant==='enderman'||variant==='charged_creeper'){

    const sentry=new three.Group();const crystal=new three.Mesh(new three.OctahedronGeometry(6),new three.MeshStandardMaterial({color:'#A855F7',emissive:'#A855F7',emissiveIntensity:.35}));
    crystal.position.y=-3;sentry.add(crystal);
    for(const x of[-9,9]){box(three,sentry,'#27273A',[3,12,3],[x,-3,0]);box(three,sentry,'#D946EF',[3.2,.7,3.2],[x,-1,0],true);}
    return sentry;
  }
  const root=new three.Group();root.name=`variant-${variant}`;

  if(variant==='herobrine')return root;
  if(variant==='crowned_steve'){
    for(const z of[-4.15,4.15])box(three,root,'#dfad32',[8.7,1.4,.6],[0,8.8,z]);for(const x of[-4.15,4.15])box(three,root,'#dfad32',[.6,1.4,8.7],[x,8.8,0]);
    for(const x of[-3,0,3])for(const z of[-4.15,4.15])box(three,root,'#f3d263',[1.3,1.8,.65],[x,10.1,z]);
    for(const x of[-4.15,4.15])for(const z of[-3,0,3])box(three,root,'#f3d263',[.65,1.8,1.3],[x,10.1,z]);
    box(three,root,'#cf345c',[1.3,1,.25],[0,8.8,4.55],true);for(const x of[-2.9,2.9])box(three,root,'#42c5c7',[.8,.8,.25],[x,8.8,4.55]);
  }else if(variant==='null'||variant==='entity303'){
    if(variant==='entity303'){
      box(three,root,'#e6e8e9',[8.7,8.7,1],[0,4,-4]);box(three,root,'#f2f2ee',[1.5,8.7,8.7],[-3.7,4,0]);box(three,root,'#f2f2ee',[1.5,8.7,8.7],[3.7,4,0]);box(three,root,'#eff0ec',[8.7,1.3,8.7],[0,7.8,0]);box(three,root,'#14151b',[5.8,6,.5],[0,3.4,4.35]);
    }
    const eyeColor=variant==='entity303'?'#ff253f':'#ffffff';
    for(const x of[-2,2])box(three,root,eyeColor,[1.8,.8,.16],[x,4.2,variant==='entity303'?4.7:4.12],true);
  }
  return root;
}
