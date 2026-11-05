use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::{Path, PathBuf}};
use tauri::{AppHandle, Manager};

const RELEASE: &str = include_str!("tp_client_release.json");
const JAR: &str = "tp-client-0.1.0.jar";
const RECEIPT: &str = ".tp-client-install.json";
const MAX_JAR: u64 = 16 * 1024 * 1024;
static INSTALL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Release { schema_version: u32, version: String, minecraft: String, loader: String, java: u8, sha256: String }

#[derive(Clone, Serialize)]
pub struct ClientTarget {
    pub id: String, pub name: String, pub version_id: String, pub pack_id: Option<String>,
    pub installed: bool, pub fabric_api: bool, pub companion: bool, pub mode: String,
}

#[derive(Serialize)]
pub struct ClientWorkspace {
    pub artifact_available: bool, pub version: String, pub minecraft: String,
    pub loader: String, pub java: u8, pub targets: Vec<ClientTarget>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32, sha256: String, mode: String,
    previous_config: Option<String>, previous_jar: bool,
    companion_file: Option<String>, companion_sha256: Option<String>,
    #[serde(default)] applied_config_sha256: Option<String>,
}

fn release() -> Result<Release, String> {
    let value: Release = serde_json::from_str(RELEASE).map_err(|_| "El manifiesto de TP Client no es válido")?;
    if value.schema_version != 1 || value.minecraft != "1.21.1" || value.loader != "fabric" || value.java != 21 || value.version != "0.1.0" {
        return Err("La distribución de TP Client no coincide con esta versión del launcher".into());
    }
    Ok(value)
}

fn digest(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|_| "No se pudo leer un archivo de TP Client")?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit {
        return Err("TP Client rechazó un archivo enlazado, inválido o demasiado grande".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path).and_then(|file| file.take(limit + 1).read_to_end(&mut bytes)).map_err(|_| "No se pudo leer el archivo de TP Client")?;
    if bytes.len() as u64 > limit { return Err("El archivo de TP Client supera el límite".into()); }
    Ok(bytes)
}

fn safe_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        let meta = fs::symlink_metadata(path).map_err(|_| "No se pudo revisar la carpeta de la instancia")?;
        if !meta.is_dir() || meta.file_type().is_symlink() { return Err("La carpeta de TP Client no puede ser un enlace".into()); }
    } else { fs::create_dir_all(path).map_err(|_| "No se pudo preparar la carpeta de TP Client")?; }
    Ok(())
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    if path.exists() { bounded(path, 2 * MAX_JAR)?; }
    let pending = path.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&pending)?;
        file.write_all(bytes)?; file.sync_all()?; drop(file);
        fs::rename(&pending, path)
    })();
    if result.is_err() { let _ = fs::remove_file(&pending); }
    result.map_err(|_| "No se pudo confirmar el archivo de TP Client".into())
}

fn metadata(bytes: &[u8]) -> Result<Value, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|_| "TP Client no es un JAR válido")?;
    let mut data = Vec::new();
    zip.by_name("fabric.mod.json").map_err(|_| "Faltan los metadatos Fabric de TP Client")?.take(16385).read_to_end(&mut data).map_err(|_| "No se pudieron leer los metadatos Fabric")?;
    if data.len() > 16384 { return Err("Metadatos Fabric demasiado grandes".into()); }
    serde_json::from_slice(&data).map_err(|_| "Metadatos Fabric inválidos".into())
}

fn verify_artifact(bytes: &[u8], expected: &Release) -> Result<(), String> {
    if expected.sha256.len() != 64 || !expected.sha256.bytes().all(|value| value.is_ascii_hexdigit()) || digest(bytes) != expected.sha256 {
        return Err("La huella SHA-256 de TP Client no coincide con la compilación del launcher".into());
    }
    let meta = metadata(bytes)?;
    if meta["id"] != "tp_client" || meta["version"] != expected.version || meta["environment"] != "client"
        || meta.pointer("/depends/minecraft").and_then(Value::as_str) != Some("1.21.1") {
        return Err("El JAR no corresponde a TP Client para Fabric 1.21.1".into());
    }
    Ok(())
}

fn artifact(app: &AppHandle) -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.join("tp-client/build/libs").join(JAR);
        if path.is_file() { return Some(path); }
    }
    app.path().resource_dir().ok().map(|path| path.join("clients").join(JAR)).filter(|path| path.is_file())
}

