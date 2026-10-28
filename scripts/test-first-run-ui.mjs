import assert from 'node:assert/strict';
import fs from 'node:fs';
import {chromium} from 'playwright-core';
const executablePath=['C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe','C:/Program Files/Microsoft/Edge/Application/msedge.exe'].find(fs.existsSync);
assert.ok(executablePath,'An installed Edge browser is required for UI tests.');
const browser=await chromium.launch({executablePath,headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader','--no-first-run']});
const port=Number(process.env.TP_TEST_PORT??1420);assert.ok(Number.isInteger(port)&&port>=1024&&port<=65535);
try{
 const page=await browser.newPage({viewport:{width:1280,height:900}});
 const errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.addInitScript(()=>{
  let settings={language:'',translate_dynamic_content:false,launch_behavior:'keep',data_root:'C:/TP-test',discord_presence:false,discord_application_id:'',automatic_updates:false,first_run_completed:false,ui_scale_percent:100,launcher_width:1280,launcher_height:820,launcher_maximized:false,resolution_width:1280,resolution_height:720,fullscreen:false,ram_gb:4,fps_boost:false,gc_mode:'g1gc'};
  let account=null;window.testCalls=[];window.testMicrosoftFailure=true;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
  window.__TAURI_INTERNALS__={metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(cmd,args={})=>{
   window.testCalls.push({cmd,args});
   if(cmd==='get_launcher_settings')return settings;
   if(cmd==='save_launcher_settings'){settings=args.settings;return settings;}
   if(cmd==='get_system_locale')return 'es-CL';
   if(cmd==='get_system_summary')return {total_memory_gb:16,recommended_ram_gb:8};
   if(cmd==='available_java_versions')return [8,11,17,21,22,25,26];
   if(cmd==='get_game_session')return {running:false,active_games:0,username:'',version:'',server:''};
   if(cmd==='get_offline_eligibility')return {allowed:true,message:''};
   if(cmd==='list_launcher_accounts')return account?[account]:[];
   if(cmd==='add_offline_account'){account={id:'offline-test',name:args.name,kind:'offline',active:true};return account;}
   if(cmd==='activate_launcher_account')return {account,profile:null};
   if(cmd==='begin_microsoft_login'){
    if(window.testMicrosoftFailure)throw new Error('Minecraft Services (HTTP 403): revisa la aprobación del Client ID.');
    const profile={id:'0123456789abcdef0123456789abcdef',name:'Fariiuh',skins:[],capes:[]};
    account={id:'microsoft-'+profile.id,name:profile.name,kind:'microsoft',active:true};return profile;
   }
   if(cmd==='get_microsoft_profile'||cmd==='studio_current_operation')return null;
   if(cmd==='get_root_path')return 'C:/TP-test';
   if(cmd==='get_custom_background'||cmd==='get_custom_app_icon')return '';
   if(cmd==='get_version_java_requirement')return 21;
   if(cmd==='check_java')return {installed:false,major:0,version:'',path:'',is_portable:false};
   if(cmd==='get_playtime_stats')return {total_seconds:0,active_games:0,versions:[]};
   if(cmd.includes('is_maximized'))return false;
   if(cmd.startsWith('plugin:'))return 1;
   if(/appearance|artwork|skin/.test(cmd))throw new Error('No local test artwork');
   return [];
  }};
 });
 await page.goto(`http://127.0.0.1:${port}/`,{waitUntil:'domcontentloaded'});
 const wizard=page.locator('.wizard-shell');await wizard.waitFor();
 await page.getByRole('heading',{name:'Selecciona tu idioma',exact:true}).waitFor();
 assert.equal(await page.evaluate(()=>document.documentElement.lang),'es-419');
 for(const code of ['en-US','pt-BR','fr-FR','de-DE','ru-RU','it-IT','zh-CN','ja-JP','ko-KR','es-419']){
  await wizard.locator('tbody tr').filter({hasText:code}).locator('button').click();
  await page.waitForFunction(code=>document.documentElement.lang===code,code);
  const title=await wizard.locator('.wizard-body h2').textContent();
  if(code!=='es-419')assert.notEqual(title,'Selecciona tu idioma','Wizard title must be translated: '+code);
 }

 await wizard.getByRole('button',{name:'Continuar',exact:true}).click();
 await page.getByRole('heading',{name:'Java y memoria',exact:true}).waitFor();
 await wizard.getByText('Java 26',{exact:true}).waitFor();
 await wizard.getByRole('button',{name:'Continuar',exact:true}).click();
 await page.getByRole('heading',{name:'Elige cómo jugar',exact:true}).waitFor();
 await wizard.locator('input[maxlength="16"]').fill('LocalTest');
 await wizard.getByRole('button',{name:'Empezar a jugar',exact:true}).click();
 await wizard.waitFor({state:'hidden'});
 const calls=await page.evaluate(()=>window.testCalls);
 assert.ok(calls.some(call=>call.cmd==='save_launcher_settings'&&call.args.settings.first_run_completed));
 assert.ok(calls.some(call=>call.cmd==='add_offline_account'&&call.args.name==='LocalTest'));
 assert.ok(!calls.some(call=>/install_java/.test(call.cmd)));
 await page.getByRole('button',{name:'Minimizar',exact:true}).click();
 assert.ok((await page.evaluate(()=>window.testCalls)).some(call=>call.cmd.includes('minimize')));
 await page.getByRole('button',{name:'Cambiar cuenta',exact:true}).click();
 const settingsView=page.locator('.settings-suite');
 await settingsView.getByRole('button',{name:/Añadir cuenta Microsoft/}).click();
 await settingsView.locator('.status.failed').filter({hasText:'Minecraft Services (HTTP 403)'}).waitFor();
 assert.notEqual((await settingsView.locator('.status').textContent()).trim(),'Listo');
 await page.reload({waitUntil:'domcontentloaded'});
 await wizard.getByRole('button',{name:'Continuar',exact:true}).click();
 await wizard.getByRole('button',{name:'Continuar',exact:true}).click();
 await wizard.getByRole('button',{name:/Microsoft · Minecraft Premium/}).click();
 await wizard.getByRole('button',{name:/Conectar Microsoft/}).click();
 await wizard.getByText(/Minecraft Services \(HTTP 403\)/).waitFor();
 assert.ok(!await wizard.getByText('Cuenta activa',{exact:true}).count());
 await page.evaluate(()=>window.testMicrosoftFailure=false);
 await wizard.getByRole('button',{name:/Conectar Microsoft/}).click();
 await wizard.getByText('Cuenta activa',{exact:true}).waitFor();
 await wizard.getByRole('button',{name:'Empezar a jugar',exact:true}).click();
 await wizard.waitFor({state:'hidden'});
 assert.ok((await page.evaluate(()=>window.testCalls)).some(call=>call.cmd==='begin_microsoft_login'));
 assert.deepEqual(errors,[],'Browser runtime errors');
 console.log('First-run UI: ten languages, optional Java, local account, Microsoft 403 surfaced without false success, successful retry, profile displayed and minimize command passed using isolated IPC fixtures.');
}finally{await browser.close();}
