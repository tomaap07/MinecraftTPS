use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Cursor,
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::State;

const MAX_SKIN_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedSkin {
    pub id: String,
    pub name: String,
    pub model: String,
    pub created_at: u64,
    pub file_name: String,
    #[serde(default, skip_deserializing)]
    pub data_url: String,
    #[serde(default, skip_deserializing, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

fn library_dir() -> Result<PathBuf, String> {
    let path = crate::root_dir()?.join("skins_library");
    fs::create_dir_all(&path).map_err(|e| format!("No se pudo crear el armario: {e}"))?;
    Ok(path)
}

fn library_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn skin_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err("Escribe un nombre de skin de entre 1 y 80 caracteres".into());
    }
    Ok(name.to_owned())
}

fn skin_model(model: &str) -> Result<String, String> {
    match model {
        "classic" | "slim" => Ok(model.to_owned()),
        _ => Err("Elige un modelo Clásico o Slim".into()),
    }
}

fn validate_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() > MAX_SKIN_BYTES || bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Elige un archivo PNG válido de menos de 2 MB".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| "PNG incompleto")?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| "PNG incompleto")?);
    if width != 64 || ![32, 64].contains(&height) {
        return Err("La skin debe medir 64×64 o 64×32 píxeles".into());
    }
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|e| format!("La imagen PNG está dañada: {e}"))?;
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(output.into_inner())
}

fn read_index() -> Result<Vec<SavedSkin>, String> {
    let path = library_dir()?.join("skins.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path).map_err(|e| format!("No se pudo leer el armario: {e}"))?;
    let skins: Vec<SavedSkin> = serde_json::from_slice(&bytes)
        .map_err(|e| format!("El índice del armario no se pudo leer: {e}"))?;
    for skin in &skins {
        uuid::Uuid::parse_str(&skin.id)
            .map_err(|_| "El armario contiene un identificador inválido")?;
        if skin.file_name != format!("{}.png", skin.id) {
            return Err("El armario contiene una ruta inválida".into());
        }
    }
    Ok(skins)
}

fn write_index(skins: &[SavedSkin]) -> Result<(), String> {
    let dir = library_dir()?;
    let mut clean = skins.to_vec();
    for skin in &mut clean {
        skin.data_url.clear();
        skin.error.clear();
    }
    let bytes = serde_json::to_vec_pretty(&clean).map_err(|e| e.to_string())?;
    let pending = dir.join("skins.json.pending");
    fs::write(&pending, bytes).map_err(|e| format!("No se pudo guardar el armario: {e}"))?;
    fs::rename(pending, dir.join("skins.json")).map_err(|e| e.to_string())
}

fn data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(bytes))
}

fn persist_skin(name: String, model: String, bytes: Vec<u8>) -> Result<SavedSkin, String> {
    let name = skin_name(&name)?;
    let model = skin_model(&model)?;
    let bytes = validate_png(&bytes)?;
    let _guard = library_lock()
        .lock()
        .map_err(|_| "El armario está ocupado")?;
    let mut skins = read_index()?;
    let id = uuid::Uuid::new_v4().to_string();
    let file_name = format!("{id}.png");
    let path = library_dir()?.join(&file_name);
    fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    let skin = SavedSkin {
        id,
        name,
        model,
        created_at: crate::unix_time_seconds(),
        file_name,
        data_url: data_url(&bytes),
        error: String::new(),
    };
    skins.push(skin.clone());
    if let Err(error) = write_index(&skins) {
        let _ = fs::remove_file(path);
        return Err(error);
    }
    Ok(skin)
}

#[tauri::command]
pub fn list_saved_skins() -> Result<Vec<SavedSkin>, String> {
    let _guard = library_lock()
        .lock()
        .map_err(|_| "El armario está ocupado")?;
    let mut skins = read_index()?;
    let dir = library_dir()?;
    for skin in &mut skins {
        match fs::read(dir.join(&skin.file_name))
            .map_err(|e| e.to_string())
            .and_then(|bytes| validate_png(&bytes))
        {
            Ok(bytes) => {
                skin.data_url = data_url(&bytes);
                skin.error.clear();
            }
            Err(_) => {
                skin.data_url.clear();
                skin.error = "El PNG falta o está dañado. Puedes volver a importarlo o eliminar esta entrada.".into();
            }
        }
    }
    skins.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(skins)
}