fn jar_id(path: &Path) -> Option<String> {
    let meta = fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 128 * 1024 * 1024 { return None; }
    let mut zip = zip::ZipArchive::new(fs::File::open(path).ok()?).ok()?;
    let mut bytes = Vec::new();
    zip.by_name("fabric.mod.json").ok()?.take(16385).read_to_end(&mut bytes).ok()?;
    if bytes.len() > 16384 { return None; }
    serde_json::from_slice::<Value>(&bytes).ok()?["id"].as_str().map(str::to_owned)
}

fn mods(dir: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let folder = dir.join("mods");
    if !folder.exists() { return Ok(Vec::new()); }
    safe_directory(&folder)?;
    let mut result = Vec::new();
    for (index, entry) in fs::read_dir(folder).map_err(|_| "No se pudo revisar los mods instalados")?.enumerate() {
        if index > 2048 { return Err("Hay demasiados archivos para revisar la instalación de TP Client".into()); }
        let path = entry.map_err(|_| "No se pudo revisar un mod instalado")?.path();
        if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("jar")) {
            if let Some(id) = jar_id(&path) { result.push((path, id)); }
        }
    }
    Ok(result)
}

pub(crate) fn resolve_game_dir(username: &str, version_id: &str, pack_id: Option<&str>) -> Result<PathBuf, String> {
    let root = crate::root_dir()?;
    let profile = crate::list_installed_versions_inner()?.into_iter().find(|version| version.id == version_id && version.json && version.jar)
        .ok_or("Selecciona una versión instalada de Minecraft")?;
    if profile.loader != "fabric" || profile.game_version != "1.21.1" || !crate::content_targets::installed_profile(&root, version_id) {
        return Err("TP Client requiere una instalación Fabric de Minecraft 1.21.1; Vanilla y otras versiones permanecen intactas".into());
    }
    let path = if let Some(id) = pack_id {
        let pack = crate::mod_studio::studio_list_packs()?.into_iter().find(|pack| pack.id == id && pack.version_id == version_id && pack.install_state == "ready" && pack.game_version == "1.21.1" && pack.loader == "fabric")
            .ok_or("El modpack no está listo o no es compatible con TP Client")?;
        crate::mod_studio::pack_game_dir(&pack.id)?
    } else { crate::version_game_dir(&root, username, version_id)? };
    safe_directory(&path)?;
    let mut cursor=path.as_path();
    loop {
        if fs::symlink_metadata(cursor).is_ok_and(|metadata|metadata.file_type().is_symlink()) { return Err("La instancia contiene un enlace de carpetas; no se modificará".into()); }
        if cursor==root { break; }
        cursor=cursor.parent().filter(|parent|parent.starts_with(&root)).ok_or("La instancia queda fuera de la carpeta de Minecraft")?;
    }
    for folder in ["mods", "config"] { if path.join(folder).exists() { safe_directory(&path.join(folder))?; } }
    Ok(path)
}

fn target_state(mut target: ClientTarget, dir: &Path) -> Result<ClientTarget, String> {
    let installed = mods(dir)?;
    target.fabric_api = installed.iter().any(|(_, id)| id == "fabric-api");
    target.companion = installed.iter().any(|(_, id)| id == "tp_companion");
    target.installed = installed.iter().any(|(_, id)| id == "tp_client");
    if let Ok(bytes) = bounded(&dir.join("config/tp-client.json"), 512 * 1024) {
        if let Ok(value) = serde_json::from_slice::<Value>(&bytes) { target.mode = value["mode"].as_str().filter(|mode| ["minimal", "full"].contains(mode)).unwrap_or("minimal").into(); }
    }
    Ok(target)
}

