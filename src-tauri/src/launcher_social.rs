
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::{Read, Write}, path::PathBuf, sync::Mutex};
static PROFILE_LOCK: Mutex<()> = Mutex::new(());
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SocialLinks { pub discord:String, pub youtube:String, pub twitch:String, pub twitter:String, pub github:String }
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TpCosmetics { pub cape:String, pub wings:bool, pub aura:bool }
impl Default for TpCosmetics {fn default()->Self{Self{cape:"none".into(),wings:false,aura:false}}}
#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SocialProfile {pub schema_version:u32,pub show_links:bool,pub links:SocialLinks,pub cosmetics:TpCosmetics}
impl Default for SocialProfile {fn default()->Self{Self{schema_version:1,show_links:true,links:SocialLinks::default(),cosmetics:TpCosmetics::default()}}}
fn path(player_id:&str)->Result<PathBuf,String>{
    if player_id.trim().is_empty()||player_id.len()>128||player_id.chars().any(char::is_control){return Err("Identidad de perfil no válida.".into());}
    let folder=crate::launcher_settings::data_root()?.join("profiles");fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
    Ok(folder.join(format!("{:x}.json",Sha256::digest(player_id.trim().to_lowercase().as_bytes()))))
}
fn validate_link(value:&mut String,hosts:&[&str],discord:bool)->Result<(),String>{
    *value=value.trim().to_string();
    if value.is_empty(){return Ok(());}
    if value.len()>512||value.chars().any(char::is_control){return Err("Cada enlace debe tener menos de 512 caracteres y no contener controles.".into());}
    if discord&&!value.contains("://"){
        if value.len()<=64&&value.chars().all(|c|c.is_ascii_alphanumeric()||matches!(c,'.'|'_'|'#'|'-')){return Ok(());}
        return Err("Usa un nombre de Discord o un enlace https://discord.com/users/ID.".into());
    }
    let url=url::Url::parse(value).map_err(|_|"Introduce un enlace HTTPS completo a tu perfil.".to_string())?;
    let host=url.host_str().unwrap_or_default().trim_start_matches("www.");
    if url.scheme()!="https"||!url.username().is_empty()||url.password().is_some()||url.port().is_some()
        ||!hosts.contains(&host)||url.path().trim_matches('/').is_empty()
        ||(discord&&!url.path().starts_with("/users/")){
        return Err("El enlace no corresponde a la red social seleccionada.".into());
    }
    *value=url.to_string();Ok(())
}
fn validate(profile:&mut SocialProfile)->Result<(),String>{
    if profile.schema_version!=1||!["none","end"].contains(&profile.cosmetics.cape.as_str()){return Err("Perfil social o cosmético no compatible.".into());}
    validate_link(&mut profile.links.discord,&["discord.com"],true)?;
    validate_link(&mut profile.links.youtube,&["youtube.com","youtu.be"],false)?;
    validate_link(&mut profile.links.twitch,&["twitch.tv"],false)?;
    validate_link(&mut profile.links.twitter,&["x.com","twitter.com"],false)?;
    validate_link(&mut profile.links.github,&["github.com"],false)?;Ok(())
}
#[tauri::command]
pub fn get_social_profile(player_id:String)->Result<SocialProfile,String>{
    let _guard=PROFILE_LOCK.lock().map_err(|_|"Perfil ocupado.")?;let path=path(&player_id)?;
    if !path.exists(){return Ok(SocialProfile::default());}
    let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(65537).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len()>65536{return Err("El perfil supera 64 KiB.".into());}
    let mut profile:SocialProfile=serde_json::from_slice(&bytes).map_err(|_|"No se pudo leer el perfil social.")?;validate(&mut profile)?;Ok(profile)
}
#[tauri::command]
pub fn save_social_profile(player_id:String,mut profile:SocialProfile)->Result<SocialProfile,String>{
    let _guard=PROFILE_LOCK.lock().map_err(|_|"Perfil ocupado.")?;validate(&mut profile)?;let path=path(&player_id)?;
    let pending=path.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));
    let bytes=serde_json::to_vec_pretty(&profile).map_err(|e|e.to_string())?;
    let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&pending)?;
        file.write_all(&bytes)?;file.sync_all()?;drop(file);fs::rename(&pending,&path)})();
    result.map_err(|e:std::io::Error|{let _=fs::remove_file(&pending);e.to_string()})?;Ok(profile)
}
