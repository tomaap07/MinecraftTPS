import {randomUUID} from 'node:crypto';
import {rpc} from './db.js';
import {parseAddress,pingMinecraft} from './minecraft.js';
import type {ServerRow} from './types.js';

export async function syncBatch(options:{batch?:number;concurrency?:number;budgetMs?:number}={}){
  const bounded=(value:number,fallback:number,max:number)=>Number.isFinite(value)?Math.max(1,Math.min(max,Math.floor(value))):fallback;
  const batch=bounded(options.batch??Number(process.env.TP_WORKER_BATCH_SIZE??96),96,96);
  const concurrency=bounded(options.concurrency??Number(process.env.TP_WORKER_CONCURRENCY??8),8,12);

  const lease=randomUUID(),started=Date.now(),budget=options.budgetMs??30_000;
  const entries=await rpc<ServerRow[]>('tp_claim_servers',{p_limit:batch,p_lease:lease});
  let next=0,updated=0,online=0,failed=0,deferred=0;
  await Promise.all(Array.from({length:Math.min(concurrency,entries.length)},async()=>{
    while(next<entries.length){
      if(Date.now()-started>budget){deferred=entries.length-next;break;}
      const entry=entries[next++];
      let state:unknown;
      try{state=await pingMinecraft(parseAddress(entry.ip,entry.port));online++;}
      catch{state={is_online:false};failed++;}
      const saved=await rpc<boolean>('tp_finish_server_ping',{p_id:entry.id,p_lease:lease,p_state:state});
      if(saved)updated++;
    }
  }));
  return {claimed:entries.length,updated,online,failed,deferred,elapsed_ms:Date.now()-started};
}
