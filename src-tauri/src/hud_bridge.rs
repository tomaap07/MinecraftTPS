
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

const CLIENT: &str = "net.minecraft.client.Minecraft";
const MAX_MAPPING: usize = 48 * 1024 * 1024;
#[derive(Clone, Debug)]
struct Binding {
    owner: String,
    name: String,
    descriptor: String,
}
type Bindings = BTreeMap<String, Binding>;

const ROLES: &[(&str, &str, &str, bool)] = &[
    ("client.gui","net.minecraft.client.Minecraft","gui",false),
    ("client.screen","net.minecraft.client.Minecraft","screen",false),
    ("client.window","net.minecraft.client.Minecraft","getWindow",true),
    ("window.fullscreen","com.mojang.blaze3d.platform.Window","isFullscreen",true),
    ("window.handle","com.mojang.blaze3d.platform.Window","getWindow",true),
    ("window.handle","com.mojang.blaze3d.platform.Window","handle",true),
    ("window.toggle_fullscreen","com.mojang.blaze3d.platform.Window","toggleFullScreen",true),
    ("screen.children","net.minecraft.client.gui.screens.Screen","children",true),
    ("widget.message","net.minecraft.client.gui.components.AbstractWidget","getMessage",true),
    ("component.contents","net.minecraft.network.chat.Component","getContents",true),
    ("translation.key","net.minecraft.network.chat.contents.TranslatableContents","getKey",true),
    ("button.press","net.minecraft.client.gui.components.Button","onPress",true),
    ("item.translation","net.minecraft.world.item.Item","getDescriptionId",true),
    ("stack.item","net.minecraft.world.item.ItemStack","getItem",true),
    ("stack.count","net.minecraft.world.item.ItemStack","getCount",true),
    ("player.mainhand","net.minecraft.world.entity.LivingEntity","getMainHandItem",true),
    ("player.offhand","net.minecraft.world.entity.LivingEntity","getOffhandItem",true),
    ("inventory.size","net.minecraft.world.entity.player.Inventory","getContainerSize",true),
    ("inventory.stack","net.minecraft.world.entity.player.Inventory","getItem",true),
    ("world.brightness","net.minecraft.world.level.BlockAndTintGetter","getBrightness",true),
    ("light.block","net.minecraft.world.level.LightLayer","BLOCK",false),
    ("light.sky","net.minecraft.world.level.LightLayer","SKY",false),
    ("player.max_health","net.minecraft.world.entity.LivingEntity","getMaxHealth",true),
    ("player.absorption","net.minecraft.world.entity.LivingEntity","getAbsorptionAmount",true),
    ("player.armor_points","net.minecraft.world.entity.LivingEntity","getArmorValue",true),
    ("entity.sprinting","net.minecraft.world.entity.Entity","isSprinting",true),
    ("entity.sneaking","net.minecraft.world.entity.Entity","isCrouching",true),
    ("entity.swimming","net.minecraft.world.entity.Entity","isSwimming",true),
    ("entity.on_ground","net.minecraft.world.entity.Entity","onGround",true),
    ("player.xp_level","net.minecraft.world.entity.player.Player","experienceLevel",false),
    ("player.xp_progress","net.minecraft.world.entity.player.Player","experienceProgress",false),
    ("player.xp_total","net.minecraft.world.entity.player.Player","totalExperience",false),
    ("player.fall_distance","net.minecraft.world.entity.Entity","fallDistance",false),
    ("player.hurt_ticks","net.minecraft.world.entity.LivingEntity","hurtTime",false),
    ("player.effects","net.minecraft.world.entity.LivingEntity","getActiveEffects",true),
    ("food.exhaustion","net.minecraft.world.food.FoodData","getExhaustionLevel",true),
    ("world.difficulty","net.minecraft.world.level.Level","getDifficulty",true),
    ("world.rain","net.minecraft.world.level.Level","getRainLevel",true),
    ("world.thunder","net.minecraft.world.level.Level","getThunderLevel",true),
    ("option.get","net.minecraft.client.OptionInstance","get",true),
    ("connection.players","net.minecraft.client.multiplayer.ClientPacketListener","getOnlinePlayers",true),
    ("client.packs",CLIENT,"getResourcePackRepository",true),
    ("options.packs","net.minecraft.client.Options","resourcePacks",false),
    ("options.refresh_packs","net.minecraft.client.Options","loadSelectedResourcePacks",true),
    ("packs.reload","net.minecraft.server.packs.repository.PackRepository","reload",true),
    ("options.fov","net.minecraft.client.Options","fov",true),
    ("options.fov_field","net.minecraft.client.Options","fov",false),
    ("options.chunks","net.minecraft.client.Options","renderDistance",true),
    ("options.gamma","net.minecraft.client.Options","gamma",true),
    ("options.gui","net.minecraft.client.Options","guiScale",true),
    ("options.fps_limit","net.minecraft.client.Options","framerateLimit",true),
    ("options.save","net.minecraft.client.Options","save",true),
    ("option.set","net.minecraft.client.OptionInstance","set",true),
    ("client.reload",CLIENT,"reloadResourcePacks",true),

    ("options.audio","net.minecraft.client.Options","getSoundSourceOptionInstance",true),
    ("audio.master","net.minecraft.sounds.SoundSource","MASTER",false),
    ("audio.music","net.minecraft.sounds.SoundSource","MUSIC",false),
    ("vec.x","net.minecraft.world.phys.Vec3","x",false),
    ("vec.y","net.minecraft.world.phys.Vec3","y",false),
    ("vec.z","net.minecraft.world.phys.Vec3","z",false),
    ("player.cooldown","net.minecraft.world.entity.player.Player","getAttackStrengthScale",true),
    ("client.hit",CLIENT,"hitResult",false),
    ("hit.location","net.minecraft.world.phys.HitResult","getLocation",true),
    ("entity.eye","net.minecraft.world.entity.Entity","getEyePosition",true),
    ("client.target",CLIENT,"crosshairPickEntity",false),
    ("client.integrated",CLIENT,"getSingleplayerServer",true),
    ("server.world_path","net.minecraft.server.MinecraftServer","getWorldPath",true),
    ("storage.root","net.minecraft.world.level.storage.LevelResource","ROOT",false),
    ("client.local",CLIENT,"isLocalServer",true),
    ("client.server",CLIENT,"getCurrentServer",true),
    ("client.options",CLIENT,"options",false),
    ("options.pause","net.minecraft.client.Options","pauseOnLostFocus",false),
    ("client.execute",CLIENT,"execute",true),
    ("server.address","net.minecraft.client.multiplayer.ServerData","ip",false),
    ("world.day_time","net.minecraft.world.level.Level","getDayTime",true),
    ("world.game_time","net.minecraft.world.level.Level","getGameTime",true),
    ("world.dimension","net.minecraft.world.level.Level","dimension",true),
    ("world.biome","net.minecraft.world.level.LevelReader","getBiome",true),
    ("entity.block_pos","net.minecraft.world.entity.Entity","blockPosition",true),
    ("entity.velocity","net.minecraft.world.entity.Entity","getDeltaMovement",true),
    ("entity.water","net.minecraft.world.entity.Entity","isInWater",true),
    ("entity.lava","net.minecraft.world.entity.Entity","isInLava",true),
    ("entity.air","net.minecraft.world.entity.Entity","getAirSupply",true),
    ("entity.fire","net.minecraft.world.entity.Entity","getRemainingFireTicks",true),
    ("player.health","net.minecraft.world.entity.LivingEntity","getHealth",true),
    ("player.food","net.minecraft.world.entity.player.Player","getFoodData",true),
    ("food.saturation","net.minecraft.world.food.FoodData","getSaturationLevel",true),
    ("food.level","net.minecraft.world.food.FoodData","getFoodLevel",true),
    ("holder.key","net.minecraft.core.Holder","unwrapKey",true),
    ("key.location","net.minecraft.resources.ResourceKey","location",true),
    ("client.player", CLIENT, "player", false),
    ("client.world", CLIENT, "level", false),
    ("client.fps", CLIENT, "fps", false),
    ("client.fps_method", CLIENT, "getFps", true),
    ("client.instance", CLIENT, "getInstance", true),
    ("client.connection", CLIENT, "getConnection", true),
    ("client.reduced", CLIENT, "showOnlyReducedInfo", true),
    (
        "entity.x",
        "net.minecraft.world.entity.Entity",
        "getX",
        true,
    ),
    (
        "entity.y",
        "net.minecraft.world.entity.Entity",
        "getY",
        true,
    ),
    (
        "entity.z",
        "net.minecraft.world.entity.Entity",
        "getZ",
        true,
    ),
    (
        "entity.yaw",
        "net.minecraft.world.entity.Entity",
        "getYRot",
        true,
    ),
    (
        "entity.uuid",
        "net.minecraft.world.entity.Entity",
        "getUUID",
        true,
    ),
    (
        "player.inventory_method",
        "net.minecraft.world.entity.player.Player",
        "getInventory",
        true,
    ),
    (
        "player.inventory",
        "net.minecraft.world.entity.player.Player",
        "inventory",
        false,
    ),
    (
        "player.reduced",
        "net.minecraft.world.entity.player.Player",
        "isReducedDebugInfo",
        true,
    ),
    (
        "player.armor",
        "net.minecraft.world.entity.LivingEntity",
        "getArmorSlots",
        true,
    ),
    (
        "player.equipment_slot",
        "net.minecraft.world.entity.LivingEntity",
        "getItemBySlot",
        true,
    ),
    (
        "slot.head",
        "net.minecraft.world.entity.EquipmentSlot",
        "HEAD",
        false,
    ),
    (
        "slot.chest",
        "net.minecraft.world.entity.EquipmentSlot",
        "CHEST",
        false,
    ),
    (
        "slot.legs",
        "net.minecraft.world.entity.EquipmentSlot",
        "LEGS",
        false,
    ),
    (
        "slot.feet",
        "net.minecraft.world.entity.EquipmentSlot",
        "FEET",
        false,
    ),
    (
        "inventory.armor",
        "net.minecraft.world.entity.player.Inventory",
        "armor",
        false,
    ),
    (
        "stack.empty",
        "net.minecraft.world.item.ItemStack",
        "isEmpty",
        true,
    ),
    (
        "stack.max",
        "net.minecraft.world.item.ItemStack",
        "getMaxDamage",
        true,
    ),
    (
        "stack.damage",
        "net.minecraft.world.item.ItemStack",
        "getDamageValue",
        true,
    ),
    (
        "stack.name",
        "net.minecraft.world.item.ItemStack",
        "getHoverName",
        true,
    ),
    (
        "component.text",
        "net.minecraft.network.chat.Component",
        "getString",
        true,
    ),
    (
        "connection.player_info",
        "net.minecraft.client.multiplayer.ClientPacketListener",
        "getPlayerInfo",
        true,
    ),
    (
        "info.ping",
        "net.minecraft.client.multiplayer.PlayerInfo",
        "getLatency",
        true,
    ),
];

