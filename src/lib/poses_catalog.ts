
import type { HeldItemId } from './held_items';
export type Vec3 = [number, number, number];
export const JOINTS = ["head", "body", "rightArm", "leftArm", "rightLeg", "leftLeg", "cape"] as const;
export type Joint = typeof JOINTS[number];
export interface Pose { id: string; name: string; category: string; joints: Record<Joint, Vec3>; position: Vec3; rotation: Vec3; equipment?: { right: HeldItemId; left: HeldItemId } }
export const POSE_CATEGORIES = ["Presentación y bienvenida", "Miniaturas y épicas", "Combate y PvP", "Celebración y emotes", "Parkour y acción", "Descanso y chill", "Supervivencia y construcción", "Divertidas y dramáticas"];
export const JOINT_LABELS: Record<Joint, string> = {head:"Cabeza",body:"Torso",rightArm:"Brazo derecho",leftArm:"Brazo izquierdo",rightLeg:"Pierna derecha",leftLeg:"Pierna izquierda",cape:"Capa"};
type Row = [string, Vec3, Vec3, Vec3, Vec3, Vec3, Vec3, Vec3?, Vec3?, Vec3?];
const Z: Vec3 = [0,0,0];



const legacyRows: [string, number, Row][] = [
  ["pose-1-1", 0, ["Hola, aventurero",[-5,12,-5],Z,[-155,5,-22],[-8,0,8],[0,0,-4],[0,0,4]]],
  ["pose-1-4", 0, ["Brazos abiertos",[-8,0,0],Z,[-20,0,-65],[-20,0,65],[0,0,-8],[0,0,8]]],
  ["pose-1-14", 0, ["Listo para empezar",[-5,0,0],[-5,0,0],[-20,0,-12],[-20,0,12],[-10,0,-8],[10,0,8]]],
  ["pose-1-15", 0, ["Bienvenida elegante",[8,-15,-4],[12,-8,0],[-42,-25,-15],[15,0,12],[0,0,-3],[-12,0,4]]],
  ["pose-2-3", 1, ["Apuntando a cámara",[-5,-10,0],[0,-12,0],[-93,8,-8],[10,0,12],[-6,0,-8],[6,0,8]]],
  ["pose-2-4", 1, ["Héroe del horizonte",[-18,35,0],[0,12,0],[-10,0,-15],[-35,0,20],[0,0,-12],[-8,0,12],[50,8,0]]],
  ["pose-2-9", 1, ["Rey del servidor",[-10,0,0],[0,15,0],[-65,-15,-25],[-65,15,25],[0,0,-15],[0,0,15],[30,0,0]]],
  ["pose-2-12", 1, ["Mirada de portada",[0,-30,0],[0,25,0],[-10,0,-8],[5,0,12],[0,0,-5],[0,0,5],[20,0,0]]],
  ["pose-2-15", 1, ["Invencible",[-12,10,0],[-6,-10,0],[-15,-20,-28],[-15,20,28],[-10,0,-18],[10,0,18],[55,0,0]]],
  ["pose-3-1", 2, ["Guardia de espada",[5,-25,0],[8,-8,0],[-75,0,-22],[-30,0,20],[-22,0,-6],[28,0,6]]],
  ["pose-3-3", 2, ["Bloqueo con escudo",[0,15,0],[5,8,0],[-40,0,-18],[-68,0,12],[-10,0,-6],[20,0,6]]],
  ["pose-3-14", 2, ["Puño directo",[5,0,0],[6,-12,0],[-95,0,-8],[-32,0,16],[-20,0,-6],[26,0,6]]],
  ["pose-4-1", 3, ["Puño al cielo",[-20,15,0],[-10,0,-8],[-175,0,-15],[-30,0,25],[-10,0,-10],[15,0,10]]],
  ["pose-4-12", 3, ["Saludo de victoria",[-12,-20,0],[-6,-10,0],[-175,-10,-40],[-45,0,15],[0,0,-8],[-25,0,8]]],
  ["pose-6-4", 5, ["Dormitando",[28,0,12],[8,0,4],[8,0,-5],[-5,0,5],[0,0,-3],[8,0,3]]],
  ["pose-6-5", 5, ["Apoyado de espaldas",[-10,15,0],[-8,0,0],[15,0,-15],[15,0,15],[0,0,-5],[-25,0,8]]],
  ["pose-6-7", 5, ["Paseo tranquilo",[-4,20,0],Z,[-20,0,-5],[20,0,5],[18,0,-3],[-18,0,3]]],
  ["pose-6-8", 5, ["Descanso merecido",[8,-20,4],[0,0,-3],[-50,0,-16],[-8,0,12],[0,0,-4],[-20,0,4]]],
  ["pose-7-7", 6, ["Pescando",[0,-15,0],[5,0,0],[-65,-15,-15],[-35,20,15],[0,0,-8],[15,0,8]]],
  ["pose-8-4", 7, ["Estatua solemne",[-5,0,0],Z,[-5,0,-3],[-5,0,3],[0,0,-1],[0,0,1]]],
];





