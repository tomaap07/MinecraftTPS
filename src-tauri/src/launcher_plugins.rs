

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io::Read, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_opener::OpenerExt;

static PROFILE_LOCK: Mutex<()> = Mutex::new(());
const MODULE_IDS:&[&str]= &["fps", "coordinates", "keystrokes", "armor", "clock", "ping", "direction", "minimap", "custom_font", "transparent_menu", "enhanced_chat", "optical_zoom", "sodium", "lithium", "ferritecore", "days", "biome", "entityculling", "immediatelyfast", "dynamicfps", "iris", "no_fog", "item_physics", "m_health", "m_food", "m_saturation", "m_oxygen", "m_xp_level", "m_inventory_free", "m_count_arrow", "m_count_ender_pearl", "m_count_golden_apple", "m_count_totem_of_undying", "m_count_firework_rocket", "m_attack_cooldown", "m_block_light", "m_sky_light", "m_nether_coords", "m_chunk_coords", "m_chunk_local", "m_world_dimension", "m_target_distance", "m_combo", "skin_layers", "wavey_capes", "old_animations", "freelook", "block_outline", "motion_blur", "low_fire", "clean_view", "hit_color", "particle_multiplier", "self_nametag", "ragdolls", "tnt_timer", "light_levels"];
const ANCHORS: [&str; 6] = [
    "top-left",
    "top-center",
    "top-right",
    "bottom-left",
    "bottom-center",
    "bottom-right",
];

