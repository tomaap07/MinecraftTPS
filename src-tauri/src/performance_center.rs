use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::{Read, Write}, path::{Path, PathBuf}, sync::Mutex};

static LOCK: Mutex<()> = Mutex::new(());
const LIMIT: u64 = 1024 * 1024;

#[derive(Serialize)]
pub struct Preset { pub id: String, pub name: String, pub explanation: String, pub changes: BTreeMap<String, String> }
#[derive(Serialize)]
pub struct Workspace {
    pub game_version: String, pub loader: String, pub options_exist: bool, pub backup_available: bool,
    pub manual_changes: bool, pub current: BTreeMap<String, String>, pub presets: Vec<Preset>,
    pub mods: usize, pub recommended_ram_gb: u64, pub total_ram_gb: Option<u64>, pub available_ram_gb: Option<u64>,
    pub logical_threads: usize, pub note: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transaction { schema_version: u32, before_existed: bool, before_hash: String, applied_hash: String, preset: String }
#[derive(Serialize, Clone)]
pub struct Finding { pub code: String, pub title: String, pub explanation: String, pub action: String }
#[derive(Serialize)]
pub struct Diagnosis { pub findings: Vec<Finding>, pub inspected: usize, pub source: String, pub note: String }
#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub schema_version: u32, pub timestamp_ms: u64, pub pid: u32, pub elapsed_seconds: f64,
    pub has_world: bool, pub focused: bool, pub fps: f64,
    #[serde(default)] pub frame_time_ms: Option<f64>,
    pub heap_used_mb: f64, pub heap_max_mb: f64,
    #[serde(default)] pub gc_pause_ms: Option<f64>,
    #[serde(default)] pub fps_limit: Option<f64>,
}
#[derive(Serialize)]
pub struct FpsReport { pub sample: Option<Sample>, pub samples: usize, pub findings: Vec<Finding>, pub note: String }

