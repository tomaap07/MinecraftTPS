
import type { CharacterVariant, HeldItemId } from './held_items';

export type SecretTrigger = 'logo'|'console'|'studio'|'play'|'skin'|'night'|'date'|'startup';
export type EasterEgg = Readonly<{
  id:string;kind:'appearance'|'splash'|'console'|'date'|'achievement';category:string;
  title:string;text:string;chance:number;variant?:CharacterVariant;item?:HeldItemId;
  leftItem?:HeldItemId;durationMs?:number;trigger?:SecretTrigger;dateRule?:string;
}>;
type Reference = readonly [id:string,subject:string,scene:string,detail:string];
const REFERENCES:readonly Reference[]=[
  ['farlands','las Far Lands','en el borde imposible del mundo','un paisaje que nació de un límite numérico'],
  ['herobrine','Herobrine','entre los árboles de una niebla Alpha','dos ojos blancos que nunca estuvieron en el parche'],
  ['packpng','pack.png','junto a la colina del pack de texturas','la semilla detrás de una imagen inolvidable'],
  ['aether','el portal al Aether','frente al marco de glowstone','el cubo de agua que prometía otro cielo'],
  ['alpha','la hierba de Alpha','en un mundo de verdes imposibles','un horizonte que todavía cabe en la nostalgia'],
  ['c418','las melodías de C418','al caer la tarde sobre tu primer refugio','unas notas capaces de detener una aventura'],
  ['sweden','Sweden','cuando el mundo se queda en silencio','la música de volver a casa'],
  ['wet_hands','Wet Hands','junto a una ventana de tablones','un piano para una noche de lluvia'],
  ['mice_on_venus','Mice on Venus','más allá de los primeros chunks','la banda sonora de perder la noción del tiempo'],
  ['disceleven','el disco 11','al fondo de una mina abandonada','una grabación que termina demasiado pronto'],
  ['disc13','el disco 13','en un cofre cubierto de polvo','un silencio que también forma parte de la música'],
  ['redstone','la redstone','detrás de una pared aparentemente normal','un circuito que convierte una idea en movimiento'],
  ['repeater','el repetidor','en el corazón de una granja automática','cuatro posiciones para ordenar el tiempo'],
  ['piston','el pistón pegajoso','bajo una puerta secreta','un bloque que sabe guardar secretos'],
  ['observer','el observador','a un cambio de estado de la sorpresa','dos puntos rojos esperando el próximo pulso'],
  ['hopper','la tolva','debajo de un sistema de almacenamiento','el inventario que se ordena mientras exploras'],
  ['comparator','el comparador','al lado de un cofre casi lleno','una señal que también sabe contar historias'],
  ['mlg','el MLG de agua','a un instante del suelo','un cubo que cabe entre el miedo y la victoria'],
  ['speedrun','el speedrun','antes de que aparezca el primer portal','cada decisión convertida en segundos'],
  ['enderpearl','la perla de Ender','al otro lado de un salto imposible','un arco verde que decide el siguiente paso'],
  ['blazerod','la vara de Blaze','sobre un puente de fortaleza','el ingrediente que acerca el final'],
  ['stronghold','la fortaleza del portal','bajo una biblioteca sin lectores','los ojos que señalan un destino'],
  ['dragon','el dragón del End','sobre la obsidiana del último combate','un final que deja espacio para otra aventura'],
  ['credits','los créditos del End','después de la última lluvia de experiencia','el universo hablando con quien acaba de jugar'],
  ['elytra','las élitros','en la proa de un barco del End','el instante en que el vacío se vuelve camino'],
  ['beacon','el faro','sobre una pirámide terminada al amanecer','una columna de luz para encontrar el hogar'],
  ['nether','el primer portal al Nether','frente a un resplandor violeta','ocho bloques de distancia por cada paso'],
  ['ghast','el Ghast','al escuchar un eco sobre la lava','una lágrima detrás de una bola de fuego'],
  ['strider','el Strider','sobre un océano de lava','un paseo que exige una caña muy particular'],
  ['pigstep','Pigstep','en las ruinas de un bastión','el ritmo de Lena Raine entre piedra negra'],
  ['ancientcity','la ciudad ancestral','debajo de una montaña sin nombre','un silencio que conviene respetar'],
  ['warden','el Warden','cuando se apaga la última antorcha','un paso que vale más que una espada'],
  ['allay','el Allay','cerca de un bloque de notas','un ayudante azul que reconoce su canción'],
  ['sniffer','el Sniffer','sobre una tierra llena de historia','una semilla que regresó de otro tiempo'],
  ['wolf','el lobo del primer viaje','junto a la puerta de tu refugio','un compañero que recuerda el camino a casa'],
  ['cat','el gato del almacén','sobre el cofre que querías abrir','un guardián con sus propias prioridades'],
  ['axolotl','el ajolote','en una cueva exuberante','un pequeño aliado que respira aventura'],
  ['villager','el aldeano bibliotecario','entre atriles y estanterías','un intercambio que puede cambiar la partida'],
  ['creeper','el Creeper','justo detrás de una obra terminada','un sonido que convierte a todos en velocistas'],
  ['charged','el Creeper cargado','bajo una tormenta demasiado cercana','un brillo azul que pide distancia'],
  ['steve','Steve','al despertar en una playa desconocida','el primer bloque de una historia sin guion'],
  ['alex','Alex','al cruzar un bosque al amanecer','otra mirada para el mismo mundo infinito'],
  ['dinnerbone','Dinnerbone','al leer una etiqueta con nombre','una forma diferente de mirar el cielo'],
  ['grumm','Grumm','en un corral que desafía la gravedad','el mundo del revés sin cambiar de semilla'],
  ['jeb','la oveja jeb_','en un prado de colores cambiantes','un arcoíris que cabe en una etiqueta'],
  ['toast','el conejo Toast','entre zanahorias y flores','una pequeña memoria convertida en compañero'],
  ['technoblade','el recuerdo de Technoblade','bajo una corona que no se oxida','Technoblade Never Dies'],
  ['stonecutter','el cortapiedras','en el taller de una gran construcción','el bloque exacto para el último detalle'],
  ['cobblestone','el generador de adoquín','entre una corriente de agua y lava','el comienzo de una isla que no deja de crecer'],
  ['skyblock','SkyBlock','sobre unos pocos bloques suspendidos','un mundo enorme construido a partir de casi nada'],
  ['hardcore','el corazón de Hardcore','al cerrar una puerta antes de la noche','una sola vida para una historia irrepetible'],
  ['f3','la pantalla F3','entre coordenadas y direcciones','los números que guían el regreso'],
];
const SPLASH_TEMPLATES:readonly ((subject:string,scene:string,detail:string)=>string)[]=[
  (s,p,d)=>`¿Recuerdas ${s}? Te espera ${p}: ${d}.`,
  (s,p,d)=>`Hay una historia sobre ${s} ${p}. Su pista es ${d}.`,
  (s,p,d)=>`Guarda una captura de ${s} ${p}; mañana recordarás ${d}.`,
  (s,p,d)=>`Un rumor viaja desde ${s}: ${d}. Búscalo ${p}.`,
  (s,p,d)=>`El diario de hoy empieza con ${s}, ${p}, y termina con ${d}.`,
  (s,p,d)=>`Si el tiempo se detiene ${p}, quizá sea por ${s}: ${d}.`,
  (s,p,d)=>`Hay mundos que se recuerdan por ${s}; otros, por ${d}. Nos vemos ${p}.`,
  (s,p,d)=>`Una aventura secreta: encuentra ${s} ${p} y conserva ${d}.`,
  (s,p,d)=>`El mapa no marca ${s}. Solo deja una nota ${p}: ${d}.`,
  (s,p,d)=>`El próximo refugio necesita una historia sobre ${s}, ${p}: ${d}.`,
  (s,p,d)=>`Mira alrededor ${p}. Quizá descubras ${s} y ${d}.`,
  (s,p,d)=>`Hoy el mundo susurra sobre ${s}. Escucha ${p}: ${d}.`,
  (s,p,d)=>`No todo está en las coordenadas: ${s} guarda ${d} ${p}.`,
  (s,p,d)=>`Si pudieras guardar un instante de ${s}, sería ${p}: ${d}.`,
  (s,p,d)=>`La siguiente página del álbum tiene ${s}, ${p}, y una frase: ${d}.`,
  (s,p,d)=>`Los veteranos reconocen ${s}; los exploradores encuentran ${d} ${p}.`,
  (s,p,d)=>`Bajo el mismo cielo, ${s} conecta otra partida con ${d}. El encuentro es ${p}.`,
  (s,p,d)=>`Lleva provisiones para buscar ${s}. ${p[0].toUpperCase()+p.slice(1)} aguarda ${d}.`,
  (s,p,d)=>`Un mundo, mil recuerdos: ${s}; una dirección: ${p}; una razón: ${d}.`,
  (s,p,d)=>`El servidor cerró, pero quedó una leyenda sobre ${s} ${p}: ${d}.`,
];