fn companion_artifact(app:&AppHandle)->Option<PathBuf>{
    let bundled=app.path().resource_dir().ok().map(|dir|dir.join("companions").join("tp-companion-0.1.0.jar"));
    if cfg!(debug_assertions){
        let local=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.join("tp-companion/build/libs/tp-companion-0.1.0.jar");
        if local.is_file(){return Some(local);}
    }
    bundled.filter(|path|path.is_file())
}
fn install_companion(artifact:&std::path::Path,game_dir:&std::path::Path)->Result<(),String>{
    let metadata=fs::metadata(artifact).map_err(|e|e.to_string())?;
    if metadata.len()>8*1024*1024{return Err("El complemento TP supera el tamaño esperado.".into());}
    let mut archive=zip::ZipArchive::new(fs::File::open(artifact).map_err(|e|e.to_string())?).map_err(|_|"El complemento TP no es un JAR válido.")?;
    let mut data=Vec::new();archive.by_name("fabric.mod.json").map_err(|_|"Faltan los metadatos del complemento TP.")?.take(16385).read_to_end(&mut data).map_err(|e|e.to_string())?;
    if data.len()>16384{return Err("Metadatos del complemento TP demasiado grandes.".into());}
    let value:serde_json::Value=serde_json::from_slice(&data).map_err(|_|"Metadatos del complemento TP no válidos.")?;
    if value["id"]!="tp_companion"||value["version"]!="0.1.0"||value.pointer("/depends/minecraft").and_then(serde_json::Value::as_str)!=Some("1.21.1") {return Err("El JAR no corresponde a TP Companion 0.1.0 para Minecraft 1.21.1.".into());}
    drop(archive);
    let folder=game_dir.join("mods");fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
    let bytes=fs::read(artifact).map_err(|e|e.to_string())?;let target=folder.join("tp-companion-0.1.0.jar");
    if fs::read(&target).ok().as_deref()==Some(bytes.as_slice()){return Ok(());}
    let pending=folder.join(format!(".tp-companion-{}.pending",uuid::Uuid::new_v4()));
    fs::write(&pending,&bytes).and_then(|_|fs::rename(&pending,&target)).map_err(|e|{let _=fs::remove_file(&pending);e.to_string()})
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HudModule {
    pub id: String,
    pub enabled: bool,
    pub anchor: String,
    #[serde(default)]pub x:Option<f64>,
    #[serde(default)]pub y:Option<f64>,
    #[serde(default)]pub scale:Option<f64>,
    #[serde(default)]pub opacity:Option<f64>,
    #[serde(default)]pub background:Option<bool>,
}

#[derive(Clone,Serialize,Deserialize)]
#[serde(default,deny_unknown_fields)]
pub struct FogSettings{pub enabled:bool,pub overworld:bool,pub nether:bool,pub end:bool,pub water:bool,pub lava:bool,pub strength:f64}
impl Default for FogSettings{fn default()->Self{Self{enabled:false,overworld:true,nether:true,end:true,water:true,lava:true,strength:1.0}}}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HudProfile {
    #[serde(default="yes")]pub title_theme:bool,
    #[serde(default)]pub fog:FogSettings,
    #[serde(default)]pub low_latency:bool,
    #[serde(default)]pub crosshair:crate::hud_crosshair::Crosshair,
    pub schema_version: u32,
    pub name: String,
    pub scale: f64,
    pub opacity: f64,
    pub background: bool,
    pub modules: Vec<HudModule>,
    #[serde(default)]
    pub native_overlay: bool,
    #[serde(default)]
    pub companion_enabled: bool,
    #[serde(default)]
    pub auto_borderless: bool,
    #[serde(default)]pub day_offset:i32,
    #[serde(default="yes")]pub days_with_fps:bool,
    #[serde(default="accent")]pub accent:String,
    #[serde(default="text_color")]pub text_color:String,
    #[serde(default="background_color")]pub background_color:String,
    #[serde(default="menu_transparency")]pub menu_transparency:f64,
    #[serde(default="yes")]pub menu_blur:bool,
    #[serde(default)]pub chroma:bool,
    #[serde(default)]pub menu_offset_x:f64,
    #[serde(default)]pub menu_offset_y:f64,
}
fn yes()->bool{true}
fn accent()->String{"#A855F7".into()}fn text_color()->String{"#F3EDF8".into()}fn background_color()->String{"#0B0B0F".into()}fn menu_transparency()->f64{0.35}

impl Default for HudProfile {
    fn default() -> Self {
        let placements = [
            "top-left",
            "top-left",
            "top-right",
            "bottom-right",
            "bottom-left",
            "top-right",
        ];
        Self {
            title_theme:true,fog:FogSettings::default(),low_latency:false,
            crosshair:crate::hud_crosshair::Crosshair::default(),
            schema_version: 1,
            name: "Mi HUD".into(),
            scale: 1.0,
            opacity: 0.9,
            background: true,
            native_overlay: false,
            companion_enabled: false,
            auto_borderless: false,
            day_offset:0,days_with_fps:true,
            accent:accent(),text_color:text_color(),background_color:background_color(),menu_transparency:menu_transparency(),menu_blur:true,chroma:false,menu_offset_x:0.0,menu_offset_y:0.0,
            modules: MODULE_IDS
                .iter()
                .enumerate()
                .map(|(index, id)| HudModule {
                    id: (*id).into(),
                    enabled: index < 4||*id=="days",
                    anchor: placements.get(index).unwrap_or(&"top-left").to_string(),
                    x:None,y:None,scale:None,opacity:None,background:None,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct PluginWorkspace {
    pub profile: HudProfile,
    pub profile_path: String,
    pub profile_saved: bool,
    pub companion_available: bool,
    pub bridge_available: bool,
}

pub(super) fn profile_path() -> Result<PathBuf, String> {
    let folder = crate::launcher_settings::data_root()?.join("plugins");
    fs::create_dir_all(&folder)
        .map_err(|error| format!("No se pudo preparar la carpeta de plugins: {error}"))?;
    Ok(folder.join("hud-profile.json"))
}


fn normalize_modules(profile:&mut HudProfile){
    profile.modules.retain(|module|MODULE_IDS.contains(&module.id.as_str()));
    for &id in MODULE_IDS{if !profile.modules.iter().any(|module|module.id==id){profile.modules.push(HudModule{id:id.into(),enabled:id=="days",anchor:"top-left".into(),x:None,y:None,scale:None,opacity:None,background:None});}}
}

fn validate(profile: &HudProfile) -> Result<(), String> {
    if profile.schema_version != 1 {
        return Err("Este perfil de HUD pertenece a una versión incompatible del launcher.".into());
    }
    if profile.name.trim().is_empty()
        || profile.name.chars().count() > 60
        || profile.name.chars().any(char::is_control)
        || !profile.scale.is_finite()
        || !(0.75..=1.5).contains(&profile.scale)
        || !profile.opacity.is_finite()
        || !(0.35..=1.0).contains(&profile.opacity)
        || profile.modules.len() != MODULE_IDS.len()
        || !profile.fog.strength.is_finite()||!(0.0..=1.0).contains(&profile.fog.strength)
        || !profile.crosshair.valid()
        || profile.day_offset.unsigned_abs()>1_000_000
        || [&profile.accent,&profile.text_color,&profile.background_color].iter().any(|s|s.len()!=7||!s.starts_with('#')||!s[1..].chars().all(|c|c.is_ascii_hexdigit()))
        || !profile.menu_transparency.is_finite()||!(0.1..=0.95).contains(&profile.menu_transparency)
        || [profile.menu_offset_x,profile.menu_offset_y].iter().any(|v|!v.is_finite()||!(-0.5..=0.5).contains(v))
    {
        return Err("Revisa el nombre, el tamaño y la transparencia del HUD.".into());
    }
    let mut seen = HashSet::new();
    for module in &profile.modules {
        if !MODULE_IDS.contains(&module.id.as_str())
            || !seen.insert(&module.id)
            || !ANCHORS.contains(&module.anchor.as_str())
            || [module.x,module.y].iter().flatten().any(|n|!n.is_finite()||!(0.0..=1.0).contains(n))
            || module.scale.is_some_and(|v|!v.is_finite()||!(0.5..=2.5).contains(&v))
            || module.opacity.is_some_and(|v|!v.is_finite()||!(0.1..=1.0).contains(&v))
        {
            return Err("El perfil contiene un plugin o una posición no compatible.".into());
        }
    }
    Ok(())
}

pub fn disable_pause_on_focus_loss(game_dir:&std::path::Path)->Result<(),String>{
    let path=game_dir.join("options.txt");
    if fs::symlink_metadata(&path).is_ok_and(|m|m.file_type().is_symlink()||m.len()>1024*1024){return Err("No se puede actualizar options.txt de esta instancia".into());}
    let current=match fs::read_to_string(&path){Ok(value)=>value,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>String::new(),Err(e)=>return Err(e.to_string())};
    let mut lines:Vec<_>=current.lines().filter(|line|!line.starts_with("pauseOnLostFocus:")).map(str::to_owned).collect();lines.push("pauseOnLostFocus:false".into());let value=lines.join("\n")+"\n";
    if value==current{return Ok(());}
    let pending=game_dir.join(format!(".options-hud-{}.tmp",uuid::Uuid::new_v4()));fs::write(&pending,value).and_then(|_|fs::rename(&pending,&path)).map_err(|e|{let _=fs::remove_file(&pending);e.to_string()})
}

#[tauri::command]
pub fn get_launcher_plugins(app:AppHandle) -> Result<PluginWorkspace, String> {
    let _guard = PROFILE_LOCK
        .lock()
        .map_err(|_| "El perfil del HUD está ocupado.")?;
    let path = profile_path()?;
    let profile = if path.exists() {
        let mut bytes = Vec::new();
        fs::File::open(&path)
            .map_err(|e| format!("No se pudo abrir el perfil: {e}"))?
            .take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 65_536 {
            return Err("El perfil del HUD supera el límite de 64 KiB.".into());
        }
        let mut profile: HudProfile = serde_json::from_slice(&bytes)
            .map_err(|error| format!("No se pudo leer el perfil del HUD: {error}"))?;
        let previous:Vec<_>=profile.modules.iter().map(|module|module.id.clone()).collect();
        normalize_modules(&mut profile);
        validate(&profile)?;
        if previous!=profile.modules.iter().map(|module|module.id.clone()).collect::<Vec<_>>(){
            let pending=path.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));
            let bytes=serde_json::to_vec_pretty(&profile).map_err(|e|e.to_string())?;
            fs::write(&pending,bytes).and_then(|_|fs::rename(&pending,&path)).map_err(|e|{let _=fs::remove_file(&pending);format!("No se pudo actualizar el perfil de HUD: {e}")})?;
        }
        profile
    } else {
        HudProfile::default()
    };
    Ok(PluginWorkspace {
        profile,
        profile_saved: path.exists(),
        profile_path: path.to_string_lossy().into_owned(),
        companion_available: companion_artifact(&app).is_some(),
        bridge_available: crate::hud_bridge::available(&app),
    })
}

#[tauri::command]
pub fn save_launcher_plugins(app: AppHandle, mut profile: HudProfile) -> Result<HudProfile, String> {
    let _guard = PROFILE_LOCK
        .lock()
        .map_err(|_| "El perfil del HUD está ocupado.")?;
    profile.name = profile.name.trim().to_owned();
    normalize_modules(&mut profile);
    validate(&profile)?;
    let path = profile_path()?;
    let pending = path.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(&profile).map_err(|e| e.to_string())?;
    use std::io::Write;
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&pending, &path)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&pending);
        return Err(format!("No se pudo guardar el perfil del HUD: {error}"));
    }
    let _=app.emit("tp-hud-profile",&profile);
    Ok(profile)
}

#[tauri::command]
pub fn open_launcher_plugins_folder(app: AppHandle) -> Result<(), String> {
    let path = profile_path()?;
    let folder = path
        .parent()
        .ok_or("No se pudo localizar la carpeta de plugins.")?;
    app.opener()
        .open_path(folder.to_string_lossy(), None::<&str>)
        .map_err(|e| format!("No se pudo abrir la carpeta de plugins: {e}"))
}

#[derive(Serialize)]
pub struct PluginPreparation {pub installed:Vec<String>,pub skipped:Vec<String>,pub launch_version:Option<String>,pub compatibility_warnings:Vec<String>}
#[derive(Clone,Deserialize)]
struct Addon{id:String,name:String,projects:Vec<String>}
impl Addon{fn new(id:&str,name:&str,project:&str)->Self{Self{id:id.into(),name:name.into(),projects:vec![project.into()]}}}
fn compatible_addon_version<'a>(rows:&'a[serde_json::Value],module:&str,game:&str,loader:&str)->Option<&'a serde_json::Value>{
 let eligible=|row:&&serde_json::Value|{
  let matches=|key:&str,value:&str|row[key].as_array().is_some_and(|items|items.iter().any(|item|item.as_str()==Some(value)));
  let primary=row["files"].as_array().and_then(|files|files.iter().find(|file|file["primary"]==true).or_else(||files.first()));
  matches("game_versions",game)&&matches("loaders",loader)&&primary.is_some_and(|file|file.pointer("/hashes/sha1").and_then(serde_json::Value::as_str).is_some_and(|hash|!crate::addon_compatibility::blocked(game,loader,module,hash)))
 };
 rows.iter().filter(eligible).find(|row|row["version_type"]=="release").or_else(||rows.iter().find(eligible))
}
async fn addon_version(client:&reqwest::Client,addon:&Addon,game:&str,loader:&str)->Result<Option<serde_json::Value>,String>{
    for project in &addon.projects{
        let response=client.get(format!("https://api.modrinth.com/v2/project/{project}/version"))
            .query(&[("game_versions",serde_json::json!([game]).to_string()),("loaders",serde_json::json!([loader]).to_string())])
            .header("Accept","application/json").send().await.map_err(|_|format!("No se pudo consultar {}; revisa la conexión.",addon.name))?;
        if response.status()==reqwest::StatusCode::NOT_FOUND{continue;}
        if !response.status().is_success(){return Err(format!("{}: Modrinth respondió HTTP {}",addon.name,response.status().as_u16()));}
        let rows:Vec<serde_json::Value>=response.json().await.map_err(|_|"Modrinth respondió con versiones no válidas")?;
        if let Some(version)=compatible_addon_version(&rows,&addon.id,game,loader){return Ok(Some(version.clone()));}
    }Ok(None)
}
static PREPARE:tokio::sync::Mutex<()>=tokio::sync::Mutex::const_new(());

