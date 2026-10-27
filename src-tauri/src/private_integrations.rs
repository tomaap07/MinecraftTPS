
use std::{fs, path::PathBuf};

pub(crate) fn curseforge_api_key() -> Result<String, String> {
    if let Ok(value) = std::env::var("CURSEFORGE_API_KEY") {
        if !value.trim().is_empty() { return Ok(value.trim().to_owned()); }
    }


    let mut paths: Vec<PathBuf> = Vec::new();
    if cfg!(debug_assertions) {
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join(".env"));
            if cwd.file_name().is_some_and(|name| name == "src-tauri") {
                if let Some(parent) = cwd.parent() { paths.push(parent.join(".env")); }
            }
        }
    }
    if let Ok(root) = crate::root_dir() { paths.push(root.join("integrations.env")); }
    for path in paths {
        let Ok(metadata) = fs::metadata(&path) else { continue };
        if !metadata.is_file() || metadata.len() > 32 * 1024 { continue; }
        let Ok(contents) = fs::read_to_string(path) else { continue };
        for line in contents.lines() {
            let line = line.trim().strip_prefix("export ").unwrap_or(line.trim());
            let Some((key, value)) = line.split_once('=') else { continue };
            if key.trim() != "CURSEFORGE_API_KEY" { continue; }
            let value = value.trim();
            let value = if value.len() >= 2 && ((value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\''))) {
                &value[1..value.len()-1]
            } else { value };
            if !value.is_empty() { return Ok(value.to_owned()); }
        }
    }
    Err("CurseForge necesita una clave privada autorizada en CURSEFORGE_API_KEY o integrations.env del dispositivo. Puedes continuar con Modrinth o visitar curseforge.com.".into())
}
