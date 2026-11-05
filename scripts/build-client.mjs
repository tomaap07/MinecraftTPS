import {existsSync,readdirSync,readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve,dirname} from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=resolve(import.meta.dirname,'..'),folder=resolve(root,'tp-client');
const windows=process.platform==='win32',binary=windows?'javac.exe':'javac';
const candidates=[process.env.JAVA_HOME,process.env.JDK_HOME];
const cache=resolve(root,'.codex-tmp','jdk21');
if(existsSync(cache))for(const entry of readdirSync(cache,{withFileTypes:true}))if(entry.isDirectory())candidates.push(resolve(cache,entry.name));
const found=spawnSync(windows?'where':'which',[binary],{encoding:'utf8',windowsHide:true});
if(found.status===0)for(const line of found.stdout.trim().split(/\r?\n/))candidates.push(dirname(dirname(line)));
const javaHome=candidates.find(path=>{
  if(!path||!existsSync(resolve(path,'bin',binary)))return false;
  const version=spawnSync(resolve(path,'bin',binary),['-version'],{encoding:'utf8',windowsHide:true});
  return version.status===0&&Number((version.stdout+version.stderr).match(/javac (\d+)/)?.[1])===21;
});
if(!javaHome){console.error('TP Client 1.21.1 requiere JDK 21 con javac. Configura JAVA_HOME.');process.exit(1);}
const result=spawnSync(windows?'cmd.exe':'./gradlew',windows?['/d','/c','gradlew.bat','--no-daemon','build']:['--no-daemon','build'],{cwd:folder,stdio:'inherit',windowsHide:true,env:{...process.env,JAVA_HOME:javaHome}});
if(result.error)console.error(result.error.message);
if(result.status!==0)process.exit(result.status??1);
const jar=resolve(folder,'build/libs/tp-client-0.1.0.jar');
if(!existsSync(jar))throw new Error('Gradle no produjo el JAR esperado');
const sha256=createHash('sha256').update(readFileSync(jar)).digest('hex');
const manifest={schema_version:1,version:'0.1.0',minecraft:'1.21.1',loader:'fabric',java:21,sha256};
mkdirSync(resolve(folder,'build'),{recursive:true});
writeFileSync(resolve(folder,'build/tp-client-release.json'),JSON.stringify(manifest,null,2)+'\n');
writeFileSync(resolve(root,'src-tauri/src/tp_client_release.json'),JSON.stringify(manifest,null,2)+'\n');
console.log(`TP Client verified: ${jar}\nSHA-256 ${sha256}`);
