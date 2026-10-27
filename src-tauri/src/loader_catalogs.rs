
use crate::studio_progress::{INSTALL_LOCK, Operation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
    fs,
    io::Read,
    path::Path,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use tauri::AppHandle;

const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const QUILT_GAMES: &str = "https://meta.quiltmc.org/v3/versions/game";
const QUILT_LOADERS: &str = "https://meta.quiltmc.org/v3/versions/loader";
const NEOFORGE: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
const NEOFORGE_LEGACY: &str =
    "https://maven.neoforged.net/releases/net/neoforged/forge/maven-metadata.xml";
const SOURCE_LIMIT: usize = 4 * 1024 * 1024;
const MAX_ROWS: usize = 30_000;
const CACHE_TTL: Duration = Duration::from_secs(10 * 60);

#[derive(Clone, Debug, Serialize)]
pub struct ExtraLoaderVersion {
    pub id: String,
    pub loader_version: String,
    pub stable: bool,
}

#[derive(Clone)]
struct GameVersion {
    id: String,
    stable: bool,
}

#[derive(Clone, Deserialize)]
struct PublishedLoader {
    version: String,
    stable: Option<bool>,
}

#[derive(Clone, Default)]
struct Catalog {
    rows: Vec<ExtraLoaderVersion>,
    allowed: BTreeMap<String, Vec<String>>,
}

static MEMORY: OnceLock<Mutex<BTreeMap<String, (Instant, Catalog)>>> = OnceLock::new();

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))
}

fn parse_games(text: &str) -> Result<Vec<GameVersion>, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let rows = value["versions"]
        .as_array()
        .ok_or("Manifiesto de Mojang inválido")?;
    if rows.len() > MAX_ROWS {
        return Err("El manifiesto excede el límite de versiones".into());
    }
    let mut seen = HashSet::new();
    let result: Vec<_> = rows
        .iter()
        .filter_map(|row| {
            let id = row["id"].as_str()?;
            let kind = row["type"].as_str()?;
            (valid_id(id) && ["release", "snapshot"].contains(&kind) && seen.insert(id.to_owned()))
                .then(|| GameVersion {
                    id: id.into(),
                    stable: kind == "release",
                })
        })
        .collect();
    if result.is_empty() {
        Err("Mojang no devolvió versiones válidas".into())
    } else {
        Ok(result)
    }
}

fn parse_quilt_games(text: &str) -> Result<HashSet<String>, String> {
    let rows: Vec<PublishedLoader> = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if rows.is_empty() || rows.len() > MAX_ROWS || rows.iter().any(|r| !valid_id(&r.version)) {
        return Err("Catálogo de juegos de Quilt inválido".into());
    }
    Ok(rows.into_iter().map(|r| r.version).collect())
}

fn parse_quilt_loaders(text: &str) -> Result<Vec<PublishedLoader>, String> {
    let rows: Vec<PublishedLoader> = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if rows.is_empty() || rows.len() > MAX_ROWS || rows.iter().any(|r| !valid_id(&r.version)) {
        return Err("Catálogo de cargadores de Quilt inválido".into());
    }
    Ok(rows)
}

fn parse_maven(text: &str) -> Result<Vec<String>, String> {
    if !text.contains("<metadata") || !text.contains("</metadata>") {
        return Err("Metadatos Maven inválidos".into());
    }
    let regex = regex::Regex::new(r"<version>\s*([0-9A-Za-z.+_-]{1,160})\s*</version>")
        .map_err(|e| e.to_string())?;
    let result: Vec<_> = regex
        .captures_iter(text)
        .map(|cap| cap[1].to_owned())
        .collect();
    if result.is_empty() || result.len() > MAX_ROWS {
        Err("Maven no devolvió versiones válidas".into())
    } else {
        Ok(result)
    }
}

fn stable_build(version: &str) -> bool {
    let version = version.strip_prefix("1.20.1-").unwrap_or(version);
    !version.contains('-') && !version.contains('+')
}


