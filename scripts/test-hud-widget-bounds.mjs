import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import ts from 'typescript';
import {compile} from 'svelte/compiler';
const root=resolve(import.meta.dirname,'..');
const read=path=>readFileSync(resolve(root,path),'utf8');
const source=ts.transpileModule(read('src/lib/hud_widget_bounds.ts'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext}}).outputText;
const {visibleWidgetPosition:fit}=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64'));
const close=(a,b)=>assert.ok(Math.abs(a-b)<1e-9,`${a} must be approximately ${b}`);
const saved=Object.freeze({x:.8994791666666667,y:.8});
const size={width:128,height:80};

assert.deepEqual(fit(saved,{width:1920,height:1080},size,1.5),saved);

const narrow=fit(saved,{width:1280,height:720},size,1.5);
close(narrow.x,.85);close(narrow.x*1280+size.width*1.5,1280);
const small=fit(saved,{width:800,height:450},size,1.5);
close(small.x,.76);close(small.y,1-120/450);
assert.deepEqual(fit(saved,{width:1920,height:1080},size,1.5),saved,'Restoring the viewport must restore the saved position');
assert.equal(saved.x,.8994791666666667,'Viewport fitting never rewrites the profile');

assert.deepEqual(fit(saved,{width:1280,height:720},size,.5),saved);
const edge={x:.95,y:.95};
close(fit(edge,{width:1000,height:600},{width:200,height:100},1).x,.8);
close(fit(edge,{width:1000,height:600},{width:200,height:200},1.5).y,.5);
assert.deepEqual(fit({x:1,y:1},{width:100,height:80},{width:200,height:100},1.5),{x:0,y:0});
assert.deepEqual(fit({x:-1,y:NaN},{width:0,height:0},size,1),{x:0,y:0});
assert.deepEqual(fit(saved,{width:0,height:0},size,1.5),saved,'Unmeasured containers must preserve normalized saved placement');

for(const width of [320,854,1280,1920,3440])for(const height of [240,480,720,1080])for(const scale of [.5,1,1.5,2.5]){
 const box={width:117,height:93};
 const p=fit({x:.97,y:.98},{width,height},box,scale);
 assert.ok(p.x>=0&&p.y>=0);
 if(box.width*scale<=width)assert.ok(p.x*width+box.width*scale<=width+1e-8);
 if(box.height*scale<=height)assert.ok(p.y*height+box.height*scale<=height+1e-8);
}
const widget=read('src/lib/HudWidgets.svelte');
compile(widget,{filename:'HudWidgets.svelte',generate:'client'});
assert.ok(widget.includes('use:measureViewport'),'Native/preview viewport resizes must refresh placement');
assert.ok(widget.includes('m.anchor===module.anchor'),'Automatic anchor stacking must remain available');
assert.ok(widget.includes('scene.setPointerCapture(event.pointerId)'),'Pointer dragging must retain capture');
console.log('HUD widget bounds: viewport/DPI changes, global and individual scales, content growth, restored placement without profile mutation, edge limits and Svelte syntax verified.');