const MEMORY_TITLES:readonly string[]=[
  'El primer recuerdo','La historia que quedó','Una captura para mañana',
  'Rumores entre chunks','El diario del explorador','Un instante irrepetible',
  'Mundos que se encuentran','Tras una aventura secreta','La marca en el mapa',
  'Historias junto al refugio','Una mirada diferente','Lo que el mundo susurra',
  'Más allá de las coordenadas','El álbum del viajero','La página perdida',
  'El secreto de los veteranos','Bajo el mismo cielo','Provisiones para el viaje',
  'Mil razones para volver','La leyenda del último servidor',
];
const splashes:EasterEgg[]=REFERENCES.flatMap(([id,subject,scene,detail])=>SPLASH_TEMPLATES.map((template,index)=>Object.freeze({
  id:`splash:${id}:${['recuerdo','historia','captura','rumor','diario','instante','mundos','aventura','mapa','refugio','mirada','susurro','coordenadas','album','pagina','veteranos','cielo','provisiones','razon','leyenda'][index]}`,
  kind:'splash' as const,category:'Memorias de Minecraft',title:`${MEMORY_TITLES[index]} · ${subject}`,text:template(subject,scene,detail),chance:.005,trigger:'startup' as const,durationMs:14000,
})));
const appearances:EasterEgg[]=[
  {id:'appearance:herobrine',title:'Nunca estuvo aquí',text:'No había nadie entre los árboles. Al menos, eso decía el registro.',variant:'herobrine'},
  {id:'appearance:entity303',title:'El visitante del abrigo blanco',text:'Una sombra blanca cruzó el mundo. El servidor no registró su nombre.',variant:'entity303'},
  {id:'appearance:null',title:'Un nombre vacío',text:'La lista de jugadores contiene un espacio que nadie recuerda haber añadido.',variant:'null'},
  {id:'appearance:crown',title:'Una corona que permanece',text:'Technoblade Never Dies.',variant:'crowned_steve',item:'diamond_sword'},
  {id:'appearance:dinnerbone',title:'La gravedad tenía otros planes',text:'Dinnerbone pasó a saludar. Mira el mundo desde otra perspectiva.',variant:'dinnerbone',durationMs:9000},
  {id:'appearance:grumm',title:'El cielo bajo los pies',text:'Grumm comprobó la gravedad. El resultado fue inesperado.',variant:'grumm',durationMs:9000},
  {id:'appearance:charged-creeper',title:'Pronóstico: tormenta eléctrica',text:'Se recomienda observar este brillo azul desde una distancia prudente.',variant:'charged_creeper'},
  {id:'appearance:enderman',title:'Un bloque prestado',text:'No sabe quién dejó este bloque aquí. Pero piensa llevárselo de todos modos.',variant:'enderman',item:'grass_block'},
].map(egg=>Object.freeze({...egg,kind:'appearance' as const,category:'Visitantes extraordinarios',chance:.005,trigger:'startup' as const}) as EasterEgg);
const consoleSecrets:EasterEgg[]=[
  ['save-all','Guardar también es una aventura','[recuerdo] /save-all: que esta aventura tenga una próxima sesión.'],
  ['gamemode','Un mundo, cuatro miradas','[recuerdo] Supervivencia, Creativo, Aventura y Espectador: cambia la mirada, conserva la curiosidad.'],
  ['tp','Coordenadas del hogar','[recuerdo] /tp no conoce el camino de regreso que guardaste en la memoria.'],
  ['seed','La primera coordenada','[recuerdo] /seed: una cifra puede contener una historia infinita.'],
  ['time-set','El amanecer pendiente','[recuerdo] /time set day: algunos amaneceres también se construyen.'],
  ['weather','Lluvia sobre el refugio','[recuerdo] /weather clear: el cielo no es el único que necesitaba una pausa.'],
  ['locate','La brújula curiosa','[recuerdo] /locate: la aventura empieza después de encontrar la dirección.'],
  ['gamerule','Las reglas de tu mundo','[recuerdo] keepInventory no conserva los recuerdos; de eso te encargas tú.'],
  ['achievement','Logro desbloqueado','[recuerdo] Hacer inventario fue el primer logro de muchas grandes aventuras.'],
  ['punch-tree','El primer árbol','[recuerdo] Se recomienda empezar por un árbol. Las manos recordarán el resto.'],
  ['never-down','Una regla escrita en piedra','[recuerdo] Si vas a cavar hacia abajo, recuerda quién te lo advirtió.'],
  ['torch-right','El camino iluminado','[recuerdo] Las antorchas de la pared siguen contando el camino de vuelta.'],
  ['bed-explosion','Dormir en otra dimensión','[recuerdo] El Nether y el End tienen opiniones muy firmes sobre las camas.'],
  ['boat-ice','La autopista del hielo','[recuerdo] Ese bote no esperaba encontrar una pista de hielo en el Nether.'],
  ['water-nether','Agua en el Nether','[recuerdo] El cubo volvió vacío. La lección, no.'],
  ['portal-shape','El marco incompleto','[recuerdo] Las esquinas de obsidiana eran opcionales. La curiosidad nunca lo fue.'],
  ['music-off','El disco que faltaba','[recuerdo] Hay silencios que esperan el momento exacto para convertirse en música.'],
  ['dog-home','El compañero que espera','[recuerdo] Antes de salir del mundo, recuerda a quien dejaste sentado en casa.'],
  ['last-block','Solo un bloque más','[recuerdo] La frase «solo termino este techo» acaba de extender otra sesión.'],
  ['inventory-full','Inventario lleno','[recuerdo] No necesitas todas esas piedras. Pero tal vez el siguiente cofre sí.'],
  ['night-three','La tercera noche','[recuerdo] Dormir de vez en cuando también forma parte de un buen plan de exploración.'],
  ['spawnpoint','Punto de regreso','[recuerdo] /spawnpoint: un lugar puede ser especial antes de tener una casa.'],
  ['coordinates','Tres números importantes','[recuerdo] X, Y, Z. A veces basta con tres números para volver a casa.'],
  ['end-poem','El juego que sigue','[recuerdo] El portal final no es el final de tu mundo.'],
].map(([id,title,text])=>Object.freeze({id:`console:${id}`,kind:'console' as const,category:'Susurros de consola',title,text,chance:.005,trigger:'console' as const,durationMs:12000}));
const dates:EasterEgg[]=[
  ['minecraft-birthday','05-17','Un mundo que sigue creciendo','Un 17 de mayo comenzó una historia hecha de bloques. Gracias por seguir construyéndola.'],
  ['halloween','10-31','Una calabaza en la puerta','Esta noche conviene mirar dos veces antes de abrir el refugio.'],
  ['christmas-eve','12-24','Una luz en la ventana','Que siempre haya un refugio iluminado al terminar la aventura.'],
  ['christmas','12-25','Cofres para compartir','Hoy el mejor botín puede estar en la partida de tus amigos.'],
  ['new-year-eve','12-31','El último bloque del año','Antes de cerrar este mundo, deja espacio para la próxima historia.'],
  ['new-year','01-01','Una nueva semilla','Primer día, inventario vacío, posibilidades intactas.'],
  ['pi-day','03-14','Círculos a base de cuadrados','Hoy los círculos de tu construcción tienen una excusa matemática.'],
  ['earth-day','04-22','Un bioma para cuidar','Replanta ese árbol. Hasta un mundo de bloques merece un bosque.'],
  ['april-fools','04-01','Una actualización sospechosa','Si este mundo parece extraño, comprueba primero la fecha del calendario.'],
  ['palindrome-night','03:33','Una hora demasiado simétrica','03:33. La niebla está exactamente donde la dejaste. ¿Seguro?'],
  ['midnight','00:00','Un chunk entre dos días','Medianoche: el mundo sigue girando aunque todavía no hayas terminado el techo.'],
  ['sunrise','06:00','Ya vuelve la luz','Un amanecer real y otro de bloques: buen momento para guardar la aventura.'],
].map(([id,dateRule,title,text])=>Object.freeze({id:`date:${id}`,kind:'date' as const,category:'Calendario del mundo',dateRule,title,text,chance:.02,trigger:dateRule.includes(':')?'night' as const:'date' as const,durationMs:16000}));
const achievements:EasterEgg[]=[
  ['logo','logo','La marca tenía un secreto','Cinco toques despertaron los ojos blancos y las letras del End.'],
  ['studio','studio','Director de tu propio mundo','La siguiente miniatura empieza con una pose.'],
  ['console','console','Entre líneas','También hay aventuras esperando entre los registros.'],
  ['play','play','Otra partida, otra historia','El botón de jugar sigue siendo el mejor portal.'],
  ['skin','skin','Un rostro para tu aventura','Tu personaje está listo para un nuevo capítulo.'],
].map(([id,trigger,title,text])=>Object.freeze({id:`achievement:${id}`,kind:'achievement' as const,category:'Pequeños descubrimientos',trigger:trigger as SecretTrigger,title,text,chance:id==='logo'?.02:.005,durationMs:10000}));

