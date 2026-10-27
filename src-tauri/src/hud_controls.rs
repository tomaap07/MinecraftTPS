
use serde::{Deserialize,Serialize};
use std::{collections::BTreeMap,fs,io::Read,path::Path,time::{Duration,Instant}};
static COMMAND:tokio::sync::Mutex<()>=tokio::sync::Mutex::const_new(());
#[tauri::command]pub fn hud_title_background(pid:u32)->Result<String,String>{
 use base64::Engine;use std::io::Cursor;
 let dir=crate::game_session::directory(pid).ok_or("Partida no disponible")?;
 let path=dir.join("config/tp-launcher/background.png");if !path.exists(){return Ok(String::new());}
 let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>20*1024*1024{return Err("Fondo no permitido".into());}
 let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(20*1024*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
 let image=crate::hud_preview::decode(&bytes)?.thumbnail(1920,1080);let mut png=Cursor::new(Vec::new());image.write_to(&mut png,image::ImageFormat::Png).map_err(|e|e.to_string())?;
 Ok(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(png.into_inner())))
}
#[tauri::command]pub fn hud_item_icons(pid:u32,items:Vec<String>)->Result<BTreeMap<String,String>,String>{
 use base64::Engine;
 let(_,version)=crate::game_session::identity(pid).ok_or("La partida ya no está abierta")?;
 if items.len()>8||items.iter().any(|s|s.len()>80||!s.starts_with("minecraft:")||!s[10..].bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_')){return Err("Identificador de objeto no válido".into());}
 let root=crate::root_dir()?;let(game,_)=crate::resolve_mod_game_version(&root,&version)?;
 let path=root.join("versions").join(&game).join(format!("{game}.jar"));
 let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;
 if meta.file_type().is_symlink()||meta.len()>192*1024*1024{return Err("Archivo del cliente no permitido".into());}
 let mut jar=zip::ZipArchive::new(fs::File::open(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 let mut result=BTreeMap::new();for item in items{for dir in ["item","items","block","blocks"]{let path=format!("assets/minecraft/textures/{dir}/{}.png",&item[10..]);if let Ok(file)=jar.by_name(&path){if file.size()>512*1024{continue;}let mut bytes=Vec::new();file.take(512*1024+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;if bytes.len()<=512*1024&&bytes.starts_with(b"\x89PNG\r\n\x1a\n"){result.insert(item.clone(),format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes)));break;}}}}Ok(result)
}
#[tauri::command]pub async fn hud_menu_background(pid:u32,data_url:String)->Result<String,String>{
 let dir=crate::game_session::directory(pid).ok_or("No hay una partida activa")?;
 tokio::task::spawn_blocking(move||{use base64::Engine;
  if data_url.len()>28*1024*1024{return Err("La imagen es demasiado grande".into());}
  let source=data_url.strip_prefix("data:image/png;base64,").ok_or("Selecciona una imagen compatible")?;
  let bytes=base64::engine::general_purpose::STANDARD.decode(source).map_err(|_|"Imagen inválida")?;
  let image=crate::hud_preview::decode(&bytes)?.thumbnail(3840,2160);
  let folder=dir.join("config/tp-launcher");fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
  let path=folder.join("background.png");let tmp=folder.join(format!("background-{}.pending",uuid::Uuid::new_v4()));
  image.save_with_format(&tmp,image::ImageFormat::Png).map_err(|e|e.to_string())?;
  fs::rename(tmp,path).map_err(|e|e.to_string())?;
  Ok("Fondo guardado para el menú principal. GIF y WebP animados usan una imagen fija.".into())
 }).await.map_err(|e|e.to_string())?
}
#[derive(Serialize)]pub struct Pack{pub name:String,pub enabled:bool,pub order:usize}
#[derive(Serialize)]pub struct Server{pub name:String,pub address:String}
#[derive(Serialize)]pub struct Content{pub game_version:String,pub loader:String,pub shader_loader:Option<String>,pub resourcepacks:Vec<Pack>,pub shaderpacks:Vec<Pack>,pub servers:Vec<Server>,pub options:BTreeMap<String,String>}
pub(crate) fn shader_loader(dir:&Path,loader:&str)->Option<String>{
 if loader=="optifine"{return Some("OptiFine".into());}
 if !["fabric","forge","neoforge","quilt"].contains(&loader){return None;}
 for entry in fs::read_dir(dir.join("mods")).ok()?.take(512).filter_map(Result::ok){
  let Ok(meta)=entry.metadata()else{continue};if entry.file_type().is_ok_and(|v|v.is_symlink())||!meta.is_file()||meta.len()>128*1024*1024||entry.path().extension().is_none_or(|s|s!="jar"){continue;}
  let Ok(file)=fs::File::open(entry.path())else{continue};let Ok(mut jar)=zip::ZipArchive::new(file)else{continue};
  if let Ok(file)=jar.by_name("fabric.mod.json"){let mut bytes=Vec::new();let _=file.take(16385).read_to_end(&mut bytes);if bytes.len()<=16384{if let Ok(value)=serde_json::from_slice::<serde_json::Value>(&bytes){if value["id"]=="iris"{return Some("Iris".into());}}}}
  let name=entry.file_name().to_string_lossy().to_lowercase();if name.starts_with("oculus-"){return Some("Oculus".into());}if name.starts_with("optifine_"){return Some("OptiFine".into());}
 }None
}
fn read(path:&Path)->Result<String,String>{if !path.exists(){return Ok(String::new());}let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;if meta.file_type().is_symlink()||!meta.is_file()||meta.len()>1024*1024{return Err("Archivo de opciones no permitido".into());}fs::read_to_string(path).map_err(|e|e.to_string())}
fn options(text:&str)->BTreeMap<String,String>{text.lines().filter_map(|line|line.split_once(':').map(|(k,v)|(k.to_string(),v.to_string()))).collect()}
fn packs(dir:&Path,selected:&[String])->Vec<Pack>{let mut rows=Vec::new();if let Ok(entries)=fs::read_dir(dir){for e in entries.take(4096).filter_map(Result::ok){let Ok(kind)=e.file_type()else{continue};let name=e.file_name().to_string_lossy().into_owned();if name.len()>240||name.chars().any(char::is_control)||kind.is_symlink()||(!kind.is_dir()&&!name.to_lowercase().ends_with(".zip")){continue;}let order=selected.iter().position(|id|id==&name||id==&format!("file/{name}"));rows.push(Pack{name,enabled:order.is_some(),order:order.unwrap_or(usize::MAX)});}}rows.sort_by(|a,b|a.order.cmp(&b.order).then_with(||a.name.cmp(&b.name)));rows}
#[tauri::command]pub fn hud_game_content(pid:u32)->Result<Content,String>{let dir=crate::game_session::directory(pid).ok_or("No hay una partida activa")?;let values=options(&read(&dir.join("options.txt"))?);let selected:Vec<String>=values.get("resourcePacks").and_then(|v|serde_json::from_str(v).ok()).unwrap_or_default();let mut shader=read(&dir.join("optionsshaders.txt"))?.lines().find_map(|line|line.strip_prefix("shaderPack=").map(str::to_owned)).unwrap_or_default();let iris=read(&dir.join("config/iris.properties")).unwrap_or_default();if let Some(enabled)=iris.lines().find_map(|line|line.strip_prefix("enableShaders=")){shader=if enabled.trim()=="true"{iris.lines().find_map(|line|line.strip_prefix("shaderPack=").map(str::to_owned)).unwrap_or_default()}else{String::new()};}let mut allowed=BTreeMap::new();for key in ["fov","gamma","renderDistance","guiScale","maxFps","soundCategory_master","soundCategory_music"]{if let Some(value)=values.get(key){allowed.insert(key.to_string(),value.clone());}}let servers=crate::servers::list_at(&dir).map(|list|list.entries.into_iter().take(64).map(|e|Server{name:e.name,address:e.address}).collect()).unwrap_or_default();let (_,version)=crate::game_session::identity(pid).ok_or("Partida no disponible")?;let(game_version,loader)=crate::resolve_mod_game_version(&crate::root_dir()?,&version)?;Ok(Content{shader_loader:shader_loader(&dir,&loader),game_version,loader,servers,resourcepacks:packs(&dir.join("resourcepacks"),&selected),shaderpacks:packs(&dir.join("shaderpacks"),&[shader]),options:allowed})}
fn property(value:&str)->String{value.replace('\\',"\\\\").replace('\n',"\\n").replace('\r',"\\r").replace('=',"\\=").replace(':',"\\:")}
#[derive(Deserialize)]struct Ack{id:String,ok:bool,message:String}
#[tauri::command]pub async fn hud_game_command(pid:u32,action:String,values:BTreeMap<String,String>)->Result<String,String>{
 let _lock=COMMAND.lock().await;let _dir=crate::game_session::directory(pid).ok_or("La partida ya no está abierta")?;
 let valid=match action.as_str(){
  "title"=>values.len()==1&&values.get("target").is_some_and(|v|["singleplayer","multiplayer","options","quit"].contains(&v.as_str())),
  "connect"=>values.len()==1&&values.get("address").is_some_and(|v|crate::servers::parse_address(v).is_ok()),
  "option"=>{values.len()==2&&values.get("name").is_some_and(|n|["fov","chunks","gamma","gui","volume","music","fps_limit"].contains(&n.as_str()))&&values.get("value").and_then(|v|v.parse::<f64>().ok()).is_some_and(|v|v.is_finite()&&(0.0..=1000.0).contains(&v))},
  "packs"=>values.len()<=128&&values.iter().all(|(k,v)|k.strip_prefix("pack.").is_some_and(|v|v.parse::<usize>().is_ok())&&v.len()<=250&&!v.chars().any(char::is_control)),
  "shader"=>values.len()==1&&values.get("name").is_some_and(|v|v.len()<=240&&!v.contains(['/', '\\'])&&!v.chars().any(char::is_control)),
  "memory_gc"|"reload_resources"=>values.is_empty(),_=>false
 };
 if !valid{return Err("Control o parámetros no permitidos".into());}
 let folder=crate::launcher_plugins::profile_path()?.parent().ok_or("Perfil sin carpeta")?.to_path_buf();let id=uuid::Uuid::new_v4().to_string();
 let adapter=if action!="title"&&crate::launcher_overlay::telemetry(pid).is_some_and(|v|v.capabilities.iter().any(|c|c=="controls")){"companion"}else{"bridge"};
 let mut text=format!("id={id}\naction={action}\nadapter={adapter}\n");for(k,v)in values{text.push_str(&format!("{}={}\n",property(&k),property(&v)));}
 if text.len()>16384{return Err("El control supera el límite permitido".into());}
 let path=folder.join(format!("command-{pid}.properties"));let pending=folder.join(format!("command-{pid}-{id}.pending"));fs::write(&pending,text).and_then(|_|fs::rename(&pending,&path)).map_err(|e|{let _=fs::remove_file(&pending);e.to_string()})?;
 let ack=folder.join(format!("ack-{pid}.json"));let start=Instant::now();
 while start.elapsed()<Duration::from_secs(30){if !crate::game_session::tracked_pids().contains(&pid){return Err("Minecraft se cerró antes de aplicar el control".into());}if let Ok(file)=fs::File::open(&ack){let mut bytes=Vec::new();let _=file.take(4097).read_to_end(&mut bytes);if bytes.len()<=4096{if let Ok(value)=serde_json::from_slice::<Ack>(&bytes){if value.id==id{return if value.ok{Ok(value.message)}else{Err(value.message)};}}}}tokio::time::sleep(Duration::from_millis(60)).await;}
 Err("El adaptador de esta versión no confirmó el cambio. No se marcó como aplicado.".into())
}
