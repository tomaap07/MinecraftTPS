use base64::Engine;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Instant,
};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct PlayRecord {
    pub seconds: u64,
    pub sessions: u64,
}
#[derive(Default, Clone, Serialize, Deserialize)]
struct PlayDatabase {
    players: BTreeMap<String, BTreeMap<String, PlayRecord>>,
}
struct ActiveSession {
    username: String,
    version: String,
    started: Instant,
    accounted: u64,
}
#[derive(Default)]
struct PlayState {
    database: Option<PlayDatabase>,
    active: BTreeMap<u32, ActiveSession>,
}
fn play_state() -> &'static Mutex<PlayState> {
    static STATE: OnceLock<Mutex<PlayState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(PlayState::default()))
}
pub fn active_game_count() -> usize {
    play_state()
        .lock()
        .map(|state| state.active.len())
        .unwrap_or(0)
}
#[cfg(test)]
pub fn test_session_pids(username: &str) -> Vec<u32> {
    play_state()
        .lock()
        .map(|s| {
            s.active
                .iter()
                .filter(|(_, session)| session.username.eq_ignore_ascii_case(username))
                .map(|(pid, _)| *pid)
                .collect()
        })
        .unwrap_or_default()
}
fn load_database(state: &mut PlayState) -> Result<(), String> {
    if state.database.is_none() {
        let file = crate::root_dir()?.join("playtime.json");
        state.database = Some(if file.exists() {
            serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?)
                .map_err(|e| format!("El registro de tiempo está dañado: {e}"))?
        } else {
            PlayDatabase::default()
        });
    }
    Ok(())
}
fn save_database(state: &PlayState) -> Result<(), String> {
    let root = crate::root_dir()?;
    let file = root.join("playtime.json");
    let bytes = serde_json::to_vec_pretty(&state.database).map_err(|e| e.to_string())?;
    atomic_write_playtime(&file, &bytes)
}