fn targets(username: &str) -> Result<Vec<ClientTarget>, String> {
    let root = crate::root_dir()?;
    crate::version_game_dir(&root, username, "validation")?;
    let mut result = Vec::new();
    if root.join("versions").is_dir() {
        for version in crate::list_installed_versions_inner()? {
            if version.loader != "fabric" || version.game_version != "1.21.1" || !version.jar || !version.json || !crate::content_targets::installed_profile(&root, &version.id) { continue; }
            let dir = resolve_game_dir(username, &version.id, None)?;
            result.push(target_state(ClientTarget { id:format!("version:{}",version.id), name:version.id.clone(), version_id:version.id, pack_id:None, installed:false, fabric_api:false, companion:false, mode:"minimal".into() }, &dir)?);
        }
    }
    if root.join("instances").is_dir() {
        for pack in crate::mod_studio::studio_list_packs()? {
            if pack.loader != "fabric" || pack.game_version != "1.21.1" || pack.install_state != "ready" || !crate::content_targets::installed_profile(&root, &pack.version_id) { continue; }
            let dir = resolve_game_dir(username, &pack.version_id, Some(&pack.id))?;
            result.push(target_state(ClientTarget { id:pack.id.clone(), name:pack.name, version_id:pack.version_id, pack_id:Some(pack.id), installed:false, fabric_api:false, companion:false, mode:"minimal".into() }, &dir)?);
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn tp_client_workspace(app: AppHandle, username: String) -> Result<ClientWorkspace, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let release = release()?;
        let available = artifact(&app).and_then(|path| bounded(&path, MAX_JAR).ok()).is_some_and(|bytes| verify_artifact(&bytes, &release).is_ok());
        Ok(ClientWorkspace { artifact_available:available, version:release.version, minecraft:release.minecraft, loader:release.loader, java:release.java, targets:targets(&username)? })
    }).await.map_err(|_| "No se pudo revisar TP Client".to_owned())?
}

fn install_at(dir: &Path, bytes: &[u8], expected: &Release, mode: &str, replace_companion: bool) -> Result<(), String> {
    if !["minimal", "full"].contains(&mode) { return Err("Modo de TP Client no válido".into()); }
    verify_artifact(bytes, expected)?;
    safe_directory(dir)?;
    for folder in ["mods", "config"] { safe_directory(&dir.join(folder))?; }
    let found = mods(dir)?;
    let target = dir.join("mods").join(JAR);
    for (path, id) in &found {
        if id == "tp_client" && path != &target { return Err("Existe otro JAR de TP Client. Retíralo desde Mods antes de instalar esta compilación".into()); }
    }
    if target.exists() { verify_artifact(&bounded(&target, MAX_JAR)?, expected)?; }
    let companions: Vec<_> = found.iter().filter(|(_, id)| id == "tp_companion").collect();
    if companions.len() > 1 { return Err("Hay varias instalaciones de TP Companion; revisa la carpeta Mods".into()); }
    if !companions.is_empty() && !replace_companion { return Err("TP Companion está instalado. Confirma su sustitución para evitar dos HUDs dentro del juego".into()); }
    let config_path = dir.join("config/tp-client.json");
    let old_config = if config_path.exists() { Some(String::from_utf8(bounded(&config_path, 512*1024)?).map_err(|_| "La configuración existente de TP Client no es válida")?) } else { None };
    let mut config = match &old_config { Some(text) => serde_json::from_str::<Value>(text).map_err(|_| "La configuración existente está dañada; se conserva sin modificar")?, None => json!({"schemaVersion":1}) };
    if config["schemaVersion"].as_u64().is_some_and(|version|version!=1) { return Err("La configuración pertenece a otra versión de TP Client; se conserva sin modificar".into()); }
    let object = config.as_object_mut().ok_or("La configuración existente de TP Client no es un objeto JSON")?;
    object.insert("mode".into(), mode.into()); object.insert("cosmeticsEnabled".into(), (mode == "full").into());
    let mut receipt = if dir.join(RECEIPT).exists() {
        let previous:Receipt=serde_json::from_slice(&bounded(&dir.join(RECEIPT),1024*1024)?).map_err(|_|"El registro anterior de TP Client está dañado; se conserva sin modificar")?;
        if previous.schema_version!=1 || previous.sha256!=expected.sha256 { return Err("La instalación anterior pertenece a otra compilación de TP Client".into()); }
        previous
    } else { Receipt { schema_version:1, sha256:expected.sha256.clone(), mode:mode.into(), previous_config:old_config.clone(), previous_jar:target.exists(), companion_file:None, companion_sha256:None, applied_config_sha256:None } };
    receipt.mode=mode.into();
    let config_bytes=serde_json::to_vec_pretty(&config).map_err(|_| "No se pudo preparar la configuración de TP Client")?;
    receipt.applied_config_sha256=Some(digest(&config_bytes));
    let old_receipt=if dir.join(RECEIPT).exists(){Some(bounded(&dir.join(RECEIPT),1024*1024)?)}else{None};
    let old_target=if target.exists(){Some(bounded(&target,MAX_JAR)?)}else{None};
    let backup_path=dir.join("config/tp-client-previous-companion.jar");
    let mut backup_written=None;
    if let Some((path, _)) = companions.first() {
        let filename = path.file_name().and_then(|name| name.to_str()).ok_or("Nombre de TP Companion inválido")?;
        let backup = dir.join("config/tp-client-previous-companion.jar");
        if backup.exists() { return Err("Existe un respaldo de TP Companion. Restaura o conserva esa instalación antes de sustituirla otra vez".into()); }
        let previous = bounded(path, MAX_JAR)?;
        receipt.companion_file = Some(filename.into()); receipt.companion_sha256 = Some(digest(&previous));
        atomic(&backup, &previous)?;
        backup_written=Some(previous);
    }
    let receipt_bytes=serde_json::to_vec_pretty(&receipt).map_err(|_| "No se pudo preparar el registro de instalación")?;
    let mut target_written=false;let mut config_written=false;let mut receipt_written=false;
    let result:Result<(),String>=(|| {
        atomic(&target, bytes)?;target_written=true;
        atomic(&config_path,&config_bytes)?;config_written=true;
        atomic(&dir.join(RECEIPT),&receipt_bytes)?;receipt_written=true;
        for (path, _) in companions {
            if receipt.companion_sha256.as_deref()!=Some(digest(&bounded(path,MAX_JAR)?).as_str()) { return Err("TP Companion cambió durante la instalación; se conservan el archivo y el respaldo".into()); }
            fs::remove_file(path).map_err(|_| "TP Companion sigue ocupado; cierra el juego antes de sustituirlo")?;
        }
        Ok(())
    })();
    if let Err(error)=result {
        let mut restored=true;
        if receipt_written {restored &= restore_written(&dir.join(RECEIPT),&receipt_bytes,old_receipt.as_deref()).is_ok();}
        if config_written {restored &= restore_written(&config_path,&config_bytes,old_config.as_deref().map(str::as_bytes)).is_ok();}
        if target_written {restored &= restore_written(&target,bytes,old_target.as_deref()).is_ok();}
        if restored {if let Some(previous)=backup_written {let _=restore_written(&backup_path,&previous,None);}}
        return Err(if restored {format!("{error}. La instalación se revirtió sin cambiar los archivos anteriores.")}else{format!("{error}. No se pudo completar toda la reversión; se conservaron los respaldos para revisión manual.")});
    }
    Ok(())
}

