import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
const root=path.resolve(import.meta.dirname,'..');
if(process.platform!=='win32')throw new Error('La firma Authenticode debe ejecutarse en Windows.');
const thumbprint=(process.env.TP_SIGN_CERT_THUMBPRINT??'').replace(/\s/g,'').toUpperCase();
if(!/^[0-9A-F]{40}$/.test(thumbprint))throw new Error('Falta TP_SIGN_CERT_THUMBPRINT: primero obtén e instala un certificado de firma de código confiable con su clave privada en CurrentUser/My. No se generará una firma autofirmada.');
const env={...process.env,TP_SIGN_CERT_THUMBPRINT:thumbprint};
const powershell=path.join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe');
const run=code=>execFileSync(powershell,['-NoLogo','-NoProfile','-NonInteractive','-Command',code],{cwd:root,env,windowsHide:true,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
run(`$ErrorActionPreference='Stop'; $cert=Get-Item -LiteralPath ('Cert:/CurrentUser/My/'+$env:TP_SIGN_CERT_THUMBPRINT) -ErrorAction Stop;
if(-not $cert.HasPrivateKey -or $cert.NotAfter -le (Get-Date) -or $cert.NotBefore -gt (Get-Date)){throw 'El certificado no dispone de clave privada utilizable o no está vigente.'};
if(-not ($cert.EnhancedKeyUsageList | Where-Object {$_.ObjectId -eq '1.3.6.1.5.5.7.3.3'})){throw 'El certificado no tiene uso de firma de código.'};
if(-not $cert.Verify()){throw 'Windows no confía en la cadena del certificado. No se cambiará el almacén de confianza.'}`);
console.log('Certificado de firma vigente y confiable confirmado.');
if(!process.argv.includes('--check-only')){
 const config=JSON.parse(fs.readFileSync(path.join(root,'src-tauri/tauri.conf.json'),'utf8'));
 const dir=path.join(root,'.codex-tmp/signing');fs.mkdirSync(dir,{recursive:true});
 const file=path.join(dir,'tauri-signing.json');
 fs.writeFileSync(file,JSON.stringify({bundle:{windows:{certificateThumbprint:thumbprint,digestAlgorithm:'sha256',timestampUrl:'http://timestamp.digicert.com',tsp:true}}},null,2));
 execFileSync(process.execPath,[path.join(root,'node_modules/@tauri-apps/cli/tauri.js'),'build','--config',file],{cwd:root,windowsHide:true,stdio:'inherit',env});
 for(const file of [path.join(root,'src-tauri/target/release/launchertps.exe'),path.join(root,'src-tauri/target/release/bundle/nsis',`${config.productName}_${config.version}_x64-setup.exe`)]){
  env.TP_SIGN_VERIFY_PATH=file;
  run(`$signature=Get-AuthenticodeSignature -LiteralPath $env:TP_SIGN_VERIFY_PATH;
if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ne $env:TP_SIGN_CERT_THUMBPRINT -or -not $signature.TimeStamperCertificate){throw 'La firma final no es válida, no coincide con el certificado o carece de sello de tiempo.'}`);
 }
 console.log('Ejecutable e instalador: firmas Authenticode válidas y selladas en el tiempo.');
}
