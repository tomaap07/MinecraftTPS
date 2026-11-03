use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::{Cursor, Read, Write}, path::{Path, PathBuf}, sync::Mutex};

static LOCK: Mutex<()> = Mutex::new(());
const LIMIT: usize = 256 * 1024;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cape {
    pub id: String,
    pub name: String,
    pub original: bool,
    #[serde(default)] pub data_url: String,
}
fn safe_dir(path: &Path) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() || !meta.is_dir() { return Err("La carpeta de cosméticos no es un directorio seguro".into()); }
    } else { fs::create_dir_all(path).map_err(|_| "No se pudo crear la carpeta de cosméticos")?; }
    Ok(())
}
fn folder() -> Result<PathBuf, String> {
    let path = crate::launcher_settings::data_root()?.join("cosmetics_library");
    safe_dir(&path)?; Ok(path)
}
fn valid_id(id: &str) -> bool { id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) }
fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|_| "No se pudo leer el cosmético")?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit as u64 { return Err("Archivo de cosmético no permitido".into()); }
    let mut bytes = Vec::new(); fs::File::open(path).map_err(|_| "No se pudo abrir el cosmético")?.take(limit as u64 + 1).read_to_end(&mut bytes).map_err(|_| "No se pudo leer el cosmético")?;
    if bytes.len() > limit { return Err("Archivo de cosmético demasiado grande".into()); } Ok(bytes)
}
fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if fs::symlink_metadata(path).is_ok_and(|meta| !meta.is_file() || meta.file_type().is_symlink()) { return Err("Destino de cosmético no permitido".into()); }
    let staged = path.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    let result = (|| { let mut f = fs::OpenOptions::new().write(true).create_new(true).open(&staged)?; f.write_all(bytes)?; f.sync_all()?; drop(f); fs::rename(&staged, path) })();
    result.map_err(|_: std::io::Error| { let _ = fs::remove_file(staged); "No se pudo guardar el cosmético".into() })
}
fn png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() < 33 || bytes.len() > LIMIT || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" { return Err("Elige un PNG válido de hasta 256 KiB".into()); }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if width != 64 || height != 32 { return Err("La capa TP debe medir exactamente 64 × 32 píxeles".into()); }
    let mut reader=image::ImageReader::with_format(Cursor::new(bytes),image::ImageFormat::Png);
    let mut limits=image::Limits::default();limits.max_image_width=Some(64);limits.max_image_height=Some(32);limits.max_alloc=Some(4*1024*1024);reader.limits(limits);
    let image = reader.decode().map_err(|_| "La imagen PNG está dañada o supera los límites de memoria")?.to_rgba8();
    let mut output = Cursor::new(Vec::new()); image.write_to(&mut output, image::ImageFormat::Png).map_err(|_| "No se pudo normalizar el PNG")?; Ok(output.into_inner())
}
fn add(dir: &Path, name: &str, bytes: &[u8], original: bool) -> Result<Cape, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 60 || name.chars().any(char::is_control) { return Err("Escribe un nombre de 1 a 60 caracteres".into()); }
    let bytes = png(bytes)?; let id = format!("{:x}", Sha256::digest(&bytes));
    let metadata = dir.join(format!("{id}.json"));
    if metadata.exists() {
        let mut cape: Cape = serde_json::from_slice(&read_bounded(&metadata, 4096)?).map_err(|_| "Metadatos de capa dañados")?;
        let saved=png(&read_bounded(&dir.join(format!("{id}.png")), LIMIT)?)?;
        if cape.id!=id || digest_png(&saved)!=id {return Err("La capa guardada cambió fuera del armario; no se reemplazará automáticamente".into());}
        cape.data_url = format!("data:image/png;base64,{}", STANDARD.encode(saved));
        return Ok(cape);
    }
    atomic(&dir.join(format!("{id}.png")), &bytes)?;
    let mut cape = Cape { id, name: name.into(), original, data_url: String::new() };
    atomic(&metadata, &serde_json::to_vec(&cape).map_err(|_| "No se pudo guardar la ficha")?)?;
    cape.data_url = format!("data:image/png;base64,{}", STANDARD.encode(bytes)); Ok(cape)
}
fn digest_png(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn starter() -> Vec<u8> {
    let mut image = image::RgbaImage::from_pixel(64, 32, image::Rgba([12, 8, 24, 255]));
    for y in 0..32 { for x in 0..64 {
        let glow = ((x * 13 + y * 7) % 37) as u8;
        image.put_pixel(x, y, image::Rgba([24 + glow, 12 + glow / 2, 54 + glow * 2, 255]));
        if (x * 17 + y * 31) % 89 == 0 { image.put_pixel(x, y, image::Rgba([224, 204, 255, 255])); }
    }}
    for base in [1, 12] { for y in 2..17 { for x in 0i32..10 { if (x - 4).abs() + (y - 9i32).abs() / 2 <= 3 { image.put_pixel(base + x as u32, y as u32, image::Rgba([168, 85, 247, 255])); } } } }
    let mut bytes = Cursor::new(Vec::new()); image.write_to(&mut bytes, image::ImageFormat::Png).expect("small original PNG"); bytes.into_inner()
}
#[tauri::command]
pub fn list_tp_capes() -> Result<Vec<Cape>, String> {
    let _lock = LOCK.lock().map_err(|_| "Armario ocupado")?; let dir = folder()?;
    add(&dir, "Nebulosa Infinita", &starter(), true)?;
    let mut capes = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|_| "No se pudo abrir el armario")?.take(1024) {
        let path = entry.map_err(|_| "No se pudo leer el armario")?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") { continue; }
        let id = path.file_stem().and_then(|v| v.to_str()).unwrap_or_default(); if !valid_id(id) { continue; }
        let mut cape: Cape = serde_json::from_slice(&read_bounded(&path, 4096)?).map_err(|_| "Ficha de capa dañada")?;
        if cape.id != id { return Err("Identidad de capa no válida".into()); }
        let bytes = png(&read_bounded(&dir.join(format!("{id}.png")), LIMIT)?)?;
        if format!("{:x}", Sha256::digest(&bytes)) != id { return Err("La capa cambió fuera del armario; impórtala de nuevo".into()); }
        cape.data_url = format!("data:image/png;base64,{}", STANDARD.encode(bytes)); capes.push(cape);
    }
    capes.sort_by(|a,b| b.original.cmp(&a.original).then_with(|| a.name.cmp(&b.name))); Ok(capes)
}
#[tauri::command]
pub fn save_tp_cape(name: String, data_url: String) -> Result<Cape, String> {
    let _lock = LOCK.lock().map_err(|_| "Armario ocupado")?;
    if data_url.len() > LIMIT * 2 { return Err("PNG demasiado grande".into()); }
    let data = data_url.strip_prefix("data:image/png;base64,").ok_or("Se requiere un PNG local")?;
    let bytes = STANDARD.decode(data).map_err(|_| "PNG no válido")?; add(&folder()?, &name, &bytes, false)
}
#[tauri::command]
pub fn equip_tp_cosmetics(username: String, version_id: String, pack_id: Option<String>, cape_id: Option<String>, wings: bool, crown: bool) -> Result<String, String> {
    let _lock = LOCK.lock().map_err(|_| "Armario ocupado")?;
    if crate::game_session::is_running() { return Err("Cierra Minecraft para cambiar el equipamiento de la instancia desde el launcher".into()); }
    let game_dir = crate::tp_client::resolve_game_dir(&username, &version_id, pack_id.as_deref())?;
    if !crate::tp_client::installed_in(&game_dir) { return Err("Instala TP Client en esa instancia antes de equipar cosméticos".into()); }
    let config = game_dir.join("config"); safe_dir(&config)?;
    let texture_folder = config.join("tp-client-cosmetics"); safe_dir(&texture_folder)?;
    let file = match cape_id {
        Some(id) => {
            if !valid_id(&id) { return Err("Identificador de capa no válido".into()); }
            let bytes = png(&read_bounded(&folder()?.join(format!("{id}.png")), LIMIT)?)?;
            if format!("{:x}", Sha256::digest(&bytes)) != id { return Err("La capa no supera la comprobación de integridad".into()); }
            let name = format!("{id}.png"); atomic(&texture_folder.join(&name), &bytes)?; name
        }, None => String::new(),
    };
    let path = config.join("tp-client-cosmetics.json");
    if path.exists() {
        let backup = config.join(format!("tp-client-cosmetics.{}.backup.json", uuid::Uuid::new_v4()));
        atomic(&backup, &read_bounded(&path, 16384)?)?;
    }
    atomic(&path, &serde_json::to_vec_pretty(&serde_json::json!({"schemaVersion":1,"equippedCape":file,"wings":wings,"crown":crown})).map_err(|_| "No se pudo guardar el equipamiento")?)?;
    Ok("Cosméticos locales guardados. Habilita Cosméticos en el menú de TP Client. No se modifica el inventario de capas oficiales ni se publica el PNG.".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn validates_png_and_reencodes_original_art() {
        let bytes = starter(); assert_eq!(png(&bytes).unwrap(), bytes);
        assert!(png(b"not-png").is_err()); let mut forged = bytes.clone(); forged[16..20].copy_from_slice(&1000000u32.to_be_bytes()); assert!(png(&forged).is_err());
        assert!(!valid_id("../outside")); assert!(!valid_id(&"A".repeat(64)));
    }
    #[test] fn local_library_deduplicates_by_content_and_preserves_original_provenance() {
        let dir = std::env::temp_dir().join(format!("tp-cape-test-{}", uuid::Uuid::new_v4())); fs::create_dir(&dir).unwrap();
        let original = add(&dir,"Nebulosa Infinita",&starter(),true).unwrap(); let same = add(&dir,"rename",&starter(),false).unwrap();
        assert_eq!(same.id, original.id); assert!(same.original); assert_eq!(same.name,"Nebulosa Infinita");
        assert_eq!(fs::read_dir(&dir).unwrap().count(),2); assert!(add(&dir," ",&starter(),false).is_err()); fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn existing_tampered_texture_cannot_bypass_deduplication() {
        let dir=std::env::temp_dir().join(format!("tp-cape-test-{}",uuid::Uuid::new_v4()));fs::create_dir(&dir).unwrap();
        let original=add(&dir,"Original",&starter(),true).unwrap();let file=dir.join(format!("{}.png",original.id));fs::write(&file,b"not-png").unwrap();
        assert!(add(&dir,"Repeat",&starter(),false).is_err());assert_eq!(fs::read(file).unwrap(),b"not-png");
        fs::remove_dir_all(dir).unwrap();
    }
}