fn restore_written(path:&Path,written:&[u8],previous:Option<&[u8]>)->Result<(),String>{
    if bounded(path,2*MAX_JAR)?.as_slice()!=written{return Err("El archivo cambió durante la operación; no se sobrescribirá".into());}
    if let Some(bytes)=previous{atomic(path,bytes)}else{fs::remove_file(path).map_err(|_|"No se pudo revertir el archivo nuevo".into())}
}

fn remove_at(dir: &Path, expected: &Release, restore: bool) -> Result<(), String> {
    safe_directory(dir)?;
    for folder in ["mods", "config"] { safe_directory(&dir.join(folder))?; }
    let target = dir.join("mods").join(JAR);
    if target.exists() { verify_artifact(&bounded(&target, MAX_JAR)?, expected)?; }
    let receipt_path = dir.join(RECEIPT);
    let receipt: Receipt = serde_json::from_slice(&bounded(&receipt_path, 1024*1024)?).map_err(|_| "No existe un registro válido de instalación propia")?;
    if receipt.schema_version != 1 || receipt.sha256 != expected.sha256 { return Err("No se reconoce el registro de instalación de TP Client".into()); }
    let config = dir.join("config/tp-client.json");
    let current_config=if config.exists(){Some(bounded(&config,512*1024)?)}else{None};
    if restore && receipt.previous_config.is_some() && current_config.as_ref().is_none_or(|bytes|receipt.applied_config_sha256.as_deref()!=Some(digest(bytes).as_str())) {
        return Err("La configuración de TP Client cambió después de instalarlo. Desmarca «restaurar la configuración previa» para conservar tus ajustes y retirar solo el mod".into());
    }
    let mut companion_restore=None;
    if restore {
        if let (Some(name), Some(hash)) = (&receipt.companion_file, &receipt.companion_sha256) {
            if Path::new(name).components().count()!=1 || name.contains(['/', '\\', ':']) { return Err("Nombre del respaldo de TP Companion no permitido".into()); }
            let backup = dir.join("config/tp-client-previous-companion.jar");
            let bytes = bounded(&backup, MAX_JAR)?;
            if digest(&bytes) != *hash || metadata(&bytes)?["id"] != "tp_companion" { return Err("El respaldo de TP Companion fue modificado; no se restaura".into()); }
            let previous = dir.join("mods").join(name);
            if previous.exists() { return Err("Ya existe un archivo en el destino del respaldo de TP Companion".into()); }
            companion_restore=Some((previous,backup,bytes));
        }
    }
    if restore {if let Some(previous)=&receipt.previous_config{atomic(&config,previous.as_bytes())?;}}
    if let Some((previous,_,bytes))=&companion_restore {
        if let Err(error)=atomic(previous,bytes){
            if restore {if let (Some(old),Some(written))=(&current_config,&receipt.previous_config){let _=restore_written(&config,written.as_bytes(),Some(old));}}
            return Err(error);
        }
    }
    if target.exists() {
        if fs::remove_file(&target).is_err() {
            if let Some((previous,_,bytes))=&companion_restore{let _=restore_written(previous,bytes,None);}
            if restore{if let (Some(old),Some(written))=(&current_config,&receipt.previous_config){let _=restore_written(&config,written.as_bytes(),Some(old));}}
            return Err("Cierra Minecraft para retirar TP Client; se conserva su instalación y el respaldo".into());
        }
    }
    if fs::remove_file(&receipt_path).is_err(){
        return Err("TP Client se retiró, pero Windows mantuvo ocupado el registro. Se conservan los respaldos; revisa la instancia antes de reinstalar".into());
    }
    if let Some((_,backup,_))=companion_restore{let _=fs::remove_file(backup);}
    Ok(())
}

