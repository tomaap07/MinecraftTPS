

use base64::Engine;
use hickory_resolver::TokioAsyncResolver;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Seek, SeekFrom, Write},
    net::{IpAddr, Ipv6Addr, SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{Mutex as AsyncMutex, Semaphore},
    time::timeout,
};

const MAX_NBT: usize = 8 * 1024 * 1024;
const MAX_SERVERS: usize = 4096;
const PING_TIMEOUT: Duration = Duration::from_secs(7);
const PING_CACHE_TTL: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub name: String,
    pub address: String,
    #[serde(default)]
    pub icon: String,
}
#[derive(Serialize)]
pub struct ServerList {
    pub entries: Vec<ServerEntry>,
    pub revision: String,
    pub path: String,
}
#[derive(Clone, Debug)]
pub struct ServerAddress {
    pub host: String,
    pub port: u16,
    explicit_port: bool,
}
impl ServerAddress {
    pub fn display(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]:{}", self.host, self.port)
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
}

pub fn parse_address(input: &str) -> Result<ServerAddress, String> {
    let value = input.trim();
    if value.is_empty()
        || value.len() > 320
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
        || value.contains(['/', '\\', '@', '?', '#'])
    {
        return Err(
            "Introduce un dominio, IPv4 o IPv6 y un puerto opcional; sin http:// ni espacios."
                .into(),
        );
    }
    let (host, port, explicit_port) = if value.starts_with('[') {
        let end = value
            .find(']')
            .ok_or("La dirección IPv6 necesita un corchete de cierre.")?;
        let host = &value[1..end];
        host.parse::<Ipv6Addr>()
            .map_err(|_| "La dirección IPv6 no es válida.")?;
        let rest = &value[end + 1..];
        let (port, explicit) = if rest.is_empty() {
            (25565, false)
        } else {
            (
                parse_port(rest.strip_prefix(':').ok_or("Puerto IPv6 no válido.")?)?,
                true,
            )
        };
        (host.to_string(), port, explicit)
    } else if value.parse::<Ipv6Addr>().is_ok() {
        (value.to_string(), 25565, false)
    } else {
        let (host, port, explicit) = match value.split_once(':') {
            Some((host, port)) => (host, parse_port(port)?, true),
            None => (value, 25565, false),
        };
        let host = host.trim_end_matches('.');
        if host.is_empty()
            || host.len() > 253
            || !host.split('.').all(|part| {
                !part.is_empty()
                    && part.len() <= 63
                    && !part.starts_with('-')
                    && !part.ends_with('-')
                    && part
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
        {
            return Err("El dominio o la dirección IP no es válido.".into());
        }
        (host.to_ascii_lowercase(), port, explicit)
    };
    Ok(ServerAddress {
        host,
        port,
        explicit_port,
    })
}
fn parse_port(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .ok()
        .filter(|p| *p != 0)
        .ok_or_else(|| "El puerto debe estar entre 1 y 65535.".into())
}
fn game_dir(username: &str, version: &str, instance: Option<&str>) -> Result<PathBuf, String> {
    if let Some(id) = instance.filter(|id| !id.is_empty()) {
        crate::mod_studio::pack_game_dir(id)
    } else {
        crate::version_game_dir(&crate::root_dir()?, username, version)
    }
}


#[derive(Clone, Debug, PartialEq)]
enum Tag {
    Raw(u8, Vec<u8>),
    List(u8, Vec<Tag>),
    Compound(Vec<(String, Tag)>),
}
impl Tag {
    fn kind(&self) -> u8 {
        match self {
            Self::Raw(ty, _) => *ty,
            Self::List(..) => 9,
            Self::Compound(_) => 10,
        }
    }
    fn string(value: &str) -> Result<Self, String> {
        let mut bytes = Vec::new();
        put_string(&mut bytes, value)?;
        Ok(Self::Raw(8, bytes))
    }
    fn as_string(&self) -> Option<String> {
        if let Self::Raw(8, bytes) = self {
            read_string(&mut Reader::new(bytes)).ok()
        } else {
            None
        }
    }
    fn field(&self, name: &str) -> Option<&Tag> {
        if let Self::Compound(fields) = self {
            fields
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, tag)| tag)
        } else {
            None
        }
    }
    fn set(&mut self, name: &str, value: Tag) -> Result<(), String> {
        let Self::Compound(fields) = self else {
            return Err("Registro NBT de servidor inválido.".into());
        };
        if let Some((_, tag)) = fields.iter_mut().find(|(key, _)| key == name) {
            *tag = value;
        } else {
            fields.push((name.into(), value));
        }
        Ok(())
    }
}
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    nodes: usize,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            nodes: 0,
        }
    }
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(count)
            .filter(|end| *end <= self.data.len())
            .ok_or("servers.dat está incompleto o tiene longitudes inválidas.")?;
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn length(&mut self) -> Result<usize, String> {
        let count = i32::from_be_bytes(self.take(4)?.try_into().unwrap());
        usize::try_from(count).map_err(|_| "servers.dat contiene una longitud negativa.".into())
    }
}
fn read_string(reader: &mut Reader<'_>) -> Result<String, String> {
    let length = u16::from_be_bytes(reader.take(2)?.try_into().unwrap()) as usize;
    let bytes = reader.take(length)?;
    let mut units = Vec::with_capacity(length);
    let mut i = 0;
    while i < bytes.len() {
        let a = bytes[i];
        i += 1;
        let unit = if a < 0x80 {
            a as u16
        } else if a & 0xe0 == 0xc0 {
            let b = *bytes.get(i).ok_or("Cadena NBT truncada.")?;
            i += 1;
            if b & 0xc0 != 0x80 {
                return Err("Codificación NBT inválida.".into());
            }
            (((a & 0x1f) as u16) << 6) | ((b & 0x3f) as u16)
        } else if a & 0xf0 == 0xe0 {
            let b = *bytes.get(i).ok_or("Cadena NBT truncada.")?;
            let c = *bytes.get(i + 1).ok_or("Cadena NBT truncada.")?;
            i += 2;
            if b & 0xc0 != 0x80 || c & 0xc0 != 0x80 {
                return Err("Codificación NBT inválida.".into());
            }
            (((a & 15) as u16) << 12) | (((b & 63) as u16) << 6) | ((c & 63) as u16)
        } else {
            return Err("Codificación Java NBT no válida.".into());
        };
        units.push(unit);
    }
    String::from_utf16(&units).map_err(|_| "Cadena UTF-16 NBT inválida.".into())
}
fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let mut bytes = Vec::new();
    for unit in value.encode_utf16() {
        if (1..=0x7f).contains(&unit) {
            bytes.push(unit as u8);
        } else if unit <= 0x7ff {
            bytes.extend([0xc0 | (unit >> 6) as u8, 0x80 | (unit & 63) as u8]);
        } else {
            bytes.extend([
                0xe0 | (unit >> 12) as u8,
                0x80 | ((unit >> 6) & 63) as u8,
                0x80 | (unit & 63) as u8,
            ]);
        }
    }
    let length =
        u16::try_from(bytes.len()).map_err(|_| "Una cadena de servidores es demasiado larga.")?;
    out.extend(length.to_be_bytes());
    out.extend(bytes);
    Ok(())
}
fn read_tag(reader: &mut Reader<'_>, ty: u8, depth: usize) -> Result<Tag, String> {
    reader.nodes += 1;
    if depth > 64 || reader.nodes > 150_000 {
        return Err("servers.dat supera los límites de estructura NBT.".into());
    }
    let start = reader.pos;
    match ty {
        1 => {
            reader.take(1)?;
        }
        2 => {
            reader.take(2)?;
        }
        3 | 5 => {
            reader.take(4)?;
        }
        4 | 6 => {
            reader.take(8)?;
        }
        7 | 11 | 12 => {
            let count = reader.length()?;
            let size = match ty {
                11 => 4,
                12 => 8,
                _ => 1,
            };
            reader.take(
                count
                    .checked_mul(size)
                    .ok_or("Array NBT demasiado largo.")?,
            )?;
        }
        8 => {
            read_string(reader)?;
        }
        9 => {
            let kind = reader.byte()?;
            let count = reader.length()?;
            if count > 150_000 || (kind == 0 && count != 0) {
                return Err("Lista NBT inválida.".into());
            }
            let mut list = Vec::with_capacity(count.min(4096));
            for _ in 0..count {
                list.push(read_tag(reader, kind, depth + 1)?);
            }
            return Ok(Tag::List(kind, list));
        }
        10 => {
            let mut fields = Vec::new();
            loop {
                let kind = reader.byte()?;
                if kind == 0 {
                    break;
                }
                let name = read_string(reader)?;
                fields.push((name, read_tag(reader, kind, depth + 1)?));
            }
            return Ok(Tag::Compound(fields));
        }
        _ => {
            return Err(format!(
                "Tipo NBT desconocido: {ty}. El archivo se conserva sin modificar."
            ));
        }
    }
    Ok(Tag::Raw(ty, reader.data[start..reader.pos].to_vec()))
}
fn write_tag(out: &mut Vec<u8>, tag: &Tag) -> Result<(), String> {
    match tag {
        Tag::Raw(_, payload) => out.extend(payload),
        Tag::List(kind, list) => {
            out.push(*kind);
            out.extend((list.len() as i32).to_be_bytes());
            for item in list {
                write_tag(out, item)?;
            }
        }
        Tag::Compound(fields) => {
            for (name, tag) in fields {
                out.push(tag.kind());
                put_string(out, name)?;
                write_tag(out, tag)?;
            }
            out.push(0);
        }
    };
    Ok(())
}
fn parse_nbt(bytes: &[u8]) -> Result<(String, Tag), String> {
    if bytes.is_empty() {
        return Ok((
            String::new(),
            Tag::Compound(vec![("servers".into(), Tag::List(10, vec![]))]),
        ));
    }
    if bytes.len() > MAX_NBT {
        return Err("servers.dat es demasiado grande: máximo 8 MB.".into());
    }
    let mut reader = Reader::new(bytes);
    if reader.byte()? != 10 {
        return Err(
            "servers.dat debe ser un compuesto NBT sin comprimir. No se modificó el archivo."
                .into(),
        );
    }
    let name = read_string(&mut reader)?;
    let root = read_tag(&mut reader, 10, 0)?;
    if reader.pos != bytes.len() {
        return Err("servers.dat contiene datos sobrantes. No se modificó el archivo.".into());
    }
    Ok((name, root))
}
fn nbt_bytes(name: &str, tag: &Tag) -> Result<Vec<u8>, String> {
    let mut bytes = vec![10];
    put_string(&mut bytes, name)?;
    write_tag(&mut bytes, tag)?;
    Ok(bytes)
}
fn tag_id(index: usize, tag: &Tag) -> String {
    let mut bytes = Vec::new();
    let _ = write_tag(&mut bytes, tag);
    format!("{index}-{:x}", Sha1::digest(bytes))
}
fn server_tags(root: &Tag) -> Result<&[Tag], String> {
    match root.field("servers") {
        Some(Tag::List(10, list)) => Ok(list),
        Some(Tag::List(0, list)) if list.is_empty() => Ok(list),
        None => Ok(&[]),
        _ => Err("La lista servers no contiene compuestos NBT válidos.".into()),
    }
}
fn entries_of(root: &Tag) -> Result<Vec<ServerEntry>, String> {
    let list = server_tags(root)?;
    if list.len() > MAX_SERVERS {
        return Err("La lista supera los 4096 servidores permitidos.".into());
    }
    Ok(list
        .iter()
        .enumerate()
        .map(|(index, tag)| ServerEntry {
            id: tag_id(index, tag),
            name: tag
                .field("name")
                .and_then(Tag::as_string)
                .unwrap_or_else(|| "Servidor Minecraft".into()),
            address: tag.field("ip").and_then(Tag::as_string).unwrap_or_default(),
            icon: tag
                .field("icon")
                .and_then(Tag::as_string)
                .filter(|icon| icon.len() <= 128 * 1024)
                .map(|icon| {
                    let image = format!(
                        "data:image/png;base64,{}",
                        icon.trim_start_matches("data:image/png;base64,")
                    );
                    valid_favicon(Some(&image))
                })
                .unwrap_or_default(),
        })
        .collect())
}
fn revision(bytes: &[u8]) -> String {
    format!("{:x}", Sha1::digest(bytes))
}
fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    match fs::File::open(path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take((MAX_NBT + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > MAX_NBT {
                Err("servers.dat supera 8 MB.".into())
            } else {
                Ok(bytes)
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        Err(error) => Err(format!("No se pudo leer servers.dat: {error}")),
    }
}
pub(crate) fn list_at(dir: &Path) -> Result<ServerList, String> {
    let path = dir.join("servers.dat");
    let bytes = read_file(&path)?;
    let (_, root) = parse_nbt(&bytes)?;
    Ok(ServerList {
        entries: entries_of(&root)?,
        revision: revision(&bytes),
        path: path.to_string_lossy().into_owned(),
    })
}
#[tauri::command]
pub async fn servers_list(
    username: String,
    version_id: String,
    instance_id: Option<String>,
) -> Result<ServerList, String> {
    tokio::task::spawn_blocking(move || {
        list_at(&game_dir(&username, &version_id, instance_id.as_deref())?)
    })
    .await
    .map_err(|e| e.to_string())?
}
fn save_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
fn save_at(
    dir: &Path,
    entries: Vec<ServerEntry>,
    expected_revision: &str,
) -> Result<ServerList, String> {
    let _guard = save_lock()
        .lock()
        .map_err(|_| "No se pudo bloquear la lista de servidores.")?;
    if entries.len() > MAX_SERVERS {
        return Err("No se pueden guardar más de 4096 servidores.".into());
    }
    let path = dir.join("servers.dat");
    let original = read_file(&path)?;
    if revision(&original) != expected_revision {
        return Err("Minecraft u otro programa cambió la lista. Actualiza antes de guardar para conservar esos cambios.".into());
    }
    let (name, mut root) = parse_nbt(&original)?;
    let originals = server_tags(&root)?
        .iter()
        .enumerate()
        .map(|(i, tag)| (tag_id(i, tag), tag.clone()))
        .collect::<HashMap<_, _>>();
    let mut used = HashSet::new();
    let mut tags = Vec::with_capacity(entries.len());
    for entry in entries {
        if entry.name.trim().is_empty()
            || entry.name.chars().count() > 128
            || entry.name.chars().any(|c| c.is_control())
        {
            return Err("El nombre del servidor debe tener entre 1 y 128 caracteres.".into());
        }
        let address = parse_address(&entry.address)?;
        let mut tag = if entry.id.is_empty() {
            Tag::Compound(vec![])
        } else {
            if !used.insert(entry.id.clone()) {
                return Err("La lista contiene identificadores duplicados.".into());
            }
            originals
                .get(&entry.id)
                .cloned()
                .ok_or("Un servidor ya no coincide con el archivo original. Actualiza la lista.")?
        };
        tag.set("name", Tag::string(entry.name.trim())?)?;

        let stored = if address.explicit_port {
            address.display()
        } else {
            address.host.clone()
        };
        tag.set("ip", Tag::string(&stored)?)?;
        tags.push(tag);
    }
    root.set("servers", Tag::List(10, tags))?;
    let bytes = nbt_bytes(&name, &root)?;
    if bytes.len() > MAX_NBT {
        return Err("La lista generada supera 8 MB.".into());
    }
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let temporary = dir.join(format!(".servers-{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        if read_file(&path)? != original {
            return Err(
                "La lista cambió mientras se guardaba; actualiza e inténtalo de nuevo.".into(),
            );
        }
        if path.exists() {
            fs::copy(&path, dir.join("servers.dat_old"))
                .map_err(|e| format!("No se pudo guardar la copia de seguridad: {e}"))?;
        }

        fs::rename(&temporary, &path).map_err(|e| format!("No se pudo reemplazar servers.dat. Cierra la lista multijugador de Minecraft e inténtalo de nuevo: {e}"))?;
        list_at(dir)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
#[tauri::command]
pub async fn servers_save(
    username: String,
    version_id: String,
    instance_id: Option<String>,
    entries: Vec<ServerEntry>,
    revision: String,
) -> Result<ServerList, String> {
    tokio::task::spawn_blocking(move || {
        save_at(
            &game_dir(&username, &version_id, instance_id.as_deref())?,
            entries,
            &revision,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Clone, Serialize, Debug)]
pub struct MotdSpan {
    text: String,
    color: String,
    bold: bool,
    italic: bool,
    underlined: bool,
    strikethrough: bool,
}
fn default_span() -> MotdSpan {
    MotdSpan {
        text: String::new(),
        color: "#d7deee".into(),
        bold: false,
        italic: false,
        underlined: false,
        strikethrough: false,
    }
}
fn color(value: &str) -> Option<String> {
    const COLORS: [&str; 16] = [
        "#000000", "#0000aa", "#00aa00", "#00aaaa", "#aa0000", "#aa00aa", "#ffaa00", "#aaaaaa",
        "#555555", "#5555ff", "#55ff55", "#55ffff", "#ff5555", "#ff55ff", "#ffff55", "#ffffff",
    ];
    const NAMES: [&str; 16] = [
        "black",
        "dark_blue",
        "dark_green",
        "dark_aqua",
        "dark_red",
        "dark_purple",
        "gold",
        "gray",
        "dark_gray",
        "blue",
        "green",
        "aqua",
        "red",
        "light_purple",
        "yellow",
        "white",
    ];
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        Some(value.to_string())
    } else {
        NAMES
            .iter()
            .position(|name| *name == value)
            .map(|i| COLORS[i].to_string())
    }
}
fn legacy_text(text: &str, inherited: &MotdSpan, out: &mut Vec<MotdSpan>) {
    let mut style = inherited.clone();
    style.text.clear();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '§' {
            if style.text.chars().count() < 4096 {
                style.text.push(character);
            }
            continue;
        }
        let Some(code) = chars.next() else { break };
        let code = code.to_ascii_lowercase();
        if !style.text.is_empty() {
            out.push(style.clone());
            style.text.clear();
        }
        if let Some(index) = code.to_digit(16) {
            style = default_span();
            style.color = [
                "#000000", "#0000aa", "#00aa00", "#00aaaa", "#aa0000", "#aa00aa", "#ffaa00",
                "#aaaaaa", "#555555", "#5555ff", "#55ff55", "#55ffff", "#ff5555", "#ff55ff",
                "#ffff55", "#ffffff",
            ][index as usize]
                .into();
        } else {
            match code {
                'l' => style.bold = true,
                'o' => style.italic = true,
                'n' => style.underlined = true,
                'm' => style.strikethrough = true,
                'r' => style = default_span(),
                _ => {}
            }
        }
    }
    if !style.text.is_empty() {
        out.push(style);
    }
}
fn component_text(value: &Value, inherited: &MotdSpan, out: &mut Vec<MotdSpan>, depth: usize) {
    if depth > 24 || out.len() > 128 {
        return;
    }
    if let Some(text) = value.as_str() {
        legacy_text(text, inherited, out);
    } else if let Some(parts) = value.as_array() {
        for part in parts.iter().take(128) {
            component_text(part, inherited, out, depth + 1);
        }
    } else if value.is_object() {
        let mut style = inherited.clone();
        if let Some(new_color) = value["color"].as_str().and_then(color) {
            style.color = new_color;
        }
        style.bold = value["bold"].as_bool().unwrap_or(style.bold);
        style.italic = value["italic"].as_bool().unwrap_or(style.italic);
        style.underlined = value["underlined"].as_bool().unwrap_or(style.underlined);
        style.strikethrough = value["strikethrough"]
            .as_bool()
            .unwrap_or(style.strikethrough);
        if let Some(text) = value["text"]
            .as_str()
            .or_else(|| value["translate"].as_str())
        {
            legacy_text(text, &style, out);
        }
        if let Some(extra) = value["extra"].as_array() {
            for part in extra.iter().take(128) {
                component_text(part, &style, out, depth + 1);
            }
        }
    }
}
#[derive(Clone, Serialize)]
pub struct ServerStatus {
    pub address: String,
    pub online: bool,
    pub latency_ms: u64,
    pub players_online: u64,
    pub players_max: u64,
    pub version: String,
    pub protocol: i64,
    pub motd: Vec<MotdSpan>,
    pub icon: String,
    pub error: String,

    pub checked_at: u64,
}

type CachedPing = Arc<AsyncMutex<Option<(Instant, ServerStatus)>>>;
type PingCache = HashMap<String, (Instant, CachedPing)>;

fn ping_cache_entry(key: &str) -> Result<CachedPing, String> {
    static CACHE: OnceLock<Mutex<PingCache>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()))
        .lock().map_err(|_| "No se pudo consultar la caché de servidores.")?;
    if let Some((accessed, entry)) = cache.get_mut(key) {
        *accessed = Instant::now();
        return Ok(Arc::clone(entry));
    }


    cache.retain(|_, (accessed, entry)| accessed.elapsed() < Duration::from_secs(60)
        || Arc::strong_count(entry) > 1);
    if cache.len() >= 256 {
        let oldest = cache.iter().filter(|(_, (_, entry))| Arc::strong_count(entry) == 1)
            .min_by_key(|(_, (accessed, _))| *accessed).map(|(key, _)| key.clone());
        if let Some(oldest) = oldest { cache.remove(&oldest); }
        else { return Err("Hay muchas consultas de servidores pendientes. Inténtalo de nuevo en un momento.".into()); }
    }
    let entry = Arc::new(AsyncMutex::new(None));
    cache.insert(key.to_owned(), (Instant::now(), Arc::clone(&entry)));
    Ok(entry)
}
fn varint(out: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let mut byte = (value & 127) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 128;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}
fn slice_varint(bytes: &[u8], offset: &mut usize) -> Result<i32, String> {
    let mut value = 0u32;
    for shift in (0..35).step_by(7) {
        let byte = *bytes
            .get(*offset)
            .ok_or("Respuesta del servidor incompleta.")?;
        *offset += 1;
        value |= ((byte & 127) as u32) << shift;
        if byte & 128 == 0 {
            return Ok(value as i32);
        }
    }
    Err("VarInt inválido en la respuesta del servidor.".into())
}
async fn stream_varint(stream: &mut TcpStream) -> Result<i32, String> {
    let mut bytes = Vec::new();
    for _ in 0..5 {
        let byte = stream.read_u8().await.map_err(|e| e.to_string())?;
        bytes.push(byte);
        if byte & 128 == 0 {
            return slice_varint(&bytes, &mut 0);
        }
    }
    Err("El servidor respondió con un tamaño inválido.".into())
}
async fn packet(stream: &mut TcpStream, payload: &[u8]) -> Result<(), String> {
    let mut framed = Vec::new();
    varint(&mut framed, payload.len() as i32);
    framed.extend(payload);
    stream.write_all(&framed).await.map_err(|e| e.to_string())
}
async fn read_packet(stream: &mut TcpStream, max: usize) -> Result<Vec<u8>, String> {
    let length = stream_varint(stream).await?;
    if length <= 0 || length as usize > max {
        return Err("La respuesta del servidor supera el límite permitido.".into());
    }
    let mut data = vec![0; length as usize];
    stream
        .read_exact(&mut data)
        .await
        .map_err(|e| e.to_string())?;
    Ok(data)
}
pub(crate) async fn endpoint(address: &ServerAddress) -> (String, u16) {
    if !address.explicit_port && address.host.parse::<IpAddr>().is_err() {
        if let Ok(resolver) = TokioAsyncResolver::tokio_from_system_conf() {
            if let Ok(Ok(records)) = timeout(
                Duration::from_secs(2),
                resolver.srv_lookup(format!("_minecraft._tcp.{}", address.host)),
            )
            .await
            {
                if let Some(record) = records
                    .iter()
                    .filter(|r| r.target().to_utf8() != ".")
                    .min_by_key(|r| r.priority())
                {
                    return (
                        record.target().to_utf8().trim_end_matches('.').into(),
                        record.port(),
                    );
                }
            }
        }
    }
    (address.host.clone(), address.port)
}
async fn connect(host: &str, port: u16) -> Result<TcpStream, String> {
    TcpStream::connect((host, port))
        .await
        .map_err(|e| format!("No se pudo conectar: {e}"))
}
fn valid_favicon(value: Option<&str>) -> String {
    value
        .filter(|icon| icon.len() <= 128 * 1024 && icon.starts_with("data:image/png;base64,"))
        .filter(|icon| {
            base64::engine::general_purpose::STANDARD
                .decode(icon.trim_start_matches("data:image/png;base64,"))
                .ok()
                .is_some_and(|bytes| {
                    bytes.starts_with(b"\x89PNG\r\n\x1a\n")
                        && bytes.len() >= 24
                        && u32::from_be_bytes(bytes[16..20].try_into().unwrap()) <= 256
                        && u32::from_be_bytes(bytes[20..24].try_into().unwrap()) <= 256
                })
        })
        .unwrap_or_default()
        .to_string()
}
async fn modern_ping(
    address: &ServerAddress,
    endpoint: &(String, u16),
) -> Result<ServerStatus, String> {
    let start = Instant::now();
    let mut stream = connect(&endpoint.0, endpoint.1).await?;
    let mut handshake = vec![0];
    varint(&mut handshake, -1);
    varint(&mut handshake, address.host.len() as i32);
    handshake.extend(address.host.as_bytes());
    handshake.extend(address.port.to_be_bytes());
    varint(&mut handshake, 1);
    packet(&mut stream, &handshake).await?;
    packet(&mut stream, &[0]).await?;
    let data = read_packet(&mut stream, 1024 * 1024).await?;
    let mut offset = 0;
    if slice_varint(&data, &mut offset)? != 0 {
        return Err("Respuesta de estado Minecraft inválida.".into());
    }
    let length = slice_varint(&data, &mut offset)?;
    if length <= 0 || offset.checked_add(length as usize) != Some(data.len()) {
        return Err("Estado JSON de Minecraft incompleto.".into());
    }
    let value: Value = serde_json::from_slice(&data[offset..])
        .map_err(|e| format!("El servidor devolvió un estado JSON inválido: {e}"))?;
    let mut motd = vec![];
    component_text(&value["description"], &default_span(), &mut motd, 0);
    let mut ping = vec![1];
    ping.extend(0x4c545053_u64.to_be_bytes());
    let ping_start = Instant::now();
    packet(&mut stream, &ping).await?;

    let latency = match timeout(Duration::from_secs(1), read_packet(&mut stream, 64)).await {
        Ok(Ok(pong)) if pong == ping => ping_start.elapsed(),
        _ => start.elapsed(),
    };
    Ok(ServerStatus {
        address: address.display(),
        online: true,
        latency_ms: latency.as_millis() as u64,
        players_online: value
            .pointer("/players/online")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        players_max: value
            .pointer("/players/max")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        version: value
            .pointer("/version/name")
            .and_then(Value::as_str)
            .unwrap_or("Minecraft Java")
            .chars()
            .take(128)
            .collect(),
        protocol: value
            .pointer("/version/protocol")
            .and_then(Value::as_i64)
            .unwrap_or(-1),
        motd,
        icon: valid_favicon(value["favicon"].as_str()),
        error: String::new(),
        checked_at: 0,
    })
}
async fn legacy_ping(
    address: &ServerAddress,
    endpoint: &(String, u16),
) -> Result<ServerStatus, String> {
    let start = Instant::now();
    let mut stream = connect(&endpoint.0, endpoint.1).await?;
    stream
        .write_all(&[0xfe, 0x01])
        .await
        .map_err(|e| e.to_string())?;
    if stream.read_u8().await.map_err(|e| e.to_string())? != 0xff {
        return Err("No se recibió un estado Minecraft clásico.".into());
    }
    let length = stream.read_u16().await.map_err(|e| e.to_string())? as usize;
    if length > 32768 {
        return Err("Estado clásico demasiado largo.".into());
    }
    let mut bytes = vec![0; length * 2];
    stream
        .read_exact(&mut bytes)
        .await
        .map_err(|e| e.to_string())?;
    let text = String::from_utf16(
        &bytes
            .chunks_exact(2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .collect::<Vec<_>>(),
    )
    .map_err(|_| "Estado clásico inválido.")?;
    let (version, protocol, description, online, max) = if text.starts_with("§1\0") {
        let parts = text.split('\0').collect::<Vec<_>>();
        if parts.len() < 6 {
            return Err("Estado clásico incompleto.".into());
        }
        (
            parts[2].to_string(),
            parts[1].parse().unwrap_or(-1),
            parts[3].to_string(),
            parts[4].parse().unwrap_or(0),
            parts[5].parse().unwrap_or(0),
        )
    } else {
        let mut parts = text.rsplitn(3, '§');
        let max = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        let online = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        (
            "Minecraft clásico".into(),
            -1,
            parts.next().unwrap_or(&text).to_string(),
            online,
            max,
        )
    };
    let mut motd = vec![];
    legacy_text(&description, &default_span(), &mut motd);
    Ok(ServerStatus {
        address: address.display(),
        online: true,
        latency_ms: start.elapsed().as_millis() as u64,
        players_online: online,
        players_max: max,
        version,
        protocol,
        motd,
        icon: String::new(),
        error: String::new(),
        checked_at: 0,
    })
}
#[tauri::command]
pub async fn server_ping(address: String) -> Result<ServerStatus, String> {
    let parsed = parse_address(&address)?;
    let entry = ping_cache_entry(&parsed.display().to_ascii_lowercase())?;


    let mut cached = entry.lock().await;
    if let Some((measured, status)) = cached.as_ref() {
        if measured.elapsed() < PING_CACHE_TTL { return Ok(status.clone()); }
    }
    static PING_SLOTS: OnceLock<Semaphore> = OnceLock::new();
    let _slot = timeout(Duration::from_secs(10), PING_SLOTS.get_or_init(|| Semaphore::new(4)).acquire())
        .await.map_err(|_| "Las consultas de servidores están ocupadas. Se actualizará en el próximo intento.")?
        .map_err(|_| "No se pudo iniciar la consulta del servidor.")?;
    let result = timeout(PING_TIMEOUT, async {
        let endpoint = endpoint(&parsed).await;
        match timeout(Duration::from_secs(3), modern_ping(&parsed, &endpoint)).await {
            Ok(Ok(status)) => Ok(status),
            _ => legacy_ping(&parsed, &endpoint).await,
        }
    })
    .await;
    let mut status = match result {
        Ok(Ok(status)) => status,
        other => ServerStatus {address:parsed.display(),online:false,latency_ms:0,players_online:0,players_max:0,version:String::new(),protocol:-1,motd:vec![],icon:String::new(),error:match other { Ok(Err(e)) => e, _ => "El servidor no respondió en 7 segundos. Comprueba su dirección y que esté encendido.".into() },checked_at:0},
    };
    status.checked_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()
        .as_millis().min(u64::MAX as u128) as u64;
    *cached = Some((Instant::now(), status.clone()));
    Ok(status)
}

#[derive(Clone, Default)]
struct LanObservation {
    port: Option<u16>,
    public_address: Option<String>,
    updated: u64,
}
fn observations() -> &'static Mutex<HashMap<PathBuf, LanObservation>> {
    static OBSERVED: OnceLock<Mutex<HashMap<PathBuf, LanObservation>>> = OnceLock::new();
    OBSERVED.get_or_init(|| Mutex::new(HashMap::new()))
}
fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn parse_lan_line(line: &str, state: &mut LanObservation) -> bool {
    static PORT: OnceLock<regex::Regex> = OnceLock::new();
    static DOMAIN: OnceLock<regex::Regex> = OnceLock::new();
    let port_pattern = PORT.get_or_init(|| regex::Regex::new(r"(?i)(?:local game hosted on port|started serving on|partida local alojada en (?:el )?puerto|servidor local (?:iniciado|abierto) en (?:el )?puerto)\s*:?\s*(\d{1,5})").unwrap());
    let domain_pattern = DOMAIN.get_or_init(|| regex::Regex::new(r"(?i)\b([a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?(?:\.[a-z0-9-]+)*\.e4mc\.(?:link|me|net))\b").unwrap());
    let mut changed = false;
    if let Some(port) = port_pattern
        .captures(line)
        .and_then(|c| c[1].parse::<u16>().ok())
        .filter(|p| *p > 0)
    {
        state.port = Some(port);
        state.public_address = None;
        changed = true;
    }
    if let Some(domain) = domain_pattern.captures(line) {
        state.public_address = Some(domain[1].to_ascii_lowercase());
        changed = true;
    }
    if changed {
        state.updated = time_now();
    }
    changed
}

pub fn observe_game_log(game_dir: &Path, line: &str) {
    if let Ok(mut observed) = observations().lock() {
        if observed.len() > 128 {
            observed.retain(|_, value| time_now().saturating_sub(value.updated) < 86400);
        }
        let state = observed.entry(game_dir.to_path_buf()).or_default();
        parse_lan_line(line, state);
    }
}
fn log_observation(dir: &Path) -> LanObservation {
    let mut result = LanObservation::default();
    let path = dir.join("logs").join("latest.log");
    if let Ok(mut file) = fs::File::open(path) {
        let mut modified = 0;
        if let Ok(metadata) = file.metadata() {
            modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|time| time.as_secs())
                .unwrap_or(0);
            let _ = file.seek(SeekFrom::Start(metadata.len().saturating_sub(512 * 1024)));
        }
        let mut tail = Vec::new();
        let _ = file.take(512 * 1024).read_to_end(&mut tail);
        for line in String::from_utf8_lossy(&tail).lines() {
            parse_lan_line(line, &mut result);
        }
        result.updated = modified;
    }
    if let Some(live) = observations()
        .lock()
        .ok()
        .and_then(|map| map.get(dir).cloned())
    {
        if live.updated >= result.updated {
            return live;
        }
    }
    result
}
fn local_addresses() -> Vec<String> {
    let mut addresses = Vec::new();

    for target in ["192.0.2.1:9", "8.8.8.8:53"] {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(target).is_ok() {
                if let Ok(address) = socket.local_addr() {
                    if !address.ip().is_loopback() && !address.ip().is_unspecified() {
                        addresses.push(address.ip().to_string());
                    }
                }
            }
        }
    }
    if let Ok(socket) = UdpSocket::bind("[::]:0") {
        if socket.connect("[2001:4860:4860::8888]:53").is_ok() {
            if let Ok(address) = socket.local_addr() {
                if !address.ip().is_loopback() && !address.ip().is_unspecified() {
                    addresses.push(address.ip().to_string());
                }
            }
        }
    }
    addresses.sort();
    addresses.dedup();
    addresses
}
#[derive(Serialize)]
pub struct LanStatus {
    pub port: Option<u16>,
    pub listening: bool,
    pub local_addresses: Vec<String>,
    pub public_address: Option<String>,
    pub e4mc_installed: bool,
    pub message: String,
}
#[tauri::command]
pub async fn lan_status(
    username: String,
    version_id: String,
    instance_id: Option<String>,
    manual_port: Option<u16>,
) -> Result<LanStatus, String> {
    if manual_port == Some(0) {
        return Err("El puerto debe estar entre 1 y 65535.".into());
    }
    let dir = game_dir(&username, &version_id, instance_id.as_deref())?;
    let observation = log_observation(&dir);
    let port = manual_port.or(observation.port);
    let listening = if let Some(port) = port {
        timeout(Duration::from_millis(500), async {
            if TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port)))
                .await
                .is_ok()
            {
                true
            } else {
                TcpStream::connect(SocketAddr::from((Ipv6Addr::LOCALHOST, port)))
                    .await
                    .is_ok()
            }
        })
        .await
        .unwrap_or(false)
    } else {
        false
    };
    let local_addresses = port
        .filter(|_| listening)
        .map(|port| {
            local_addresses()
                .into_iter()
                .map(|ip| {
                    if ip.contains(':') {
                        format!("[{ip}]:{port}")
                    } else {
                        format!("{ip}:{port}")
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let e4mc_installed = fs::read_dir(dir.join("mods"))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            let file = entry.file_name().to_string_lossy().to_ascii_lowercase();
            file.contains("e4mc") && file.ends_with(".jar")
        });
    let public_address = if listening && (manual_port.is_none() || manual_port == observation.port)
    {
        observation.public_address
    } else {
        None
    };
    let message = if listening && public_address.is_some() {
        "Minecraft tiene un puerto LAN abierto y e4mc publicó una dirección en su registro. La conexión dura mientras mantengas el mundo abierto."
    } else if listening {
        "Puerto LAN abierto. Los jugadores de tu misma red pueden usar la dirección local. Abre el mundo en LAN con e4mc para obtener un dominio público."
    } else if port.is_some() {
        "El puerto detectado no está escuchando. Abre tu mundo y elige Abrir en LAN; un registro anterior no confirma una sesión activa."
    } else {
        "Abre un mundo de Minecraft y selecciona Abrir en LAN. Detectaremos el puerto y la dirección pública de e4mc cuando aparezcan en el registro."
    };
    Ok(LanStatus {
        port,
        listening,
        local_addresses,
        public_address,
        e4mc_installed,
        message: message.into(),
    })
}
#[tauri::command]
pub async fn install_e4mc(
    app: AppHandle,
    username: String,
    version_id: String,
    instance_id: Option<String>,
    game_version: String,
    loader: String,
) -> Result<String, String> {
    let (actual_game, actual_loader) =
        if let Some(id) = instance_id.as_deref().filter(|s| !s.is_empty()) {
            let pack = crate::mod_studio::read_pack(id)?;
            (pack.game_version, pack.loader)
        } else {
            crate::resolve_mod_game_version(&crate::root_dir()?, &version_id)?
        };
    if actual_game != game_version || actual_loader != loader {
        return Err(
            "La instalación seleccionada cambió. Actualiza el panel antes de instalar e4mc.".into(),
        );
    }
    if !["fabric", "forge", "neoforge", "quilt"].contains(&actual_loader.as_str()) {
        return Err("e4mc necesita una instalación Fabric, Forge, NeoForge o Quilt compatible. Selecciónala primero en Jugar.".into());
    }
    let client = reqwest::Client::builder()
        .user_agent("TP LAUNCHER/1.0 (Minecraft launcher; e4mc integration)")
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get("https://api.modrinth.com/v2/project/e4mc/version")
        .query(&[
            ("loaders", serde_json::to_string(&[&actual_loader]).unwrap()),
            (
                "game_versions",
                serde_json::to_string(&[&actual_game]).unwrap(),
            ),
            ("include_changelog", "false".into()),
        ])
        .send()
        .await
        .map_err(|e| format!("No se pudo consultar e4mc en Modrinth: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Modrinth no pudo devolver las versiones de e4mc: {e}"))?;
    if response.content_length().unwrap_or(0) > 5 * 1024 * 1024 {
        return Err("El catálogo de e4mc es demasiado grande.".into());
    }
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > 5 * 1024 * 1024 {
        return Err("El catálogo de e4mc es demasiado grande.".into());
    }
    let versions: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let version = versions.iter().find(|v| v["version_type"] == "release").or_else(|| versions.first()).ok_or_else(|| format!("e4mc no publica una versión compatible con Minecraft {actual_game} / {actual_loader}. No se descargó ningún archivo incompatible."))?;
    let file_id = version["id"]
        .as_str()
        .ok_or("La versión de e4mc no tiene identificador.")?
        .to_string();
    let result = crate::mod_studio::studio_install_version(
        app,
        "modrinth".into(),
        "qANg5Jrr".into(),
        file_id,
        "mod".into(),
        instance_id,
        username,
        version_id,
    )
    .await?;
    Ok(format!(
        "{result}. Reinicia Minecraft y abre tu mundo en LAN; la dirección pública aparecerá cuando e4mc establezca su conexión."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addresses_support_ipv6_and_reject_urls_and_ports() {
        assert_eq!(parse_address("example.org").unwrap().port, 25565);
        assert_eq!(
            parse_address("[2001:db8::1]:25566").unwrap().display(),
            "[2001:db8::1]:25566"
        );
        assert_eq!(parse_address("::1").unwrap().port, 25565);
        for bad in [
            "https://example.org",
            "host:0",
            "host:65536",
            "hello world",
            "../bad",
            "-bad.net",
            "[::1]:no",
        ] {
            assert!(parse_address(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn nbt_modified_utf8_and_unknown_tags_roundtrip() {
        let root = Tag::Compound(vec![
            (
                "servers".into(),
                Tag::List(
                    10,
                    vec![Tag::Compound(vec![
                        ("name".into(), Tag::string("Mi mundo 🌍\0").unwrap()),
                        ("ip".into(), Tag::string("localhost").unwrap()),
                        (
                            "acceptedCodeOfConduct".into(),
                            Tag::Raw(3, 42_i32.to_be_bytes().to_vec()),
                        ),
                        ("hidden".into(), Tag::Raw(1, vec![1])),
                    ])],
                ),
            ),
            (
                "futureMetadata".into(),
                Tag::Raw(12, vec![0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 7]),
            ),
        ]);
        let bytes = nbt_bytes("servidores", &root).unwrap();
        let (name, decoded) = parse_nbt(&bytes).unwrap();
        assert_eq!(name, "servidores");
        assert_eq!(root, decoded);
        assert_eq!(nbt_bytes(&name, &decoded).unwrap(), bytes);
        assert_eq!(entries_of(&decoded).unwrap()[0].name, "Mi mundo 🌍\0");
    }
    #[test]
    fn save_preserves_metadata_backup_order_and_detects_external_changes() {
        let dir = std::env::temp_dir().join(format!("tps-servers-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        let root = Tag::Compound(vec![
            (
                "servers".into(),
                Tag::List(
                    10,
                    vec![Tag::Compound(vec![
                        ("name".into(), Tag::string("Original").unwrap()),
                        ("ip".into(), Tag::string("example.org").unwrap()),
                        ("hidden".into(), Tag::Raw(1, vec![1])),
                    ])],
                ),
            ),
            ("custom".into(), Tag::string("conservar").unwrap()),
        ]);
        let original = nbt_bytes("", &root).unwrap();
        fs::write(dir.join("servers.dat"), &original).unwrap();
        let loaded = list_at(&dir).unwrap();
        let mut entries = loaded.entries;
        entries[0].name = "Renombrado".into();
        entries.insert(
            0,
            ServerEntry {
                id: String::new(),
                name: "Nuevo".into(),
                address: "localhost:25566".into(),
                icon: String::new(),
            },
        );
        let saved = save_at(&dir, entries, &loaded.revision).unwrap();
        assert_eq!(saved.entries[0].name, "Nuevo");
        assert_eq!(saved.entries[1].name, "Renombrado");
        assert_eq!(fs::read(dir.join("servers.dat_old")).unwrap(), original);
        let (_, newroot) = parse_nbt(&fs::read(dir.join("servers.dat")).unwrap()).unwrap();
        assert_eq!(
            newroot.field("custom").and_then(Tag::as_string).unwrap(),
            "conservar"
        );
        assert_eq!(
            server_tags(&newroot).unwrap()[1].field("hidden"),
            Some(&Tag::Raw(1, vec![1]))
        );
        assert!(save_at(&dir, saved.entries, &loaded.revision).is_err());
        for name in ["servers.dat", "servers.dat_old"] {
            fs::remove_file(dir.join(name)).unwrap();
        }
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn nbt_rejects_truncation_bad_types_and_oversized_lists() {
        assert!(parse_nbt(&[10, 0, 0, 9, 0, 1, b'x', 10, 0x7f, 0xff, 0xff, 0xff, 0]).is_err());
        assert!(parse_nbt(&[10, 0, 0, 99]).is_err());
        assert!(parse_nbt(&[10, 0, 0, 8, 0, 1, b'x', 0, 5, b'a']).is_err());
    }
    #[test]
    fn motd_retains_colors_and_styles_without_html() {
        let mut spans = vec![];
        component_text(
            &serde_json::json!({"text":"Hola ","color":"gold","extra":[{"text":"§aMinecraft","bold":true},{"text":"<script>","italic":true}]}),
            &default_span(),
            &mut spans,
            0,
        );
        assert_eq!(spans[0].color, "#ffaa00");
        assert!(
            spans
                .iter()
                .any(|s| s.text == "Minecraft" && s.color == "#55ff55")
        );
        assert!(spans.iter().any(|s| s.text == "<script>" && s.italic));
    }
    #[test]
    fn lan_requires_real_log_patterns_and_does_not_invent_tunnels() {
        let mut state = LanObservation::default();
        assert!(!parse_lan_line("no tunnel yet", &mut state));
        assert!(state.public_address.is_none());
        parse_lan_line(
            "[Server thread/INFO]: Local game hosted on port 54321",
            &mut state,
        );
        assert_eq!(state.port, Some(54321));
        parse_lan_line("[e4mc] Domain assigned: amber-fox.eu.e4mc.link", &mut state);
        assert_eq!(
            state.public_address.as_deref(),
            Some("amber-fox.eu.e4mc.link")
        );
        parse_lan_line("Partida local alojada en el puerto 25567", &mut state);
        assert_eq!(state.port, Some(25567));
        assert!(state.public_address.is_none());
    }
    #[tokio::test]
    async fn status_ping_reads_a_real_tcp_fixture() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let _ = read_packet(&mut socket, 1024).await.unwrap();
            assert_eq!(read_packet(&mut socket, 1024).await.unwrap(), vec![0]);
            let body=serde_json::to_vec(&serde_json::json!({"version":{"name":"Fixture 1.21","protocol":767},"players":{"online":3,"max":20},"description":{"text":"Servidor real TCP","color":"green"}})).unwrap();
            let mut response = vec![0];
            varint(&mut response, body.len() as i32);
            response.extend(body);
            packet(&mut socket, &response).await.unwrap();
            let pong = read_packet(&mut socket, 64).await.unwrap();
            packet(&mut socket, &pong).await.unwrap();
        });
        let status = server_ping(format!("127.0.0.1:{port}")).await.unwrap();
        assert!(status.online);
        assert_eq!(status.players_online, 3);
        assert_eq!(status.players_max, 20);
        assert_eq!(status.protocol, 767);
        assert_eq!(status.motd[0].text, "Servidor real TCP");
        server.await.unwrap();
    }
}