const commandBlockSecret:EasterEgg=Object.freeze({
  id:'equipment:command-block',kind:'achievement',category:'Objetos extraordinarios',
  title:'Un bloque fuera del inventario',text:'Hay un bloque de comandos en tu mano. Este no aparece todos los días.',
  chance:.002,trigger:'startup',item:'command_block',leftItem:'none',
});
export const EASTER_EGGS:readonly EasterEgg[]=Object.freeze([...splashes,...appearances,...consoleSecrets,...dates,...achievements,commandBlockSecret]);
const BY_ID=new Map(EASTER_EGGS.map(egg=>[egg.id,egg]));
const STORAGE_KEY='launchertps.secrets.v1';
export type SecretState=Readonly<{discovered:readonly string[];count:number;total:number}>;
let discovered:Set<string>|undefined;
const subscribers=new Set<(state:SecretState)=>void>();
let listeningForStorage=false;
function loadDiscovered():Set<string>{
  if(discovered)return discovered;
  try{const value=JSON.parse(globalThis.localStorage?.getItem(STORAGE_KEY)??'[]');discovered=new Set(Array.isArray(value)?value.filter((id):id is string=>typeof id==='string'&&BY_ID.has(id)).slice(0,EASTER_EGGS.length):[]);}
  catch{discovered=new Set();}return discovered;
}
export function getSecretState():SecretState{const found=[...loadDiscovered()];return Object.freeze({discovered:Object.freeze(found),count:found.length,total:EASTER_EGGS.length});}
function broadcast(){const snapshot=getSecretState();for(const listener of subscribers)listener(snapshot);}
export function subscribeSecretState(listener:(state:SecretState)=>void):()=>void{
  subscribers.add(listener);listener(getSecretState());
  if(!listeningForStorage&&typeof window!=='undefined'){
    listeningForStorage=true;window.addEventListener('storage',event=>{if(event.key===STORAGE_KEY){discovered=undefined;broadcast();}});
  }
  return()=>subscribers.delete(listener);
}
export function discoverSecret(id:string):boolean{
  if(!BY_ID.has(id))return false;const state=loadDiscovered();if(state.has(id))return false;state.add(id);
  try{globalThis.localStorage?.setItem(STORAGE_KEY,JSON.stringify([...state]));}catch{ }
  broadcast();return true;
}
export function randomFraction():number{
  const sample=new Uint32Array(1);globalThis.crypto.getRandomValues(sample);return sample[0]/0x100000000;
}
function sample(random:()=>number){const value=random();return Number.isFinite(value)?Math.max(0,Math.min(.999999999,value)):.999999999;}
function choose(items:readonly EasterEgg[],random:()=>number):EasterEgg|null{return items.length?items[Math.floor(sample(random)*items.length)]:null;}
export function dateMatches(rule:string,now:Date):boolean{
  const pad=(value:number)=>String(value).padStart(2,'0');
  return rule.includes(':')?rule===`${pad(now.getHours())}:${pad(now.getMinutes())}`:rule===`${pad(now.getMonth()+1)}-${pad(now.getDate())}`;
}

