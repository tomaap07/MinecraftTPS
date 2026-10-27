
export function categoryFilter(input:string):string{
  const aliases:Record<string,string>={
    all:'all',survival:'Survival',pvp:'PvP',minigames:'Minijuegos',rpg:'RPG',mmorpg:'RPG',
    anarchy:'Anarquía',anarquia:'Anarquía',skyblock:'SkyBlock',boxpvp:'BoxPvP',
    smp:'SMP',prison:'Prison',factions:'Factions',parkour:'Parkour',pixelmon:'Pixelmon',
    economy:'Economía',hardcore:'Hardcore',lifesteal:'Lifesteal',towny:'Towny'
  };
  return aliases[input.toLowerCase()]??input;
}