fn read(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|_| "No se pudo leer el archivo local.")?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > max { return Err("El archivo es un enlace, supera el límite o no es regular.".into()); }
    let mut bytes = Vec::new(); fs::File::open(path).map_err(|e|e.to_string())?.take(max + 1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
    if bytes.len() as u64 > max { return Err("El archivo supera el límite.".into()); } Ok(bytes)
}
fn safe_tree(root: &Path, path: &Path) -> Result<(), String> {
    if !path.starts_with(root) { return Err("La instancia queda fuera de la carpeta de Minecraft.".into()); }
    let mut cursor = path.to_path_buf();
    loop {
        if fs::symlink_metadata(&cursor).is_ok_and(|m|m.file_type().is_symlink()) { return Err("No se modifican instancias que contengan enlaces de carpetas.".into()); }
        if cursor == root { break; }
        if !cursor.pop() { return Err("Ruta de instancia inválida.".into()); }
    }
    Ok(())
}
fn target(username: &str, version_id: &str, pack_id: Option<&str>) -> Result<(PathBuf, String, String), String> {
    if username.is_empty() || username.len()>64 || !username.chars().all(|c|c.is_ascii_alphanumeric() || c=='_') { return Err("Nombre de perfil inválido.".into()); }
    if version_id.len()>160 || version_id.is_empty() || version_id=="." || version_id==".." || !version_id.chars().all(|c|c.is_ascii_alphanumeric() || "-._+".contains(c)) { return Err("Versión inválida.".into()); }
    let root = crate::root_dir()?;
    if !crate::content_targets::installed_profile(&root, version_id) { return Err("Selecciona una versión instalada y completa.".into()); }
    let (path, game, loader) = if let Some(id) = pack_id {
        let pack = crate::mod_studio::read_pack(id)?;
        if pack.version_id != version_id || pack.install_state != "ready" { return Err("El modpack no está listo o no corresponde a esta versión.".into()); }
        (crate::mod_studio::pack_game_dir(id)?, pack.game_version, pack.loader)
    } else {
        let (game,loader) = crate::resolve_mod_game_version(&root, version_id)?;
        (crate::version_game_dir(&root, username, version_id)?,game,loader)
    };
    safe_tree(&root,&path)?;
    Ok((path,game,loader))
}
fn hash(data: &[u8]) -> String { format!("{:x}", Sha256::digest(data)) }
fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));
    let result = (|| { let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?; file.write_all(bytes)?;file.sync_all()?;drop(file);fs::rename(&tmp,path) })();
    result.map_err(|e:std::io::Error| {let _=fs::remove_file(tmp);format!("No se pudo confirmar el archivo: {e}")})
}
fn options(bytes: &[u8]) -> Result<BTreeMap<String,String>, String> {
    let value = std::str::from_utf8(bytes).map_err(|_|"options.txt no usa UTF-8; no se modificará.")?;
    Ok(value.lines().filter_map(|line|line.split_once(':')).map(|(key,value)|(key.into(),value.into())).collect())
}
fn patch(bytes: &[u8], changes: &BTreeMap<String,String>) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_|"options.txt no usa UTF-8; no se modificará.")?;
    let newline = if text.contains("\r\n") {"\r\n"} else {"\n"};
    let mut remaining = changes.clone(); let mut result = String::new();
    for line in text.lines() {
        if let Some((key,_)) = line.split_once(':') {
            if let Some(value) = changes.get(key) { result.push_str(&format!("{key}:{value}{newline}"));remaining.remove(key);continue; }
        }
        result.push_str(line);result.push_str(newline);
    }
    for (key,value) in remaining { result.push_str(&format!("{key}:{value}{newline}")); }
    Ok(result.into_bytes())
}
fn minor(game: &str) -> Option<u32> { let mut p=game.split('.'); if p.next()!=Some("1"){return None;}p.next()?.parse().ok() }
fn supported_options(game:&str)->bool{
    if let Some(minor)=minor(game){return (7..=21).contains(&minor);}
    if game.starts_with("26."){return true;}
    let bytes=game.as_bytes();
    bytes.len()==6&&bytes[2]==b'w'&&bytes[..2].iter().all(u8::is_ascii_digit)&&bytes[3..5].iter().all(u8::is_ascii_digit)&&bytes[5].is_ascii_lowercase()
        &&game[..2].parse::<u32>().is_ok_and(|year|(14..=26).contains(&year))
}
fn presets(game: &str) -> Vec<Preset> {
    if !supported_options(game){return Vec::new();}
    let modern = minor(game).is_none_or(|m|m>=18); let graphics = if minor(game).is_some_and(|m|m<16){"fancyGraphics"}else{"graphicsMode"};
    [("balanced","Equilibrado",12,8,1,"1",true,120),("performance","Rendimiento",8,5,2,"0",false,240),("quality","Calidad",16,12,0,"1",true,120)].into_iter().map(|(id,name,distance,simulation,particles,quality,vsync,cap)| {
        let mut changes = BTreeMap::from([("renderDistance".into(),distance.to_string()),("particles".into(),particles.to_string()),(graphics.into(),if graphics=="fancyGraphics" {(quality=="1").to_string()}else{quality.into()}),("enableVsync".into(),vsync.to_string()),("maxFps".into(),cap.to_string())]);
        if modern { changes.insert("simulationDistance".into(),simulation.to_string()); }
        Preset {id:id.into(),name:name.into(),explanation:format!("Render: {distance} chunks; {}partículas {particles}; VSync {}; límite {cap} FPS. No instala mods ni cambia Java o shaders.",if modern{format!("simulación: {simulation} chunks; ")}else{String::new()},if vsync{"activado"}else{"desactivado"}),changes}
    }).collect()
}
fn tx_paths(dir: &Path) -> Result<(PathBuf, PathBuf), String> {
    let folder=dir.join("config/tp-performance");safe_tree(dir,&folder)?;Ok((folder.join("transaction.json"),folder.join("options.before")))
}
fn load_tx(path: &Path) -> Result<Option<Transaction>, String> {
    if !path.exists(){return Ok(None);}
    let tx:Transaction=serde_json::from_slice(&read(path,8192)?).map_err(|_|"El registro de optimización está dañado; conserva la copia antes de repararlo.")?;
    if tx.schema_version!=1 || [&tx.before_hash,&tx.applied_hash].iter().any(|h|h.len()!=64 || !h.bytes().all(|b|b.is_ascii_hexdigit())) {return Err("Registro de optimización inválido.".into());}Ok(Some(tx))
}
fn current(dir: &Path) -> Result<(bool,Vec<u8>),String> { let p=dir.join("options.txt");if p.exists(){Ok((true,read(&p,LIMIT)?))}else{Ok((false,Vec::new()))} }
fn apply_at(dir: &Path, preset: &Preset) -> Result<(), String> {
    safe_tree(dir,&dir.join("options.txt"))?;let (existed,bytes)=current(dir)?;let next=patch(&bytes,&preset.changes)?;
    let (record,backup)=tx_paths(dir)?;let previous=load_tx(&record)?;
    if let Some(tx)=&previous { if hash(&bytes)!=tx.applied_hash {return Err("options.txt cambió fuera de TP. Se conserva tu edición: restaura o revisa la copia manualmente antes de aplicar otro perfil.".into());} }
    fs::create_dir_all(record.parent().ok_or("Falta el directorio de copias.")?).map_err(|e|e.to_string())?;
    let tx=if let Some(tx)=previous { if hash(&read(&backup,LIMIT)?)!=tx.before_hash {return Err("La copia original no supera la verificación SHA-256.".into());}Transaction{applied_hash:hash(&next),preset:preset.id.clone(),..tx} }else{
        if backup.exists(){return Err("Ya existe una copia sin registro; no se sobrescribirá.".into());}atomic(&backup,&bytes)?;
        Transaction{schema_version:1,before_existed:existed,before_hash:hash(&bytes),applied_hash:hash(&next),preset:preset.id.clone()}
    };
    atomic(&record,&serde_json::to_vec_pretty(&tx).map_err(|e|e.to_string())?)?;
    if let Err(error)=atomic(&dir.join("options.txt"),&next) {return Err(format!("{error}. La copia original sigue intacta en config/tp-performance/options.before."));}
    Ok(())
}
fn restore_at(dir:&Path) -> Result<(),String> {
    safe_tree(dir,&dir.join("options.txt"))?;let(record,backup)=tx_paths(dir)?;let tx=load_tx(&record)?.ok_or("No hay un perfil aplicado por TP para restaurar.")?;
    let(_,bytes)=current(dir)?;if hash(&bytes)!=tx.applied_hash{return Err("Se detectaron cambios manuales en options.txt. No se sobrescribirá tu configuración.".into());}
    let original=read(&backup,LIMIT)?;if hash(&original)!=tx.before_hash{return Err("La copia original no supera SHA-256; no se restaurará.".into());}
    if tx.before_existed {atomic(&dir.join("options.txt"),&original)?;}else{fs::remove_file(dir.join("options.txt")).map_err(|e|e.to_string())?;}
    fs::remove_file(record).map_err(|e|e.to_string())?;fs::remove_file(backup).map_err(|e|e.to_string())?;Ok(())
}
fn memory() -> (Option<u64>,Option<u64>) {
    #[cfg(windows)] { #[repr(C)] struct Status {length:u32,load:u32,total:u64,available:u64,page_total:u64,page_available:u64,virtual_total:u64,virtual_available:u64,extended:u64}
        #[link(name="kernel32")] unsafe extern "system"{fn GlobalMemoryStatusEx(status:*mut Status)->i32;}
        let mut s=Status{length:std::mem::size_of::<Status>() as u32,load:0,total:0,available:0,page_total:0,page_available:0,virtual_total:0,virtual_available:0,extended:0};
        if unsafe{GlobalMemoryStatusEx(&mut s)}>0{return(Some(s.total/(1024*1024*1024)),Some(s.available/(1024*1024*1024)));}
    }
    (None,None)
}
fn ram(mods:usize,total:Option<u64>,available:Option<u64>)->u64 {
    let desired=if mods>150{8}else if mods>60{6}else if mods>10{4}else{2};
    desired.min(total.map(|n|n.saturating_sub(3).min(n/2).max(1)).unwrap_or(2)).min(available.map(|n|n.saturating_sub(2).max(1)).unwrap_or(2))
}
fn workspace(dir:&Path,game:String,loader:String)->Result<Workspace,String>{
    let(existed,bytes)=current(dir)?;let current=options(&bytes)?;let(record,_)=tx_paths(dir)?;let tx=load_tx(&record)?;
    let mods=fs::read_dir(dir.join("mods")).ok().into_iter().flatten().take(4096).filter_map(Result::ok).filter(|e|e.path().extension().is_some_and(|e|e=="jar")).count();
    let(total,available)=memory();
    Ok(Workspace{game_version:game.clone(),loader,options_exist:existed,backup_available:tx.is_some(),manual_changes:tx.is_some_and(|t|hash(&bytes)!=t.applied_hash),current:current.into_iter().filter(|(k,_)|["renderDistance","simulationDistance","particles","graphicsMode","fancyGraphics","enableVsync","maxFps"].contains(&k.as_str())).collect(),presets:presets(&game),mods,recommended_ram_gb:ram(mods,total,available),total_ram_gb:total,available_ram_gb:available,logical_threads:std::thread::available_parallelism().map(usize::from).unwrap_or(1),note:"La memoria sugerida es orientativa y reserva RAM para Windows. No cambia el máximo asignado ni instala aceleradores. Vanilla comparte options.txt entre sus versiones del mismo perfil; los modpacks y perfiles de loader usan su carpeta existente.".into()})
}
#[tauri::command]
pub async fn performance_workspace(username:String,version_id:String,pack_id:Option<String>)->Result<Workspace,String>{tauri::async_runtime::spawn_blocking(move||{let _lock=LOCK.lock().map_err(|_|"Centro de rendimiento ocupado.")?;let(dir,game,loader)=target(&username,&version_id,pack_id.as_deref())?;workspace(&dir,game,loader)}).await.map_err(|e|e.to_string())?}
#[tauri::command]
pub async fn performance_apply_preset(username:String,version_id:String,pack_id:Option<String>,preset:String)->Result<Workspace,String>{tauri::async_runtime::spawn_blocking(move||{let _lock=LOCK.lock().map_err(|_|"Centro de rendimiento ocupado.")?;if crate::game_session::is_running(){return Err("Cierra Minecraft antes de cambiar options.txt.".into());}let(dir,game,loader)=target(&username,&version_id,pack_id.as_deref())?;let selected=presets(&game).into_iter().find(|p|p.id==preset).ok_or("Perfil de rendimiento inválido.")?;apply_at(&dir,&selected)?;workspace(&dir,game,loader)}).await.map_err(|e|e.to_string())?}
#[tauri::command]
pub async fn performance_restore(username:String,version_id:String,pack_id:Option<String>)->Result<Workspace,String>{tauri::async_runtime::spawn_blocking(move||{let _lock=LOCK.lock().map_err(|_|"Centro de rendimiento ocupado.")?;if crate::game_session::is_running(){return Err("Cierra Minecraft antes de restaurar options.txt.".into());}let(dir,game,loader)=target(&username,&version_id,pack_id.as_deref())?;restore_at(&dir)?;workspace(&dir,game,loader)}).await.map_err(|e|e.to_string())?}