export function rollStartupEgg(random:()=>number=randomFraction,now=new Date()):EasterEgg{
  const gate=sample(random);

  if(gate<.001)return appearances[0];

  if(gate<.003)return commandBlockSecret;
  const kind=sample(random);
  if(kind<.30)return choose(appearances.slice(1),random)??splashes[0];
  const calendar=dates.filter(egg=>dateMatches(egg.dateRule!,now));
  if(kind>.92&&calendar.length)return choose(calendar,random)??splashes[0];
  if(kind>.84)return choose(consoleSecrets,random)??splashes[0];
  return choose(splashes,random)??splashes[0];
}

export function onSecretInteraction(trigger:SecretTrigger,random:()=>number=randomFraction):EasterEgg|null{
  const candidates=achievements.filter(egg=>egg.trigger===trigger);
  if(!candidates.length)return null;

  if(trigger!=='logo'&&sample(random)>=.01)return null;
  const egg=choose(candidates,random);if(egg)discoverSecret(egg.id);return egg;
}

export function startEggClock(callback?:(egg:EasterEgg)=>void,random:()=>number=randomFraction):()=>void{
  if(typeof window==='undefined')return()=>{};
  let lastMinute='';const tried=new Set<string>();
  const tick=()=>{
    const now=new Date();const minute=`${now.getFullYear()}-${now.getMonth()}-${now.getDate()}-${now.getHours()}-${now.getMinutes()}`;
    if(minute===lastMinute)return;lastMinute=minute;
    const eligible=dates.filter(egg=>dateMatches(egg.dateRule!,now)&&!tried.has(egg.id));
    if(!eligible.length)return;for(const egg of eligible)tried.add(egg.id);
    if(sample(random)>=.02)return;const egg=choose(eligible,random);if(egg){discoverSecret(egg.id);callback?.(egg);}
  };
  tick();const timer=window.setInterval(tick,15000);return()=>window.clearInterval(timer);
}