fn artifact(app: &AppHandle) -> Option<PathBuf> {
    let bundled = app
        .path()
        .resource_dir()
        .ok()?
        .join("companions/tp-bridge-0.1.0.jar");
    let local=Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.join("tp-bridge/build/tp-bridge-0.1.0.jar");
    let path=if cfg!(debug_assertions)&&local.is_file(){local}else if bundled.is_file(){bundled}else{return None};
    let bytes = fs::read(&path).ok()?;
    if bytes.len() > 1024 * 1024 {
        return None;
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let mut manifest = String::new();
    archive
        .by_name("META-INF/MANIFEST.MF")
        .ok()?
        .take(8192)
        .read_to_string(&mut manifest)
        .ok()?;
    manifest
        .contains("Premain-Class: dev.tplauncher.bridge.TpBridge")
        .then_some(path)
}
pub fn available(app: &AppHandle) -> bool {
    artifact(app).is_some()
}

fn descriptor(name: &str, classes: &BTreeMap<String, String>) -> String {
    if let Some(element) = name.strip_suffix("[]") {
        return format!("[{}", descriptor(element, classes));
    }
    match name {
        "void" => "V",
        "int" => "I",
        "double" => "D",
        "float" => "F",
        "boolean" => "Z",
        "long" => "J",
        "byte" => "B",
        "short" => "S",
        "char" => "C",
        _ => {
            return format!(
                "L{};",
                classes
                    .get(name)
                    .map(String::as_str)
                    .unwrap_or(name)
                    .replace('.', "/")
            );
        }
    }
    .into()
}

fn proguard(text: &str) -> (Option<String>, Bindings) {
    let mut classes = BTreeMap::new();
    for line in text
        .lines()
        .filter(|line| !line.starts_with(' ') && !line.starts_with('#'))
    {
        if let Some((named, obfuscated)) = line.trim().split_once(" -> ") {
            if let Some(obfuscated) = obfuscated.strip_suffix(':') {
                classes.insert(named.to_string(), obfuscated.to_string());
            }
        }
    }
    let mut owner = "";
    let mut result = Bindings::new();
    for line in text.lines() {
        if line.trim_start().starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            owner = line.split_once(" -> ").map(|p| p.0).unwrap_or("");
            continue;
        }
        let Some((left, obfuscated)) = line.trim().split_once(" -> ") else {
            continue;
        };
        let parts: Vec<_> = left.split_whitespace().collect();
        if parts.len() != 2 {
            continue;
        }
        let is_method = parts[1].contains('(');
        let member = parts[1].split('(').next().unwrap_or("");
        let Some((role, _, _, _)) = ROLES.iter().find(|(_, class, name, method)| {
            *class == owner && *name == member && *method == is_method
        }) else {
            continue;
        };
        let field_type = parts[0].rsplit(':').next().unwrap_or("");
        let desc = if is_method {
            let Some(args) = parts[1]
                .split_once('(')
                .and_then(|p| p.1.split_once(')'))
                .map(|p| p.0)
            else {
                continue;
            };
            if !args.is_empty() && !(*role=="world.biome"&&args=="net.minecraft.core.BlockPos") && !(*role=="client.execute"&&args=="java.lang.Runnable") && !(*role=="option.set"&&args=="java.lang.Object") && !(*role=="options.refresh_packs"&&args=="net.minecraft.server.packs.repository.PackRepository") && !(*role=="server.world_path"&&args=="net.minecraft.world.level.storage.LevelResource") && !(*role=="player.cooldown"&&args=="float") && !(*role=="options.audio"&&args=="net.minecraft.sounds.SoundSource")
                && !(*role == "world.brightness" && args == "net.minecraft.world.level.LightLayer,net.minecraft.core.BlockPos")
                && !(*role == "connection.player_info" && args == "java.util.UUID")
                && !(*role == "player.equipment_slot"
                    && args == "net.minecraft.world.entity.EquipmentSlot")
            {
                continue;
            }
            format!(
                "({}){}",
                args.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| descriptor(s, &classes))
                    .collect::<String>(),
                descriptor(field_type, &classes)
            )
        } else {
            descriptor(field_type, &classes)
        };
        if let Some(class) = classes.get(owner) {
            result.insert(
                role.to_string(),
                Binding {
                    owner: class.replace('.', "/"),
                    name: obfuscated.into(),
                    descriptor: desc,
                },
            );
        }
    }
    (classes.get(CLIENT).cloned(), result)
}

