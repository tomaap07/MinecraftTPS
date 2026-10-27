export type HudSize={width:number;height:number};
export type HudPosition={x:number;y:number};

const length=(value:number)=>Number.isFinite(value)&&value>0?value:0;
const normalized=(value:number)=>Number.isFinite(value)?Math.max(0,Math.min(1,value)):0;




export function visibleWidgetPosition(saved:HudPosition,viewport:HudSize,widget:HudSize,scale:number):HudPosition{
 const factor=Number.isFinite(scale)&&scale>0?scale:1;
 const fit=(coordinate:number,available:number,size:number)=>{
  const value=normalized(coordinate),extent=length(available);
  return extent?Math.min(value,Math.max(0,1-length(size)*factor/extent)):value;
 };
 return{x:fit(saved.x,viewport.width,widget.width),y:fit(saved.y,viewport.height,widget.height)};
}
