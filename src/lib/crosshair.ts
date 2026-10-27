export type CrosshairShape='dot'|'cross'|'circle'|'diamond'|'chevron'|'cross-dot'|'pixels';
export type CrosshairSettings={enabled:boolean;shape:CrosshairShape;grid:15|17;pixels:boolean[];size:number;thickness:number;gap:number;dot:number;outline:number;opacity:number;color:string;chroma:boolean;dynamic:boolean;target_color:boolean;hostile_color:string;passive_color:string;range_color:string;cooldown:'off'|'ring'|'bar';hitmarker:boolean;hit_color:string;hit_duration:number};
export const CROSSHAIR_SHAPES:{id:CrosshairShape;name:string}[]=[{id:'dot',name:'Punto'},{id:'cross',name:'Cruz competitiva'},{id:'circle',name:'Círculo'},{id:'diamond',name:'Rombo'},{id:'chevron',name:'Arquero'},{id:'cross-dot',name:'Cruz y punto'},{id:'pixels',name:'Mi diseño pixel-art'}];
export function defaultCrosshair():CrosshairSettings{return{enabled:false,shape:'cross',grid:15,pixels:Array(225).fill(false),size:6,thickness:1.5,gap:3,dot:1.5,outline:1,opacity:1,color:'#FFFFFF',chroma:false,dynamic:false,target_color:false,hostile_color:'#EF4444',passive_color:'#10B981',range_color:'#F59E0B',cooldown:'off',hitmarker:false,hit_color:'#D946EF',hit_duration:180};}

export function crosshairPath(c:CrosshairSettings,spread=0):string{
 const g=c.gap+spread,l=c.size,r=g+l;
 switch(c.shape){
 case 'dot':return'';
 case 'circle':return `M ${-r} 0 a ${r} ${r} 0 1 0 ${r*2} 0 a ${r} ${r} 0 1 0 ${-r*2} 0`;
 case 'diamond':return `M 0 ${-r} L ${r} 0 0 ${r} ${-r} 0 Z`;
 case 'chevron':return `M ${-r} ${l/2} L 0 ${-l/2} ${r} ${l/2}`;
 case 'pixels':return'';
 default:return `M ${-r} 0 H ${-g} M ${g} 0 H ${r} M 0 ${-r} V ${-g} M 0 ${g} V ${r}`;
 }
}
