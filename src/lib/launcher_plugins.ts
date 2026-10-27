import visualAddons from './data/visual_addons.json';
import {METRIC_MODULES,type MetricModuleId} from './hud_metrics';
import {defaultCrosshair,type CrosshairSettings} from './crosshair';
export type HudAnchor='top-left'|'top-center'|'top-right'|'bottom-left'|'bottom-center'|'bottom-right';
export type HudModuleId='light_levels'|'skin_layers'|'wavey_capes'|'old_animations'|'freelook'|'block_outline'|'motion_blur'|'low_fire'|'clean_view'|'hit_color'|'particle_multiplier'|'self_nametag'|'ragdolls'|'tnt_timer'|MetricModuleId|'entityculling'|'immediatelyfast'|'dynamicfps'|'iris'|'no_fog'|'item_physics'|'days'|'biome'|'fps'|'coordinates'|'direction'|'keystrokes'|'armor'|'minimap'|'clock'|'ping'|'custom_font'|'transparent_menu'|'enhanced_chat'|'optical_zoom'|'sodium'|'lithium'|'ferritecore';
export type PluginCategory='hud'|'interface'|'performance'|'survival'|'pvp'|'technical'|'visual';
export type HudProfile={title_theme:boolean;fog:{enabled:boolean;overworld:boolean;nether:boolean;end:boolean;water:boolean;lava:boolean;strength:number};low_latency:boolean;crosshair:CrosshairSettings;schema_version:1;name:string;scale:number;opacity:number;background:boolean;native_overlay:boolean;companion_enabled:boolean;auto_borderless:boolean;day_offset:number;days_with_fps:boolean;accent:string;text_color:string;background_color:string;menu_transparency:number;menu_blur:boolean;chroma:boolean;menu_offset_x:number;menu_offset_y:number;modules:{id:HudModuleId;enabled:boolean;anchor:HudAnchor;x?:number|null;y?:number|null;scale?:number|null;opacity?:number|null;background?:boolean|null}[]};
export type PluginWorkspace={profile:HudProfile;profile_path:string;profile_saved:boolean;companion_available:boolean;bridge_available:boolean};
export const HUD_ANCHORS:{id:HudAnchor;label:string}[]=[{id:'top-left',label:'Arriba a la izquierda'},{id:'top-center',label:'Arriba al centro'},{id:'top-right',label:'Arriba a la derecha'},{id:'bottom-left',label:'Abajo a la izquierda'},{id:'bottom-center',label:'Abajo al centro'},{id:'bottom-right',label:'Abajo a la derecha'}];
export const PLUGIN_CATEGORIES:{id:PluginCategory;name:string;icon:string}[]=[{id:'hud',name:'HUDs del juego',icon:'grid'},{id:'interface',name:'Interfaz y menú',icon:'image'},{id:'performance',name:'Optimizadores',icon:'bolt'},{id:'survival',name:'Supervivencia',icon:'compass'},{id:'pvp',name:'Combate',icon:'shield'},{id:'technical',name:'Técnico',icon:'settings'},{id:'visual',name:'Efectos visuales',icon:'image'}];
export const LAUNCHER_PLUGINS:ReadonlyArray<{id:HudModuleId;name:string;icon:string;category:PluginCategory;description:string;delivery:'companion'|'installer';project?:string}>=[
{id:'entityculling',name:'EntityCulling',icon:'ender_eye',category:'performance',description:'Descarga compatible: omite entidades o bloques fuera de la vista.',delivery:'installer'},
{id:'immediatelyfast',name:'ImmediatelyFast',icon:'redstone',category:'performance',description:'Descarga compatible: optimiza las llamadas de renderizado.',delivery:'installer'},
{id:'dynamicfps',name:'Dynamic FPS',icon:'clock',category:'performance',description:'Reduce recursos cuando el juego está en segundo plano.',delivery:'installer'},
{id:'iris',name:'Iris Shaders',icon:'sky_sun',category:'performance',description:'Shaders sobre un cargador compatible, con dependencias verificadas.',delivery:'installer'},
{id:'no_fog',name:'Control de niebla',icon:'glass',category:'interface',description:'Configura cada dimensión y entorno desde Packs, shaders y control rápido.',delivery:'companion'},
{id:'item_physics',name:'Físicas visuales de objetos',icon:'diamond_sword',category:'interface',description:'Objetos apoyados en el suelo y giro durante la caída, con TP Companion.',delivery:'companion'},
{id:'days',name:'Días del mundo',icon:'sky_sun',category:'hud',description:'Día real del mundo, junto a los FPS o como widget independiente.',delivery:'companion'},
{id:'biome',name:'Bioma',icon:'grass_block',category:'hud',description:'Bioma que el cliente recibe en tu posición.',delivery:'companion'},
{id:'fps',name:'Contador de FPS',icon:'clock',category:'hud',description:'Fotogramas por segundo reales de Minecraft.',delivery:'companion'},
{id:'coordinates',name:'Coordenadas',icon:'compass',category:'hud',description:'Posición y altura de tu personaje.',delivery:'companion'},
{id:'keystrokes',name:'Teclas y clics',icon:'redstone',category:'hud',description:'Movimiento y clics para tu HUD de combate.',delivery:'companion'},
{id:'armor',name:'Armadura y herramientas',icon:'diamond_sword',category:'hud',description:'Icono y porcentaje de durabilidad de cada pieza equipada y de tus manos.',delivery:'companion'},
{id:'clock',name:'Reloj',icon:'sky_sun',category:'hud',description:'La hora del equipo dentro de Minecraft.',delivery:'companion'},
{id:'ping',name:'Latencia · MS',icon:'ender_eye',category:'hud',description:'Milisegundos de respuesta del servidor; en un mundo local muestra 0 ms.',delivery:'companion'},
{id:'direction',name:'Dirección',icon:'compass',category:'hud',description:'Brújula y orientación al explorar.',delivery:'companion'},
{id:'minimap',name:'Minimapa',icon:'map',category:'hud',description:'Superficie de 96 × 96 bloques recibidos, con norte y posición del jugador.',delivery:'companion'},
{id:'custom_font',name:'Fuente personalizada',icon:'book',category:'interface',description:'Preferencia de tipografía para menús y HUD.',delivery:'companion'},
{id:'transparent_menu',name:'Menú transparente',icon:'glass',category:'interface',description:'Personalización de fondos de menú.',delivery:'companion'},
{id:'enhanced_chat',name:'Chat mejorado',icon:'name_tag',category:'interface',description:'Hora local en cada nuevo mensaje, conservando el estilo del texto.',delivery:'companion'},
{id:'optical_zoom',name:'Zoom óptico',icon:'spyglass',category:'interface',description:'Mantén C para acercar la cámara con TP Companion.',delivery:'companion'},
{id:'sodium',name:'Sodium',icon:'redstone',category:'performance',description:'Solo se descarga una edición compatible con tu cargador y versión.',delivery:'installer'},
{id:'lithium',name:'Lithium',icon:'slime_ball',category:'performance',description:'Optimizaciones de lógica del juego; compatibilidad comprobada en Modrinth.',delivery:'installer'},
{id:'ferritecore',name:'FerriteCore',icon:'iron_ingot',category:'performance',description:'Optimizaciones de memoria para versiones y cargadores compatibles.',delivery:'installer'},
...visualAddons.map(row=>({id:row.id as HudModuleId,name:row.name,icon:'ender_eye',category:('category' in row?row.category:'visual') as PluginCategory,description:row.description+' Se prepara al próximo inicio.',delivery:'installer' as const,project:'https://modrinth.com/mod/'+row.slug})),
...METRIC_MODULES.map(row=>({id:row.id,name:row.name,icon:row.group==='pvp'?'diamond_sword':'compass',category:row.group as PluginCategory,description:row.description,delivery:'companion' as const}))
];
const widgetIds=new Set(['fps','coordinates','direction','armor','keystrokes','clock','ping','minimap','days','biome',...METRIC_MODULES.map(row=>row.id)]);
export function isHudWidget(id:string):boolean{return widgetIds.has(id);}

