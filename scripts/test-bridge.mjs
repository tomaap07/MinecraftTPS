import assert from 'node:assert/strict';
import{createSocket}from'node:dgram';
import{randomBytes}from'node:crypto';
import {existsSync,readFileSync,readdirSync,mkdirSync,writeFileSync,realpathSync,rmSync} from 'node:fs';
import {resolve,sep} from 'node:path';
import {spawn,spawnSync} from 'node:child_process';
import {randomUUID} from 'node:crypto';
const root=realpathSync(resolve(import.meta.dirname,'..')),win=process.platform==='win32';
const cache=resolve(root,'.codex-tmp/jdk21');
const homes=[process.env.JAVA_HOME,process.env.JDK_HOME,...(existsSync(cache)?readdirSync(cache,{withFileTypes:true}).filter(v=>v.isDirectory()).map(v=>resolve(cache,v.name)):[])];
const home=homes.find(v=>v&&existsSync(resolve(v,'bin',win?'javac.exe':'javac')));
assert.ok(home,'Set JAVA_HOME to run the bridge verification.');
const runtime=process.env.TP_BRIDGE_JAVA||resolve(home,'bin',win?'java.exe':'java');assert.ok(existsSync(runtime),'TP_BRIDGE_JAVA does not exist.');
const build=resolve(root,'tp-bridge/build'),dir=resolve(build,'test-'+randomUUID());mkdirSync(dir,{recursive:true});
const artifact=resolve(build,'tp-bridge-0.1.0.jar');assert.ok(existsSync(artifact),'Run npm run build:bridge first.');
const javac=spawnSync(resolve(home,'bin',win?'javac.exe':'javac'),['--release','8','-Xlint:-options','-encoding','UTF-8','-cp',artifact,'-d',dir,resolve(root,'tp-bridge/src/test/java/dev/tplauncher/bridge/WorldTimeTest.java'),...['FakeMinecraft','ObfuscatedClient','LegacyClient','EquipmentClient','RenderOnlyClient','LegacyRenderOnlyClient'].map(name=>resolve(root,`tp-bridge/src/test/java/fixture/${name}.java`)),...['glfw/GLFW','opengl/Display','opengl/GL11'].map(name=>resolve(root,`tp-bridge/src/test/java/org/lwjgl/${name}.java`))],{encoding:'utf8',windowsHide:true});
assert.equal(javac.status,0,javac.stderr);
const escape=s=>s.replaceAll('\\','\\\\').replaceAll(':','\\:').replaceAll('=','\\=');
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
async function waitFor(test,label){const until=Date.now()+7000;let last;while(Date.now()<until){try{const result=test();if(result)return result;}catch(e){last=e;}await sleep(80);}throw new Error(label,{cause:last});}
const children=[];const packets=[];const token=randomBytes(32).toString('hex'),udp=createSocket('udp4');udp.on('message',message=>{try{const p=JSON.parse(message);if(p.token===token)packets.push(p.payload);}catch{}});await new Promise(r=>udp.bind(0,'127.0.0.1',r));
async function launch(type,bindings={},renderOnly=false){
 const work=resolve(dir,type);mkdirSync(work);const profile=resolve(work,'hud-profile.json'),phase=resolve(work,'phase.txt');
 writeFileSync(profile,JSON.stringify({native_overlay:true}));writeFileSync(phase,'world');
 const config=resolve(work,'bindings.properties');writeFileSync(config,Object.entries({profile,'client.classes':'fixture.'+type,'udp.port':String(udp.address().port),'udp.token':token,...bindings}).map(([k,v])=>`${escape(k)}=${escape(v)}\n`).join(''));
 const child=spawn(runtime,[`-javaagent:${artifact}=${config}`,'-cp',dir,'fixture.'+type,phase],{stdio:['ignore','ignore','pipe'],windowsHide:true});children.push(child);
 let errors='';child.stderr.on('data',chunk=>errors=(errors+chunk).slice(-4096));child.testErrors=()=>errors;
 const file=resolve(work,`bridge-${child.pid}.json`);const sample=()=>JSON.parse(readFileSync(file,'utf8'));
 const frame=await waitFor(()=>{const f=sample();return (renderOnly?f.game_fps>20:f.has_world&&f.capabilities.includes('coordinates'))&&f;},type+' did not connect: '+errors);
 assert.equal(frame.pid,child.pid);assert.ok(Date.now()-frame.timestamp_ms<3500);assert.equal(frame.source,'TP Bridge');
 return{child,frame,phase,profile,file,sample};
}
try{
 const nbt=spawnSync(runtime,['-cp',dir+(win?';':':')+artifact,'dev.tplauncher.bridge.WorldTimeTest',resolve(dir,'nbt-fixture')],{encoding:'utf8',windowsHide:true});assert.equal(nbt.status,0,nbt.stderr);console.log(nbt.stdout.trim());
 const named=await launch('FakeMinecraft');assert.equal(named.frame.game_fps,137);assert.equal(named.frame.x,-12.75);assert.equal(named.frame.y,64.5);assert.equal(named.frame.z,45.125);assert.equal(named.frame.ping,42);assert.equal(named.frame.armor[0].remaining,87);assert.ok(named.frame.armor[0].name.includes('漢字'));assert.equal(named.frame.direction,'west');
 assert.equal(named.frame.world_time,48000);assert.equal(named.frame.biome,'minecraft:plains');assert.deepEqual(named.frame.metrics,{});assert.equal(named.frame.world_metrics.block_light,7);assert.equal(named.frame.world_metrics.sky_light,15);assert.equal(named.frame.armor[0].item_id,'minecraft:diamond_helmet');
 await waitFor(()=>packets.some(p=>p.pid===named.child.pid&&p.world_time===48000),'Authenticated UDP telemetry did not arrive');
 const settings=()=>JSON.parse(readFileSync(resolve(named.file,'../settings.json'),'utf8'));await waitFor(()=>settings().pause===false,'Focus setting was not updated on the game executor');
 const commandId=randomUUID();writeFileSync(resolve(named.file,`../command-${named.child.pid}.properties`),`id=${commandId}\nadapter=bridge\naction=option\nname=fov\nvalue=90\n`);
 await waitFor(()=>JSON.parse(readFileSync(resolve(named.file,`../ack-${named.child.pid}.json`),'utf8')).id===commandId,'Option change was not acknowledged');assert.equal(settings().fov,90);
 writeFileSync(named.phase,'modern-options');await sleep(100);
 const modernId=randomUUID();writeFileSync(resolve(named.file,`../command-${named.child.pid}.properties`),`id=${modernId}\nadapter=bridge\naction=option\nname=fov\nvalue=105\n`);await waitFor(()=>settings().fov===105,'Modern integer OptionInstance was not updated');
 const audioId=randomUUID();writeFileSync(resolve(named.file,`../command-${named.child.pid}.properties`),`id=${audioId}\nadapter=bridge\naction=option\nname=volume\nvalue=0.35\n`);await waitFor(()=>settings().volume===0.35,'Modern enum-based audio option was not updated').catch(error=>{throw new Error(error.message+' '+readFileSync(resolve(named.file,`../ack-${named.child.pid}.json`),'utf8'),{cause:error});});
 const saveCount=settings().saved;writeFileSync(named.profile,JSON.stringify({native_overlay:true,modules:[{id:'optical_zoom',enabled:true}]}));
 writeFileSync(named.phase,'zoom-modern');await waitFor(()=>settings().fov===35,'Holding C did not apply reversible modern FOV zoom');assert.ok(named.sample().capabilities.includes('optical_zoom'));
 writeFileSync(named.phase,'zoom-modern-unfocused');await waitFor(()=>settings().fov===105,'Alt-Tab did not restore FOV while C remains held');
 writeFileSync(named.phase,'zoom-modern');await waitFor(()=>settings().fov===35,'C zoom did not resume after focus returned');writeFileSync(named.phase,'zoom-modern-screen');await waitFor(()=>settings().fov===105,'Opening a game screen did not restore FOV');
 writeFileSync(named.phase,'zoom-modern');await waitFor(()=>settings().fov===35,'C zoom did not resume after closing the screen');writeFileSync(named.phase,'modern-options');await waitFor(()=>settings().fov===105,'Releasing C did not restore FOV');
 writeFileSync(named.phase,'zoom-modern');await waitFor(()=>settings().fov===35,'A second zoom did not engage');writeFileSync(named.profile,JSON.stringify({native_overlay:true,modules:[{id:'optical_zoom',enabled:false}]}));await waitFor(()=>settings().fov===105,'Disabling zoom did not restore FOV');assert.equal(settings().saved,saveCount,'Transient zoom was written into saved game options');
 writeFileSync(named.phase,'world');writeFileSync(named.profile,JSON.stringify({native_overlay:true,modules:[{id:'optical_zoom',enabled:true}]}));await sleep(1100);writeFileSync(named.phase,'zoom');await waitFor(()=>settings().fov===30,'Legacy float FOV was not zoomed');writeFileSync(named.phase,'world');await waitFor(()=>settings().fov===90,'Legacy FOV was not restored');
 writeFileSync(named.profile,JSON.stringify({native_overlay:true,fog:{enabled:true,overworld:true,nether:false,end:false,water:false,lava:false,strength:1}}));await waitFor(()=>settings().fog>1000000,'Live fog wrapper was not configured');
 writeFileSync(named.phase,'local');await waitFor(()=>{const f=named.sample();return f.singleplayer&&f.ping===0;},'Singleplayer did not report Local / 0 ms');
 writeFileSync(named.phase,'restricted');await waitFor(()=>{const f=named.sample();return f.coordinates_restricted&&!f.capabilities.includes('coordinates')&&f.x===0;},'Reduced debug info leaked coordinates');
 writeFileSync(named.phase,'menu');await waitFor(()=>{const f=named.sample();return !f.has_world&&f.game_fps===137&&!f.capabilities.includes('coordinates');},'Main menu fabricated a world');
 writeFileSync(named.profile,JSON.stringify({native_overlay:false}));await waitFor(()=>!existsSync(named.file),'Disabling HUD did not remove telemetry');
 writeFileSync(named.profile,JSON.stringify({native_overlay:true}));writeFileSync(named.phase,'world');await waitFor(()=>named.sample().capabilities.includes('coordinates'),'Bridge did not resume');
 const obf=await launch('ObfuscatedClient',{'class.official':'fixture.ObfuscatedClient','official.client.player':'p','official.client.world':'w','official.client.fps':'f','official.entity.x':'a','official.entity.y':'b','official.entity.z':'c'});
 assert.equal(obf.frame.game_fps,83);assert.equal(obf.frame.x,20.25);assert.equal(obf.frame.y,-4.5);assert.equal(obf.frame.z,-22.25);assert.ok(!obf.frame.capabilities.includes('armor'),'Unsupported armor was presented as an empty inventory');
 const legacy=await launch('LegacyClient',{'class.official':'fixture.LegacyClient','official.client.player':'p','official.client.world':'w','official.client.debug':'debug','official.entity.pos_x':'a','official.entity.pos_y':'b','official.entity.pos_z':'c','official.entity.yaw_field':'d'});
 assert.equal(legacy.frame.game_fps,73);assert.equal(legacy.frame.x,-10.5);assert.equal(legacy.frame.z,40.75);assert.equal(legacy.frame.direction,'east');
 const equipment=await launch('EquipmentClient',{'class.official':'fixture.EquipmentClient','official.player.equipment_slot':'e','official.slot.head':'CAP'});
 assert.equal(equipment.frame.armor.length,2,'Unexpected equipment rows');assert.equal(equipment.frame.armor[0].slot,'head');assert.equal(equipment.frame.armor[1].slot,'mainhand');assert.equal(equipment.frame.armor[0].remaining,73);assert.equal(equipment.frame.armor[1].item_id,'minecraft:diamond_sword');assert.ok(equipment.frame.capabilities.includes('armor'));
 for(const type of ['RenderOnlyClient','LegacyRenderOnlyClient']){const render=await launch(type,{},true);assert.ok(render.frame.game_fps>20&&render.frame.game_fps<1000);assert.equal(render.frame.frame_time_ms,undefined);assert.equal(render.frame.fps_low,undefined);assert.equal(render.frame.has_world,false);}
 console.log('TP Bridge verified in real JVM processes: actual FPS/XYZ/ping, mapped equipment and local item IDs, light levels, C zoom with release/focus/menu/disable restoration and no saved changes, no CPU/1% low telemetry, authenticated UDP, reduced-debug privacy and main-thread controls.');
}catch(error){for(const child of children)if(child.testErrors?.())console.error(child.testErrors());throw error;}finally{
 udp.close();
 for(const child of children){child.kill();await new Promise(r=>{if(child.exitCode!==null||child.signalCode!==null)r();else child.once('exit',r);});}
 const actual=realpathSync(dir),base=realpathSync(build);if(!actual.startsWith(base+sep))throw new Error('Test directory escaped workspace.');rmSync(actual,{recursive:true,force:true});
}
