
use base64::Engine;
use serde::Serialize;
use std::{
    fs,
    io::{Cursor, Read},
    path::PathBuf,
};

#[derive(Serialize)]
pub struct HudPreview {
    pub data_url: String,
    pub label: String,
    pub custom: bool,
}
fn path() -> Result<PathBuf, String> {
    Ok(crate::launcher_plugins::profile_path()?
        .parent()
        .ok_or("Carpeta del HUD no disponible")?
        .join("hud-preview.png"))
}
pub(crate) fn decode(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    if bytes.len() > 20 * 1024 * 1024 {
        return Err("La captura debe ocupar menos de 20 MB".into());
    }
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Selecciona una captura PNG de Minecraft".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| "PNG incompleto")?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| "PNG incompleto")?);
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || u64::from(width) * u64::from(height) > 40_000_000
    {
        return Err("La captura supera las dimensiones admitidas".into());
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|_| "La captura PNG está dañada".into())
}
fn encoded(image: &image::DynamicImage) -> Result<Vec<u8>, String> {
    let mut out = Cursor::new(Vec::new());
    image
        .thumbnail(1280, 720)
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(out.into_inner())
}
fn response(bytes: &[u8], label: String, custom: bool) -> HudPreview {
    HudPreview {
        data_url: format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ),
        label,
        custom,
    }
}
#[tauri::command]
pub async fn get_hud_preview() -> Result<HudPreview, String> {
    tokio::task::spawn_blocking(|| {
        let target = path()?;
        if let Ok(file) = fs::File::open(target) {
            if file.metadata().is_ok_and(|m| m.len() <= 20 * 1024 * 1024) {
                let mut bytes = Vec::new();
                if file
                    .take(20 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)
                    .is_ok()
                {
                    if let Ok(image) = decode(&bytes) {
                        return Ok(response(
                            &encoded(&image)?,
                            "Captura personal · guardada en este equipo".into(),
                            true,
                        ));
                    }
                }
            }
        }
        if let Some((image, name)) = crate::launcher_tools::latest_hud_screenshot() {
            return Ok(response(&encoded(&image)?, format!("F2 · {name}"), false));
        }
        Ok(HudPreview {
            data_url: String::new(),
            label: "Elige una captura tuya o toma una con F2 dentro de Minecraft.".into(),
            custom: false,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn set_hud_preview(image_base64: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let target = path()?;
        if image_base64.is_empty() {
            if target.exists() {
                fs::remove_file(target).map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
        if image_base64.len() > 28 * 1024 * 1024 {
            return Err("La captura debe ocupar menos de 20 MB".into());
        }
        let source = image_base64
            .strip_prefix("data:image/png;base64,")
            .ok_or("Selecciona una captura PNG")?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(source)
            .map_err(|_| "Captura PNG inválida")?;
        let image = decode(&bytes)?;
        let bytes = encoded(&image)?;
        let pending =
            target.with_file_name(format!(".hud-preview-{}.pending", uuid::Uuid::new_v4()));
        fs::write(&pending, bytes)
            .and_then(|_| fs::rename(&pending, target))
            .map_err(|e| {
                let _ = fs::remove_file(pending);
                e.to_string()
            })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_fake_and_oversized_images() {
        assert!(decode(b"not a PNG").is_err());
        let mut bytes = b"\x89PNG\r\n\x1a\n12345678".to_vec();
        bytes.extend_from_slice(&40000u32.to_be_bytes());
        bytes.extend_from_slice(&40000u32.to_be_bytes());
        assert!(decode(&bytes).is_err());
    }
    #[test]
    fn private_preview_roundtrips_and_bounds_resolution() {
        let image = image::DynamicImage::new_rgb8(1920, 1080);
        let bytes = encoded(&image).unwrap();
        let decoded = decode(&bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (1280, 720));
        let view = response(&bytes, "Personal".into(), true);
        assert!(view.custom);
        assert!(view.data_url.starts_with("data:image/png;base64,"));
    }
}
