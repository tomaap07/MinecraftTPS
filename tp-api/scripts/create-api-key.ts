import {createHash,randomBytes} from 'node:crypto';
import {mkdir,writeFile} from 'node:fs/promises';
import {join,resolve} from 'node:path';
import {database} from '../lib/db.js';
const args=process.argv.slice(2),value=(name:string)=>args.find(item=>item.startsWith(`${name}=`))?.slice(name.length+1);
async function main(){
  const owner=value('--owner')?.trim();if(!owner||owner.length>100)throw new Error('Indica --owner=nombre (máximo 100 caracteres).');
  const tier=value('--tier')??'free';if(!['free','partner','admin'].includes(tier))throw new Error('Tier no válido.');
  const limit=Number(value('--limit')??5000);if(!Number.isInteger(limit)||limit<1||limit>10000000)throw new Error('Límite diario no válido.');
  const key=`tp_${randomBytes(32).toString('base64url')}`,hash=createHash('sha256').update(key).digest('hex');
  const inserted=await database<{id:string}[]>('tp_api_keys?select=id',{method:'POST',headers:{Prefer:'return=representation'},body:JSON.stringify({owner_name:owner,api_key_hash:hash,tier,daily_limit:limit})});
  const folder=resolve('.local','api-keys');await mkdir(folder,{recursive:true});const path=join(folder,`${inserted[0].id}.txt`);
  await writeFile(path,key+'\n',{mode:0o600,flag:'wx'});console.log(`API Key creada. Valor guardado solo en: ${path}`);
}
void main().catch(error=>{console.error(error instanceof Error?error.message:'No se pudo crear la API Key.');process.exitCode=1;});
