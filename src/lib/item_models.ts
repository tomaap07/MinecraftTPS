import type * as THREE from 'three';
export type ItemVector=[number,number,number];
export type ItemDisplayTransform={rotation:ItemVector;translation:ItemVector;scale:ItemVector};
export type ItemFace={texture:string;uv:[number,number,number,number];rotation?:number;tintindex?:number};
export type ItemElement={from:ItemVector;to:ItemVector;faces:Record<string,ItemFace>;rotation?:{origin:ItemVector;axis:'x'|'y'|'z';angle:number;rescale:boolean}};
export type LocalItemModel={kind:'sprite'|'model'|'shield'|'trident';textures:Record<string,string>;elements:ItemElement[];display:{right:ItemDisplayTransform;left:ItemDisplayTransform};source_model:string;tint_color?:string};
export type ItemPixels={width:number;height:number;data:Uint8ClampedArray};
export type ItemImages=ReadonlyMap<string,ItemPixels>;
type Three=typeof import('three');
const FACE_NAMES=['north','south','east','west','up','down'] as const;
function texture(three:Three,image:ItemPixels):THREE.DataTexture{
 const result=new three.DataTexture(new Uint8Array(image.data),image.width,image.height,three.RGBAFormat);
 result.colorSpace=three.SRGBColorSpace;result.flipY=true;result.magFilter=result.minFilter=three.NearestFilter;result.generateMipmaps=false;result.needsUpdate=true;return result;
}
function corners(direction:string,a:ItemVector,b:ItemVector):number[][]{
 const[x,y,z]=a,[X,Y,Z]=b;
 switch(direction){
  case'north':return[[X,Y,z],[X,y,z],[x,y,z],[x,Y,z]];
  case'south':return[[x,Y,Z],[x,y,Z],[X,y,Z],[X,Y,Z]];
  case'east':return[[X,Y,Z],[X,y,Z],[X,y,z],[X,Y,z]];
  case'west':return[[x,Y,z],[x,y,z],[x,y,Z],[x,Y,Z]];
  case'up':return[[x,Y,z],[x,Y,Z],[X,Y,Z],[X,Y,z]];
  default:return[[x,y,Z],[x,y,z],[X,y,z],[X,y,Z]];
 }
}
function faceGeometry(three:Three,element:ItemElement,direction:string,face:ItemFace,uvSize:[number,number]=[16,16]):THREE.BufferGeometry{
 const position=corners(direction,element.from,element.to).flatMap(p=>p.map(v=>v-8));
 const[u,v,U,V]=face.uv,[w,h]=uvSize;const uv=[[u/w,1-v/h],[u/w,1-V/h],[U/w,1-V/h],[U/w,1-v/h]];
 const angle=face.rotation??0;const offset=[0,90,180,270].includes(angle)?angle/90:0;
 const geometry=new three.BufferGeometry();geometry.setAttribute('position',new three.Float32BufferAttribute(position,3));geometry.setAttribute('uv',new three.Float32BufferAttribute(uv.flatMap((_,i)=>uv[(i+offset)%4]),2));geometry.setIndex([0,1,2,0,2,3]);geometry.computeVertexNormals();return geometry;
}
function material(three:Three,map:THREE.Texture,tint?:string,overlay=false):THREE.MeshStandardMaterial{
 return new three.MeshStandardMaterial({map,color:tint??'#ffffff',roughness:.8,alphaTest:.5,transparent:false,side:three.DoubleSide,depthWrite:true,polygonOffset:overlay,polygonOffsetFactor:overlay?-1:0,polygonOffsetUnits:overlay?-1:0});
}
function elementsModel(three:Three,root:THREE.Group,model:LocalItemModel,images:ItemImages):void{
 const textures=new Map<string,THREE.Texture>();for(const[key,image]of images)textures.set(key,texture(three,image));const used=new Set<THREE.Texture>();
 model.elements.forEach((element,index)=>{
  const group=new three.Group();group.name=`element-${index}`;
  for(const direction of FACE_NAMES){const face=element.faces[direction];if(!face)continue;const map=textures.get(face.texture.replace(/^#/,''));if(!map)continue;used.add(map);
   const mesh=new three.Mesh(faceGeometry(three,element,direction,face),material(three,map,face.tintindex!=null?model.tint_color:undefined,index>0));mesh.castShadow=mesh.receiveShadow=true;mesh.name=direction;group.add(mesh);
  }
  if(element.rotation){const{origin,axis,angle,rescale}=element.rotation;const pivot=new three.Vector3(...origin).subScalar(8);group.position.copy(pivot);for(const child of group.children)child.position.sub(pivot);group.rotation[axis]=angle*Math.PI/180;
   if(rescale){const factor=1/Math.max(.1,Math.abs(Math.cos(angle*Math.PI/180)));for(const other of ['x','y','z'] as const)if(other!==axis)group.scale[other]=factor;}
  }if(group.children.length)root.add(group);
 });
 for(const map of textures.values())if(!used.has(map))map.dispose();
}
function spriteModel(three:Three,root:THREE.Group,images:ItemImages):void{


 for(const[key,image]of images){if(!/^layer\d+$/.test(key))continue;
  const frame=Math.min(image.width,image.height),resolution=Math.min(frame,64),pixel=16/resolution;type Pixel={x:number;y:number;color:THREE.Color};const pixels:Pixel[]=[];
  for(let y=0;y<resolution;y++)for(let x=0;x<resolution;x++){
   const px=Math.min(frame-1,Math.floor((x+.5)*frame/resolution)),py=Math.min(frame-1,Math.floor((y+.5)*frame/resolution));const i=(py*image.width+px)*4;if(image.data[i+3]<128)continue;
   pixels.push({x,y,color:new three.Color().setRGB(image.data[i]/255,image.data[i+1]/255,image.data[i+2]/255,three.SRGBColorSpace)});
  }
  if(!pixels.length)continue;const geometry=new three.BoxGeometry(pixel,pixel,1),mat=new three.MeshStandardMaterial({color:'#ffffff',roughness:.8});const mesh=new three.InstancedMesh(geometry,mat,pixels.length),matrix=new three.Matrix4();mesh.name=`local-${key}`;
  pixels.forEach((p,i)=>{matrix.makeTranslation((p.x+.5)*pixel-8,8-(p.y+.5)*pixel,Number(key.slice(5))*.02);mesh.setMatrixAt(i,matrix);mesh.setColorAt(i,p.color);});mesh.castShadow=mesh.receiveShadow=true;mesh.computeBoundingSphere();root.add(mesh);
 }
}


function entityBox(three:Three,root:THREE.Group,map:THREE.Texture,from:ItemVector,size:ItemVector,u:number,v:number,textureHeight:number):void{
 const[w,h,d]=size,[x,y,z]=from;const element:ItemElement={from:[x+8,-y-h+8,-z-d+8],to:[x+w+8,-y+8,-z+8],faces:{}};
 const faces:Record<string,[number,number,number,number]>={up:[u+d,v,u+d+w,v+d],down:[u+d+w,v,u+d+w*2,v+d],east:[u+w+d,v+d,u+w+d*2,v+d+h],west:[u,v+d,u+d,v+d+h],south:[u+d,v+d,u+d+w,v+d+h],north:[u+w+d*2,v+d,u+w*2+d*2,v+d+h]};
 for(const direction of FACE_NAMES){const mesh=new three.Mesh(faceGeometry(three,element,direction,{texture:'entity',uv:faces[direction]},[64,textureHeight]),material(three,map));mesh.castShadow=mesh.receiveShadow=true;root.add(mesh);}
}
function entityModel(three:Three,root:THREE.Group,model:LocalItemModel,images:ItemImages):void{
 const image=images.get('entity');if(!image)return;const map=texture(three,image);
 if(model.kind==='shield'){
  entityBox(three,root,map,[-6,-11,-2],[12,22,1],0,0,64);entityBox(three,root,map,[-1,-3,-1],[2,6,6],26,0,64);root.userData.grip=[0,0,-2];
 }else{
  entityBox(three,root,map,[-.5,2,-.5],[1,25,1],0,6,32);entityBox(three,root,map,[-1.5,0,-.5],[3,2,1],4,0,32);
  entityBox(three,root,map,[-2.5,-3,-.5],[1,4,1],4,3,32);entityBox(three,root,map,[-.5,-4,-.5],[1,4,1],0,0,32);entityBox(three,root,map,[1.5,-3,-.5],[1,4,1],4,3,32);root.userData.grip=[0,-12,0];
 }
}
export function buildLocalItemModel(three:Three,model:LocalItemModel,images:ItemImages):THREE.Group|null{
 const root=new three.Group();root.name=`local-item-${model.source_model}`;root.userData.localArtwork=true;root.userData.kind=model.kind;
 if(model.kind==='sprite')spriteModel(three,root,images);else if(model.kind==='model')elementsModel(three,root,model,images);else entityModel(three,root,model,images);
 return root.children.length?root:null;
}
export function displayForHand(model:LocalItemModel,hand:'right'|'left'):ItemDisplayTransform{
 const source=model.display[hand]??model.display.right,sign=hand==='left'?-1:1;
 return{rotation:[source.rotation[0],source.rotation[1]*sign,source.rotation[2]*sign],translation:[source.translation[0]*sign,source.translation[1],source.translation[2]],scale:[...source.scale]};
}