fn build_order(a: &str, b: &str) -> Ordering {
    fn parts(value: &str) -> Vec<&str> {
        value
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect()
    }
    let a = parts(a);
    let b = parts(b);
    for (a, b) in a.iter().zip(&b) {
        let order = match (a.parse::<u64>(), b.parse::<u64>()) {
            (Ok(a), Ok(b)) => a.cmp(&b),
            _ => a.cmp(b),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len())
}

fn best_loader(versions: &[PublishedLoader]) -> Option<&PublishedLoader> {
    versions.iter().max_by(|a, b| {
        a.stable
            .unwrap_or_else(|| stable_build(&a.version))
            .cmp(&b.stable.unwrap_or_else(|| stable_build(&b.version)))
            .then_with(|| build_order(&a.version, &b.version))
    })
}

fn quilt_catalog(
    games: &[GameVersion],
    supported: &HashSet<String>,
    loaders: &[PublishedLoader],
) -> Catalog {
    let mut catalog = Catalog::default();
    let Some(best) = best_loader(loaders) else {
        return catalog;
    };
    for game in games.iter().filter(|game| supported.contains(&game.id)) {
        catalog.rows.push(ExtraLoaderVersion {
            id: game.id.clone(),
            loader_version: best.version.clone(),
            stable: game.stable && best.stable.unwrap_or_else(|| stable_build(&best.version)),
        });
        catalog.allowed.insert(
            game.id.clone(),
            loaders.iter().map(|l| l.version.clone()).collect(),
        );
    }
    catalog
}

fn neoforge_catalog(games: &[GameVersion], modern: &[String], legacy: &[String]) -> Catalog {
    let mut catalog = Catalog::default();
    for game in games {
        let versions: Vec<_> = (if game.id == "1.20.1" { legacy } else { modern })
            .iter()
            .filter(|version| crate::mod_studio::neoforge_version_matches(&game.id, version))
            .map(|version| PublishedLoader {
                version: version.clone(),
                stable: Some(stable_build(version)),
            })
            .collect();
        let Some(best) = best_loader(&versions) else {
            continue;
        };
        catalog.rows.push(ExtraLoaderVersion {
            id: game.id.clone(),
            loader_version: best.version.clone(),
            stable: game.stable && best.stable.unwrap_or(false),
        });
        catalog.allowed.insert(
            game.id.clone(),
            versions.into_iter().map(|v| v.version).collect(),
        );
    }
    catalog
}

fn cached_source(path: &Path, validate: fn(&str) -> bool) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > SOURCE_LIMIT as u64 {
        return None;
    }
    let mut text = String::new();
    file.take(SOURCE_LIMIT as u64 + 1)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() <= SOURCE_LIMIT && validate(&text)).then_some(text)
}

