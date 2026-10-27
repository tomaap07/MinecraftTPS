import { invoke } from '@tauri-apps/api/core';
import { addressKey } from './server_directory';

export type MotdSpan = {text:string;color:string;bold:boolean;italic:boolean;underlined:boolean;strikethrough:boolean};
export type LiveServerStatus = {
  address:string;online:boolean;latency_ms:number;players_online:number;players_max:number;
  version:string;protocol:number;motd:MotdSpan[];icon:string;error:string;checked_at:number;
  last_successful_at?:number;
  measurement_available?:boolean;refresh_error?:string;refresh_failed_at?:number;
};
type Target = {id:string;address:string;java_compatible?:boolean};
export type LiveServerSnapshot = {statuses:Record<string,LiveServerStatus>;pinging:Record<string,boolean>;enabled:boolean};
export const SERVER_REFRESH_MS = 15_000;


export function createLiveServerMonitor(publish:(snapshot:LiveServerSnapshot)=>void) {
  let targets:Target[]=[];
  const cache=new Map<string,LiveServerStatus>();
  const queue=new Set<string>();
  const inFlight=new Set<string>();
  let running=0,started=false,disposed=false,enabled=false;
  let timer:ReturnType<typeof setInterval>|undefined;
  const available=()=>started&&!disposed&&document.visibilityState!=='hidden'&&document.hasFocus()&&navigator.onLine!==false;
  const selected=(key:string)=>targets.find(target=>addressKey(target.address)===key);
  const emit=()=>{
    if(disposed)return;
    const statuses:Record<string,LiveServerStatus>={},pinging:Record<string,boolean>={};
    for(const target of targets){
      const key=addressKey(target.address),status=cache.get(key);
      if(status)statuses[target.id]=status;
      if(inFlight.has(key)||queue.has(key))pinging[target.id]=true;
    }
    publish({statuses,pinging,enabled});
  };
  function keep(key:string,status:LiveServerStatus){
    cache.delete(key);cache.set(key,status);
    while(cache.size>64)cache.delete(cache.keys().next().value!);
  }
  async function query(key:string,address:string){
    try{
      const result=await invoke<LiveServerStatus>('server_ping',{address});
      if(disposed)return;
      const previous=cache.get(key);
      keep(key,{...result,checked_at:result.checked_at||Date.now(),measurement_available:true,

        icon:result.online?result.icon:result.icon||previous?.icon||'',
        motd:result.online?(result.motd??[]):result.motd?.length?result.motd:previous?.motd??[],
        last_successful_at:result.online?(result.checked_at||Date.now()):previous?.last_successful_at});
    }catch{
      if(disposed)return;
      const previous=cache.get(key);

      const retry={refresh_error:'No se pudo actualizar el estado en este momento.',refresh_failed_at:Date.now()};
      keep(key,previous?{...previous,...retry}:{address,online:false,latency_ms:0,players_online:0,players_max:0,version:'',
        protocol:-1,motd:[],icon:'',error:'',checked_at:0,measurement_available:false,...retry});
    }finally{
      running--;inFlight.delete(key);
      if(!disposed){emit();pump();}
    }
  }
  function pump(){
    if(!enabled||disposed)return;
    for(const key of queue){
      if(running>=4)break;
      queue.delete(key);
      const target=selected(key);
      if(!target||inFlight.has(key))continue;
      inFlight.add(key);running++;
      void query(key,target.address);
    }
  }
  function refresh(address?:string){
    if(!enabled||disposed)return;
    for(const target of targets){
      const key=addressKey(target.address);
      if((!address||key===addressKey(address))&&!inFlight.has(key))queue.add(key);
    }
    pump();emit();
  }
  function visibilityChanged(){
    const wasEnabled=enabled;enabled=available();
    if(!enabled)queue.clear();
    if(enabled&&!wasEnabled)refresh();
    else emit();
  }
  return {
    setTargets(next:readonly Target[]){
      targets=next.filter(target=>target.java_compatible!==false);
      queue.clear();

      if(enabled)for(const target of targets){const key=addressKey(target.address),cached=cache.get(key);
        if(!inFlight.has(key)&&(!cached||Date.now()-cached.checked_at>=10_000))queue.add(key);
      }
      pump();emit();
    },
    refresh,
    start(){
      if(started||disposed)return;
      started=true;
      document.addEventListener('visibilitychange',visibilityChanged);
      window.addEventListener('online',visibilityChanged);
      window.addEventListener('offline',visibilityChanged);
      window.addEventListener('focus',visibilityChanged);
      window.addEventListener('blur',visibilityChanged);
      visibilityChanged();
      timer=setInterval(()=>{if(enabled)refresh();},SERVER_REFRESH_MS);
    },
    stop(){
      disposed=true;enabled=false;queue.clear();cache.clear();clearInterval(timer);
      if(started){document.removeEventListener('visibilitychange',visibilityChanged);window.removeEventListener('online',visibilityChanged);
        window.removeEventListener('offline',visibilityChanged);window.removeEventListener('focus',visibilityChanged);window.removeEventListener('blur',visibilityChanged);}
    },
  };
}