#[tauri::command]
pub fn save_library_skin(
    name: String,
    skin_base64: String,
    model: String,
) -> Result<SavedSkin, String> {
    let encoded = skin_base64
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&skin_base64);
    if encoded.len() > MAX_SKIN_BYTES * 2 {
        return Err("El PNG supera el límite de 2 MB".into());
    }
    persist_skin(
        name,
        model,
        STANDARD
            .decode(encoded)
            .map_err(|_| "El archivo PNG no se pudo leer")?,
    )
}

fn allowed_texture_url(url: &url::Url) -> bool {
    url.scheme() == "https"
        && url.port_or_known_default() == Some(443)
        && matches!(
            url.host_str(),
            Some("textures.minecraft.net" | "mc-heads.net" | "minotar.net")
        )
}

async fn texture_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut parsed = url::Url::parse(url).map_err(|_| "La URL de la skin no es válida")?;
    if parsed.scheme() == "http"
        && matches!(
            parsed.host_str(),
            Some("textures.minecraft.net" | "mc-heads.net" | "minotar.net")
        )
    {
        parsed
            .set_scheme("https")
            .map_err(|_| "La URL de la skin no es válida")?;
    }
    if !allowed_texture_url(&parsed) {
        return Err("La skin debe proceder de Minecraft, MCHeads o Minotar por HTTPS".into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() > 4 || !allowed_texture_url(attempt.url()) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(parsed)
        .send()
        .await
        .map_err(|e| format!("No se pudo descargar la skin: {e}"))?
        .error_for_status()
        .map_err(|e| format!("El servidor no pudo entregar la skin: {e}"))?;
    if response.content_length().unwrap_or(0) > MAX_SKIN_BYTES as u64 {
        return Err("La skin supera los 2 MB".into());
    }
    let mut chunks = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        if bytes.len() + chunk.len() > MAX_SKIN_BYTES {
            return Err("La skin supera los 2 MB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    validate_png(&bytes)
}

#[tauri::command]
pub async fn save_current_skin(
    name: String,
    skin_url: String,
    model: String,
) -> Result<SavedSkin, String> {
    let bytes = if skin_url.starts_with("data:image/png;base64,") {
        if skin_url.len() > MAX_SKIN_BYTES * 2 {
            return Err("El PNG supera el límite de 2 MB".into());
        }
        STANDARD
            .decode(skin_url.trim_start_matches("data:image/png;base64,"))
            .map_err(|_| "La skin no se pudo leer")?
    } else {
        texture_bytes(&skin_url).await?
    };
    persist_skin(name, model, bytes)
}

#[tauri::command]
pub async fn import_library_skin(nick: String, name: String) -> Result<SavedSkin, String> {
    let nick = nick.trim();
    if !(3..=16).contains(&nick.len())
        || !nick.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err("El nick debe tener entre 3 y 16 letras, números o guiones bajos".into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{nick}"
        ))
        .send()
        .await
        .map_err(|e| format!("No se pudo consultar el perfil premium: {e}"))?;
    if response.status().as_u16() == 204 || response.status().as_u16() == 404 {
        return Err("No existe una cuenta premium con ese nick".into());
    }
    let account: serde_json::Value = response
        .error_for_status()
        .map_err(|e| format!("Mojang no pudo resolver ese nick: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let id = account["id"]
        .as_str()
        .filter(|id| id.len() == 32 && id.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or("Mojang no devolvió un perfil premium válido")?;
    let mut model = "classic".to_owned();
    let mut urls = Vec::new();
    if let Ok(response) = client
        .get(format!(
            "https://sessionserver.mojang.com/session/minecraft/profile/{id}"
        ))
        .send()
        .await
    {
        if let Ok(response) = response.error_for_status() {
            if let Ok(profile) = response.json::<serde_json::Value>().await {
                if let Some(value) = profile["properties"]
                    .as_array()
                    .and_then(|p| p.iter().find(|p| p["name"] == "textures"))
                    .and_then(|p| p["value"].as_str())
                {
                    if let Ok(bytes) = STANDARD.decode(value) {
                        if let Ok(textures) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                            if let Some(url) = textures
                                .pointer("/textures/SKIN/url")
                                .and_then(|v| v.as_str())
                            {
                                urls.push(url.replacen("http://", "https://", 1));
                            }
                            if textures
                                .pointer("/textures/SKIN/metadata/model")
                                .and_then(|v| v.as_str())
                                == Some("slim")
                            {
                                model = "slim".into();
                            }
                        }
                    }
                }
            }
        }
    }
    urls.push(format!("https://mc-heads.net/skin/{id}"));
    urls.push(format!("https://minotar.net/skin/{id}"));
    for url in urls {
        if let Ok(bytes) = texture_bytes(&url).await {
            let label = if name.trim().is_empty() {
                account["name"].as_str().unwrap_or(nick).to_owned()
            } else {
                name.clone()
            };
            return persist_skin(label, model, bytes);
        }
    }
    Err("El perfil existe, pero no se pudo descargar su skin. Intenta de nuevo más tarde".into())
}

#[tauri::command]
pub fn rename_library_skin(id: String, name: String) -> Result<(), String> {
    let name = skin_name(&name)?;
    let _guard = library_lock()
        .lock()
        .map_err(|_| "El armario está ocupado")?;
    let mut skins = read_index()?;
    skins
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or("La skin ya no está en el armario")?
        .name = name;
    write_index(&skins)
}

#[tauri::command]
pub fn delete_library_skin(id: String) -> Result<(), String> {
    let _guard = library_lock()
        .lock()
        .map_err(|_| "El armario está ocupado")?;
    let mut skins = read_index()?;
    let index = skins
        .iter()
        .position(|s| s.id == id)
        .ok_or("La skin ya no está en el armario")?;
    let removed = skins.remove(index);
    write_index(&skins)?;
    let path = library_dir()?.join(removed.file_name);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn apply_library_skin(
    auth: State<'_, crate::AuthState>,
    id: String,
) -> Result<crate::MinecraftProfile, String> {
    crate::ensure_microsoft_session(&auth).await?;
    let (skin, bytes) = {
        let _guard = library_lock()
            .lock()
            .map_err(|_| "El armario está ocupado")?;
        let skin = read_index()?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or("La skin ya no está en el armario")?;
        let bytes = validate_png(
            &fs::read(library_dir()?.join(&skin.file_name)).map_err(|e| e.to_string())?,
        )?;
        (skin, bytes)
    };
    crate::upload_minecraft_skin(auth, data_url(&bytes), skin.model).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wardrobe_accepts_classic_and_modern_pngs_but_rejects_oversized_dimensions() {
        for height in [32, 64] {
            let image = image::DynamicImage::new_rgba8(64, height);
            let mut encoded = Cursor::new(Vec::new());
            image
                .write_to(&mut encoded, image::ImageFormat::Png)
                .unwrap();
            assert!(validate_png(encoded.get_ref()).is_ok());
        }
        let image = image::DynamicImage::new_rgba8(128, 128);
        let mut encoded = Cursor::new(Vec::new());
        image
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        assert!(validate_png(encoded.get_ref()).is_err());
        assert!(validate_png(b"not a PNG").is_err());
    }

    #[test]
    fn texture_downloads_only_allow_known_https_hosts() {
        assert!(allowed_texture_url(
            &url::Url::parse("https://textures.minecraft.net/texture/abc").unwrap()
        ));
        for url in [
            "http://mc-heads.net/skin/Steve",
            "https://textures.minecraft.net.evil.test/a",
            "https://localhost/skin.png",
            "https://mc-heads.net:444/skin/Steve",
        ] {
            assert!(
                !allowed_texture_url(&url::Url::parse(url).unwrap()),
                "{url}"
            );
        }
    }
}