export function normalizeHudProfile(profile:HudProfile):HudProfile{
 const existing=new Map(profile.modules.map(module=>[module.id,module]));
 const defaults=defaultHudProfile();
 return{...profile,modules:defaults.modules.map(module=>existing.get(module.id)??module)};
}
export const FUTURE_PLUGIN_TOOLS=[{name:'Adaptador del juego',icon:'crafting_table',description:'Conectará los módulos del HUD y la interfaz con Minecraft.'},{name:'Cosméticos compartidos',icon:'golden_helmet',description:'Las vistas previas TP necesitan un complemento para mostrarse dentro del juego.'}] as const;
export function defaultHudProfile():HudProfile{
return{title_theme:true,fog:{enabled:false,overworld:true,nether:true,end:true,water:true,lava:true,strength:1},low_latency:false,crosshair:defaultCrosshair(),schema_version:1,name:'Mi HUD',scale:1,opacity:.9,background:true,native_overlay:false,companion_enabled:false,auto_borderless:false,day_offset:0,days_with_fps:false,accent:'#A855F7',text_color:'#F3EDF8',background_color:'#0B0B0F',menu_transparency:.35,menu_blur:true,chroma:false,menu_offset_x:0,menu_offset_y:0,modules:LAUNCHER_PLUGINS.map(plugin=>({id:plugin.id,enabled:['fps','ping','coordinates','keystrokes','armor'].includes(plugin.id),anchor:({keystrokes:'top-right',armor:'bottom-right',clock:'bottom-left',ping:'top-left'} as Record<string,HudAnchor>)[plugin.id]??'top-left'}))};
}
export function deliveryHelp(id:HudModuleId):string{
 if(visualAddons.some(a=>a.id===id))return 'Modrinth · mod externo · próximo inicio compatible';
 if(id.startsWith("m_"))return "Dato real · disponibilidad según adaptador";
 if(id==='keystrokes'||id==='clock'||id==='custom_font')return 'Overlay Universal · cualquier versión';
 if(['days','biome'].includes(id))return 'Datos reales · conector del juego';
 if(id==='fps')return 'FPS reales · puente local / TP Companion';
 if(['sodium','lithium','ferritecore','entityculling','immediatelyfast','dynamicfps','iris'].includes(id))return 'Se prepara en el próximo inicio compatible';
 if(['coordinates','direction','armor','ping'].includes(id))return 'Datos internos · puente local / TP Companion';
 return 'Funciones del motor · TP Companion compatible';
}
export const HUD_PRESETS=[{id:'explore',name:'Exploración'},{id:'pvp',name:'Combate'},{id:'minimal',name:'Minimalista'},{id:'technical',name:'Técnico'},{id:'build',name:'Construcción'}] as const;
export type HudPreset=typeof HUD_PRESETS[number]['id'];
export function applyHudPreset(profile:HudProfile,kind:HudPreset):HudProfile{
 const next=normalizeHudProfile(structuredClone(profile));
 const enabled:Record<HudPreset,string[]>={explore:['fps','ping','days','coordinates','direction','armor'],pvp:['fps','keystrokes','armor','ping'],minimal:['fps','ping'],technical:['light_levels','fps','ping','coordinates','m_chunk_coords','m_chunk_local','m_block_light','m_sky_light'],build:['fps','ping','coordinates','direction','days','m_inventory_free']};
 next.name='Mi HUD · '+HUD_PRESETS.find(p=>p.id===kind)!.name;
 for(const module of next.modules)if(isHudWidget(module.id)||module.id==='light_levels')module.enabled=enabled[kind].includes(module.id);return next;
}
export function setAllPlugins(profile:HudProfile,enabled:boolean):void{profile.modules=normalizeHudProfile(profile).modules;for(const module of profile.modules)module.enabled=enabled;profile.fog.enabled=enabled;}