fn tiny(text: &str, wanted: &Bindings) -> (Option<String>, BTreeMap<String, String>) {
    let mut current = "";
    let mut client = None;
    let mut result = BTreeMap::new();
    for line in text.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        if columns.first() == Some(&"c") && columns.len() >= 3 {
            current = columns[1];
            if wanted.values().any(|b| b.owner == current)
                && current
                    == wanted
                        .get("client.player")
                        .map(|b| b.owner.as_str())
                        .unwrap_or("")
            {
                client = Some(columns[2].replace('/', "."));
            }
        } else if columns.first() == Some(&"")
            && columns.len() >= 5
            && ["f", "m"].contains(&columns[1])
        {
            for (role, binding) in wanted {
                if binding.owner == current
                    && binding.name == columns[3]
                    && binding.descriptor == columns[2]
                {
                    result.insert(role.clone(), columns[4].into());
                }
            }
        }
    }
    (client, result)
}

fn tsrg(text: &str, wanted: &Bindings) -> BTreeMap<String, String> {
    let mut owner = "";
    let mut result = BTreeMap::new();
    for line in text.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.is_empty() || line.starts_with("tsrg") {
            continue;
        }
        if !line.starts_with('\t') && !line.starts_with(' ') {
            owner = fields[0];
            continue;
        }
        if line.starts_with("\t\t") {
            continue;
        }
        for (role, binding) in wanted {
            if binding.owner != owner || binding.name != fields[0] {
                continue;
            }
            if fields.len() >= 3 && fields[1].starts_with('(') {
                if fields[1] == binding.descriptor {
                    result.insert(role.clone(), fields[2].into());
                }
            } else if fields.len() >= 2 && !binding.descriptor.starts_with('(') {
                result.insert(role.clone(), fields[1].into());
            }
        }
    }
    result
}