type CuratedRow = [string, number, number, number, number, number, number, number, number, HeldItemId?, HeldItemId?, number?, number?];
const curatedRows: CuratedRow[][] = [
  [
    ["Un nuevo comienzo",-4,8,-12,-8,-8,9,-3,3],
    ["Saludo desde el sendero",-6,-12,-145,-18,-12,7,-8,8],
    ["Bienvenido al campamento",-2,16,-35,-34,-28,32,0,0,'torch'],
    ["Tu próxima expedición",-7,-18,-68,-18,-10,10,-10,12,'diamond_pickaxe'],
    ["La primera herramienta",8,12,-38,-14,-6,7,0,-6,'wooden_sword'],
    ["Una luz para el camino",-10,-8,-115,-18,-15,9,-5,5,'lantern'],
    ["Un regalo de bienvenida",7,0,-64,-18,-20,13,0,0,'enchanted_golden_apple'],
    ["Todo listo en la mochila",-3,20,-20,-13,-48,19,6,-6,'iron_sword','totem'],
    ["El guía del refugio",-5,-20,-45,-26,-17,11,-9,4,'torch','none',2,-4],
    ["Saludo con el pico",-8,10,-120,-16,-14,10,-4,7,'diamond_pickaxe'],
    ["Al otro lado del portal",-4,24,-30,-16,-57,17,0,8,'ender_pearl'],
    ["La aventura te espera",-9,-14,-54,-24,-30,21,-14,14,'diamond_sword','none',-3,3],
    ["Bienvenida del constructor",2,8,-58,-20,-13,10,-7,5,'oak_planks_block'],
  ],
  [
    ["Guardián del amanecer",-12,24,-24,-13,-35,14,-8,10,'netherite_sword','totem',-2,-5],
    ["La última antorcha",-8,-26,-132,-20,-22,12,6,-8,'torch','none',0,5],
    ["Tesoro descubierto",12,-8,-65,-20,-28,18,-4,4,'diamond_block','none',4,0],
    ["Maza del desafío",-10,18,-76,-18,-25,15,-12,14,'mace','none',0,-4],
    ["Custodio del océano",-14,-22,-42,-18,-26,19,3,-7,'trident','none',-3,4],
    ["Centinela de la fortaleza",-5,12,-35,-16,-62,12,-7,8,'iron_sword','shield',3,-7],
    ["Rumbo al End",-12,30,-48,-21,-28,16,-10,12,'ender_pearl','totem',-2,-8],
    ["El filo de la leyenda",-7,-16,-110,-18,-18,14,5,-9,'diamond_sword','none',0,6],
    ["Arquero del horizonte",-8,25,-48,-24,-20,10,-6,8,'bow','none',0,-7],
    ["Un bloque de posibilidades",1,-18,-78,-22,-16,12,3,-5,'grass_block','none',2,5],
    ["Viajero bajo las estrellas",-18,8,-16,-12,-64,18,-5,7,'lantern','ender_pearl',-2,-3],
    ["El trofeo de la expedición",-11,-6,-142,-19,-25,15,-9,11,'diamond_axe','totem',0,4],
    ["Dos mundos por explorar",-5,20,-39,-25,-43,23,0,0,'stone_block','oak_planks_block',0,-6],
  ],
  [
    ["Espada en guardia",0,-14,-58,-15,-25,16,-16,18,'diamond_sword','none',5,4],
    ["Escudo preparado",2,16,-40,-18,-68,12,-12,14,'iron_sword','shield',4,-7],
    ["Defensa de netherite",-3,-10,-67,-20,-58,14,-18,16,'netherite_sword','shield',5,5],
    ["El hacha del guardián",0,12,-72,-18,-30,14,-13,15,'diamond_axe','totem',3,-5],
    ["Maza lista para el duelo",3,-12,-92,-20,-32,20,-14,18,'mace','none',4,4],
    ["Vigía con ballesta",-2,22,-73,-14,-22,13,-10,13,'crossbow_arrow','none',3,-6],
    ["Patrulla con arco",-4,-24,-46,-22,-16,11,-20,18,'bow','none',2,7],
    ["Lanza del arrecife",0,15,-82,-23,-29,18,-15,19,'trident','none',3,-5],
    ["La reserva del tótem",4,8,-54,-16,-55,18,-9,12,'netherite_axe','totem',4,-3],
    ["La primera escaramuza",1,-18,-69,-19,-29,17,-21,17,'wooden_sword','none',6,5],
    ["Vigilar el flanco",-2,32,-47,-18,-60,13,8,-12,'diamond_sword','shield',2,-9],
    ["Paso de espadachín",0,-8,-78,-21,-27,16,-24,22,'iron_sword','none',5,3],
    ["Guardia de la armería",-3,10,-33,-15,-52,12,0,0,'crossbow','none',1,-3],
    ["Avance con escudo",2,-16,-56,-17,-74,14,-22,20,'diamond_axe','shield',6,5],
    ["Perla para la retirada",3,20,-65,-16,-34,19,14,-16,'ender_pearl','totem',1,-6],
    ["Duelo al atardecer",-5,-26,-96,-20,-23,15,-17,21,'netherite_sword','none',3,8],
  ],
  [
    ["Brindis con manzana dorada",-8,12,-125,-20,-17,13,2,-4,'enchanted_golden_apple'],
    ["Diamante conseguido",-10,-12,-138,-21,-30,20,-7,9,'diamond_block'],
    ["Victoria del constructor",-6,8,-118,-24,-100,23,4,-6,'oak_planks_block'],
    ["Un saludo al equipo",-3,-5,-106,-35,-103,34,-3,3],
    ["Baile junto a la hoguera",-5,18,-85,-28,-46,28,-20,18,'torch','none',0,-3],
    ["Meta alcanzada",-9,-14,-150,-18,-70,21,-10,10],
    ["El pescador afortunado",-4,8,-113,-25,-20,14,-2,6,'fishing_rod'],
    ["Un tótem para celebrarlo",-8,-9,-140,-22,-44,19,5,-8,'totem'],
    ["Alegría del minero",-11,13,-125,-19,-113,26,-9,7,'diamond_pickaxe','torch'],
    ["La victoria del arquero",-6,-18,-112,-29,-35,18,-6,10,'bow'],
    ["Saludo del campeón",-7,16,-144,-16,-39,15,1,-5,'none','netherite_sword'],
    ["Conquista compartida",-5,0,-128,-26,-134,24,0,0,'diamond_sword','totem'],
  ],
  [
    ["Paso ligero",-5,8,-34,-10,24,10,25,-25,'none','none',4,0],
    ["Comienzo de la carrera",-7,-8,29,-11,-38,12,-28,26,'none','none',6,2],
    ["Perla en movimiento",-3,18,-66,-18,20,12,-23,21,'ender_pearl','none',5,-4],
    ["Paseo por las copas",-8,-15,-24,-25,-30,28,18,-18,'none','none',3,3],
    ["Equilibrio tranquilo",2,12,-16,-48,-20,46,0,-22,'none','none',0,-3],
    ["Sendero de la antorcha",-4,-12,-72,-19,25,10,22,-24,'torch','none',4,3],
    ["Atravesando el valle",-6,20,24,-12,-35,13,-26,25,'diamond_pickaxe','none',5,-5],
    ["Llegada al refugio",-3,-6,-39,-14,15,11,14,-18,'lantern','none',2,2],
    ["Un salto pequeño",-7,0,-64,-20,-60,21,-18,-15,'none','none',3,0],
    ["El próximo peldaño",5,-10,-46,-17,18,13,-29,8,'none','none',4,2],
    ["Cruzar el puente",3,14,-26,-39,-32,37,19,-17,'none','none',2,-4],
    ["Hacia la aventura",-6,-20,-42,-15,28,12,28,-27,'iron_sword','none',6,5],
  ],
  [
    ["Pausa al atardecer",-12,18,-7,-9,-13,10,0,-7],
    ["El silencio del lago",-4,-24,-27,-14,-12,9,-4,6,'fishing_rod'],
    ["Mirar las estrellas",-22,12,-14,-11,-17,12,2,-4],
    ["Una manzana y un descanso",10,-5,-48,-17,-8,9,0,8,'enchanted_golden_apple'],
    ["Luz cálida en casa",-3,16,-23,-13,-47,15,-5,4,'none','lantern'],
    ["Conversación tranquila",0,-15,-31,-19,-20,17,6,-5],
    ["La calma tras el combate",4,10,-16,-14,-25,15,0,7,'iron_sword','shield'],
    ["Contemplar la construcción",-10,-28,-10,-10,-16,11,-6,4,'stone_block'],
    ["Un paseo sin prisa",-2,14,-17,-8,14,9,16,-16],
    ["Refugio bajo la linterna",-15,-8,-76,-16,-9,8,3,-5,'lantern'],
  ],
  [
    ["Primer bloque de la casa",10,-10,-62,-19,-20,13,-6,8,'oak_planks_block','none',4,3],
    ["Muro de adoquín",6,15,-76,-23,-24,18,-8,10,'cobblestone_block','none',3,-4],
    ["Pico en la entrada de la mina",-6,-15,-57,-18,-33,16,-12,10,'diamond_pickaxe','torch',3,4],
    ["Inspeccionar el siguiente bloque",16,8,-50,-20,-19,12,-5,6,'stone_block','none',5,-2],
    ["Un jardín por construir",12,-18,-72,-21,-17,12,-11,9,'grass_block','none',4,5],
    ["Iluminar la nueva habitación",-12,14,-122,-20,-26,17,-5,8,'torch','none',0,-4],
    ["Preparando la cantera",9,-9,-84,-19,-30,18,-15,13,'diamond_pickaxe','none',6,3],
    ["El carpintero del poblado",4,20,-46,-18,-26,15,-7,9,'diamond_axe','oak_planks_block',2,-6],
    ["Cofre lleno de diamantes",12,-7,-67,-22,-40,20,0,5,'diamond_block','none',3,2],
    ["Obras en la fortaleza",-5,18,-98,-24,-25,17,-12,14,'stone_block','none',1,-5],
    ["El farero del puerto",-8,-22,-109,-18,-16,12,-2,6,'lantern','none',0,6],
    ["Materiales para el puente",7,10,-58,-23,-43,21,-7,8,'oak_planks_block','cobblestone_block',4,-3],
    ["Pescando desde la orilla",3,-20,-74,-16,-22,15,-4,9,'fishing_rod','none',2,5],
    ["La antorcha del explorador",-10,26,-91,-23,-34,18,-10,12,'torch','diamond_pickaxe',2,-7],
  ],
  [
    ["¿Ese bloque hacía ruido?",-3,28,-65,-25,-30,22,5,-8,'tnt_block'],
    ["La manzana era para después",14,-12,-52,-20,-24,18,0,6,'enchanted_golden_apple'],
    ["¿Dónde dejé la perla?",10,24,-60,-18,-32,20,-4,7,'ender_pearl'],
    ["El tótem tiene un plan",8,-18,-33,-19,-70,21,4,-6,'none','totem'],
    ["Un diamante muy sospechoso",15,12,-75,-24,-25,16,-7,8,'diamond_block'],
    ["No era una espada de juguete",6,-22,-82,-21,-36,23,6,-10,'wooden_sword'],
    ["La última flecha",3,18,-58,-22,-55,20,0,5,'bow','arrow'],
    ["Constructor en apuros",4,-26,-88,-27,-68,25,-12,11,'grass_block'],
    ["Todo bajo control, casi",-4,14,-113,-26,-44,20,-9,12,'torch','shield'],
    ["¿Quién apagó la cueva?",-5,-30,-118,-22,-39,20,8,-11,'lantern','diamond_pickaxe'],
  ],
];

