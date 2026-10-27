use serde::Serialize;
use std::{collections::HashMap, path::Path};

#[derive(Serialize)]
pub struct InstallTarget {
    id: String,
    name: String,
    game_version: String,
    loader: String,
    version_id: String,
    pack_id: Option<String>,
    shader_loader: Option<String>,
}

pub(crate) fn file_compatible(kind: &str, games: &[String], engines: &[String], game: &str, loader: &str, shader: Option<&str>) -> bool {
    if !games.iter().any(|value| value == game) { return false; }
    let engines = engines.iter().map(|value| value.to_ascii_lowercase()).collect::<Vec<_>>();
    let shader = shader.map(str::to_ascii_lowercase);
    let supports = |value: &String| value == loader || shader.as_ref().is_some_and(|shader| value == shader || (value == "iris" && shader == "oculus"));
    match kind {
        "mod" => ["fabric", "forge", "neoforge", "quilt"].contains(&loader) && engines.iter().any(|value| value == loader),
        "shader" => shader.is_some() && (engines.is_empty() || engines.iter().any(supports)),
        "resourcepack" => engines.is_empty() || engines.iter().any(|value| ["minecraft", "vanilla"].contains(&value.as_str()) || supports(value)),
        _ => false,
    }
}

pub(crate) fn installed_profile(root: &Path, id: &str) -> bool {
    let valid = |value: &str| !value.is_empty() && value != "." && value != ".." && value.chars().all(|c|c.is_ascii_alphanumeric() || "-._+".contains(c));
    if !valid(id) { return false; }
    let Ok(bytes) = std::fs::read(root.join("versions").join(id).join(format!("{id}.json"))) else { return false; };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else { return false; };
    let base = value["inheritsFrom"].as_str().or_else(||value["jar"].as_str()).unwrap_or(id);
    valid(id) && valid(base) && (root.join("versions").join(id).join(format!("{id}.jar")).is_file() || root.join("versions").join(base).join(format!("{base}.jar")).is_file())
}

fn list(username: &str) -> Result<Vec<InstallTarget>, String> {
    let root = crate::root_dir()?;
    crate::version_game_dir(&root, username, "validation")?;
    let mut result = Vec::new();
    let mut shaders = HashMap::new();
    if root.join("versions").is_dir() {
        for version in crate::list_installed_versions_inner()? {
            if !version.jar || !version.json || !installed_profile(&root,&version.id) { continue; }
            let dir = crate::version_game_dir(&root, username, &version.id)?;
            let key = (dir.clone(),version.loader.clone());
            let shader_loader = shaders.entry(key).or_insert_with(||crate::hud_controls::shader_loader(&dir,&version.loader)).clone();
            result.push(InstallTarget {id:format!("version:{}",version.id),name:version.id.clone(),game_version:version.game_version,loader:version.loader,version_id:version.id,pack_id:None,shader_loader});
        }
    }
    if root.join("instances").is_dir() {
        for pack in crate::mod_studio::studio_list_packs()? {
            if pack.install_state != "ready" || !installed_profile(&root,&pack.version_id) { continue; }
            let dir = crate::mod_studio::pack_game_dir(&pack.id)?;
            let shader_loader = crate::hud_controls::shader_loader(&dir,&pack.loader);
            result.push(InstallTarget {id:pack.id.clone(),name:pack.name,game_version:pack.game_version,loader:pack.loader,version_id:pack.version_id,pack_id:Some(pack.id),shader_loader});
        }
    }
    result.sort_by(|a,b|b.pack_id.is_some().cmp(&a.pack_id.is_some()).then_with(||a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(result)
}

#[tauri::command]
pub async fn studio_install_targets(username: String) -> Result<Vec<InstallTarget>, String> {
    tauri::async_runtime::spawn_blocking(move || list(&username)).await.map_err(|error|error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_other_games_loaders_and_missing_shader_engines() {
        let games=vec!["1.21.1".into()];
        assert!(file_compatible("mod",&games,&["fabric".into()],"1.21.1","fabric",None));
        assert!(!file_compatible("mod",&games,&["fabric".into()],"1.21.1","quilt",None));
        assert!(!file_compatible("mod",&games,&[],"1.21.1","fabric",None));
        assert!(!file_compatible("mod",&games,&["forge".into()],"1.21.1","neoforge",None));
        assert!(!file_compatible("resourcepack",&games,&["minecraft".into()],"1.20.1","vanilla",None));
        assert!(!file_compatible("shader",&games,&["iris".into()],"1.21.1","fabric",None));
        assert!(file_compatible("shader",&games,&["iris".into()],"1.21.1","forge",Some("Oculus")));
        assert!(!file_compatible("shader",&games,&["canvas".into()],"1.21.1","fabric",Some("Iris")));
        assert!(file_compatible("resourcepack",&games,&["minecraft".into()],"1.21.1","vanilla",None));
        assert!(!file_compatible("resourcepack",&games,&["optifine".into()],"1.21.1","vanilla",None));
    }
}