fn atomic_write_playtime(file: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = file.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        output.write_all(bytes).map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);


        fs::rename(&temporary, &file)
            .map_err(|e| format!("No se pudo guardar el tiempo de juego: {e}"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
pub fn start_play_session(pid: u32, username: &str, version: &str) -> Result<(), String> {
    let mut state = play_state()
        .lock()
        .map_err(|_| "Registro de sesiones ocupado")?;
    load_database(&mut state)?;
    if state.active.contains_key(&pid) {
        return Ok(());
    }
    let record = state
        .database
        .as_mut()
        .unwrap()
        .players
        .entry(username.to_ascii_lowercase())
        .or_default()
        .entry(version.to_owned())
        .or_default();
    record.sessions = record.sessions.saturating_add(1);
    state.active.insert(
        pid,
        ActiveSession {
            username: username.to_ascii_lowercase(),
            version: version.to_owned(),
            started: Instant::now(),
            accounted: 0,
        },
    );
    save_database(&state)
}
pub fn checkpoint_play_session(pid: u32, finished: bool) -> Result<(), String> {
    let mut state = play_state()
        .lock()
        .map_err(|_| "Registro de sesiones ocupado")?;
    load_database(&mut state)?;
    if let Some(session) = state.active.get_mut(&pid) {
        let elapsed = session.started.elapsed().as_secs();
        let update = (
            session.username.clone(),
            session.version.clone(),
            elapsed.saturating_sub(session.accounted),
        );
        session.accounted = elapsed;
        let record = state
            .database
            .as_mut()
            .unwrap()
            .players
            .entry(update.0)
            .or_default()
            .entry(update.1)
            .or_default();
        record.seconds = record.seconds.saturating_add(update.2);
    }
    if finished {
        state.active.remove(&pid);
    }
    save_database(&state)
}
#[derive(Serialize)]
pub struct PlaytimeSummary {
    pub total_seconds: u64,
    pub session_seconds: u64,
    pub sessions: u64,
    pub active_games: usize,
    pub versions: BTreeMap<String, PlayRecord>,
}
#[tauri::command]
pub fn get_playtime(username: String) -> Result<PlaytimeSummary, String> {
    let mut state = play_state()
        .lock()
        .map_err(|_| "Registro de sesiones ocupado")?;
    load_database(&mut state)?;
    let key = username.to_ascii_lowercase();
    let mut versions = state
        .database
        .as_ref()
        .unwrap()
        .players
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let mut session_seconds = 0_u64;
    let mut active_games = 0;
    for session in state.active.values().filter(|s| s.username == key) {
        let seconds = session.started.elapsed().as_secs();
        session_seconds = session_seconds.saturating_add(seconds);
        active_games += 1;
        let record = versions.entry(session.version.clone()).or_default();
        record.seconds = record
            .seconds
            .saturating_add(seconds.saturating_sub(session.accounted));
    }
    Ok(PlaytimeSummary {
        total_seconds: versions
            .values()
            .fold(0_u64, |total, v| total.saturating_add(v.seconds)),
        sessions: versions
            .values()
            .fold(0_u64, |total, v| total.saturating_add(v.sessions)),
        session_seconds,
        active_games,
        versions,
    })
}

#[cfg(test)]
pub fn boost_jvm_flags(java_major: u8, arguments: &mut Vec<String>) {
    if java_major < 8 {
        return;
    }

    if !arguments
        .iter()
        .any(|a| a.starts_with("-XX:+Use") && a.ends_with("GC"))
    {
        arguments.push("-XX:+UseG1GC".into());
    }
    for flag in [
        "-XX:+UnlockExperimentalVMOptions",
        "-XX:+ParallelRefProcEnabled",
        "-XX:MaxGCPauseMillis=200",
        "-XX:+DisableExplicitGC",
    ] {
        let key = flag.split('=').next().unwrap_or(flag);
        if !arguments.iter().any(|a| a.starts_with(key)) {
            arguments.push(flag.into());
        }
    }
}

#[derive(Clone, Serialize)]
pub struct PlayerAppearance {
    pub name: String,
    pub skin_data: String,
    pub cape_data: String,
    pub model: String,
    pub avatar_data: String,
}
const STEVE_UUID:&str="c06f89064c8a49119c29ea1dbd1aab82";
const ALEX_UUID:&str="6ab4317889fd490597f60f67d9d76fd9";
static DEFAULT_APPEARANCE_LOCK:tokio::sync::Mutex<()>=tokio::sync::Mutex::const_new(());
fn appearance_uuid(value:&str)->Result<String,String>{
 let id=uuid::Uuid::parse_str(value.trim()).map_err(|_|"La apariencia requiere un UUID válido de Minecraft")?;
 if id.is_nil(){return Err("La apariencia requiere un UUID de jugador, no un nombre".into());}Ok(id.simple().to_string())
}
fn crafatar_skin_url(id:&str,variant:&str)->Result<String,String>{
 let id=appearance_uuid(id)?;let fallback=if variant=="alex"{"MHF_Alex"}else{"MHF_Steve"};
 Ok(format!("https://crafatar.com/skins/{id}?default={fallback}"))
}
fn public_appearance_client()->Result<reqwest::Client,String>{
 reqwest::Client::builder().connect_timeout(std::time::Duration::from_secs(3)).timeout(std::time::Duration::from_secs(6)).user_agent("TP LAUNCHER/0.1")
 .redirect(reqwest::redirect::Policy::custom(|attempt|{
  if attempt.previous().len()>3||attempt.url().scheme()!="https"||!matches!(attempt.url().host_str(),Some("api.mojang.com"|"api.minecraftservices.com"|"sessionserver.mojang.com"|"textures.minecraft.net"|"assets.mojang.com"|"crafatar.com"|"mc-heads.net"|"minotar.net"))||!attempt.url().username().is_empty()||attempt.url().password().is_some()||attempt.url().port_or_known_default()!=Some(443){attempt.stop()}else{attempt.follow()}
 })).build().map_err(|e|e.to_string())
}
fn skin_model(image:&image::DynamicImage)->&'static str{
 if image.height()==64{let pixels=image.to_rgba8();if [(54,20),(55,20),(46,52),(47,52)].iter().all(|&(x,y)|pixels.get_pixel(x,y)[3]==0){return "slim";}}"default"
}
fn default_appearance(variant:&str,skin:&image::DynamicImage)->Result<PlayerAppearance,String>{
 Ok(PlayerAppearance{name:if variant=="alex"{"Alex"}else{"Steve"}.into(),skin_data:png_data(skin)?,cape_data:String::new(),model:skin_model(skin).into(),avatar_data:png_data(&avatar_image(skin))?})
}
fn default_cache_path(variant:&str)->Result<PathBuf,String>{
 let folder=crate::launcher_settings::config_root()?.join("appearance-defaults-v1");fs::create_dir_all(&folder).map_err(|e|e.to_string())?;Ok(folder.join(format!("{variant}.png")))
}
fn cached_default(path:&Path)->Option<(image::DynamicImage,bool)>{
 let meta=fs::symlink_metadata(path).ok()?;if meta.file_type().is_symlink()||!meta.is_file()||meta.len()>2*1024*1024{return None;}
 let fresh=meta.modified().ok().and_then(|v|v.elapsed().ok()).is_some_and(|age|age<std::time::Duration::from_secs(24*60*60));
 let mut bytes=Vec::new();fs::File::open(path).ok()?.take(2*1024*1024+1).read_to_end(&mut bytes).ok()?;
 let image=decode_bounded_png(&bytes,64,64,4096,1024*1024).ok()?;if image.width()!=64||![32,64].contains(&image.height()){return None;}Some((image,fresh))
}
fn cache_default(path:&Path,image:&image::DynamicImage)->Result<(),String>{
 let pending=path.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));
 let mut bytes=Cursor::new(Vec::new());image.write_to(&mut bytes,image::ImageFormat::Png).map_err(|e|e.to_string())?;
 fs::write(&pending,bytes.into_inner()).and_then(|_|fs::rename(&pending,path)).map_err(|e|{let _=fs::remove_file(pending);e.to_string()})
}
#[tauri::command]
pub async fn get_default_player_appearance(variant:Option<String>)->Result<PlayerAppearance,String>{
 let variant=variant.unwrap_or_else(||"steve".into()).to_ascii_lowercase();if !["steve","alex"].contains(&variant.as_str()){return Err("Elige Steve o Alex como apariencia predeterminada".into());}
 let _guard=DEFAULT_APPEARANCE_LOCK.lock().await;let path=default_cache_path(&variant)?;let cached=cached_default(&path);
 if let Some((ref skin,true))=cached{return default_appearance(&variant,skin);}
 let client=public_appearance_client()?;let id=if variant=="alex"{ALEX_UUID}else{STEVE_UUID};
 let result=tokio::time::timeout(std::time::Duration::from_secs(16),async{


  for address in [format!("https://assets.mojang.com/SkinTemplates/{variant}.png"),crafatar_skin_url(id,&variant)?]{
   if let Ok(image)=texture_image(&client,&address).await{if image.width()==64&&[32,64].contains(&image.height()){return Ok::<_,String>(image);}}
  }Err("No se pudo descargar la skin predeterminada desde Mojang o Crafatar".into())
 }).await.unwrap_or_else(|_|Err("La descarga de la skin predeterminada tardó demasiado".into()));
 match result{Ok(image)=>{let _=cache_default(&path,&image);default_appearance(&variant,&image)},Err(error)=>cached.map(|(skin,_)|default_appearance(&variant,&skin)).unwrap_or_else(||Err(format!("{error}. No hay una copia local guardada todavía")))}
}
struct AppearanceCacheEntry {
    cached_at: Instant,
    lifetime: std::time::Duration,
    result: Result<Option<PlayerAppearance>, String>,
}
fn appearance_cache() -> &'static Mutex<BTreeMap<String, AppearanceCacheEntry>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, AppearanceCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn appearance_cache_size(entry: &AppearanceCacheEntry) -> usize {
    match &entry.result {
        Ok(Some(appearance)) => {
            appearance.skin_data.len() + appearance.cape_data.len() + appearance.avatar_data.len()
        }
        Err(error) => error.len(),
        _ => 0,
    }
}
fn cache_appearance(nick: &str, result: Result<Option<PlayerAppearance>, String>) {
    let Ok(mut cache) = appearance_cache().lock() else {
        return;
    };
    cache.retain(|_, entry| entry.cached_at.elapsed() < entry.lifetime);

    let lifetime = std::time::Duration::from_secs(match &result {
        Ok(Some(_)) => 10 * 60,
        Ok(None) => 60,
        Err(_) => 10,
    });
    cache.insert(
        nick.to_ascii_lowercase(),
        AppearanceCacheEntry {
            cached_at: Instant::now(),
            lifetime,
            result,
        },
    );
    while cache.len() > 128
        || cache.values().map(appearance_cache_size).sum::<usize>() > 16 * 1024 * 1024
    {
        let Some(oldest) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.cached_at)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        cache.remove(&oldest);
    }
}
fn png_data(image: &image::DynamicImage) -> Result<String, String> {
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(buffer.into_inner())
    ))
}
async fn texture_image(
    client: &reqwest::Client,
    address: &str,
) -> Result<image::DynamicImage, String> {
    let url = appearance_texture_url(address)?;
    let response = if url.host_str()==Some("crafatar.com"){



        let response=client.get(url).send().await.map_err(|e|e.to_string())?;
        if !response.status().is_success()&&response.status()!=reqwest::StatusCode::INTERNAL_SERVER_ERROR{return Err("Crafatar no pudo servir la apariencia".into());}response
    }else{appearance_response(client, url).await?.ok_or("No se encontró la textura pública del jugador")?};
    let bytes = appearance_response_bytes(response, 2 * 1024 * 1024).await?;
    decode_bounded_png(&bytes, 2048, 2048, 4 * 1024 * 1024, 32 * 1024 * 1024)
}
fn appearance_texture_url(address: &str) -> Result<url::Url, String> {
    let mut url = url::Url::parse(address).map_err(|e| e.to_string())?;
    if !matches!(
        url.host_str(),
        Some("textures.minecraft.net" | "assets.mojang.com" | "crafatar.com" | "mc-heads.net" | "minotar.net")
    ) || !(url.scheme() == "https"
        || (url.scheme() == "http" && url.host_str() == Some("textures.minecraft.net")))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some_and(|port| {
            if url.scheme() == "http" {
                port != 80
            } else {
                port != 443
            }
        })
    {
        return Err("La textura debe proceder de Mojang, Crafatar, MCHeads o Minotar por HTTPS".into());
    }
    if url.host_str()==Some("assets.mojang.com")&&!matches!(url.path(),"/SkinTemplates/steve.png"|"/SkinTemplates/alex.png"){return Err("Plantilla predeterminada de Mojang no permitida".into());}
    if url.host_str()==Some("crafatar.com"){
        let id=url.path().strip_prefix("/skins/").ok_or("Solo se admiten skins UUID de Crafatar")?;
        appearance_uuid(id)?;
        if !url.query_pairs().all(|(key,value)|key=="default"&&["MHF_Steve","MHF_Alex"].contains(&value.as_ref())){return Err("Parámetros de Crafatar no permitidos".into());}
    }
    url.set_scheme("https")
        .map_err(|_| "Dirección de textura inválida")?;
    url.set_port(None)
        .map_err(|_| "Dirección de textura inválida")?;
    Ok(url)
}
async fn appearance_response(
    client: &reqwest::Client,
    url: url::Url,
) -> Result<Option<reqwest::Response>, String> {
    for attempt in 0..2 {
        match client.get(url.clone()).send().await {
            Ok(response) => {
                if matches!(
                    response.status(),
                    reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::NO_CONTENT
                ) {
                    return Ok(None);
                }
                if response.status().is_server_error() && attempt == 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                    continue;
                }
                return response
                    .error_for_status()
                    .map(Some)
                    .map_err(|e| e.to_string());
            }
            Err(error) if attempt == 0 && (error.is_connect() || error.is_timeout()) => {
                tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("No se pudo consultar el aspecto público del jugador".into())
}
async fn appearance_response_bytes(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, String> {
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err("Textura demasiado grande".into());
    }
    let mut bytes = Vec::new();
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        if bytes.len() + chunk.len() > limit {
            return Err("Textura demasiado grande".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
async fn appearance_json(
    client: &reqwest::Client,
    address: &str,
) -> Result<Option<serde_json::Value>, String> {
    let url = url::Url::parse(address).map_err(|e| e.to_string())?;
    let Some(response) = appearance_response(client, url).await? else {
        return Ok(None);
    };
    let bytes = appearance_response_bytes(response, 128 * 1024).await?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| e.to_string())
}

fn avatar_image(skin: &image::DynamicImage) -> image::DynamicImage {
    let mut head = skin.crop_imm(8, 8, 8, 8).to_rgba8();
    let pixels = skin.to_rgba8();


    let old_opaque_hat =
        skin.height() == 32 && (0..32).all(|y| (32..64).all(|x| pixels.get_pixel(x, y)[3] >= 128));
    if !old_opaque_hat {
        image::imageops::overlay(&mut head, &skin.crop_imm(40, 8, 8, 8).to_rgba8(), 0, 0);
    }
    image::DynamicImage::ImageRgba8(image::imageops::resize(
        &head,
        64,
        64,
        image::imageops::FilterType::Nearest,
    ))
}

fn decode_bounded_png(
    bytes: &[u8],
    max_width: u32,
    max_height: u32,
    max_pixels: u64,
    max_alloc: u64,
) -> Result<image::DynamicImage, String> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("El archivo no es una imagen PNG válida".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| "PNG incompleto")?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| "PNG incompleto")?);
    if width == 0
        || height == 0
        || width > max_width
        || height > max_height
        || u64::from(width) * u64::from(height) > max_pixels
    {
        return Err("Las dimensiones de la imagen exceden el límite de memoria".into());
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(max_width);
    limits.max_image_height = Some(max_height);
    limits.max_alloc = Some(max_alloc);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|e| format!("No se pudo leer la imagen: {e}"))
}
#[tauri::command]
pub async fn resolve_player_appearance(nick: String) -> Result<Option<PlayerAppearance>, String> {
    let nick = nick.trim();
    if !(3..=16).contains(&nick.len())
        || !nick.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Ok(None);
    }
    if let Ok(cache) = appearance_cache().lock() {
        if let Some(entry) = cache
            .get(&nick.to_ascii_lowercase())
            .filter(|entry| entry.cached_at.elapsed() < entry.lifetime)
        {
            return entry.result.clone();
        }
    }
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(22),
        resolve_public_appearance(nick),
    )
    .await
    .unwrap_or_else(|_| {
        Err("La consulta de la skin tardó demasiado; se usará Steve temporalmente".into())
    });
    cache_appearance(nick, result.clone());
    result
}
async fn resolve_public_appearance(nick: &str) -> Result<Option<PlayerAppearance>, String> {
    let client = public_appearance_client()?;


    let mut profile = None;
    let mut lookup_error = None;
    let mut absent = false;
    for address in [
        format!("https://api.mojang.com/users/profiles/minecraft/{nick}"),
        format!("https://api.minecraftservices.com/minecraft/profile/lookup/name/{nick}"),
    ] {
        match appearance_json(&client, &address).await {
            Ok(Some(account))
                if account["name"]
                    .as_str()
                    .is_some_and(|name| name.eq_ignore_ascii_case(nick))
                    && account["id"].as_str().is_some_and(|id| {
                        id.len() == 32 && id.bytes().all(|c| c.is_ascii_hexdigit())
                    }) =>
            {
                profile = Some(account);
                break;
            }
            Ok(None) => absent = true,
            Ok(Some(_)) => {
                lookup_error =
                    Some("El servicio no devolvió un perfil premium válido para ese nick".into())
            }
            Err(error) => lookup_error = Some(error),
        }
    }
    let Some(profile) = profile else {
        return if absent {
            Ok(None)
        } else {
            Err(lookup_error.unwrap_or_else(|| "No se pudo consultar el perfil premium".into()))
        };
    };
    let id = profile["id"]
        .as_str()
        .filter(|id| id.len() == 32 && id.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or("La cuenta no tiene un UUID válido")?;
    resolve_uuid_appearance(&client,id,profile["name"].as_str().unwrap_or(nick)).await.map(Some)
}
#[tauri::command]
pub async fn get_player_appearance_by_uuid(uuid:String)->Result<PlayerAppearance,String>{
 let id=appearance_uuid(&uuid)?;let key=format!("uuid:{id}");
 if let Ok(cache)=appearance_cache().lock(){if let Some(entry)=cache.get(&key).filter(|e|e.cached_at.elapsed()<e.lifetime){return entry.result.clone().and_then(|a|a.ok_or("No se encontró la apariencia de este UUID".into()));}}
 let result=tokio::time::timeout(std::time::Duration::from_secs(22),async{let client=public_appearance_client()?;resolve_uuid_appearance(&client,&id,"").await}).await.unwrap_or_else(|_|Err("La consulta pública por UUID tardó demasiado".into()));
 cache_appearance(&key,result.clone().map(Some));result
}
async fn resolve_uuid_appearance(client:&reqwest::Client,id:&str,name:&str)->Result<PlayerAppearance,String>{
    let session = appearance_json(
        &client,
        &format!("https://sessionserver.mojang.com/session/minecraft/profile/{id}"),
    )
    .await
    .ok()
    .flatten();
    let textures = session
        .as_ref()
        .filter(|session| {
            session["id"]
                .as_str()
                .is_some_and(|session_id| session_id.eq_ignore_ascii_case(id))
        })
        .and_then(|session| session["properties"].as_array())
        .and_then(|properties| {
            properties
                .iter()
                .find(|property| property["name"] == "textures")
        })
        .and_then(|property| property["value"].as_str())
        .and_then(|encoded| {
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .ok()
        })
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .unwrap_or(serde_json::Value::Null);
    let mut skin_urls = Vec::new();
    if let Some(skin_url) = textures
        .pointer("/textures/SKIN/url")
        .and_then(|v| v.as_str())
    {
        skin_urls.push(skin_url.to_owned());
    }
    skin_urls.push(crafatar_skin_url(id,"steve")?);
    skin_urls.push(format!("https://mc-heads.net/skin/{id}"));
    skin_urls.push(format!("https://minotar.net/skin/{id}"));
    let mut skin = None;
    let mut downloaded_direct=false;
    for address in skin_urls {
        if let Ok(image) = texture_image(&client, &address).await {
            if image.width() == 64 && [32, 64].contains(&image.height()) {
                downloaded_direct=textures.pointer("/textures/SKIN/url").and_then(serde_json::Value::as_str)==Some(address.as_str());
                skin = Some(image);
                break;
            }
        }
    }
    let skin = skin.ok_or("La cuenta existe, pero no se pudo descargar su skin pública")?;
    let head = avatar_image(&skin);
    let cape_data = if let Some(cape_url) = textures
        .pointer("/textures/CAPE/url")
        .and_then(|v| v.as_str())
    {
        match texture_image(&client, cape_url).await {
            Ok(cape) => png_data(&cape)?,
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };
    Ok(PlayerAppearance {
        name: if name.is_empty(){session.as_ref().and_then(|s|s["name"].as_str()).unwrap_or("Jugador")}else{name}.into(),
        skin_data: png_data(&skin)?,
        cape_data,
        model: if downloaded_direct{textures
            .pointer("/textures/SKIN/metadata/model")
            .and_then(|v| v.as_str())
            .filter(|model|["default","slim"].contains(model))
            .unwrap_or("default")}else{skin_model(&skin)}.into(),
        avatar_data: png_data(&head)?,
    })
}

#[derive(Serialize)]
pub struct Screenshot {
    pub id: String,
    pub name: String,
    pub instance: String,
    pub modified: u64,
    pub thumbnail: String,
}
fn screenshot_files() -> Result<Vec<PathBuf>, String> {
    let root = crate::root_dir()?;
    let mut pending = vec![(root.join("instances"), 0)];
    let mut folders = vec![root.join("screenshots")];
    while let Some((folder, depth)) = pending.pop() {
        if depth > 5 {
            continue;
        }
        let Ok(entries) = fs::read_dir(folder) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if entry.file_name() == "screenshots" {
                    folders.push(entry.path());
                } else if depth == 0 || entry.file_name() == "profiles" || depth == 2 {
                    pending.push((entry.path(), depth + 1));
                }
            }
        }
    }
    let mut files = Vec::new();
    for folder in folders {
        if let Ok(entries) = fs::read_dir(folder) {
            for entry in entries.filter_map(Result::ok) {
                if entry.file_type().is_ok_and(|t| t.is_file())
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("png"))
                {
                    files.push(entry.path());
                }
            }
        }
    }
    files.sort_by_key(|path| {
        std::cmp::Reverse(
            fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH),
        )
    });
    Ok(files)
}
fn safe_screenshot(id: &str) -> Result<PathBuf, String> {
    let root = crate::root_dir()?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let relative = Path::new(id);
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("Ruta de captura inválida".into());
    }
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(&root)
        || !path.is_file()
        || path
            .parent()
            .and_then(|p| p.file_name())
            .is_none_or(|name| name != "screenshots")
        || path
            .extension()
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("png"))
    {
        return Err("La imagen no es una captura del launcher".into());
    }
    Ok(path)
}

