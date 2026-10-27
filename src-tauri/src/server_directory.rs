
use base64::Engine;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use std::{collections::{HashMap, HashSet}, fs, future::Future, io::Cursor, sync::{Mutex, OnceLock}, time::{Duration, Instant}};
const PAGE_SIZE: u64 = 12;
const MAX_RESPONSE: usize = 4 * 1024 * 1024;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DirectoryMotd {
    pub text: String, pub color: String, pub bold: bool, pub italic: bool,
    pub underlined: bool, pub strikethrough: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub id: String, pub name: String, pub address: String, pub description: String,
    pub version: String, pub tags: Vec<String>, pub online: Option<bool>,
    pub players_online: Option<u64>, pub players_max: Option<u64>, pub checked_at: String,
    pub java_compatible: bool,
    #[serde(default)] pub icon: String,
    #[serde(default)] pub motd: Vec<DirectoryMotd>,
    #[serde(default)] pub category: String,
    #[serde(default)] pub region: String,
    #[serde(default)] pub language: String,
    #[serde(default)] pub featured: bool,
    #[serde(default)] pub featured_until: String,
    #[serde(default)] pub sponsor_url: String,
    #[serde(default)] pub from_cloud: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct DirectoryPage {
    pub entries: Vec<DirectoryEntry>, pub total: u64, pub page: u64, pub pages: u64,
    pub source: String, pub stale: bool, pub message: String,
}
#[derive(Serialize)]
pub struct DirectoryConfig { pub configured: bool, pub base_url: String, pub publishing_available: bool }
type PageCache = HashMap<String, (Instant, DirectoryPage)>;
fn pages_cache() -> &'static Mutex<PageCache> {
    static CACHE: OnceLock<Mutex<PageCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