#[tauri::command]
pub async fn prepare_launcher_plugins(app:AppHandle,username:String,version_id:String,instance_id:Option<String>,game_version:String,loader:String)->Result<PluginPreparation,String>{
    let _lock=PREPARE.lock().await;
    let profile=get_launcher_plugins(app.clone())?.profile;
    let mut result=PluginPreparation{installed:vec![],skipped:vec![],launch_version:None,compatibility_warnings:vec![]};
    if profile.native_overlay{if let Err(error)=crate::launcher_overlay::set_launcher_overlay(app.clone(),true).await{result.skipped.push(format!("HUD universal no disponible: {error}. Minecraft puede iniciarse igualmente."));}}
    let enabled=|id:&str|profile.modules.iter().any(|module|module.id==id&&module.enabled);
    let mut selected:Vec<Addon>=[("sodium","Sodium","sodium"),("lithium","Lithium","lithium"),("ferritecore","FerriteCore","ferrite-core"),("entityculling","EntityCulling","entityculling"),("immediatelyfast","ImmediatelyFast","immediatelyfast"),("dynamicfps","Dynamic FPS","dynamic-fps"),("iris","Iris","iris")].iter().filter(|(id,_,_)|enabled(id)).map(|(id,name,project)|Addon::new(id,name,project)).collect();
    let visual:Vec<Addon>=serde_json::from_str(include_str!("../../src/lib/data/visual_addons.json")).map_err(|_|"Catálogo de efectos inválido")?;
    selected.extend(visual.into_iter().filter(|addon|enabled(&addon.id)));
    let mut loader=loader.to_lowercase();let mut version_id=version_id;
    if loader=="vanilla"&&!selected.is_empty()&&instance_id.as_deref().is_none_or(str::is_empty)&&game_version.split('.').all(|p|!p.is_empty()&&p.bytes().all(|b|b.is_ascii_digit())){
        match crate::install_fabric(game_version.clone()).await{
            Ok(id)=>{version_id=id.clone();loader="fabric".into();result.launch_version=Some(id);result.installed.push(format!("TP Turbo: instalación Fabric preparada para Minecraft {game_version}."));}
            Err(error)=>{result.skipped.push(format!("No hay un cargador disponible para {game_version}: {error}. Se conserva el lanzamiento Vanilla."));return Ok(result);}
        }
    }
    let companion=if profile.companion_enabled&&game_version=="1.21.1"&&loader=="fabric"{
        match companion_artifact(&app){Some(path)=>{selected.insert(0,Addon::new("fabric_api","Fabric API","fabric-api"));Some(path)},None=>{result.skipped.push("TP Companion no está empaquetado.".into());None}}
    }else{if profile.companion_enabled{result.skipped.push("TP Companion: adaptador actual Fabric 1.21.1; las demás ediciones usan el puente y sus capacidades.".into());}None};
    if !["fabric","forge","neoforge","quilt"].contains(&loader.as_str()){
        if !selected.is_empty(){result.skipped.push("Los mods visuales y optimizadores requieren un cargador compatible.".into());}return Ok(result);
    }
    let dir=if let Some(id)=instance_id.as_deref().filter(|id|!id.is_empty()){crate::mod_studio::pack_game_dir(id)?}else{crate::version_game_dir(&crate::root_dir()?,&username,&version_id)?};
    let mut managed=crate::managed_addons::Store::load(&dir)?;
    let (effective,warnings)=managed.compatible_profile(&dir,&profile,&game_version,&loader)?;
    result.compatibility_warnings=warnings;
    result.skipped.extend(managed.reconcile(&dir,&effective)?);
    managed.confirm_compatibility(&dir,&game_version,&loader)?;
    let client=reqwest::Client::builder().timeout(std::time::Duration::from_secs(25)).user_agent("TP LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)").build().map_err(|e|e.to_string())?;
    for addon in selected{
        let Some(version)=addon_version(&client,&addon,&game_version,&loader).await?else{
            if addon.id=="fabric_api"&&companion.is_some(){return Err("No hay una Fabric API compatible para preparar TP Companion.".into());}
            result.skipped.push(format!("{}: sin versión compatible para Minecraft {game_version} / {loader}.",addon.name));continue;
        };
        let project_id=version["project_id"].as_str().ok_or("Proyecto sin identificador")?.to_string();let file_id=version["id"].as_str().ok_or("Archivo sin identificador")?.to_string();
        let file=version["files"].as_array().and_then(|rows|rows.iter().find(|f|f["primary"]==true).or_else(||rows.first())).ok_or("El complemento no tiene archivo")?;
        let name=file["filename"].as_str().ok_or("Falta el nombre del complemento")?;let hash=file.pointer("/hashes/sha1").and_then(serde_json::Value::as_str).ok_or("Falta la huella del complemento")?;
        if PathBuf::from(name).components().count()!=1||name.contains(['/', '\\', ':']){return Err("Nombre de complemento no permitido".into());}
        let path=dir.join("mods").join(name);let existed=path.exists();
        if fs::symlink_metadata(&path).is_ok_and(|m|m.is_file()&&!m.file_type().is_symlink()&&m.len()<=128*1024*1024){if let Ok(bytes)=fs::read(&path){use sha1::Digest;if format!("{:x}",sha1::Sha1::digest(bytes))==hash{result.skipped.push(format!("{}: instalado y verificado.",addon.name));continue;}}}
        crate::mod_studio::studio_install_version(app.clone(),"modrinth".into(),project_id,file_id,"mod".into(),instance_id.clone(),username.clone(),version_id.clone()).await?;
        if enabled(&addon.id)&&(!existed||managed.owns(&addon.id)){managed.installed(&dir,&addon.id,name,hash)?;}
        result.installed.push(format!("{} · Minecraft {game_version} / {loader}",addon.name));
    }
    if let Some(artifact)=companion{install_companion(&artifact,&dir)?;result.installed.push("TP Companion 0.1.0 · 1.21.1 Fabric".into());}
    Ok(result)
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn independent_widget_preferences_migrate_and_validate(){
  let original=HudProfile::default();let mut legacy=serde_json::to_value(&original).unwrap();
  for module in legacy["modules"].as_array_mut().unwrap(){for key in ["scale","opacity","background"]{module.as_object_mut().unwrap().remove(key);}}
  let mut migrated:HudProfile=serde_json::from_value(legacy).unwrap();assert!(validate(&migrated).is_ok());
  migrated.modules[0].scale=Some(2.0);migrated.modules[0].opacity=Some(0.4);migrated.modules[0].background=Some(false);
  assert!(validate(&migrated).is_ok());assert_eq!(migrated.scale,original.scale);assert_eq!(migrated.modules[1].scale,None);
  migrated.modules[0].scale=Some(f64::NAN);assert!(validate(&migrated).is_err());migrated.modules[0].scale=Some(2.0);migrated.modules[0].opacity=Some(0.0);assert!(validate(&migrated).is_err());
 }
 #[test]fn hud_profile_registry_matches_visible_metrics(){let p=HudProfile::default();assert!(validate(&p).is_ok());let rows:serde_json::Value=serde_json::from_str(include_str!("../../src/lib/data/hud_metrics.json")).unwrap();let kept:Vec<_>=rows.as_array().unwrap().iter().filter(|row|MODULE_IDS.contains(&row["id"].as_str().unwrap())).collect();assert_eq!(kept.len(),20);assert_eq!(p.modules.iter().filter(|m|m.enabled).count(),5);for removed in ["frame_time","fps_low","m_heap_used","m_java_cpu","m_system_cpu"]{assert!(!MODULE_IDS.contains(&removed));}}
 #[test]fn hud_profile_migrates_retired_diagnostics_without_resetting_layout(){let mut p=HudProfile::default();p.modules[0].x=Some(0.75);p.modules.push(HudModule{id:"fps_low".into(),enabled:true,anchor:"top-left".into(),x:None,y:None,scale:None,opacity:None,background:None});p.modules.push(HudModule{id:"m_system_cpu".into(),enabled:true,anchor:"top-left".into(),x:None,y:None,scale:None,opacity:None,background:None});normalize_modules(&mut p);assert!(validate(&p).is_ok());assert_eq!(p.modules[0].x,Some(0.75));assert!(!p.modules.iter().any(|m|m.id=="fps_low"||m.id=="m_system_cpu"));}
 #[test]fn hud_profile_rejects_invalid_positions_and_incomplete_registry(){let mut p=HudProfile::default();p.modules[0].x=Some(f64::INFINITY);assert!(validate(&p).is_err());p.modules[0].x=Some(0.25);assert!(validate(&p).is_ok());p.modules.pop();assert!(validate(&p).is_err());}
 #[test]fn hud_focus_preference_preserves_other_options(){let path=std::env::temp_dir().join(format!("tp-hud-options-{}",uuid::Uuid::new_v4()));fs::create_dir(&path).unwrap();fs::write(path.join("options.txt"),"music:0.3\npauseOnLostFocus:true\nresourcePacks:[\"vanilla\"]\n").unwrap();disable_pause_on_focus_loss(&path).unwrap();let first=fs::read_to_string(path.join("options.txt")).unwrap();assert!(first.contains("music:0.3\n"));assert!(first.contains("resourcePacks:[\"vanilla\"]\n"));assert_eq!(first.matches("pauseOnLostFocus:").count(),1);assert!(first.contains("pauseOnLostFocus:false"));disable_pause_on_focus_loss(&path).unwrap();assert_eq!(first,fs::read_to_string(path.join("options.txt")).unwrap());fs::remove_file(path.join("options.txt")).unwrap();fs::remove_dir(path).unwrap();}
}

#[cfg(test)]mod compatibility_tests {
 use super::*;
 #[test]fn known_bad_release_is_skipped_for_a_future_compatible_build(){
  let version=|hash:&str,game:&str,loader:&str|serde_json::json!({"version_type":"release","game_versions":[game],"loaders":[loader],"files":[{"primary":true,"hashes":{"sha1":hash}}]});
  let bad=version(crate::addon_compatibility::HIT_COLOR_263_SHA1,"26.3","fabric");let good=version(&"a".repeat(40),"26.3","fabric");let rows=vec![bad.clone(),good.clone()];
  assert_eq!(compatible_addon_version(&rows,"hit_color","26.3","fabric"),Some(&good));assert!(compatible_addon_version(&[bad.clone()],"hit_color","26.3","fabric").is_none());
  assert!(compatible_addon_version(&rows,"hit_color","26.2","fabric").is_none());assert!(compatible_addon_version(&rows,"hit_color","26.3","forge").is_none());assert_eq!(compatible_addon_version(&rows,"iris","26.3","fabric"),Some(&bad));
 }
}