#[tauri::command]
pub async fn tp_client_install(app:AppHandle, username:String, version_id:String, pack_id:Option<String>, mode:String, replace_companion:bool, install_fabric_api:bool) -> Result<String,String> {
    let _lock = INSTALL.lock().await;
    if crate::game_session::is_running() { return Err("Cierra Minecraft antes de cambiar TP Client".into()); }
    let dir = resolve_game_dir(&username, &version_id, pack_id.as_deref())?;
    let path = artifact(&app).ok_or("Esta distribución no incluye un JAR compilado de TP Client")?;
    let bytes = bounded(&path, MAX_JAR)?;
    let release = release()?;
    verify_artifact(&bytes,&release)?;
    if !["minimal","full"].contains(&mode.as_str()) { return Err("Modo de TP Client no válido".into()); }
    if !mods(&dir)?.iter().any(|(_, id)| id=="fabric-api") {
        if !install_fabric_api { return Err("Instala Fabric API para 1.21.1 desde Mods en esta misma instancia o autoriza su instalación junto a TP Client".into()); }
        let client=reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).redirect(reqwest::redirect::Policy::none()).user_agent("TP-LAUNCHER/0.1.0").build().map_err(|_|"No se pudo preparar la descarga de Fabric API")?;
        let response=client.get("https://api.modrinth.com/v2/project/fabric-api/version").query(&[("game_versions","[\"1.21.1\"]"),("loaders","[\"fabric\"]")]).send().await.map_err(|_|"No se pudo consultar Fabric API en Modrinth")?;
        if !response.status().is_success() { return Err(format!("Modrinth respondió HTTP {} al consultar Fabric API",response.status().as_u16())); }
        use futures_util::StreamExt;
        let mut chunks=response.bytes_stream();let mut data=Vec::new();
        while let Some(chunk)=chunks.next().await { let chunk=chunk.map_err(|_|"La respuesta de Fabric API se interrumpió")?;if data.len()+chunk.len()>4*1024*1024{return Err("La respuesta de Fabric API supera el tamaño esperado".into());}data.extend_from_slice(&chunk); }
        let rows:Vec<Value>=serde_json::from_slice(&data).map_err(|_|"Modrinth devolvió un catálogo no válido de Fabric API")?;
        let row=rows.iter().find(|row|row["version_type"]=="release"&&row["game_versions"].as_array().is_some_and(|rows|rows.iter().any(|value|value=="1.21.1"))&&row["loaders"].as_array().is_some_and(|rows|rows.iter().any(|value|value=="fabric"))).ok_or("No hay una versión estable compatible de Fabric API")?;
        let project=row["project_id"].as_str().ok_or("Fabric API no tiene identificador de proyecto")?.to_owned();
        let file=row["id"].as_str().ok_or("Fabric API no tiene identificador de versión")?.to_owned();
        crate::mod_studio::studio_install_version(app.clone(),"modrinth".into(),project,file,"mod".into(),pack_id.clone(),username.clone(),version_id.clone()).await?;
        if !mods(&dir)?.iter().any(|(_, id)|id=="fabric-api") { return Err("No se pudo verificar Fabric API después de descargarla; TP Client no se instaló".into()); }
    }
    install_at(&dir, &bytes, &release, &mode, replace_companion)?;
    Ok("TP Client instalado y verificado. Sus cambios se aplican en el siguiente inicio; Shift derecho abre su menú dentro del juego.".into())
}

