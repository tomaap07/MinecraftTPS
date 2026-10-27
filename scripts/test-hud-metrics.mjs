import assert from 'node:assert/strict';
import{readFileSync}from'node:fs';import{resolve}from'node:path';import ts from'typescript';
const root=resolve(import.meta.dirname,'..');
const catalog=JSON.parse(readFileSync(resolve(root,'src/lib/data/hud_metrics.json'),'utf8'));
const addons=JSON.parse(readFileSync(resolve(root,'src/lib/data/visual_addons.json'),'utf8'));assert.equal(addons.length,14);assert.equal(new Set(addons.map(a=>a.id)).size,addons.length);for(const addon of addons){assert.ok(addon.projects.length);assert.ok(addon.projects.every(p=>/^[a-zA-Z0-9_-]+$/.test(p)));}
const source=ts.transpileModule(readFileSync(resolve(root,'src/lib/hud_metrics.ts'),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText.replace(/import catalog from '[^']+';/,`const catalog=${JSON.stringify(catalog)};`);
const{metricValue,METRIC_MODULES}=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64'));
assert.equal(METRIC_MODULES.length,20);assert.equal(new Set(METRIC_MODULES.map(x=>x.id)).size,METRIC_MODULES.length);assert.equal(new Set(METRIC_MODULES.map(x=>x.name)).size,METRIC_MODULES.length);
assert.ok(METRIC_MODULES.every(row=>row.group!=='system'&&row.source!=='system'),'System counters must not appear in the player HUD');
const frame={updated_at:Date.now(),left_cps:7,right_cps:3,telemetry:{has_world:true,coordinates_restricted:false,capabilities:['coordinates','armor'],x:-1,y:64,z:-17,world_time:24000,dimension:'minecraft:overworld',armor:[{maximum:100,remaining:12}],metrics:{heap_used:1048576,heap_max:4194304},world_metrics:{health:18,stat_player_kills:4,stat_deaths:0,spawn_x:0,spawn_z:0}}};
assert.equal(metricValue('m_chunk_coords',frame),'-1 / -2');assert.equal(metricValue('m_region_coords',frame),null);assert.equal(metricValue('m_chunk_local',frame),'15 / 15');
assert.equal(metricValue('m_world_dimension',frame),'overworld');assert.equal(metricValue('m_nether_coords',frame),`${(-.125).toLocaleString(undefined,{maximumFractionDigits:1})} / ${(-2.125).toLocaleString(undefined,{maximumFractionDigits:1})}`);assert.equal(metricValue('m_heap_percent',frame),null);assert.equal(metricValue('m_java_cpu',frame),null);assert.equal(metricValue('m_kd_ratio',frame),null);
assert.equal(metricValue('m_count_totem_of_undying',frame),null,'Unknown inventory data was converted to zero');
frame.telemetry.world_metrics.count_totem_of_undying=0;assert.equal(metricValue('m_count_totem_of_undying',frame),'0','A real empty inventory count disappeared');
frame.telemetry.coordinates_restricted=true;for(const id of ['m_chunk_coords','m_chunk_local','m_nether_coords'])assert.equal(metricValue(id,frame),null,'Reduced debug coordinates leaked');
frame.telemetry.has_world=false;for(const id of ['m_health','m_world_dimension','m_nether_coords'])assert.equal(metricValue(id,frame),null,'World widget persisted in menu');assert.equal(metricValue('m_heap_percent',frame),null,'Retired system telemetry leaked in menu');
assert.equal(metricValue('m_stat_deaths',null),null);for(const row of METRIC_MODULES)assert.ok(metricValue(row.id,null,true),'Designer sample is missing: '+row.id);
console.log(`${METRIC_MODULES.length} essential metrics: no CPU/RAM counters, unique registry, negative chunks, real zero vs missing and reduced-debug restrictions verified.`);
