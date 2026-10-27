import {metricValue} from './hud_metrics';
import type {OverlaySnapshot} from './launcher_extensions';
export type HudMetricState={available:boolean;short:string;detail:string};
const internal=new Set(['coordinates','direction','armor','ping','minimap','days','biome']);
export function hudMetricState(id:string,frame:OverlaySnapshot|null,preview=false):HudMetricState{
 if(['frame_time','fps_low'].includes(id))return{available:false,short:'Retirado',detail:'El HUD de rendimiento utiliza únicamente FPS y MS.'};
 if(id.startsWith('m_')){const value=metricValue(id,frame,preview);return{available:value!=null,short:value!=null?(preview?'Ejemplo':'En vivo'):'No disponible',detail:value!=null?'Dato real de la partida o el proceso.':'El adaptador de esta edición no ha conectado este dato. Se oculta en el HUD.'};}
 if(preview)return{available:true,short:'Ejemplo',detail:'Datos de ejemplo del diseñador.'};
 if(id==='fps'){
  const available=frame?.game_fps!=null&&Number.isFinite(frame.game_fps)&&frame.game_fps>=0;
  return{available,short:available?'En vivo':'Sin datos',detail:available?frame?.fps_source||'Contador interno':frame?.fps_source||'Inicia Minecraft con el HUD activado para conectar el contador.'};
 }
 if(['transparent_menu','enhanced_chat','optical_zoom','item_physics','no_fog'].includes(id)){const available=['no_fog','optical_zoom'].includes(id)?!!frame?.telemetry?.capabilities.includes(id):frame?.telemetry?.source==='TP Companion';return{available,short:available?'Adaptador conectado':'Requiere adaptador',detail:available?(id==='optical_zoom'?'Mantén C durante la partida para acercar la cámara.':'Función interna disponible en esta edición.'):'Esta función requiere un adaptador del juego compatible. La casilla guarda la preferencia, pero no añade soporte a una edición sin conector.'};}
 if(!internal.has(id))return{available:true,short:'En vivo',detail:'Overlay local.'};
 const data=frame?.telemetry;
 if(data?.coordinates_restricted&&['coordinates','direction','minimap'].includes(id))return{available:false,short:'Restringido',detail:'El servidor ha reducido la información de depuración. El HUD respeta esa restricción.'};
 if(!data)return{available:false,short:'Sin conexión',detail:frame?.message||'Si activaste el HUD durante la partida, reinicia Minecraft desde TP LAUNCHER para conectar sus datos.'};
 if(!data.has_world)return{available:false,short:'Esperando mundo',detail:data.status||'Entra en un mundo o servidor para recibir este dato.'};
 const capabilities=data.capabilities??['fps','coordinates','direction','armor','ping',...(data.map?['minimap']:[])];
 if(!capabilities.includes(id))return{available:false,short:'No disponible',detail:id==='minimap'?'El minimapa requiere TP Companion en una edición compatible.':`Esta edición no ha conectado este dato. ${data.status||'Revisa la conexión en Plugins.'}`};
 if(id==='coordinates'&&![data.x,data.y,data.z].every(Number.isFinite))return{available:false,short:'Esperando posición',detail:'No se recibió una posición válida.'};
 if(id==='minimap'&&!data.map)return{available:false,short:'Esperando terreno',detail:'Todavía no hay chunks cargados para dibujar el mapa.'};
 if(id==='ping'&&data.ping==null)return{available:false,short:'Esperando servidor',detail:'El cliente aún no ha recibido el tiempo de respuesta del servidor.'};
 return{available:true,short:'En vivo',detail:data.source||'Integración local del juego'};
}