async fn source(
    client: &Client,
    url: &str,
    cache: &Path,
    validate: fn(&str) -> bool,
) -> Result<String, String> {
    let downloaded: Result<String, String> = async {
        let mut response = client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        if response
            .content_length()
            .is_some_and(|n| n > SOURCE_LIMIT as u64)
        {
            return Err("El catálogo excede el tamaño permitido".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            if bytes.len().saturating_add(chunk.len()) > SOURCE_LIMIT {
                return Err("El catálogo excede el tamaño permitido".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        if !validate(&text) {
            return Err("El proveedor devolvió un catálogo inválido".into());
        }
        Ok(text)
    }
    .await;
    match downloaded {
        Ok(text) => {

            let _ = fs::write(cache, &text);
            Ok(text)
        }
        Err(error) => cached_source(cache, validate).ok_or_else(|| {
            format!(
                "No se pudo consultar el catálogo oficial y no hay una copia local válida: {error}"
            )
        }),
    }
}

async fn catalog(loader: &str) -> Result<Catalog, String> {
    if !["neoforge", "quilt"].contains(&loader) {
        return Err("Cargador no compatible".into());
    }
    if let Ok(memory) = MEMORY.get_or_init(Default::default).lock() {
        if let Some((at, result)) = memory.get(loader) {
            if at.elapsed() < CACHE_TTL {
                return Ok(result.clone());
            }
        }
    }
    let folder = crate::root_dir()?.join("cache/loader-catalogs");
    let _ = fs::create_dir_all(&folder);
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(18))
        .user_agent("TP-LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)")
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| e.to_string())?;
    let manifest_file = folder.join("mojang.json");
    let game_future = source(&client, MANIFEST, &manifest_file, |s| {
        parse_games(s).is_ok()
    });
    let result = if loader == "quilt" {
        let games_file = folder.join("quilt-games.json");
        let loaders_file = folder.join("quilt-loaders.json");
        let (manifest, supported, loaders) = tokio::try_join!(
            game_future,
            source(&client, QUILT_GAMES, &games_file, |s| parse_quilt_games(s)
                .is_ok()),
            source(&client, QUILT_LOADERS, &loaders_file, |s| {
                parse_quilt_loaders(s).is_ok()
            })
        )?;
        quilt_catalog(
            &parse_games(&manifest)?,
            &parse_quilt_games(&supported)?,
            &parse_quilt_loaders(&loaders)?,
        )
    } else {
        let modern_file = folder.join("neoforge.xml");
        let legacy_file = folder.join("neoforge-legacy.xml");
        let (manifest, modern, legacy) = tokio::try_join!(
            game_future,
            source(&client, NEOFORGE, &modern_file, |s| parse_maven(s).is_ok()),
            source(&client, NEOFORGE_LEGACY, &legacy_file, |s| parse_maven(s)
                .is_ok())
        )?;
        neoforge_catalog(
            &parse_games(&manifest)?,
            &parse_maven(&modern)?,
            &parse_maven(&legacy)?,
        )
    };
    if result.rows.is_empty() {
        return Err("El catálogo oficial no incluye versiones compatibles de Minecraft".into());
    }
    if let Ok(mut memory) = MEMORY.get_or_init(Default::default).lock() {
        memory.insert(loader.into(), (Instant::now(), result.clone()));
    }
    Ok(result)
}

#[tauri::command]
pub async fn fetch_extra_loader_catalog(loader: String) -> Result<Vec<ExtraLoaderVersion>, String> {
    Ok(catalog(&loader.to_ascii_lowercase()).await?.rows)
}

#[tauri::command]
pub async fn install_extra_loader(
    app: AppHandle,
    loader: String,
    game_version: String,
    loader_version: Option<String>,
) -> Result<String, String> {
    let loader = loader.to_ascii_lowercase();
    if !valid_id(&game_version) {
        return Err("Versión de Minecraft inválida".into());
    }
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Ya hay una instalación en curso")?;
    let operation = Operation::new(Some(app), &format!("Instalar {loader} · {game_version}"));
    let result = async {
        operation.stage("Comprobando el catálogo oficial", &game_version, 1.0);
        let catalog = catalog(&loader).await?;
        let row = catalog
            .rows
            .iter()
            .find(|row| row.id == game_version)
            .ok_or("El cargador no publica una versión compatible con este Minecraft")?;
        let version = loader_version.as_deref().unwrap_or(&row.loader_version);
        if !catalog
            .allowed
            .get(&game_version)
            .is_some_and(|allowed| allowed.iter().any(|v| v == version))
        {
            return Err(
                "La versión del cargador no pertenece al catálogo oficial de este Minecraft".into(),
            );
        }
        crate::mod_studio::install_loader(&operation, &game_version, &loader, version).await
    }
    .await;
    operation.complete(result, "Cargador instalado y disponible en tu biblioteca")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn games(ids: &[&str]) -> Vec<GameVersion> {
        ids.iter()
            .map(|id| GameVersion {
                id: (*id).into(),
                stable: !id.contains('-'),
            })
            .collect()
    }
    #[test]
    fn exact_neoforge_versions_and_latest_stable_build() {

        let modern = [
            "21.1.99",
            "21.1.265",
            "21.1.266-beta",
            "26.3.0.99",
            "26.1.2.124",
            "26.1.0.0-alpha.2+snapshot-1",
        ]
        .map(str::to_owned);
        let legacy = ["1.20.1-47.1.105", "1.20.1-47.1.106"].map(str::to_owned);
        let catalog = neoforge_catalog(
            &games(&[
                "26.3",
                "26.1.2",
                "26.1-snapshot-1",
                "26.1-snapshot-2",
                "1.21.1",
                "1.21.11",
                "1.20.1",
            ]),
            &modern,
            &legacy,
        );
        let versions: BTreeMap<_, _> = catalog
            .rows
            .iter()
            .map(|r| (r.id.as_str(), r.loader_version.as_str()))
            .collect();
        assert_eq!(versions["1.20.1"], "1.20.1-47.1.106");
        assert_eq!(versions["1.21.1"], "21.1.265");
        assert_eq!(versions["26.3"], "26.3.0.99");
        assert_eq!(versions["26.1.2"], "26.1.2.124");
        assert_eq!(versions["26.1-snapshot-1"], "26.1.0.0-alpha.2+snapshot-1");
        assert!(!versions.contains_key("26.1-snapshot-2"));
        assert!(!versions.contains_key("1.21.11"));
        assert!(
            !catalog
                .rows
                .iter()
                .find(|r| r.id == "26.1-snapshot-1")
                .unwrap()
                .stable
        );
    }
    #[test]
    fn quilt_rejects_unknown_minecraft_and_prefers_stable_loader() {
        let games = parse_games(r#"{"versions":[{"id":"26.3","type":"release"},{"id":"26.3-snapshot-10","type":"snapshot"},{"id":"1.21.1","type":"release"}]}"#).unwrap();
        let supported = parse_quilt_games(r#"[{"version":"26.3","stable":true},{"version":"26.3-snapshot-10","stable":false},{"version":"1.21.1","stable":true},{"version":"999.0","stable":true}]"#).unwrap();
        let loaders = parse_quilt_loaders(
            r#"[{"version":"0.31.0-beta.4"},{"version":"0.30.1"},{"version":"0.29.2"}]"#,
        )
        .unwrap();
        let catalog = quilt_catalog(&games, &supported, &loaders);
        assert_eq!(catalog.rows.len(), 3);
        assert!(catalog.rows.iter().all(|r| r.loader_version == "0.30.1"));
        assert_eq!(catalog.rows[0].id, "26.3");
        assert!(!catalog.rows[1].stable);
        assert!(!catalog.allowed.contains_key("999.0"));
    }
    #[test]
    fn invalid_metadata_and_paths_are_rejected() {
        assert!(parse_maven("<html>Forbidden</html>").is_err());
        assert!(parse_quilt_loaders(r#"[{"version":"../bad"}]"#).is_err());
        assert!(parse_games(r#"{"versions":[]}"#).is_err());
        for id in ["../a", "..", "a/b", "C:\\file", ""] {
            assert!(!valid_id(id));
        }
        assert!(valid_id("26.1.0.0-alpha.2+snapshot-1"));
    }
}