fn clean(value: &Value, limit: usize) -> String {
    value.as_str().unwrap_or_default().chars().filter(|c| !c.is_control() || *c == '\n').take(limit).collect()
}
fn https(value: &str) -> String {
    url::Url::parse(value).ok().filter(|url| url.scheme() == "https" && url.username().is_empty() && url.password().is_none())
        .map(|url| url.to_string()).unwrap_or_default()
}
fn configured_base() -> Option<String> {
    let configured = std::env::var("TP_API_URL").ok().filter(|v| !v.trim().is_empty())
        .or_else(|| crate::root_dir().ok().and_then(|root| fs::read(root.join("tp-api.json")).ok())
            .filter(|bytes| bytes.len() <= 4096).and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .and_then(|value| value["baseUrl"].as_str().map(str::to_string)))
        .or_else(|| serde_json::from_str::<Value>(include_str!("../../static/tp-api.json")).ok()
            .and_then(|value| value["baseUrl"].as_str().map(str::to_string)))?;
    let url = url::Url::parse(configured.trim()).ok()?;
    let local = cfg!(debug_assertions) && url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]" | "::1"));
    if (url.scheme() != "https" && !local) || !url.username().is_empty() || url.password().is_some()
        || url.query().is_some() || url.fragment().is_some() || url.host_str().is_none() { return None; }
    Some(url.as_str().trim_end_matches('/').to_string())
}
#[tauri::command]
pub fn server_directory_config() -> DirectoryConfig {
    let base = configured_base();
    DirectoryConfig { configured: base.is_some(), publishing_available: base.is_some(), base_url: base.unwrap_or_default() }
}
pub(crate) async fn while_idle<T, F: Future<Output = Result<T, String>>>(future: F) -> Result<T, String> {
    if crate::game_session::is_running() { return Err("Las consultas del directorio se pausan mientras juegas.".into()); }
    let mut state = crate::game_session::subscribe_running();
    tokio::select! {
        result = future => result,
        _ = async {
            loop {
                if *state.borrow() { break; }
                if state.changed().await.is_err() { std::future::pending::<()>().await; }
            }
        } => Err("Consulta detenida al iniciar Minecraft.".into()),
    }
}
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder().timeout(Duration::from_secs(18)).connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("TP LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)")
        .build().map_err(|_| "No se pudo preparar la conexión al directorio.".into())
}
async fn response_json(response: reqwest::Response) -> Result<Value, String> {
    if !response.status().is_success() {
        return Err(if response.status().as_u16() == 429 { "El directorio solicita esperar antes de consultar de nuevo.".into() }
            else { format!("El servicio no está disponible (HTTP {}).", response.status().as_u16()) });
    }
    let mut stream = response.bytes_stream(); let mut bytes = Vec::new();
    while let Some(part) = stream.next().await {
        let part = part.map_err(|_| "La descarga del directorio se interrumpió.")?;
        if bytes.len() + part.len() > MAX_RESPONSE { return Err("La respuesta del directorio es demasiado grande.".into()); }
        bytes.extend_from_slice(&part);
    }
    serde_json::from_slice(&bytes).map_err(|_| "El servicio no devolvió JSON válido.".into())
}
fn icon(value: &Value) -> String {
    let Some(value) = value.as_str().filter(|v| v.len() <= 100_000) else { return String::new(); };
    let Some(data) = value.strip_prefix("data:image/png;base64,") else { return String::new(); };
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data) else { return String::new(); };
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default(); limits.max_image_width = Some(128); limits.max_image_height = Some(128);
    limits.max_alloc = Some(128 * 128 * 4); reader.limits(limits);
    match reader.decode() { Ok(image) if image.width() == 64 && image.height() == 64 => value.into(), _ => String::new() }
}
fn motd(value: &Value, fallback: &str) -> Vec<DirectoryMotd> {
    let spans: Vec<_> = value.as_array().map(|spans| spans.iter().take(64).map(|span| {
        let color = clean(&span["color"], 7);
        let valid_color = color.len() == 7 && color.starts_with('#') && color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit);
        DirectoryMotd { text: clean(&span["text"], 1024), color: if valid_color { color } else { "#D7DEEE".into() },
            bold: span["bold"].as_bool().unwrap_or(false), italic: span["italic"].as_bool().unwrap_or(false),
            underlined: span["underlined"].as_bool().unwrap_or(false), strikethrough: span["strikethrough"].as_bool().unwrap_or(false) }
    }).filter(|span| !span.text.is_empty()).collect()).unwrap_or_default();
    if spans.is_empty() && !fallback.is_empty() { vec![DirectoryMotd {text:fallback.chars().take(4096).collect(), color:"#D7DEEE".into(), ..Default::default()}] } else { spans }
}
fn normalized_address(ip: &str, port: u64) -> Option<String> {
    let parsed = crate::servers::parse_address(ip).ok()?;
    let port = if port == 0 {parsed.port} else {u16::try_from(port).ok().filter(|p| *p > 0)?};
    Some(if port == 25565 { parsed.host } else if parsed.host.contains(':') {format!("[{}]:{port}", parsed.host)} else {format!("{}:{port}", parsed.host)})
}
fn normalize_cloud(value: Value, requested_page: u64) -> Result<DirectoryPage, String> {
    let rows = value["servers"].as_array().ok_or("TP API devolvió un formato de catálogo no reconocido.")?;
    let mut entries = Vec::new(); let mut addresses = HashSet::new();
    for row in rows.iter().take(PAGE_SIZE as usize) {
        let Some(address) = normalized_address(&clean(&row["ip"], 320), row["port"].as_u64().unwrap_or(25565)) else {continue};
        if !addresses.insert(address.to_lowercase()) {continue;}
        let category = clean(&row["category"], 64);
        let name = clean(&row["name"], 128);
        let description = clean(&row["motd_clean"], 4096);
        let checked_at = clean(&row["last_pinged_at"], 64);
        let featured_until = row["featured_until"].as_str().and_then(|date| parse_timestamp(date));
        let featured = row["is_featured"].as_bool() == Some(true)
            && featured_until.is_some_and(|timestamp| timestamp > now_feature_key());
        entries.push(DirectoryEntry {
            id:format!("tp:{}", clean(&row["id"], 100)), name:if name.is_empty() {address.clone()} else {name},
            address, description:description.clone(), version:clean(&row["version_text"], 120), tags:vec![category.to_lowercase()],
            online:row["is_online"].as_bool(), players_online:row["players_online"].as_u64(), players_max:row["players_max"].as_u64(),
            checked_at, java_compatible:true, icon:icon(&row["favicon_url"]), motd:motd(&row["motd"], &description),
            category, region:clean(&row["region"], 40), language:clean(&row["language"], 10), featured,featured_until:clean(&row["featured_until"],64),
            sponsor_url:https(row["sponsor_url"].as_str().unwrap_or_default()), from_cloud:true,
        });
    }
    let total = value["total"].as_u64().ok_or("TP API no indicó el tamaño del catálogo.")?;
    Ok(DirectoryPage {entries,total,page:requested_page,pages:total.div_ceil(PAGE_SIZE).max(1),
        source:"TP API".into(),stale:false,message:String::new()})
}
fn now_seconds() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }
fn parse_timestamp(value: &str) -> Option<u64> {


    let digits: String = value.chars().take(19).filter(char::is_ascii_digit).collect();
    if digits.len() != 14 {return None;}


    if !value.ends_with('Z') && !value.ends_with("+00:00") {return None;}
    Some(digits.parse::<u64>().ok()?)
}
fn now_feature_key() -> u64 {

    let seconds = now_seconds(); let days = (seconds / 86400) as i64;
    let z = days + 719468; let era = if z >= 0 {z} else {z - 146096} / 146097;
    let doe = z - era * 146097; let yoe = (doe - doe/1460 + doe/36524 - doe/146096)/365;
    let mut year = yoe + era*400; let doy = doe - (365*yoe + yoe/4 - yoe/100); let mp = (5*doy+2)/153;
    let day = doy-(153*mp+2)/5+1; let month = mp+if mp<10 {3} else {-9}; year += if month<=2 {1} else {0};
    format!("{year:04}{month:02}{day:02}{:02}{:02}{:02}",seconds%86400/3600,seconds%3600/60,seconds%60).parse().unwrap_or(0)
}
fn category(mode: &str) -> &str { match mode {
    "minigames"=>"Minijuegos","pvp"=>"PvP","mmorpg"=>"RPG","anarchy"=>"Anarquía","skyblock"=>"SkyBlock",
    "survival"=>"Survival","smp"=>"SMP","boxpvp"=>"BoxPvP","economy"=>"Economía",""=> "all", other=>other,
} }
async fn search_impl(page:u64,search:String,mode:String,version:String,sort:String,lang:Option<String>,region:Option<String>,_local:Option<bool>) -> Result<DirectoryPage,String> {
    if crate::game_session::is_running(){return Err("El directorio está pausado mientras juegas.".into());}
    let page=page.clamp(1,100_000);let search:String=search.trim().chars().filter(|c|!c.is_control()).take(120).collect();
    let mode:String=mode.chars().filter(|c|c.is_ascii_alphanumeric()||*c=='-').take(40).collect();
    let version:String=version.trim().chars().filter(|c|c.is_ascii_alphanumeric()||matches!(c,'.'|'-'|' ')).take(32).collect();
    let lang=lang.unwrap_or_default().chars().take(10).collect::<String>();let region=region.unwrap_or_default().chars().take(40).collect::<String>();

    let base=configured_base();
    let query=[("page",page.to_string()),("limit",PAGE_SIZE.to_string()),("search",search.clone()),("category",category(&mode).to_string()),
        ("version",version),("sort",sort),("lang",if lang.is_empty(){"all".into()}else{lang.clone()}),("region",if region.is_empty(){"all".into()}else{region.clone()})];
    let key=format!("{:x}",Sha1::digest(serde_json::to_vec(&(&base,&query)).map_err(|e|e.to_string())?));
    if let Some((at,value))=pages_cache().lock().map_err(|_|"Caché del directorio ocupada.")?.get(&key){if at.elapsed()<Duration::from_secs(60)&&value.source=="TP API"{return Ok(value.clone());}}
    let disk=crate::root_dir()?.join("cache").join("server-directory").join(format!("tp-{key}.json"));
    let saved=fs::read(&disk).ok().filter(|bytes|bytes.len()<=MAX_RESPONSE).and_then(|bytes|serde_json::from_slice::<DirectoryPage>(&bytes).ok())
        .filter(|saved|saved.source=="TP API"&&saved.entries.iter().all(|entry|entry.from_cloud&&entry.id.starts_with("tp:")));
    let result=if let Some(base)=base {while_idle(async{
        let response=client()?.get(format!("{base}/api/v1/servers")).query(&query).header("Accept","application/json")
            .header("X-Client-ID","tp-launcher-official").send().await.map_err(|_|"No se pudo conectar a TP API.".to_string())?;
        normalize_cloud(response_json(response).await?,page)
    }).await}else{Err("TP API no está configurada. No se ha sustituido el catálogo por otra fuente.".into())};
    let value=match result {Ok(value)=>value,Err(error)=>{
        if crate::game_session::is_running(){return Err("Consulta detenida al iniciar Minecraft.".into());}
        if let Some(mut saved)=saved {saved.stale=true;for entry in &mut saved.entries{entry.featured=false;}saved.message=format!("{error} Se muestra la última página guardada de TP API; sus datos pueden haber cambiado.");saved}
        else {return Err(error);}
    }};
    if crate::game_session::is_running(){return Err("Consulta detenida al iniciar Minecraft.".into());}
    if let Ok(mut cache)=pages_cache().lock(){if cache.len()>=96{cache.clear();}cache.insert(key,(Instant::now(),value.clone()));}
    if !value.stale {if let Some(parent)=disk.parent(){let _=fs::create_dir_all(parent);if let Ok(bytes)=serde_json::to_vec(&value){let _=fs::write(disk,bytes);}}}
    Ok(value)
}
#[tauri::command]
pub async fn server_directory_search(page:u64,search:String,mode:String,version:String,sort:String,lang:Option<String>,region:Option<String>,local:Option<bool>) -> Result<DirectoryPage,String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static GENERATION:AtomicU64=AtomicU64::new(0);
    static CHANGES:OnceLock<tokio::sync::watch::Sender<u64>>=OnceLock::new();
    let sender=CHANGES.get_or_init(||tokio::sync::watch::channel(0).0);
    let id=GENERATION.fetch_add(1,Ordering::SeqCst)+1;
    let mut changes=sender.subscribe();sender.send_replace(id);
    tokio::select! {
        value=search_impl(page,search,mode,version,sort,lang,region,local)=>value,
        _=async {loop{if *changes.borrow()!=id{break;}if changes.changed().await.is_err(){std::future::pending::<()>().await;}}}
            =>Err("Consulta reemplazada por los nuevos filtros.".into()),
    }
}
#[tauri::command]
pub async fn server_directory_submit(name:String,address:String,category:String,region:String,language:String) -> Result<String,String>{
    let base=configured_base().ok_or("La publicación estará disponible cuando TP API esté conectada.")?;
    let parsed=crate::servers::parse_address(&address)?;
    if name.trim().is_empty()||name.chars().count()>120{return Err("Escribe un nombre de hasta 120 caracteres.".into());}
    let body=json!({"name":name.trim(),"ip":parsed.host,"port":parsed.port,"category":category,"region":region,"language":language});
    while_idle(async{
        let response=client()?.post(format!("{base}/api/v1/servers/submit")).header("X-Client-ID","tp-launcher-official").json(&body)
            .send().await.map_err(|_|"No se pudo publicar el servidor en TP API.".to_string())?;
        let status=response.status();
        if !status.is_success(){return Err(format!("TP API no aceptó el servidor (HTTP {}). Comprueba que sea un servidor Java público y esté encendido.",status.as_u16()));}
        response_json(response).await?;
        if let Ok(mut cache)=pages_cache().lock(){cache.clear();}
        Ok("Servidor publicado en TP API. Ya puedes encontrarlo en el catálogo.".into())
    }).await
}