function makePose(id: string, category: number, row: Row, equipment?: Pose['equipment']): Pose {
  return {
    id, name: row[0], category: POSE_CATEGORIES[category],
    joints: { head: [...row[1]], body: [...row[2]], rightArm: [...row[3]], leftArm: [...row[4]], rightLeg: [...row[5]], leftLeg: [...row[6]], cape: [...(row[7] ?? [12,0,0])] },
    position: [...(row[8] ?? Z)], rotation: [...(row[9] ?? [0,-18,0])],
    ...(equipment ? { equipment: { ...equipment } } : {}),
  };
}

export const POSES: Pose[] = [
  ...legacyRows.map(([id, category, row]) => makePose(id, category, row)),
  ...curatedRows.flatMap((rows, category) => rows.map((row, index) => {
    const [name, hx, hy, rx, rz, lx, lz, rightStride, leftStride, right = 'none', left = 'none', bodyX = 0, bodyY = 0] = row;
    return makePose(`curated-${category + 1}-${index + 1}`, category,
      [name, [hx,hy,0], [bodyX,bodyY,0], [rx,0,rz], [lx,0,lz], [rightStride,0,-4], [leftStride,0,4], [16,0,0]], { right, left });
  })),
];
export const WELCOME_POSE = POSES.find(pose => pose.id === 'curated-1-1')!;
export function clonePose(pose: Pose): Pose { return JSON.parse(JSON.stringify(pose)) as Pose; }
export const FALLBACK_SKIN = "/skins/tp-explorer.png";
export const FALLBACK_AVATAR = "/skins/tp-explorer-face.png";