fn analyze(text:&str)->Vec<Finding>{
    let lower=text.to_ascii_lowercase();
    let rules=[("java","Java incompatible","El registro contiene UnsupportedClassVersionError o un nivel de bytecode no compatible.","Selecciona el Java que exige el manifiesto de esta versión; no cambies los mods para reparar este error.",lower.contains("unsupportedclassversionerror")),("memory","Memoria insuficiente","Java registró un error de memoria; también puede existir presión de RAM o un mod con una fuga.","Revisa la memoria disponible y el tamaño del modpack. Ajusta RAM con cautela; no asignes toda la RAM del equipo.",lower.contains("outofmemoryerror")||lower.contains("could not reserve enough space")),("dependency","Dependencia o mod incompatible","El registro indica dependencias no resueltas o incompatibilidad de mods.","Revisa los identificadores y las versiones en el registro original. Desactiva únicamente el mod conflictivo tras hacer una copia.",lower.contains("incompatible mods found")||lower.contains("mod resolution encountered an incompatible mod set")||lower.contains("missing mandatory dependencies")),("graphics","Inicialización gráfica","El registro contiene un fallo explícito de GLFW/OpenGL.","Revisa el controlador gráfico y la versión OpenGL requerida; comprueba primero una instancia sin shaders.",lower.contains("glfw error")||lower.contains("failed to create window")||lower.contains("pixel format not accelerated")),("library","Clase o biblioteca ausente","Java informó una clase que no pudo cargar; puede ser una biblioteca o un conflicto de mods.","Comprueba el nombre en el registro original. Revalida los archivos de la instalación; no descargues JAR de fuentes desconocidas.",lower.contains("classnotfoundexception")||lower.contains("noclassdeffounderror"))];
    rules.into_iter().filter(|r|r.4).map(|(code,title,explanation,action,_)|Finding{code:code.into(),title:title.into(),explanation:explanation.into(),action:action.into()}).collect()
}
fn diagnose(dir:&Path)->Result<Diagnosis,String>{
    let mut text=String::new();let mut inspected=0;
    let mut candidates=vec![dir.join("logs/latest.log")];
    let mut crashes=fs::read_dir(dir.join("crash-reports")).ok().into_iter().flatten().take(256).filter_map(Result::ok).filter(|e|e.path().extension().is_some_and(|e|e=="txt")).collect::<Vec<_>>();
    crashes.sort_by_key(|e|std::cmp::Reverse(e.metadata().and_then(|m|m.modified()).ok()));candidates.extend(crashes.into_iter().take(2).map(|e|e.path()));
    for path in candidates {safe_tree(dir,&path)?;if !path.exists(){continue;}if let Ok(data)=read(&path,4*1024*1024){text.push_str(&String::from_utf8_lossy(&data));text.push('\n');inspected+=1;}}
    Ok(Diagnosis{findings:analyze(&text),inspected,source:"latest.log y hasta dos crash-reports de la instancia elegida".into(),note:"Análisis determinista local: los indicios no garantizan una causa única. El informe devuelve categorías y pasos, nunca el contenido del log, tokens, nombres, chat ni direcciones. No modifica Java, archivos o mods automáticamente.".into()})
}
#[tauri::command]
pub async fn performance_diagnose(username:String,version_id:String,pack_id:Option<String>)->Result<Diagnosis,String>{tauri::async_runtime::spawn_blocking(move||{let(dir,_,_)=target(&username,&version_id,pack_id.as_deref())?;diagnose(&dir)}).await.map_err(|e|e.to_string())?}

