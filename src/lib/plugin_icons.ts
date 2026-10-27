export type PluginIconSpec={vector:string;texture?:string};



export const PLUGIN_ICONS:Readonly<Record<string,PluginIconSpec>>={
 entityculling:{vector:'eye-off'},
 immediatelyfast:{vector:'bolt'},
 dynamicfps:{vector:'monitor'},
 iris:{vector:'sun'},
 no_fog:{vector:'wind'},
 item_physics:{vector:'hand',texture:'diamond_sword'},
 days:{vector:'calendar'},
 biome:{vector:'leaf',texture:'grass_block'},
 fps:{vector:'gauge'},
 coordinates:{vector:'locate'},
 keystrokes:{vector:'keyboard'},
 armor:{vector:'armor',texture:'diamond_chestplate'},
 clock:{vector:'clock',texture:'clock'},
 ping:{vector:'network'},
 direction:{vector:'compass',texture:'compass'},
 minimap:{vector:'map',texture:'map'},
 custom_font:{vector:'text'},
 transparent_menu:{vector:'layers'},
 enhanced_chat:{vector:'chat'},
 optical_zoom:{vector:'search',texture:'spyglass'},
 sodium:{vector:'monitor'},
 lithium:{vector:'leaf',texture:'slime_ball'},
 ferritecore:{vector:'memory',texture:'iron_ingot'},
 skin_layers:{vector:'layers'},
 wavey_capes:{vector:'wind',texture:'feather'},
 old_animations:{vector:'rewind'},
 freelook:{vector:'camera'},
 block_outline:{vector:'scan'},
 motion_blur:{vector:'wind'},
 low_fire:{vector:'flame'},
 clean_view:{vector:'eye-off'},
 hit_color:{vector:'target'},
 particle_multiplier:{vector:'sparkles'},
 self_nametag:{vector:'tag',texture:'name_tag'},
 ragdolls:{vector:'pose'},
 tnt_timer:{vector:'bomb',texture:'tnt_block'},
 light_levels:{vector:'sun',texture:'torch'},
 m_health:{vector:'heart'},
 m_food:{vector:'food',texture:'cooked_beef'},
 m_saturation:{vector:'food',texture:'golden_carrot'},
 m_oxygen:{vector:'bubbles'},
 m_xp_level:{vector:'trophy',texture:'experience_bottle'},
 m_inventory_free:{vector:'inventory',texture:'chest'},
 m_count_arrow:{vector:'bow',texture:'arrow'},
 m_count_ender_pearl:{vector:'globe',texture:'ender_pearl'},
 m_count_golden_apple:{vector:'food',texture:'enchanted_golden_apple'},
 m_count_totem_of_undying:{vector:'shield',texture:'totem'},
 m_count_firework_rocket:{vector:'rocket',texture:'firework_rocket'},
 m_attack_cooldown:{vector:'sword',texture:'iron_sword'},
 m_block_light:{vector:'sun',texture:'lantern'},
 m_sky_light:{vector:'sun'},
 m_nether_coords:{vector:'globe',texture:'obsidian'},
 m_chunk_coords:{vector:'grid'},
 m_chunk_local:{vector:'locate'},
 m_world_dimension:{vector:'globe'},
 m_target_distance:{vector:'ruler'},
 m_combo:{vector:'sword'}
};

export function pluginIconSpec(id:string):PluginIconSpec{
 return PLUGIN_ICONS[id]??{vector:'plugins'};
}


export function resolvePluginIcon(id:string,textures:Record<string,string>,failedTexture=''):{vector:string;texture:string|null}{
 const spec=pluginIconSpec(id),texture=spec.texture?textures[spec.texture]:undefined;
 return{vector:spec.vector,texture:texture?.startsWith('data:image/png;base64,')&&texture!==failedTexture?texture:null};
}
