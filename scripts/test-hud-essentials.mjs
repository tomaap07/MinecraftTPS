import assert from 'node:assert/strict';
import{readFileSync}from'node:fs';
import{resolve}from'node:path';
import ts from'typescript';
import{compile}from'svelte/compiler';
const root=resolve(import.meta.dirname,'..');
const read=path=>readFileSync(resolve(root,path),'utf8');
const uri=code=>'data:text/javascript;base64,'+Buffer.from(code).toString('base64');
const transpile=path=>ts.transpileModule(read(path),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText;
const metrics=uri(transpile('src/lib/hud_metrics.ts').replace(/import catalog from '[^']+';/,`const catalog=${read('src/lib/data/hud_metrics.json')};`));
const crosshair=uri(transpile('src/lib/crosshair.ts'));
const pluginSource=transpile('src/lib/launcher_plugins.ts')
 .replace(/import visualAddons from '[^']+';/,`const visualAddons=${read('src/lib/data/visual_addons.json')};`)
 .replace(/from ['"]\.\/hud_metrics['"]/,`from '${metrics}'`)
 .replace(/from ['"]\.\/crosshair['"]/,`from '${crosshair}'`);
const{LAUNCHER_PLUGINS,defaultHudProfile,normalizeHudProfile,applyHudPreset,setAllPlugins,isHudWidget}=await import(uri(pluginSource));
const defaults=defaultHudProfile();
for(const id of ['fps_low','frame_time','m_java_cpu','m_system_cpu','m_heap_percent','m_heap_used','m_gc_ms']){
 assert.equal(LAUNCHER_PLUGINS.some(p=>p.id===id),false,id+' must not be offered');
 assert.equal(isHudWidget(id),false,id+' must not render, including historical profiles');
}
assert.ok(defaults.modules.some(m=>m.id==='fps'&&m.enabled));assert.ok(defaults.modules.some(m=>m.id==='ping'&&m.enabled));
const old=structuredClone(defaults);old.modules.push({id:'m_java_cpu',enabled:true,anchor:'top-left'});old.modules.find(m=>m.id==='armor').opacity=.35;
const clean=normalizeHudProfile(old);assert.equal(clean.modules.some(m=>m.id==='m_java_cpu'),false);assert.equal(clean.modules.find(m=>m.id==='armor').opacity,.35,'Per-widget preferences must survive migration');
assert.deepEqual(applyHudPreset(old,'minimal').modules.filter(m=>m.enabled&&isHudWidget(m.id)).map(m=>m.id).sort(),['fps','ping']);
setAllPlugins(old,true);assert.equal(old.modules.some(m=>m.id==='m_java_cpu'),false);assert.ok(old.modules.every(m=>m.enabled));setAllPlugins(old,false);assert.ok(old.modules.every(m=>!m.enabled));
const{durabilityPercent,equipmentRows,equipmentKind}=await import(uri(transpile('src/lib/hud_equipment.ts')));
assert.equal(durabilityPercent({remaining:50,maximum:100}),50);assert.equal(durabilityPercent({remaining:1,maximum:1561}),1);assert.equal(durabilityPercent({remaining:0,maximum:100}),0);
for(const item of [{remaining:2,maximum:0},{remaining:NaN,maximum:10},{remaining:5,maximum:Infinity}])assert.equal(durabilityPercent(item),null);
assert.equal(durabilityPercent({remaining:-5,maximum:100}),0);assert.equal(durabilityPercent({remaining:500,maximum:100}),100);
const items=[{name:'Sword',item_id:'minecraft:diamond_sword',slot:'mainhand',maximum:100,remaining:80},{name:'Boots',item_id:'minecraft:diamond_boots',slot:'armor',maximum:100,remaining:75},{name:'Helmet',item_id:'minecraft:diamond_helmet',slot:'head',maximum:100,remaining:90}];
assert.deepEqual(equipmentRows(items).map(equipmentKind),['head','feet','sword']);
const stateSource=transpile('src/lib/hud_data.ts').replace(/from ['"]\.\/hud_metrics['"]/,`from '${metrics}'`);
const{hudMetricState}=await import(uri(stateSource));
assert.equal(hudMetricState('optical_zoom',{telemetry:{capabilities:['optical_zoom'],source:'TP Bridge'}}).available,true);
assert.equal(hudMetricState('optical_zoom',{telemetry:{capabilities:[],source:'TP Bridge'}}).available,false);
assert.equal(hudMetricState('fps_low',null,true).available,false);
for(const path of ['HudEquipmentIcon','HudWidgets','HudModuleSettings','HudInGameMenu','PluginsPanel','TpHudOverlay'])compile(read(`src/lib/${path}.svelte`),{filename:`${path}.svelte`,generate:'client'});
console.log('HUD essentials: historical-profile migration, FPS/MS minimal preset, isolated widget preferences, validated equipment percentages, slot ordering, zoom capability and component syntax verified.');