fn valid_sample(s:&Sample,now:u64)->bool{
    s.schema_version==1&&s.timestamp_ms<=now.saturating_add(5000)&&now.saturating_sub(s.timestamp_ms)<=30000&&s.pid>0
        &&[s.elapsed_seconds,s.fps,s.heap_used_mb,s.heap_max_mb].iter().all(|v|v.is_finite()&&*v>=0.)
        &&s.fps<=100000.&&s.elapsed_seconds<=86400.*365.&&s.heap_max_mb>0.&&s.heap_max_mb<1048576.&&s.heap_used_mb<=s.heap_max_mb
        &&[s.frame_time_ms,s.gc_pause_ms,s.fps_limit].iter().flatten().all(|v|v.is_finite()&&*v>=0.&&*v<1e9)
}
fn fps_findings(samples:&[Sample])->Vec<Finding>{
    if samples.len()<6{return Vec::new();}
    let recent=&samples[samples.len()-6..];
    if recent.iter().any(|s|s.elapsed_seconds<45.||!s.has_world||!s.focused||s.fps_limit.is_some_and(|n|n<=30.))
        || recent.windows(2).any(|pair|pair[1].timestamp_ms.saturating_sub(pair[0].timestamp_ms)>10000)
        || recent.last().unwrap().timestamp_ms.saturating_sub(recent[0].timestamp_ms)<25000{return Vec::new();}
    let mut findings=Vec::new();
    if recent.iter().all(|s|s.fps<30.){findings.push(Finding{code:"sustained-low-fps".into(),title:"FPS bajos sostenidos".into(),explanation:"Seis muestras locales válidas durante al menos 25 segundos registraron menos de 30 FPS. Esto no demuestra por sí solo un cuello de botella de CPU o GPU.".into(),action:"Prueba sin shaders y con menor distancia de render manteniendo el resto de opciones iguales. Compara otra sesión antes de cambiar más ajustes.".into()});}
    if recent.iter().all(|s|s.heap_used_mb/s.heap_max_mb>0.9){findings.push(Finding{code:"heap-pressure".into(),title:"Presión sostenida del heap".into(),explanation:"El heap utilizado supera el 90% del máximo en seis muestras; no equivale a toda la memoria del proceso.".into(),action:"Comprueba primero la RAM disponible. Considera un aumento moderado o revisar los mods si la presión se repite.".into()});}
    findings
}
#[tauri::command]
pub async fn performance_samples(username:String,version_id:String,pack_id:Option<String>)->Result<FpsReport,String>{
    tauri::async_runtime::spawn_blocking(move||{
        type History=BTreeMap<PathBuf,Vec<Sample>>;static HISTORY:std::sync::OnceLock<Mutex<History>>=std::sync::OnceLock::new();
        let(dir,_,_)=target(&username,&version_id,pack_id.as_deref())?;let path=dir.join("config/tp-client/performance.json");safe_tree(&dir,&path)?;
        let now=crate::launcher_overlay::now();
        let sample=read(&path,65536).ok().and_then(|bytes|serde_json::from_slice::<Sample>(&bytes).ok()).filter(|sample|valid_sample(sample,now)&&crate::game_session::directory(sample.pid).is_some_and(|p|p==dir));
        let mut all=HISTORY.get_or_init(||Mutex::new(BTreeMap::new())).lock().map_err(|_|"Muestras ocupadas.")?;
        if all.len()>32{all.clear();}
        let history=all.entry(dir).or_default();
        if let Some(sample)=&sample {if history.last().is_none_or(|old|old.timestamp_ms!=sample.timestamp_ms){if history.last().is_some_and(|old|old.pid!=sample.pid){history.clear();}history.push(sample.clone());if history.len()>120{history.remove(0);}}}else{history.clear();}
        Ok(FpsReport{findings:fps_findings(history),samples:history.len(),note:if sample.is_some(){"Muestras locales opt-in cada 5 s; se ignoran calentamiento, falta de foco y límite intencional de FPS. No se envían a internet. Un benchmark A/B requiere una ruta y opciones equivalentes; aún no hay una comparación válida."}else{"Sin métricas locales recientes de TP Client para esta instancia. Activa su informe local de rendimiento en el juego. No se inventan FPS ni diagnósticos."}.into(),sample})
    }).await.map_err(|e|e.to_string())?
}

