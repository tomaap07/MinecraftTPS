import {existsSync, readdirSync, mkdirSync, writeFileSync, realpathSync, rmSync,readFileSync,copyFileSync} from 'node:fs';
import {resolve, dirname, sep} from 'node:path';
import {spawnSync} from 'node:child_process';
import {randomUUID,createHash} from 'node:crypto';
import {relocateClass} from './relocate-java-class.mjs';

const root=realpathSync(resolve(import.meta.dirname,'..'));
const windows=process.platform==='win32';
const binary=windows?'javac.exe':'javac';
const candidates=[process.env.JAVA_HOME,process.env.JDK_HOME];
const cache=resolve(root,'.codex-tmp/jdk21');
if(existsSync(cache))for(const item of readdirSync(cache,{withFileTypes:true}))if(item.isDirectory())candidates.push(resolve(cache,item.name));
const found=spawnSync(windows?'where':'which',[binary],{encoding:'utf8',windowsHide:true});
if(found.status===0)for(const line of found.stdout.trim().split(/\r?\n/))candidates.push(dirname(dirname(line)));
const javaHome=candidates.find(path=>{
 if(!path||!existsSync(resolve(path,'bin',binary)))return false;
 const version=spawnSync(resolve(path,'bin',binary),['-version'],{encoding:'utf8',windowsHide:true});
 return version.status===0&&Number((version.stdout+version.stderr).match(/javac (\d+)/)?.[1])>=9;
});
if(!javaHome)throw new Error('Para compilar TP Bridge configura JAVA_HOME con un JDK 9 o superior. El JAR generado funciona con Java 8 o superior.');
const build=resolve(root,'tp-bridge/build');mkdirSync(build,{recursive:true});
const classes=resolve(build,'classes-'+randomUUID());mkdirSync(classes);
const manifest=resolve(classes,'agent.mf');
function run(tool,args,cwd=root){const result=spawnSync(resolve(javaHome,'bin',tool+(windows?'.exe':'')),args,{cwd,stdio:'inherit',windowsHide:true});if(result.status!==0)throw result.error??new Error(`${tool} terminó con código ${result.status}`);}
try{
 const dependencies=resolve(root,'.codex-tmp/java-dependencies');mkdirSync(dependencies,{recursive:true});const asm=resolve(dependencies,'asm-9.8.jar'),expected='876eab6a83daecad5ca67eb9fcabb063c97b5aeb8cf1fca7a989ecde17522051';
 if(!existsSync(asm)||createHash('sha256').update(readFileSync(asm)).digest('hex')!==expected){const response=await fetch('https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.8/asm-9.8.jar');if(!response.ok)throw new Error('ASM download failed');const bytes=Buffer.from(await response.arrayBuffer());if(bytes.length>1000000||createHash('sha256').update(bytes).digest('hex')!==expected)throw new Error('ASM SHA-256 mismatch');writeFileSync(asm,bytes);}
 const runtimeClasses=resolve(classes,'runtime'),agentClasses=resolve(classes,'agent'),asmClasses=resolve(classes,'asm');for(const dir of[runtimeClasses,agentClasses,asmClasses])mkdirSync(dir);
 run('javac',['--release','8','-Xlint:-options','-encoding','UTF-8','-d',runtimeClasses,resolve(root,'tp-bridge/src/main/java/dev/tplauncher/hud/runtime/FrameCounter.java')]);
 mkdirSync(resolve(agentClasses,'bootstrap'));run('jar',[`-J-Djava.io.tmpdir=${classes}`,'--create','--file',resolve(agentClasses,'bootstrap/tp-hud-runtime.jar'),'-C',runtimeClasses,'dev']);
 run('javac',['--release','8','-Xlint:-options','-encoding','UTF-8','-cp',asm,'-d',agentClasses,...['TpBridge','RenderHook','WorldTime'].map(name=>resolve(root,`tp-bridge/src/main/java/dev/tplauncher/bridge/${name}.java`))]);
 run('jar',['--extract','--file',asm],asmClasses);
 for(const name of readdirSync(agentClasses,{recursive:true}).filter(name=>name.endsWith('.class'))){const path=resolve(agentClasses,name);writeFileSync(path,relocateClass(readFileSync(path)));}
 for(const name of readdirSync(resolve(asmClasses,'org/objectweb/asm'),{recursive:true}).filter(name=>name.endsWith('.class'))){const path=resolve(agentClasses,'dev/tplauncher/shaded/asm',name);mkdirSync(dirname(path),{recursive:true});writeFileSync(path,relocateClass(readFileSync(resolve(asmClasses,'org/objectweb/asm',name))));}
 copyFileSync(resolve(root,'tp-bridge/ASM-LICENSE.txt'),resolve(agentClasses,'ASM-LICENSE.txt'));
 writeFileSync(manifest,'Manifest-Version: 1.0\nPremain-Class: dev.tplauncher.bridge.TpBridge\nCan-Redefine-Classes: false\nCan-Retransform-Classes: true\n\n');
 const artifact=resolve(build,'tp-bridge-0.1.0.jar');
 run('jar',[`-J-Djava.io.tmpdir=${classes}`,'--create','--file',artifact,'--manifest',manifest,'-C',agentClasses,'.','-C',root,'LICENSE']);
 console.log('TP Bridge: Java 8+, ASM aislado y contador de presentaciones LWJGL 2/3; sin código o recursos de Minecraft.');
}finally{

 const actual=realpathSync(classes),parent=realpathSync(build);
 if(!actual.startsWith(parent+sep))throw new Error('Temporary build path escaped its workspace.');
 rmSync(actual,{recursive:true,force:true});
}