fn archive_text(bytes: &[u8], entry: &str) -> Result<String, String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| "Archivo de símbolos inválido")?;
    let file = archive
        .by_name(entry)
        .map_err(|_| "No se encontró el índice de símbolos")?;
    if file.size() > MAX_MAPPING as u64 {
        return Err("El índice de símbolos es demasiado grande".into());
    }
    let mut out = String::new();
    file.take(MAX_MAPPING as u64 + 1)
        .read_to_string(&mut out)
        .map_err(|e| e.to_string())?;
    if out.len() > MAX_MAPPING {
        return Err("Índice de símbolos excesivo".into());
    }
    Ok(out)
}


fn legacy_role(owner: &str, name: &str, method: bool, desc: &str) -> Option<&'static str> {
    let class = owner.rsplit('/').next()?;
    if method
        && !desc.starts_with("()")
        && !(name == "getPlayerInfo" && desc.starts_with("(Ljava/util/UUID;)"))
        && !(name == "addScheduledTask" && desc.starts_with("(Ljava/lang/Runnable;)"))
    {
        return None;
    }
    Some(match (class, name, method) {
        ("Minecraft", "thePlayer" | "player", false) => "client.player",
        ("Minecraft", "theWorld" | "world", false) => "client.world",
        ("Minecraft", "debugFPS" | "fps", false) => "client.fps",
        ("Minecraft", "debug" | "debugString", false) => "client.debug",
        ("Minecraft", "getNetHandler", true) => "client.connection",
        ("Minecraft", "gameSettings", false) => "client.options",
        ("Minecraft", "currentScreen", false) => "client.screen",
        ("Minecraft", "addScheduledTask", true) => "client.execute",
        ("GameSettings", "fovSetting", false) => "options.fov_field",
        ("Minecraft", "isSingleplayer", true) => "client.local",
        ("World","getWorldTime",true)=>"world.day_time",
        ("World","getTotalWorldTime",true)=>"world.game_time",
        ("Minecraft", "isReducedDebug" | "isReducedDebugInfo", true) => "client.reduced",
        ("Entity", "posX", false) => "entity.pos_x",
        ("Entity", "posY", false) => "entity.pos_y",
        ("Entity", "posZ", false) => "entity.pos_z",
        ("Entity", "rotationYaw", false) => "entity.yaw_field",
        ("Entity", "getUniqueID", true) => "entity.uuid",
        ("EntityPlayer", "inventory", false) => "player.inventory",
        ("EntityPlayer", "hasReducedDebug", true) => "player.reduced",
        ("EntityPlayer" | "EntityLivingBase", "getHeldItem", true) => "player.mainhand",
        ("InventoryPlayer", "armorInventory", false) => "inventory.armor",
        ("ItemStack", "isEmpty", true) => "stack.empty",
        ("ItemStack", "getMaxDamage", true) => "stack.max",
        ("ItemStack", "getItemDamage", true) => "stack.damage",
        ("ItemStack", "getDisplayName", true) => "stack.name",
        ("ItemStack", "getItem", true) => "stack.item",
        ("Item", "getUnlocalizedName", true) => "item.translation",
        ("NetHandlerPlayClient", "getPlayerInfo", true) => "connection.player_info",
        ("NetworkPlayerInfo", "getResponseTime", true) => "info.ping",
        _ => return None,
    })
}

fn legacy_tiny(text: &str) -> (Option<String>, BTreeMap<String, String>) {
    let mut rows = text.lines();
    let header: Vec<_> = rows.next().unwrap_or("").split('\t').collect();
    if header.get(..3) != Some(&["tiny", "2", "0"][..]) {
        return (None, BTreeMap::new());
    }
    let namespaces = &header[3..];
    let Some(named) = namespaces.iter().position(|s| *s == "named") else {
        return (None, BTreeMap::new());
    };
    let Some(runtime) = namespaces
        .iter()
        .position(|s| *s == "client")
        .or_else(|| namespaces.iter().position(|s| *s == "official"))
    else {
        return (None, BTreeMap::new());
    };
    let mut owner = "";
    let mut client = None;
    let mut aliases = BTreeMap::new();
    for line in rows {
        let c: Vec<_> = line.split('\t').collect();
        if c.first() == Some(&"c") && c.len() > 1 + named && c.len() > 1 + runtime {
            owner = c[1 + named];
            if owner.rsplit('/').next() == Some("Minecraft") {
                client = Some(c[1 + runtime].replace('/', "."));
            }
        } else if c.first() == Some(&"")
            && c.len() > 3 + named
            && c.len() > 3 + runtime
            && ["f", "m"].contains(&c[1])
        {
            if let Some(role) = legacy_role(owner, c[3 + named], c[1] == "m", c[2]) {
                if !c[3 + runtime].is_empty() {
                    aliases.insert(role.into(), c[3 + runtime].into());
                }
            }
        }
    }
    (client, aliases)
}