#[cfg(test)] mod tests{
    use super::*;
    fn dir()->PathBuf{let p=std::env::temp_dir().join(format!("tp-perf-{}",uuid::Uuid::new_v4()));fs::create_dir(&p).unwrap();p}
    fn cleanup(p:PathBuf){let p=p.canonicalize().unwrap();assert_eq!(p.parent().unwrap(),std::env::temp_dir().canonicalize().unwrap());assert!(p.file_name().unwrap().to_string_lossy().starts_with("tp-perf-"));fs::remove_dir_all(p).unwrap();}
    #[test]fn exact_rollback_preserves_unknown_options_and_crlf(){let p=dir();let original=b"renderDistance:22\r\ncustom:keep:this\r\nkey_key.jump:key.keyboard.space";fs::write(p.join("options.txt"),original).unwrap();let choices=presets("1.21.1");apply_at(&p,&choices[1]).unwrap();assert!(fs::read_to_string(p.join("options.txt")).unwrap().contains("custom:keep:this\r\n"));apply_at(&p,&choices[2]).unwrap();restore_at(&p).unwrap();assert_eq!(fs::read(p.join("options.txt")).unwrap(),original);cleanup(p);}
    #[test]fn manual_edits_and_tampered_backup_fail_closed(){let p=dir();fs::write(p.join("options.txt"),b"renderDistance:18").unwrap();apply_at(&p,&presets("1.21.1")[0]).unwrap();fs::write(p.join("options.txt"),b"player edit").unwrap();assert!(restore_at(&p).is_err());assert!(apply_at(&p,&presets("1.21.1")[1]).is_err());assert_eq!(fs::read(p.join("options.txt")).unwrap(),b"player edit");cleanup(p);}
    #[test]fn restore_previously_missing_options_and_reject_invalid_encoding(){let p=dir();apply_at(&p,&presets("1.12.2")[0]).unwrap();assert!(!fs::read_to_string(p.join("options.txt")).unwrap().contains("simulationDistance"));restore_at(&p).unwrap();assert!(!p.join("options.txt").exists());assert!(patch(&[255],&BTreeMap::new()).is_err());cleanup(p);}
    #[test]fn backup_integrity_and_path_boundaries(){let p=dir();fs::write(p.join("options.txt"),b"original").unwrap();apply_at(&p,&presets("1.21.1")[0]).unwrap();fs::write(p.join("config/tp-performance/options.before"),b"tampered").unwrap();assert!(restore_at(&p).is_err());assert!(safe_tree(&p,Path::new("C:/unrelated/options.txt")).is_err());cleanup(p);}
    #[test]fn memory_never_assigns_all_system_or_available_ram(){assert_eq!(ram(200,Some(16),Some(10)),8);assert_eq!(ram(200,Some(8),Some(3)),1);assert_eq!(ram(0,None,None),2);}
    #[test]fn presets_never_write_modern_option_formats_into_legacy_versions(){for game in ["1.0","1.5.2","1.6.4","b1.7.3","a1.2.6","c0.30_01c","rd-132211","12w49a","13w37a","unknown","1.99"]{assert!(presets(game).is_empty(),"{game}");}for game in ["1.7.2","1.12.2","1.21.9","26.3","26.3-snapshot-1","24w14a"]{assert!(!presets(game).is_empty(),"{game}");}}
    #[test]fn crash_report_is_categorical_and_never_returns_private_input(){let findings=analyze("access_token=private\nC:/Users/private secret@example.com UnsupportedClassVersionError OutOfMemoryError");let encoded=serde_json::to_string(&findings).unwrap();assert_eq!(findings.len(),2);for private in ["access_token","private","secret@example"]{assert!(!encoded.contains(private));}}
    fn sample(t:u64)->Sample{Sample{schema_version:1,timestamp_ms:t,pid:1,elapsed_seconds:60.,has_world:true,focused:true,fps:20.,frame_time_ms:Some(50.),heap_used_mb:950.,heap_max_mb:1000.,gc_pause_ms:None,fps_limit:Some(120.)}}
    #[test]fn doctor_requires_sustained_real_samples_and_ignores_loading_or_limits(){let mut values=(0..6).map(|i|sample(100000+i*5000)).collect::<Vec<_>>();assert_eq!(fps_findings(&values).len(),2);assert!(fps_findings(&values[..5]).is_empty());for s in &mut values{s.fps_limit=Some(20.);}assert!(fps_findings(&values).is_empty());for s in &mut values{s.fps_limit=None;s.elapsed_seconds=10.;}assert!(fps_findings(&values).is_empty());let mut s=sample(100000);assert!(valid_sample(&s,100010));s.fps=f64::NAN;assert!(!valid_sample(&s,100010));}
}
