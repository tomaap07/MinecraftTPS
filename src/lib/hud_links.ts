import{openUrl}from '@tauri-apps/plugin-opener';

export async function openHudLink(address:string):Promise<void>{
 const url=new URL(address);
 if(url.origin!=='https://modrinth.com'||url.username||url.password)throw new Error('Este enlace no pertenece al catálogo permitido.');
 await openUrl(url.href);
}