fn legacy_srg(
    text: &str,
) -> (
    Option<String>,
    BTreeMap<String, String>,
    BTreeMap<String, String>,
) {
    if !text.lines().any(|line| line.starts_with("CL: ")) {
        let mut source = "";
        let mut target = "";
        let mut normalized = String::new();
        for line in text.lines() {
            let c: Vec<_> = line.split_whitespace().collect();
            if c.len() < 2 || line.starts_with("tsrg") || line.starts_with("\t\t") {
                continue;
            }
            if !line.starts_with('\t') && !line.starts_with(' ') {
                source = c[0];
                target = c[1];
                normalized.push_str(&format!("CL: {source} {target}\n"));
            } else if c.len() >= 3 && c[1].starts_with('(') {
                normalized.push_str(&format!(
                    "MD: {source}/{} {} {target}/{} {}\n",
                    c[0], c[1], c[2], c[1]
                ));
            } else {
                normalized.push_str(&format!("FD: {source}/{} {target}/{}\n", c[0], c[1]));
            }
        }
        if normalized.is_empty() {
            return (None, BTreeMap::new(), BTreeMap::new());
        }
        return legacy_srg(&normalized);
    }

    let symbols = [
        ("func_71356_B","isSingleplayer"),("func_72820_D","getWorldTime"),("func_82737_E","getTotalWorldTime"),
        ("field_71439_g", "thePlayer"),
        ("field_71441_e", "theWorld"),
        ("field_71470_ab", "debugFPS"),
        ("field_70165_t", "posX"),
        ("field_70163_u", "posY"),
        ("field_70161_v", "posZ"),
        ("field_70177_z", "rotationYaw"),
        ("field_71071_by", "inventory"),
        ("field_70460_b", "armorInventory"),
        ("field_71474_y", "gameSettings"),
        ("field_71462_r", "currentScreen"),
        ("field_74334_X", "fovSetting"),
        ("func_152344_a", "addScheduledTask"),
        ("func_70694_bm", "getHeldItem"),
        ("func_77973_b", "getItem"),
        ("func_77658_a", "getUnlocalizedName"),
        ("func_110124_au", "getUniqueID"),
        ("func_150729_e", "getNetHandler"),
        ("func_175102_a", "getPlayerInfo"),
        ("func_178853_c", "getResponseTime"),
        ("func_189648_am", "isReducedDebug"),
        ("func_175140_cp", "hasReducedDebug"),
        ("func_190926_b", "isEmpty"),
        ("func_77958_k", "getMaxDamage"),
        ("func_77952_i", "getItemDamage"),
        ("func_82833_r", "getDisplayName"),
    ];
    let mut client = None;
    let mut official = BTreeMap::new();
    let mut named = BTreeMap::new();
    for line in text.lines() {
        let c: Vec<_> = line.split_whitespace().collect();
        if c.len() == 3 && c[0] == "CL:" && c[2].rsplit('/').next() == Some("Minecraft") {
            client = Some(c[1].replace('/', "."));
        }
        let method = c.first() == Some(&"MD:");
        let index = if method { 3 } else { 2 };
        if (method && c.len() == 5) || (c.first() == Some(&"FD:") && c.len() == 3) {
            let Some((owner, srg)) = c[index].rsplit_once('/') else {
                continue;
            };
            let Some((_, semantic)) = symbols.iter().find(|(key, _)| *key == srg) else {
                continue;
            };
            if let Some(role) = legacy_role(owner, semantic, method, if method { c[4] } else { "" })
            {
                official.insert(role.into(), c[1].rsplit('/').next().unwrap_or("").into());
                named.insert(role.into(), srg.into());
            }
        }
    }
    (client, official, named)
}

