import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import ts from 'typescript';
import {compile} from 'svelte/compiler';
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
const {LAUNCHER_PLUGINS}=await import(uri(pluginSource));
const {PLUGIN_ICONS,resolvePluginIcon}=await import(uri(transpile('src/lib/plugin_icons.ts')));
const source=read('src/lib/UiIcon.svelte');
const paths=source.match(/const paths:[^=]+=\s*(\{[\s\S]+?\n  \});/)[1];
const vectors=Function('return ('+paths+')')();
const textureSource=read('src-tauri/src/poser_assets.rs').split('fn valid_id')[0];
for(const plugin of LAUNCHER_PLUGINS){
 assert.ok(Object.hasOwn(PLUGIN_ICONS,plugin.id),'Explicit semantic icon required: '+plugin.id);
 const icon=resolvePluginIcon(plugin.id,{});
 assert.ok(vectors[icon.vector],plugin.id+' must resolve to an existing SVG');
 assert.equal(icon.texture,null,'No local client must use the semantic vector');
 const spec=PLUGIN_ICONS[plugin.id];
 if(spec.texture){
  assert.ok(textureSource.includes('"'+spec.texture+'"'),'Rust local reader must expose '+spec.texture);
  const png='data:image/png;base64,cG5n';
  assert.equal(resolvePluginIcon(plugin.id,{[spec.texture]:png}).texture,png);
  assert.equal(resolvePluginIcon(plugin.id,{[spec.texture]:png},png).texture,null,'An image decoding failure must recover to SVG');
  assert.equal(resolvePluginIcon(plugin.id,{[spec.texture]:'https://example.invalid/art.png'}).texture,null,'Remote artwork is not a local JAR texture');
 }
}
assert.equal(PLUGIN_ICONS.fps.vector,'gauge');
assert.equal(PLUGIN_ICONS.iris.vector,'sun');
assert.equal(PLUGIN_ICONS.keystrokes.vector,'keyboard');
assert.equal(PLUGIN_ICONS.ping.vector,'network');
const quickTools=read('src/lib/PlayerQuickTools.svelte').split('<div class="quick-tools">')[1].split('</div>')[0];
const quickButtons=[...quickTools.matchAll(/<button\b[\s\S]*?<\/button>/g)].map(match=>match[0]);
assert.equal(quickButtons.length,6);
for(const button of quickButtons)assert.equal((button.match(/<UiIcon /g)??[]).length,1,'Every quick action gets exactly one icon');
assert.ok(!quickTools.includes('GameIcon'),'No purple pixel fallback in quick actions');
for(const file of ['PluginIcon','PlayerQuickTools','UiIcon','PluginsPanel','HudInGameMenu'])compile(read(`src/lib/${file}.svelte`),{filename:`${file}.svelte`,generate:'client'});
for(const file of ['PluginsPanel','HudInGameMenu'])assert.ok(read(`src/lib/${file}.svelte`).includes('<PluginIcon id='),'Both plugin menus must share the icon renderer');
console.log(`Plugin icons: ${LAUNCHER_PLUGINS.length} explicit modules, SVG availability, local artwork coverage, safe missing/broken image fallback and six single-icon quick actions verified.`);
