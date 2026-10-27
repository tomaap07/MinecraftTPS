import { EASTER_EGGS, type EasterEgg } from './easter_eggs';

export type SecretDifficulty = 'easy' | 'normal' | 'hard' | 'legendary';
export type SecretIconName = 'trophy' | 'lock' | 'spark' | 'sword' | 'block' | 'music' | 'disc' | 'portal' | 'skull' | 'eyes' | 'crown' | 'creeper' | 'enderman' | 'tree' | 'compass' | 'heart' | 'moon' | 'sun' | 'calendar' | 'terminal' | 'palette' | 'person' | 'play' | 'logo' | 'book' | 'redstone' | 'feather' | 'potion' | 'map' | 'chest';

export const SECRET_DIFFICULTIES: ReadonlyArray<{ id: SecretDifficulty; label: string; explanation: string }> = [
  { id: 'easy', label: 'Fácil', explanation: 'Interacciones que puedes probar mientras usas el launcher.' },
  { id: 'normal', label: 'Normal', explanation: 'Recuerdos y mensajes elegidos al iniciar el launcher.' },
  { id: 'hard', label: 'Difícil', explanation: 'Visitantes y encuentros vinculados a momentos especiales.' },
  { id: 'legendary', label: 'Legendario', explanation: 'Las apariciones más raras de la colección.' },
];

export function secretDifficulty(egg: EasterEgg): SecretDifficulty {
  if (egg.id === 'appearance:herobrine' || egg.id === 'equipment:command-block') return 'legendary';
  if (egg.kind === 'appearance' || egg.kind === 'date') return 'hard';
  if (egg.kind === 'achievement' && egg.trigger !== 'startup') return 'easy';
  return 'normal';
}

export function secretHint(egg: EasterEgg): string {
  if (egg.id === 'appearance:herobrine') return 'Puede aparecer al iniciar el launcher: 1 de cada 1.000 inicios, en promedio. Cada inicio es independiente.';
  if (egg.id === 'equipment:command-block') return 'Puede equiparse al iniciar el launcher: 1 de cada 500 inicios, en promedio. Cada inicio es independiente.';
  if (egg.id === 'achievement:logo') return 'Pulsa cinco veces seguidas el logo del launcher.';
  if (egg.id === 'achievement:studio') return 'Abre el estudio 3D de Poser. La interacción puede revelar este recuerdo; no se activa en cada visita.';
  if (egg.id === 'achievement:console') return 'Visita la consola. La interacción puede revelar este recuerdo; no se activa en cada visita.';
  if (egg.id === 'achievement:play') return 'Inicia una partida. La interacción puede revelar este recuerdo; no se activa en cada lanzamiento.';
  if (egg.id === 'achievement:skin') return 'Visita tu cuenta y tu personaje. La interacción puede revelar este recuerdo; no se activa en cada visita.';
  if (egg.kind === 'date' && egg.dateRule) {
    if (egg.dateRule.includes(':')) return `Mantén abierto el launcher a las ${egg.dateRule}, según la hora de tu equipo. El encuentro es aleatorio.`;
    const [month, day] = egg.dateRule.split('-').map(Number);
    const months = ['enero', 'febrero', 'marzo', 'abril', 'mayo', 'junio', 'julio', 'agosto', 'septiembre', 'octubre', 'noviembre', 'diciembre'];
    return `Visita el launcher el ${day} de ${months[month - 1]}. El encuentro es aleatorio.`;
  }
  if (egg.kind === 'appearance') return 'Observa al personaje de bienvenida al iniciar. El visitante se elige al azar.';
  if (egg.kind === 'console') return 'Lee los mensajes del launcher al iniciar. Este susurro puede aparecer entre ellos.';
  return 'Lee el recuerdo que te recibe al iniciar el launcher. La selección cambia en cada inicio.';
}


export function secretIcon(egg: EasterEgg): SecretIconName {
  const id = egg.id;
  if (id === 'achievement:logo') return 'logo';
  if (id === 'achievement:studio') return 'palette';
  if (id === 'achievement:console') return 'terminal';
  if (id === 'achievement:play') return 'play';
  if (id === 'achievement:skin') return 'person';
  if (/herobrine|entity303|:null/.test(id)) return 'eyes';
  if (/command-block/.test(id)) return 'block';
  if (/crown|technoblade/.test(id)) return 'crown';
  if (/charged|creeper/.test(id)) return 'creeper';
  if (/enderman/.test(id)) return 'enderman';
  if (/disceleven|disc13/.test(id)) return 'disc';
  if (/c418|sweden|wet_hands|mice_on_venus|pigstep|allay|music-off/.test(id)) return 'music';
  if (/aether|nether|stronghold|portal/.test(id)) return 'portal';
  if (/hardcore|wolf|cat|dog-home|axolotl/.test(id)) return 'heart';
  if (/dragon|warden|ancientcity|halloween/.test(id)) return 'skull';
  if (/redstone|repeater|observer|comparator|piston/.test(id)) return 'redstone';
  if (/elytra|ghast/.test(id)) return 'feather';
  if (/blazerod|water-nether|mlg/.test(id)) return 'potion';
  if (/farlands|f3|coordinates|seed|spawnpoint|speedrun|locate|:tp/.test(id)) return 'compass';
  if (/packpng|alpha|earth-day|punch-tree/.test(id)) return 'tree';
  if (/hopper|inventory|christmas|last-block/.test(id)) return 'chest';
  if (/night|midnight/.test(id)) return 'moon';
  if (/sunrise|time-set|weather/.test(id)) return 'sun';
  if (egg.kind === 'date') return 'calendar';
  if (/steve|alex|villager|dinnerbone|grumm|jeb|toast/.test(id)) return 'person';
  if (/credits|end-poem|gamerule|achievement/.test(id)) return 'book';
  if (/stonecutter|cobblestone|skyblock|beacon|never-down/.test(id)) return 'block';
  if (/gamemode/.test(id)) return 'sword';
  if (egg.kind === 'console') return 'terminal';
  return 'map';
}


export function secretGameIcon(egg: EasterEgg): string {
  if (/command-block/.test(egg.id)) return 'command_block_front';
  if (/disceleven/.test(egg.id)) return 'music_disc_11';
  if (/disc13/.test(egg.id)) return 'music_disc_13';
  if (/pigstep/.test(egg.id)) return 'music_disc_pigstep';
  if (/elytra/.test(egg.id)) return 'feather';
  if (/nether|portal/.test(egg.id)) return 'obsidian';
  if (/beacon/.test(egg.id)) return 'nether_star';
  if (/hopper/.test(egg.id)) return 'hopper';
  return secretIcon(egg);
}

export const ACHIEVEMENT_CATEGORIES = Object.freeze([...new Set(EASTER_EGGS.map(egg => egg.category))]);


export function secretCipher(id: string): string {
  let hash = 2166136261;
  for (const char of id) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619);
  const glyphs = '░▒▓?';
  return Array.from({ length: 13 }, (_, index) => {
    hash = Math.imul(hash ^ (index + 1), 16777619);
    return index === 4 || index === 8 ? ' ' : glyphs[(hash >>> 0) % glyphs.length];
  }).join('');
}