async fn fetch(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    sha1: Option<&str>,
    max: usize,
) -> Result<Vec<u8>, String> {
    if let Ok(bytes) = fs::read(path) {
        if bytes.len() <= max && crate::sha1_matches(&bytes, sha1) {
            return Ok(bytes);
        }
    }
    let parsed = url::Url::parse(url).map_err(|_| "URL de símbolos inválida")?;
    if parsed.scheme() != "https"
        || !matches!(
            parsed.host_str(),
            Some(
                "piston-data.mojang.com"
                    | "launcher.mojang.com"
                    | "maven.fabricmc.net"
                    | "maven.minecraftforge.net"
                    | "mcphackers.org"
            )
        )
    {
        return Err("Fuente de símbolos no autorizada".into());
    }
    let mut response = client
        .get(parsed)
        .send()
        .await
        .map_err(|_| "No se pudieron consultar los símbolos del HUD")?
        .error_for_status()
        .map_err(|_| "Símbolos del HUD no disponibles para esta edición")?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > max {
            return Err("El índice de símbolos supera el límite".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if !crate::sha1_matches(&bytes, sha1) {
        return Err("Símbolos con SHA-1 incorrecto".into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, &bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn property(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('=', "\\=")
        .replace(':', "\\:")
}



fn java_path(path: &Path) -> Result<String, String> {
    let value = path
        .to_str()
        .ok_or("La ruta del conector no se puede representar en Java")?;
    if let Some(tail) = value.strip_prefix(r"\\?\UNC\") {
        return Ok(format!(r"\\{tail}"));
    }
    if let Some(tail) = value.strip_prefix(r"\\?\") {
        let bytes = tail.as_bytes();
        if bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/')
        {
            return Ok(tail.to_string());
        }
        return Err("El conector del HUD necesita una ruta de archivo Windows convencional".into());
    }
    if value.starts_with(r"\\.\") {
        return Err("Ruta de dispositivo no válida para el conector del HUD".into());
    }
    Ok(value.to_string())
}

fn agent_option(jar: &Path, config: &Path) -> Result<String, String> {
    let jar = java_path(jar)?;
    if jar.contains('=') {
        return Err("La ruta de TP Bridge no puede contener el carácter =".into());
    }
    Ok(format!("-javaagent:{jar}={}", java_path(config)?))
}

fn probe_startup(java: &Path, option: &str) -> Result<(), String> {


    let mut command = Command::new(java);
    command
        .arg(option)
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|_| "No se pudo verificar el conector del HUD con el Java seleccionado")?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err("Java rechazó el conector del HUD durante su comprobación previa".into())
                };
            }
            Ok(None) => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("No se pudo comprobar el conector del HUD".into());
            }
        }
        if start.elapsed() > Duration::from_secs(6) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("El conector del HUD no respondió a tiempo".into());
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub async fn launch_option(
    app: &AppHandle,
    base: &Value,
    version: &str,
    forge: bool,
    java: &Path,
) -> Result<String, String> {
    let jar = artifact(app).ok_or(
        "TP Bridge no está empaquetado. Ejecuta npm run build:bridge antes de iniciar el juego.",
    )?;
    java_path(&jar)?;
    let root = crate::root_dir()?.join("plugins/bridge");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let mut config = BTreeMap::<String, String>::new();
    if let Ok((port,token))=crate::hud_transport::endpoint(){config.insert("udp.port".into(),port.to_string());config.insert("udp.token".into(),token);}
    config.insert(
        "profile".into(),
        java_path(&crate::launcher_plugins::profile_path()?)?,
    );
    let mut candidates = vec![
        CLIENT.to_string(),
        "net.minecraft.client.MinecraftClient".into(),
        "net.minecraft.class_310".into(),
    ];
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("TP-LAUNCHER/0.1.0")
        .build()
        .map_err(|e| e.to_string())?;

    use sha2::{Digest, Sha256};
    let id = format!("{:x}", Sha256::digest(version.as_bytes()));
    if let Some(mapping) = base.pointer("/downloads/client_mappings") {
        if let Some(address) = mapping["url"].as_str() {
            if let Ok(bytes) = fetch(
                &client,
                address,
                &root.join(format!("{id}.mojmap")),
                mapping["sha1"].as_str(),
                MAX_MAPPING,
            )
            .await
            {
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    let (class, bindings) = proguard(text);
                    if let Some(class) = class {
                        config.insert("class.official".into(), class.clone());
                        candidates.push(class);
                    }
                    for (role, binding) in &bindings {
                        config.insert(format!("official.{role}"), binding.name.clone());
                    }
                    let url = format!(
                        "https://maven.fabricmc.net/net/fabricmc/intermediary/{version}/intermediary-{version}-v2.jar"
                    );
                    if !version
                        .chars()
                        .any(|c| !c.is_ascii_alphanumeric() && !"._-".contains(c))
                    {
                        if let Ok(bytes) = fetch(
                            &client,
                            &url,
                            &root.join(format!("{id}.fabric.jar")),
                            None,
                            8 * 1024 * 1024,
                        )
                        .await
                        {
                            if let Ok(text) = archive_text(&bytes, "mappings/mappings.tiny") {
                                let (class, aliases) = tiny(&text, &bindings);
                                if let Some(class) = class {
                                    config.insert("class.fabric".into(), class.clone());
                                    candidates.push(class);
                                }
                                for (role, name) in aliases {
                                    config.insert(format!("fabric.{role}"), name);
                                }
                            }
                        }
                        if forge {
                            let url = format!(
                                "https://maven.minecraftforge.net/de/oceanlabs/mcp/mcp_config/{version}/mcp_config-{version}.zip"
                            );
                            if let Ok(bytes) = fetch(
                                &client,
                                &url,
                                &root.join(format!("{id}.mcp.zip")),
                                None,
                                16 * 1024 * 1024,
                            )
                            .await
                            {
                                if let Ok(text) = archive_text(&bytes, "config/joined.tsrg") {
                                    for (role, name) in tsrg(&text, &bindings) {
                                        config.insert(format!("named.{role}"), name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !config.contains_key("class.official")
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
    {


        if version.starts_with("1.") {
            let url = format!(
                "https://maven.minecraftforge.net/de/oceanlabs/mcp/mcp/{version}/mcp-{version}-srg.zip"
            );
            let mut archive = fetch(
                &client,
                &url,
                &root.join(format!("{id}.srg.zip")),
                None,
                16 * 1024 * 1024,
            )
            .await
            .ok();
            if archive.is_none()
                && version
                    .split('.')
                    .nth(1)
                    .and_then(|n| n.parse::<u32>().ok())
                    .is_some_and(|n| n >= 13)
            {
                let url = format!(
                    "https://maven.minecraftforge.net/de/oceanlabs/mcp/mcp_config/{version}/mcp_config-{version}.zip"
                );
                archive = fetch(
                    &client,
                    &url,
                    &root.join(format!("{id}.mcp.zip")),
                    None,
                    16 * 1024 * 1024,
                )
                .await
                .ok();
            }
            if let Some(bytes) = archive {
                if let Ok(text) = archive_text(&bytes, "joined.srg")
                    .or_else(|_| archive_text(&bytes, "config/joined.tsrg"))
                {
                    let (class, official, named) = legacy_srg(&text);
                    if let Some(class) = class {
                        config.insert("class.official".into(), class.clone());
                        candidates.push(class);
                    }
                    for (role, name) in official {
                        config.insert(format!("official.{role}"), name);
                    }
                    for (role, name) in named {
                        config.insert(format!("named.{role}"), name);
                    }
                }
            }
        }
        if !config.contains_key("class.official") {
            let index_path = root.join("retro-index.json");
            if fs::metadata(&index_path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|d| d > Duration::from_secs(86400))
            {
                let _ = fs::remove_file(&index_path);
            }
            if let Ok(bytes) = fetch(
                &client,
                "https://mcphackers.org/versionsV3/versions.json",
                &index_path,
                None,
                2 * 1024 * 1024,
            )
            .await
            {
                if let Ok(Value::Array(index)) = serde_json::from_slice::<Value>(&bytes) {
                    if let Some(address) = index
                        .iter()
                        .find(|v| v["id"].as_str() == Some(version))
                        .and_then(|v| v["resources"].as_str())
                    {
                        if address.starts_with("https://mcphackers.org/versionsV2/")
                            || address.starts_with("https://mcphackers.org/versionsV3/")
                        {
                            if let Ok(bytes) = fetch(
                                &client,
                                address,
                                &root.join(format!("{id}.retro.zip")),
                                None,
                                8 * 1024 * 1024,
                            )
                            .await
                            {
                                if let Ok(text) = archive_text(&bytes, "mappings.tiny") {
                                    let (class, aliases) = legacy_tiny(&text);
                                    if let Some(class) = class {
                                        config.insert("class.official".into(), class.clone());
                                        candidates.push(class);
                                    }
                                    for (role, name) in aliases {
                                        config.insert(format!("official.{role}"), name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    config.insert("client.classes".into(), candidates.join(","));
    let path = root.join(format!("{id}.properties"));
    let text = config
        .into_iter()
        .map(|(k, v)| format!("{}={}\n", property(&k), property(&v)))
        .collect::<String>();
    if text.len() > 65536 {
        return Err("La configuración del puente supera el límite".into());
    }
    fs::write(&path, text).map_err(|e| e.to_string())?;
    let option = agent_option(&jar, &path)?;
    let java = java.to_path_buf();
    let probe_option = option.clone();
    tokio::task::spawn_blocking(move || probe_startup(&java, &probe_option))
        .await
        .map_err(|_| "No se pudo verificar el conector del HUD")??;
    Ok(option)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hud_agent_paths_remove_windows_verbatim_prefixes() {
        let option = agent_option(
            Path::new(r"\\?\C:\Program Files\TP\tp-bridge.jar"),
            Path::new(r"\\?\C:\Users\Tomás\HUD\perfil.properties"),
        )
        .unwrap();
        assert_eq!(
            option,
            r"-javaagent:C:\Program Files\TP\tp-bridge.jar=C:\Users\Tomás\HUD\perfil.properties"
        );
        assert_eq!(
            java_path(Path::new(r"\\?\UNC\server\share\tp.jar")).unwrap(),
            r"\\server\share\tp.jar"
        );
        assert!(java_path(Path::new(r"\\?\Volume{123}\tp.jar")).is_err());
        assert!(java_path(Path::new(r"\\.\PhysicalDrive0")).is_err());
        assert!(agent_option(Path::new(r"C:\path=ambiguous\tp.jar"), Path::new("config")).is_err());
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "Requires built bridge and TP_HUD_TEST_JAVAS (path list)"]
    fn hud_startup_windows_verbatim_paths_with_real_jvms() {
        let java_paths = std::env::var_os("TP_HUD_TEST_JAVAS").expect("Set TP_HUD_TEST_JAVAS");
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let cache = base.join(".codex-tmp");
        fs::create_dir_all(&cache).unwrap();
        let cache = cache.canonicalize().unwrap();
        let root = cache.join(format!("hud-startup-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        struct Cleanup {
            path: PathBuf,
            base: PathBuf,
        }
        impl Drop for Cleanup {
            fn drop(&mut self) {
                if let Ok(path) = self.path.canonicalize() {
                    if path.starts_with(&self.base)
                        && path
                            .file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with("hud-startup-"))
                    {
                        let _ = fs::remove_dir_all(path);
                    }
                }
            }
        }
        let _cleanup = Cleanup {
            path: root.clone(),
            base: cache,
        };
        let folder = root.join("Carpeta con espacios");
        fs::create_dir(&folder).unwrap();
        let jar = folder.join("tp bridge.jar");
        fs::copy(base.join("tp-bridge/build/tp-bridge-0.1.0.jar"), &jar).unwrap();
        let profile = folder.join("hud-profile.json");
        fs::write(&profile, b"{\"native_overlay\":false}").unwrap();
        let config = folder.join("perfil.properties");
        fs::write(
            &config,
            format!("profile={}\n", property(&java_path(&profile).unwrap())),
        )
        .unwrap();
        let jar = jar.canonicalize().unwrap();
        let config = config.canonicalize().unwrap();
        assert!(jar.to_string_lossy().starts_with(r"\\?\"));
        let option = agent_option(&jar, &config).unwrap();
        assert!(!option.contains(r"\\?\"));
        let mut count = 0;
        for java in std::env::split_paths(&java_paths) {
            probe_startup(&java, &option)
                .unwrap_or_else(|error| panic!("{}: {error}", java.display()));
            assert!(
                probe_startup(
                    &java,
                    &format!("-javaagent:{}", folder.join("missing.jar").display())
                )
                .is_err()
            );
            count += 1;
        }
        assert!(count > 0);
        println!(
            "Verified normalized startup and safe rejection with {count} actual Java runtimes"
        );
    }
    #[test]
    fn maps_overloads_without_confusing_uuid_and_name() {
        let text = "net.minecraft.client.Minecraft -> abc:\n# {\"id\":\"sourceFile\"}\n    net.minecraft.client.player.LocalPlayer player -> p\n    int fps -> f\nnet.minecraft.client.multiplayer.ClientPacketListener -> conn:\n    1:2:net.minecraft.client.multiplayer.PlayerInfo getPlayerInfo(java.lang.String) -> a\n    3:4:net.minecraft.client.multiplayer.PlayerInfo getPlayerInfo(java.util.UUID) -> b\nnet.minecraft.client.multiplayer.PlayerInfo -> info:\n";
        let (class, b) = proguard(text);
        assert_eq!(class.as_deref(), Some("abc"));
        assert_eq!(b["connection.player_info"].name, "b");
        assert_eq!(
            b["connection.player_info"].descriptor,
            "(Ljava/util/UUID;)Linfo;"
        );
        let (_, t) = tiny(
            "tiny\t2\t0\tofficial\tintermediary\nc\tabc\tnet/minecraft/class_310\n\tf\tLnet/minecraft/client/player/LocalPlayer;\tp\tfield_player\n\tf\tI\tf\tfield_fps\n",
            &b,
        );
        assert_eq!(t["client.fps"], "field_fps");
        let s = tsrg(
            "tsrg2 obf srg id\nabc net/minecraft/client/Minecraft 1\n\tf f_123_ 2\nconn net/minecraft/client/multiplayer/ClientPacketListener 3\n\tb (Ljava/util/UUID;)Linfo; m_123_ 4\n",
            &b,
        );
        assert_eq!(s["client.fps"], "f_123_");
        assert_eq!(s["connection.player_info"], "m_123_");
    }
    #[test]
    fn escapes_windows_profile_paths() {
        assert_eq!(property("C:\\a=b\\hud.json"), "C\\:\\\\a\\=b\\\\hud.json");
    }
    #[test]
    fn maps_window_handle_and_typed_light_arguments(){
        let text="com.mojang.blaze3d.platform.Window -> win:\n    1:1:long handle() -> h\nnet.minecraft.world.level.BlockAndTintGetter -> light:\n    1:1:int getBrightness(net.minecraft.world.level.LightLayer,net.minecraft.core.BlockPos) -> a\nnet.minecraft.world.level.LightLayer -> channel:\n    net.minecraft.world.level.LightLayer BLOCK -> b\n    net.minecraft.world.level.LightLayer SKY -> s\nnet.minecraft.core.BlockPos -> pos:\n";
        let(_,bindings)=proguard(text);assert_eq!(bindings["window.handle"].name,"h");assert_eq!(bindings["world.brightness"].descriptor,"(Lchannel;Lpos;)I");assert_eq!(bindings["light.block"].name,"b");assert_eq!(bindings["light.sky"].name,"s");
    }
    #[test]
    fn legacy_namespaces_and_field_coordinates() {
        let text = "tiny\t2\t0\tnamed\tclient\tserver\nc\tnet/minecraft/client/Minecraft\tnet/minecraft/client/Minecraft\t\n\tf\tLjava/lang/String;\tdebug\tK\t\n\tf\tLnet/minecraft/src/EntityPlayer;\tthePlayer\th\t\nc\tnet/minecraft/src/Entity\tum\t\n\tf\tD\tposX\taM\t\n";
        let (class, a) = legacy_tiny(text);
        assert_eq!(class.as_deref(), Some(CLIENT));
        assert_eq!(a["client.debug"], "K");
        assert_eq!(a["entity.pos_x"], "aM");
        let text = "CL: bao net/minecraft/client/Minecraft\nFD: bao/h net/minecraft/client/Minecraft/field_71439_g\nFD: sa/s net/minecraft/entity/Entity/field_70165_t\nMD: bjb/a (Ljava/lang/String;)Lbsc; net/minecraft/client/network/NetHandlerPlayClient/func_175102_a (Ljava/lang/String;)Lnet/minecraft/client/network/NetworkPlayerInfo;\n";
        let (c, o, n) = legacy_srg(text);
        assert_eq!(c.as_deref(), Some("bao"));
        assert_eq!(o["entity.pos_x"], "s");
        assert_eq!(n["client.player"], "field_71439_g");
        assert!(!o.contains_key("connection.player_info"));
    }
    #[test]
    #[ignore = "Requires downloaded public reference mappings; set TP_HUD_REFERENCE_MAPS"]
    fn validates_reference_mapping_archives() {
        let root =
            PathBuf::from(std::env::var("TP_HUD_REFERENCE_MAPS").expect("reference directory"));
        for version in ["1.21.9", "1.21.1"] {
            let text = fs::read_to_string(root.join(format!("{version}.mojmap"))).unwrap();
            let (c, b) = proguard(&text);
            assert!(c.is_some(), "{version}");
            for role in [
                "client.player",
                "client.world",
                "client.fps",
                "entity.x",
                "entity.y",
                "entity.z",
            ] {
                assert!(b.contains_key(role), "{version}: {role}");
            }
        }
        for version in ["1.12.2", "1.8.9"] {
            let bytes = fs::read(root.join(format!("{version}.srg.zip"))).unwrap();
            let text = archive_text(&bytes, "joined.srg").unwrap();
            let (c, o, n) = legacy_srg(&text);
            assert!(c.is_some());
            for role in [
                "client.player",
                "client.world",
                "client.fps",
                "entity.pos_x",
                "entity.pos_y",
                "entity.pos_z",
            ] {
                assert!(o.contains_key(role), "{version}: {role}");
                assert!(n.contains_key(role));
            }
        }
        for version in ["1.5.2", "b1.7.3", "a1.2.6"] {
            let bytes = fs::read(root.join(format!("retro-{version}.zip"))).unwrap();
            let (c, a) = legacy_tiny(&archive_text(&bytes, "mappings.tiny").unwrap());
            assert!(c.is_some());
            for role in [
                "client.player",
                "client.world",
                "entity.pos_x",
                "entity.pos_y",
                "entity.pos_z",
            ] {
                assert!(a.contains_key(role), "{version}: {role}");
            }
        }
    }
}
