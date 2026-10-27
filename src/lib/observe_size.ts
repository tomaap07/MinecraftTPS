export function observeSize(node:HTMLElement,change:(width:number,height:number)=>void,mode:'client'|'offset'='client'):()=>void {
  let frame=0,alive=true,lastWidth=-1,lastHeight=-1;
  const schedule=()=>{
    if(frame||!alive)return;
    frame=requestAnimationFrame(()=>{
      frame=0;if(!alive)return;
      const width=mode==='offset'?node.offsetWidth:node.clientWidth,height=mode==='offset'?node.offsetHeight:node.clientHeight;
      if(width<=0||height<=0||(width===lastWidth&&height===lastHeight))return;
      lastWidth=width;lastHeight=height;change(width,height);
    });
  };
  const observer=new ResizeObserver(schedule);observer.observe(node);schedule();
  return()=>{alive=false;observer.disconnect();if(frame)cancelAnimationFrame(frame);};
}
