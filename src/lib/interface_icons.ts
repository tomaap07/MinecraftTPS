
type InterfaceIcon = { ui: string; pixel?: 'dimension' | 'release' | 'snapshot' | 'legacy' };
const aliases: Record<string, string> = {
  diamond_sword:'play', sword:'play', netherite_sword:'play', stone:'cube', block:'cube',
  diamond:'trophy', emerald:'trophy', trophy:'trophy', spark:'trophy', nether_star:'trophy',
  crafting_table:'grid', grid:'grid', name_tag:'user', person:'user', armor_stand:'pose',
  painting:'image', palette:'image', redstone:'chip', redstone_torch:'settings',
  command_block_front:'terminal', book:'folder', chest:'folder', chest_minecart:'folder',
  hopper:'download', feather:'pose', map:'globe', filled_map:'globe', compass:'globe',
  clock:'clock', calendar:'clock', sky_sun:'clock', sky_moon:'clock', sun:'clock', moon:'clock',
  creeper:'chip', enderman:'cube', eyes:'eye', skull:'shield', crown:'trophy', heart:'trophy',
  music:'volume', disc:'volume', music_disc_13:'volume', music_disc_11:'volume',
};
export function interfaceIcon(name: string): InterfaceIcon {
  if (['grass_block','grass_side','logo'].includes(name)) return { ui:'cube', pixel:'release' };
  if (['stone_block','obsidian','portal','ender_eye'].includes(name)) return { ui:'cube', pixel:'dimension' };
  if (['cobblestone','cobblestone_block'].includes(name)) return { ui:'cube', pixel:'legacy' };
  if (name === 'potion') return { ui:'chip', pixel:'snapshot' };
  return { ui: aliases[name] ?? name };
}
