import {existsSync,readdirSync} from 'node:fs';
import {resolve,dirname} from 'node:path';
import {spawnSync} from 'node:child_process';

const root=resolve(import.meta.dirname,'..'),folder=resolve(root,'tp-companion');
const windows=process.platform==='win32',binary=windows?'javac.exe':'javac';
const candidates=[process.env.JAVA_HOME,process.env.JDK_HOME];

const cache=resolve(root,'.codex-tmp','jdk21');
if(existsSync(cache))for(const entry of readdirSync(cache,{withFileTypes:true}))if(entry.isDirectory())candidates.push(resolve(cache,entry.name));
const found=spawnSync(windows?'where':'which',[binary],{encoding:'utf8',windowsHide:true});
if(found.status===0)for(const line of found.stdout.trim().split(/\r?\n/))candidates.push(dirname(dirname(line)));
const javaHome=candidates.find(path=>{
  if(!path||!existsSync(resolve(path,'bin',binary)))return false;
  const version=spawnSync(resolve(path,'bin',binary),['-version'],{encoding:'utf8',windowsHide:true});
  const major=Number((version.stdout+version.stderr).match(/javac (\d+)/)?.[1]);
  return version.status===0&&major>=21;
});
if(!javaHome){console.error('Para empaquetar TP Companion se necesita JDK 21 o superior (con javac). Configura JAVA_HOME y vuelve a ejecutar npm run build:desktop.');process.exit(1);}
const result=spawnSync(windows?'cmd.exe':'./gradlew',windows?['/d','/c','gradlew.bat','--no-daemon','build']:['--no-daemon','build'],{
  cwd:folder,stdio:'inherit',windowsHide:true,env:{...process.env,JAVA_HOME:javaHome}
});
if(result.error)console.error(result.error.message);
process.exit(result.status??1);