fn screenshot_image(path: &Path) -> Result<image::DynamicImage, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Captura demasiado grande: máximo 32 MB".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err("Captura demasiado grande: máximo 32 MB".into());
    }
    decode_bounded_png(&bytes, 16384, 16384, 40_000_000, 256 * 1024 * 1024)
}
fn collect_screenshots_from(paths: Vec<PathBuf>) -> Result<Vec<Screenshot>, String> {
    let root = crate::root_dir()?;
    let mut result = Vec::new();
    for path in paths {
        let relative = path.strip_prefix(&root).map_err(|e| e.to_string())?;
        let Ok(safe_path) = safe_screenshot(relative.to_string_lossy().as_ref()) else {
            continue;
        };
        let Ok(metadata) = fs::metadata(&safe_path) else {
            continue;
        };
        let Ok(image) = screenshot_image(&safe_path) else {
            continue;
        };
        result.push(Screenshot {
            id: relative.to_string_lossy().into(),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            instance: relative
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            modified: metadata
                .modified()
                .unwrap_or(std::time::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            thumbnail: png_data(&image.thumbnail(480, 270))?,
        });
    }
    Ok(result)
}
#[tauri::command]
pub async fn list_screenshots() -> Result<Vec<Screenshot>, String> {
    tokio::task::spawn_blocking(|| collect_screenshots_from(screenshot_files()?.into_iter().take(48).collect()))
        .await
        .map_err(|e| e.to_string())?
}
#[derive(Serialize)]
pub struct ScreenshotPage {
    pub items: Vec<Screenshot>,
    pub total: usize,
    pub page: usize,
    pub pages: usize,
}
pub(crate) fn latest_hud_screenshot() -> Option<(image::DynamicImage,String)> {

    for path in screenshot_files().ok()?.into_iter().take(12) {
        let relative=path.strip_prefix(crate::root_dir().ok()?).ok()?.to_string_lossy().into_owned();
        if let Ok(safe)=safe_screenshot(&relative) { if let Ok(image)=screenshot_image(&safe) {
            return Some((image,path.file_name()?.to_string_lossy().into_owned()));
        }}
    }
    None
}
#[tauri::command]
pub async fn screenshots_page(page: u32) -> Result<ScreenshotPage, String> {
    tokio::task::spawn_blocking(move || {
        let paths = screenshot_files()?;
        let total = paths.len();
        let pages = total.div_ceil(24).max(1);
        let page = (page.max(1) as usize).min(pages);
        let items = collect_screenshots_from(paths.into_iter().skip((page - 1) * 24).take(24).collect())?;
        Ok(ScreenshotPage { items, total, page, pages })
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn screenshot_data(id: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let path = safe_screenshot(&id)?;
        png_data(&screenshot_image(&path)?)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn open_screenshot(app: AppHandle, id: String) -> Result<(), String> {
    app.opener()
        .open_path(
            safe_screenshot(&id)?.to_string_lossy().as_ref(),
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn open_player_folder(
    app: AppHandle,
    username: String,
    version_id: String,
    instance_id: Option<String>,
    folder: String,
) -> Result<(), String> {
    let root = crate::root_dir()?;
    let instance = if let Some(id) = instance_id.filter(|s| !s.is_empty()) {
        crate::mod_studio::pack_game_dir(&id)?
    } else {
        crate::version_game_dir(&root, &username, &version_id)?
    };
    let path = match folder.as_str() {
        "root" => root,
        "instance" => instance,
        "screenshots" | "saves" | "shaderpacks" | "resourcepacks" => instance.join(folder),
        _ => return Err("Acceso rápido desconocido".into()),
    };
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crafatar_requires_real_uuid_shape_and_safe_default_parameters(){
        let expected=format!("https://crafatar.com/skins/{STEVE_UUID}?default=MHF_Steve");
        assert_eq!(crafatar_skin_url("c06f8906-4c8a-4911-9c29-ea1dbd1aab82","steve").unwrap(),expected);
        assert!(appearance_texture_url(&expected).is_ok());
        assert!(appearance_texture_url("https://assets.mojang.com/SkinTemplates/alex.png").is_ok());
        for value in ["Steve","Willyrex","https://example.com/image.png","00000000000000000000000000000000"]{assert!(appearance_uuid(value).is_err());}
        for value in ["https://crafatar.com/skins/Steve","https://crafatar.com.evil.invalid/skins/c06f89064c8a49119c29ea1dbd1aab82","http://crafatar.com/skins/c06f89064c8a49119c29ea1dbd1aab82","https://crafatar.com/skins/c06f89064c8a49119c29ea1dbd1aab82?default=https://evil.invalid/skin.png","https://assets.mojang.com/other.png"]{assert!(appearance_texture_url(value).is_err());}
    }
    #[test]
    fn default_skin_private_cache_roundtrips_and_rejects_invalid_images(){
        let folder=std::env::temp_dir().join(format!("tp-default-skin-{}",uuid::Uuid::new_v4()));fs::create_dir(&folder).unwrap();let path=folder.join("steve.png");
        let mut skin=image::RgbaImage::from_pixel(64,64,image::Rgba([120,90,70,255]));
        let classic=image::DynamicImage::ImageRgba8(skin.clone());assert_eq!(skin_model(&classic),"default");cache_default(&path,&classic).unwrap();let(cached,fresh)=cached_default(&path).unwrap();assert!(fresh);assert_eq!(cached.to_rgba8(),classic.to_rgba8());
        for (x,y) in [(54,20),(55,20),(46,52),(47,52)]{skin.get_pixel_mut(x,y)[3]=0;}assert_eq!(skin_model(&image::DynamicImage::ImageRgba8(skin)),"slim");
        fs::write(&path,b"<html>server failure</html>").unwrap();assert!(cached_default(&path).is_none());
        let oversized=image::DynamicImage::ImageRgba8(image::RgbaImage::new(65,64));cache_default(&path,&oversized).unwrap();assert!(cached_default(&path).is_none());fs::remove_file(path).unwrap();fs::remove_dir(folder).unwrap();
    }
    #[tokio::test]
    #[ignore="Consulta solo PNG públicos de Mojang/Crafatar; no escribe en carpetas del usuario"]
    async fn default_skin_live_sources_return_classic_and_slim_pngs(){
        let client=public_appearance_client().unwrap();
        for(variant,model)in[("steve","default"),("alex","slim")]{let skin=texture_image(&client,&format!("https://assets.mojang.com/SkinTemplates/{variant}.png")).await.unwrap();assert_eq!((skin.width(),skin.height()),(64,64));let result=default_appearance(variant,&skin).unwrap();assert_eq!(result.model,model);assert!(result.skin_data.starts_with("data:image/png;base64,"));assert!(result.avatar_data.starts_with("data:image/png;base64,"));}
        let mirrored=texture_image(&client,&crafatar_skin_url(STEVE_UUID,"steve").unwrap()).await.unwrap();assert_eq!(mirrored.width(),64);assert!([32,64].contains(&mirrored.height()));
    }
    #[tokio::test]
    #[ignore = "Consulta las APIs públicas de Minecraft y descarga PNG; requiere conexión"]
    async fn public_willyrex_appearance_resolves_pngs_and_reuses_cache() {
        appearance_cache().lock().unwrap().remove("willyrex");
        let first = resolve_player_appearance("Willyrex".into())
            .await
            .expect("La consulta pública de Willyrex debe responder")
            .expect("Willyrex debe tener una cuenta premium pública");
        assert_eq!(first.name, "Willyrex");
        assert!(matches!(first.model.as_str(), "default" | "slim"));
        let decode_data_url = |data: &str| {
            let encoded = data
                .strip_prefix("data:image/png;base64,")
                .expect("Debe ser PNG base64");
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .expect("Base64 válido");
            decode_bounded_png(&bytes, 2048, 2048, 4 * 1024 * 1024, 32 * 1024 * 1024)
                .expect("PNG válido")
        };
        let skin = decode_data_url(&first.skin_data);
        assert_eq!(skin.width(), 64);
        assert!([32, 64].contains(&skin.height()));
        let avatar = decode_data_url(&first.avatar_data);
        assert_eq!((avatar.width(), avatar.height()), (64, 64));
        if !first.cape_data.is_empty() {
            let cape = decode_data_url(&first.cape_data);
            assert!(cape.width() > 0 && cape.height() > 0);
        }
        let cached_at = appearance_cache().lock().unwrap()["willyrex"].cached_at;
        let second = resolve_player_appearance("wIlLyReX".into())
            .await
            .expect("La caché debe responder")
            .expect("La caché debe conservar el perfil");
        assert_eq!(second.name, first.name);
        assert_eq!(second.model, first.model);

        assert!(
            second.skin_data == first.skin_data,
            "La skin de la caché debe coincidir"
        );
        assert!(
            second.cape_data == first.cape_data,
            "La capa de la caché debe coincidir"
        );
        assert!(
            second.avatar_data == first.avatar_data,
            "El avatar de la caché debe coincidir"
        );
        assert_eq!(
            appearance_cache().lock().unwrap()["willyrex"].cached_at,
            cached_at
        );
    }
    #[test]
    fn public_skin_mirrors_are_https_only_and_reject_foreign_urls() {
        assert_eq!(
            appearance_texture_url("http://textures.minecraft.net/texture/skin")
                .unwrap()
                .scheme(),
            "https"
        );
        assert!(
            appearance_texture_url("https://mc-heads.net/skin/eea22e6e7b1a425badebc0f158ea394f")
                .is_ok()
        );
        assert!(
            appearance_texture_url("https://minotar.net/skin/eea22e6e7b1a425badebc0f158ea394f")
                .is_ok()
        );
        for address in [
            "http://mc-heads.net/skin/Steve",
            "https://mc-heads.net.evil.invalid/skin/Steve",
            "https://evil.invalid@mc-heads.net/skin/Steve",
            "https://textures.minecraft.net:444/texture/skin",
            "https://mc-heads.net:80/skin/Steve",
            "file:///skin.png",
        ] {
            assert!(
                appearance_texture_url(address).is_err(),
                "Unexpected texture URL accepted: {address}"
            );
        }
    }
    #[test]
    fn boost_avoids_conflicting_collectors_and_duplicate_flags() {
        let mut flags = vec!["-XX:+UseZGC".into(), "-XX:MaxGCPauseMillis=100".into()];
        boost_jvm_flags(21, &mut flags);
        assert!(!flags.iter().any(|flag| flag == "-XX:+UseG1GC"));
        assert_eq!(
            flags
                .iter()
                .filter(|flag| flag.starts_with("-XX:MaxGCPauseMillis="))
                .count(),
            1
        );
        let before = flags.clone();
        boost_jvm_flags(21, &mut flags);
        assert_eq!(flags, before);
    }

    #[test]
    fn legacy_opaque_hat_does_not_cover_avatar_but_modern_hat_does() {
        let mut legacy = image::RgbaImage::from_pixel(64, 32, image::Rgba([200, 50, 40, 255]));
        for y in 8..16 {
            for x in 40..48 {
                legacy.put_pixel(x, y, image::Rgba([10, 30, 240, 255]));
            }
        }
        let head = avatar_image(&image::DynamicImage::ImageRgba8(legacy.clone())).to_rgba8();
        assert_eq!(*head.get_pixel(0, 0), image::Rgba([200, 50, 40, 255]));
        legacy.put_pixel(32, 0, image::Rgba([0, 0, 0, 0]));
        let head = avatar_image(&image::DynamicImage::ImageRgba8(legacy)).to_rgba8();
        assert_eq!(*head.get_pixel(0, 0), image::Rgba([10, 30, 240, 255]));
        let modern = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            64,
            64,
            image::Rgba([10, 30, 240, 255]),
        ));
        assert_eq!(
            *avatar_image(&modern).to_rgba8().get_pixel(0, 0),
            image::Rgba([10, 30, 240, 255])
        );
    }

    #[test]
    fn oversized_png_header_is_rejected_before_decompression() {
        let mut bytes = Vec::from(&b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"[..]);
        bytes.extend_from_slice(&100_000_u32.to_be_bytes());
        bytes.extend_from_slice(&100_000_u32.to_be_bytes());
        assert!(
            decode_bounded_png(&bytes, 16384, 16384, 40_000_000, 256 * 1024 * 1024)
                .unwrap_err()
                .contains("dimensiones")
        );
    }

    #[test]
    fn playtime_replaces_existing_file_without_losing_previous_format() {
        let directory = std::env::temp_dir().join(format!(
            "minecrafttps-playtime-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&directory).unwrap();
        let file = directory.join("playtime.json");
        atomic_write_playtime(&file, b"{\"seconds\":1}").unwrap();
        atomic_write_playtime(&file, b"{\"seconds\":2}").unwrap();
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(value["seconds"], 2);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_file(file).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