#[tauri::command]
pub async fn tp_client_remove(username:String, version_id:String, pack_id:Option<String>, restore_previous:bool) -> Result<String,String> {
    let _lock = INSTALL.lock().await;
    if crate::game_session::is_running() { return Err("Cierra Minecraft antes de cambiar TP Client".into()); }
    let dir = resolve_game_dir(&username, &version_id, pack_id.as_deref())?;
    remove_at(&dir, &release()?, restore_previous)?;
    Ok("TP Client retirado. Se conservan tus mundos, los mods ajenos y la configuración personalizada.".into())
}

pub(crate) fn installed_in(dir: &Path) -> bool { mods(dir).is_ok_and(|rows| rows.iter().any(|(_, id)|id=="tp_client")) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn jar(id:&str)->Vec<u8> {
        let mut zip=zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("fabric.mod.json",zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(json!({"id":id,"version":"0.1.0","environment":"client","depends":{"minecraft":"1.21.1"}}).to_string().as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }
    fn fixture()->(PathBuf,Vec<u8>,Release) {
        let path=std::env::temp_dir().join(format!("tp-client-installer-{}",uuid::Uuid::new_v4()));
        fs::create_dir(&path).unwrap();let bytes=jar("tp_client");
        let expected=Release{schema_version:1,version:"0.1.0".into(),minecraft:"1.21.1".into(),loader:"fabric".into(),java:21,sha256:digest(&bytes)};
        (path,bytes,expected)
    }
    #[test] fn install_retains_custom_settings_and_remove_keeps_user_mods() {
        let (path,bytes,expected)=fixture();fs::create_dir(path.join("config")).unwrap();fs::create_dir(path.join("mods")).unwrap();
        fs::write(path.join("config/tp-client.json"),br#"{"schemaVersion":1,"streamerMode":true,"mode":"minimal"}"#).unwrap();
        fs::write(path.join("mods/user-mod.jar"),b"user contents").unwrap();
        install_at(&path,&bytes,&expected,"full",false).unwrap();
        let config:Value=serde_json::from_slice(&fs::read(path.join("config/tp-client.json")).unwrap()).unwrap();
        assert_eq!(config["streamerMode"],true);assert_eq!(config["cosmeticsEnabled"],true);
        remove_at(&path,&expected,false).unwrap();assert_eq!(fs::read(path.join("mods/user-mod.jar")).unwrap(),b"user contents");assert!(path.join("config/tp-client.json").exists());
        fs::remove_dir_all(path).unwrap();
    }
    #[test] fn tampered_jar_and_unknown_modes_never_install_or_delete() {
        let (path,bytes,expected)=fixture();assert!(install_at(&path,b"wrong",&expected,"minimal",false).is_err());
        assert!(install_at(&path,&bytes,&expected,"invalid",false).is_err());
        install_at(&path,&bytes,&expected,"minimal",false).unwrap();fs::write(path.join("mods").join(JAR),b"user replaced file").unwrap();
        assert!(remove_at(&path,&expected,false).is_err());assert_eq!(fs::read(path.join("mods").join(JAR)).unwrap(),b"user replaced file");
        fs::remove_dir_all(path).unwrap();
    }
    #[test] fn companion_replacement_requires_consent_and_can_be_restored() {
        let (path,bytes,expected)=fixture();fs::create_dir(path.join("mods")).unwrap();let companion=jar("tp_companion");
        fs::write(path.join("mods/tp-companion.jar"),&companion).unwrap();
        assert!(install_at(&path,&bytes,&expected,"minimal",false).is_err());
        install_at(&path,&bytes,&expected,"minimal",true).unwrap();assert!(!path.join("mods/tp-companion.jar").exists());
        remove_at(&path,&expected,true).unwrap();assert_eq!(fs::read(path.join("mods/tp-companion.jar")).unwrap(),companion);
        fs::remove_dir_all(path).unwrap();
    }
    #[test] fn corrupt_existing_configuration_is_preserved() {
        let (path,bytes,expected)=fixture();fs::create_dir(path.join("config")).unwrap();fs::write(path.join("config/tp-client.json"),b"corrupt-user-file").unwrap();
        assert!(install_at(&path,&bytes,&expected,"minimal",false).is_err());assert_eq!(fs::read(path.join("config/tp-client.json")).unwrap(),b"corrupt-user-file");
        assert!(!path.join("mods").join(JAR).exists());fs::remove_dir_all(path).unwrap();
    }
    #[test] fn removal_never_overwrites_post_install_configuration() {
        let (path,bytes,expected)=fixture();fs::create_dir(path.join("config")).unwrap();
        fs::write(path.join("config/tp-client.json"),br#"{"schemaVersion":1,"custom":"before"}"#).unwrap();
        install_at(&path,&bytes,&expected,"minimal",false).unwrap();
        let edited=br#"{"schemaVersion":1,"custom":"player-edited"}"#;fs::write(path.join("config/tp-client.json"),edited).unwrap();
        assert!(remove_at(&path,&expected,true).unwrap_err().contains("cambió después"));
        assert_eq!(fs::read(path.join("config/tp-client.json")).unwrap(),edited);assert!(path.join("mods").join(JAR).exists());
        remove_at(&path,&expected,false).unwrap();assert_eq!(fs::read(path.join("config/tp-client.json")).unwrap(),edited);
        fs::remove_dir_all(path).unwrap();
    }
    #[test] fn corrupted_companion_backup_is_rejected_before_any_restore() {
        let (path,bytes,expected)=fixture();fs::create_dir(path.join("mods")).unwrap();fs::create_dir(path.join("config")).unwrap();
        fs::write(path.join("mods/tp-companion.jar"),jar("tp_companion")).unwrap();fs::write(path.join("config/tp-client.json"),br#"{"schemaVersion":1}"#).unwrap();
        install_at(&path,&bytes,&expected,"full",true).unwrap();let installed_config=fs::read(path.join("config/tp-client.json")).unwrap();
        fs::write(path.join("config/tp-client-previous-companion.jar"),b"corrupt backup").unwrap();
        assert!(remove_at(&path,&expected,true).is_err());assert_eq!(fs::read(path.join("config/tp-client.json")).unwrap(),installed_config);
        assert_eq!(fs::read(path.join("mods").join(JAR)).unwrap(),bytes);fs::remove_dir_all(path).unwrap();
    }
    #[cfg(windows)]
    #[test] fn locked_configuration_rolls_back_new_jar_and_companion_backup() {
        use std::os::windows::fs::OpenOptionsExt;
        let (path,bytes,expected)=fixture();fs::create_dir(path.join("mods")).unwrap();fs::create_dir(path.join("config")).unwrap();
        let old=br#"{"schemaVersion":1,"custom":"preserve"}"#;fs::write(path.join("config/tp-client.json"),old).unwrap();let companion=jar("tp_companion");fs::write(path.join("mods/tp-companion.jar"),&companion).unwrap();
        let held=fs::OpenOptions::new().read(true).share_mode(1).open(path.join("config/tp-client.json")).unwrap();
        assert!(install_at(&path,&bytes,&expected,"full",true).unwrap_err().contains("se revirtió"));
        assert!(!path.join("mods").join(JAR).exists());assert!(!path.join(RECEIPT).exists());assert!(!path.join("config/tp-client-previous-companion.jar").exists());
        assert_eq!(fs::read(path.join("config/tp-client.json")).unwrap(),old);assert_eq!(fs::read(path.join("mods/tp-companion.jar")).unwrap(),companion);
        drop(held);fs::remove_dir_all(path).unwrap();
    }
}
