import fs from 'node:fs/promises';
import path from 'node:path';
import http from 'node:http';
import {fileURLToPath} from 'node:url';
import {chromium} from 'playwright-core';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const possible=['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe','C:/Program Files/Microsoft/Edge/Application/msedge.exe','C:/Program Files/BraveSoftware/Brave-Browser/Application/brave.exe'];
let executablePath=process.env.TP_RENDER_BROWSER;
if(!executablePath)for(const candidate of possible){try{await fs.access(candidate);executablePath=candidate;break;}catch{}}
if(!executablePath)throw new Error('Set TP_RENDER_BROWSER to an installed Chromium browser.');
const html=`<!doctype html><html><head><meta charset="utf-8"><style>html,body{margin:0;background:transparent}</style></head><body><canvas id="character"></canvas><script src="/viewer.js"></script><script>
window.renderCharacter=async()=>{
 const viewer=new skinview3d.SkinViewer({canvas:document.querySelector('canvas'),width:1000,height:1400,preserveDrawingBuffer:true,zoom:.70});
 viewer.autoRotate=false;viewer.controls.enabled=false;
 await viewer.loadSkin('/skin.png');
 viewer.playerObject.rotation.y=.28;
 const skin=viewer.playerObject.skin;
 skin.head.rotation.set(-.06,.12,-.05);
 skin.rightArm.rotation.set(-.15,.05,-2.2);
 skin.leftArm.rotation.set(-.10,0,.10);
 skin.rightLeg.rotation.z=-.035;skin.leftLeg.rotation.z=.035;
 viewer.globalLight.intensity=2.2;viewer.cameraLight.intensity=1.3;
 viewer.render();
 return viewer.canvas.toDataURL('image/png');
};
</script></body></html>`;
const server=http.createServer(async(req,res)=>{try{
 if(req.url==='/viewer.js'){res.setHeader('Content-Type','text/javascript');res.end(await fs.readFile(path.join(root,'node_modules/skinview3d/bundles/skinview3d.bundle.js')));}
 else if(req.url==='/skin.png'){res.setHeader('Content-Type','image/png');res.end(await fs.readFile(path.join(root,'static/branding/fariiuh-skin.png')));}
 else{res.setHeader('Content-Type','text/html');res.end(html);}
}catch{res.writeHead(500);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({executablePath,headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader','--no-first-run']});
try{
 const page=await browser.newPage({viewport:{width:1000,height:1400},deviceScaleFactor:1});
 await page.goto('http://127.0.0.1:'+server.address().port);
 const png=await page.evaluate(()=>window.renderCharacter());
 await fs.writeFile(path.join(root,'src-tauri/installer/fariiuh-welcome.png'),Buffer.from(png.split(',')[1],'base64'));
 console.log('Rendered Fariiuh welcome pose at 1000 × 1400, transparent PNG.');
}finally{await browser.close();server.close();}
