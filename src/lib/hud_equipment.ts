export type EquipmentItem={name:string;remaining:number;maximum:number;item_id?:string;slot?:string};
export type EquipmentKind='head'|'chest'|'legs'|'feet'|'sword'|'pickaxe'|'axe'|'shovel'|'bow'|'shield'|'tool';
export function equipmentKind(item:EquipmentItem):EquipmentKind{
 const id=(item.item_id??'').split(':').pop()??'';
 if(item.slot==='head'||/helmet|skull|head$/.test(id))return 'head';
 if(item.slot==='chest'||/chestplate|elytra/.test(id))return 'chest';
 if(item.slot==='legs'||/leggings/.test(id))return 'legs';
 if(item.slot==='feet'||/boots/.test(id))return 'feet';
 if(/sword|trident|mace/.test(id))return 'sword';
 if(/pickaxe/.test(id))return 'pickaxe';
 if(/axe/.test(id))return 'axe';
 if(/shovel|spade/.test(id))return 'shovel';
 if(/bow/.test(id))return 'bow';
 if(/shield/.test(id))return 'shield';
 return item.slot==='armor'?'chest':'tool';
}

export function durabilityPercent(item:EquipmentItem):number|null{
 if(!Number.isFinite(item.maximum)||item.maximum<=0||!Number.isFinite(item.remaining))return null;
 return Math.ceil(Math.max(0,Math.min(1,item.remaining/item.maximum))*100);
}
export function equipmentRows(items:EquipmentItem[]):EquipmentItem[]{
 const order:Record<string,number>={head:0,chest:1,legs:2,feet:3};
 return items.map((item,index)=>({item,index,order:order[equipmentKind(item)]??(item.slot==='offhand'?6:item.slot==='mainhand'?5:4)}))
  .sort((a,b)=>a.order-b.order||a.index-b.index).map(row=>row.item);
}
export const PREVIEW_EQUIPMENT:EquipmentItem[]=[
 {name:'Casco',item_id:'minecraft:diamond_helmet',slot:'head',remaining:330,maximum:363},
 {name:'Pechera',item_id:'minecraft:diamond_chestplate',slot:'chest',remaining:480,maximum:528},
 {name:'Grebas',item_id:'minecraft:diamond_leggings',slot:'legs',remaining:450,maximum:495},
 {name:'Botas',item_id:'minecraft:diamond_boots',slot:'feet',remaining:390,maximum:429},
 {name:'Espada',item_id:'minecraft:diamond_sword',slot:'mainhand',remaining:1405,maximum:1561}
];
