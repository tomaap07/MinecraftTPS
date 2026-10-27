
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime},
};
use tauri::AppHandle;

const MAX_JAR_BYTES: u64 = 80 * 1024 * 1024;
const MAX_DIRECTORY_ENTRIES: usize = 4096;
const MAX_PROFILE_DIRECTORIES: usize = 1024;
const MAX_INSPECTED_FILES: usize = 128;
const MAX_SCAN_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Clone)]
struct CacheEntry {
    size: u64,
    modified: Option<SystemTime>,
    candidate: Option<OptiFineCandidate>,
}
static CACHE: OnceLock<Mutex<HashMap<PathBuf, CacheEntry>>> = OnceLock::new();

#[derive(Clone, Serialize)]
pub struct OptiFineCandidate {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub file_name: String,
    pub path: String,
    pub source: String,
    pub profile_id: String,
    pub preview: bool,
    pub legacy_patch: bool,
}

fn read_small_entry(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    if file.size() > 2 * 1024 * 1024 {
        return None;
    }
    let mut bytes = Vec::new();
    file.by_ref()
        .take(2 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() <= 2 * 1024 * 1024).then_some(bytes)
}


pub(crate) fn validate(file_name: &str, bytes: &[u8]) -> Result<crate::OptiFineVersion, String> {
    if bytes.len() < 4096 || bytes.len() as u64 > MAX_JAR_BYTES {
        return Err("Selecciona el JAR original de OptiFine de hasta 80 MB".into());
    }
    let pattern = regex::Regex::new(
        r"^(?:preview_)?OptiFine_([0-9]+(?:\.[0-9]+){1,3})_HD(?:_U)?_[A-Za-z0-9_]+\.jar$",
    )
    .map_err(|e| e.to_string())?;
    let captures = pattern.captures(file_name).ok_or("El archivo no tiene el nombre original de un instalador OptiFine JAR. No lo renombres antes de importarlo")?;
    let game_version = captures[1].to_owned();
    let mut jar = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| "El archivo local no es un JAR válido")?;
    if jar.len() > 100_000
        || !jar
            .file_names()
            .any(|name| name == "optifine/OptiFineTweaker.class")
    {
        return Err("El JAR no contiene el cargador OptiFine esperado. Usa el archivo original obtenido del autor".into());
    }


    let expected = file_name
        .trim_start_matches("preview_")
        .trim_end_matches(".jar");
    let embedded = regex::Regex::new(r"OptiFine_[0-9]+(?:\.[0-9]+){1,3}_HD(?:_U)?_[A-Za-z0-9_]+")
        .map_err(|e| e.to_string())?;
    let mut explicit_builds = HashSet::new();
    for name in [
        "net/optifine/Config.class",
        "Config.class",
        "optifine/Config.class",
        "optifine/OptiFineVersion.class",
        "META-INF/MANIFEST.MF",
    ] {
        if let Some(data) = read_small_entry(&mut jar, name) {
            let text = String::from_utf8_lossy(&data);
            for build in embedded.find_iter(&text) {
                explicit_builds.insert(build.as_str().to_owned());
            }
        }
    }
    if !explicit_builds.is_empty() && !explicit_builds.contains(expected) {
        return Err(format!(
            "El nombre {file_name} no coincide con la versión declarada dentro del JAR. Conserva su nombre original"
        ));
    }
    let profile_id = crate::optifine_profile_id(&game_version, file_name);
    Ok(crate::OptiFineVersion {
        id: profile_id.clone(),
        profile_id,
        game_version,
        name: expected.replace('_', " "),
        file_name: file_name.into(),
        preview: file_name.starts_with("preview_"),
        legacy_patch: false,
    })
}

fn bounded_children(path: &Path, directories: bool) -> Vec<PathBuf> {
    let mut children = fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .take(MAX_DIRECTORY_ENTRIES)
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|kind| {
                !kind.is_symlink()
                    && if directories {
                        kind.is_dir()
                    } else {
                        kind.is_file()
                    }
            })
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    children.sort();
    if directories {
        children.truncate(MAX_PROFILE_DIRECTORIES);
    }
    children
}

fn search_directories(root: &Path, downloads: Option<PathBuf>) -> Vec<(PathBuf, &'static str)> {
    let mut directories = vec![(root.join("mods"), "mods")];

    if let Some(downloads) = downloads {
        directories.push((downloads, "downloads"));
    }
    for profile in bounded_children(&root.join("versions"), true) {
        directories.push((profile, "versions"));
    }
    for instance in bounded_children(&root.join("instances"), true) {
        directories.push((instance.join("mods"), "instance-mods"));

        for profile in bounded_children(&instance.join("profiles"), true)
            .into_iter()
            .take(128)
        {
            if directories.len() >= 2048 {
                break;
            }
            directories.push((profile.join("mods"), "profile-mods"));
        }
        if directories.len() >= 2048 {
            break;
        }
    }
    directories
}

fn fingerprint(path: &Path, bytes: &[u8]) -> String {
    let mut fingerprint = Sha1::new();
    fingerprint.update(path.to_string_lossy().as_bytes());
    fingerprint.update(Sha1::digest(bytes));
    format!("{:x}", fingerprint.finalize())
}

fn bounded_read(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_JAR_BYTES {
        return Err("El JAR supera 80 MB".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_JAR_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_JAR_BYTES {
        return Err("El JAR cambió de tamaño durante la lectura".into());
    }
    Ok(bytes)
}

fn discover(root: &Path, downloads: Option<PathBuf>) -> Vec<OptiFineCandidate> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let started = Instant::now();
    let mut inspected = 0usize;
    let mut scanned_bytes = 0u64;
    'directories: for (directory, source) in search_directories(root, downloads) {
        if started.elapsed().as_secs() >= 4 {
            break;
        }
        for path in bounded_children(&directory, false) {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !(file_name.starts_with("OptiFine_") || file_name.starts_with("preview_OptiFine_"))
                || !file_name.ends_with(".jar")
            {
                continue;
            }
            let Ok(canonical) = path.canonicalize() else {
                continue;
            };
            if !seen.insert(canonical.clone()) {
                continue;
            }
            let Ok(metadata) = fs::metadata(&canonical) else {
                continue;
            };
            if metadata.len() > MAX_JAR_BYTES {
                continue;
            }
            inspected += 1;
            if inspected > MAX_INSPECTED_FILES || started.elapsed().as_secs() >= 4 {
                break 'directories;
            }
            let modified = metadata.modified().ok();
            let cached = cache.lock().ok().and_then(|cache| {
                cache
                    .get(&canonical)
                    .filter(|entry| entry.size == metadata.len() && entry.modified == modified)
                    .cloned()
            });
            if let Some(cached) = cached {
                if let Some(mut candidate) = cached.candidate {
                    candidate.source = source.into();
                    result.push(candidate);
                }
                continue;
            }
            if scanned_bytes.saturating_add(metadata.len()) > MAX_SCAN_BYTES {
                continue;
            }
            scanned_bytes = scanned_bytes.saturating_add(metadata.len());
            let Ok(bytes) = bounded_read(&canonical) else {
                continue;
            };
            let version = validate(file_name, &bytes).ok();
            let candidate = version.map(|version| OptiFineCandidate {
                id: fingerprint(&canonical, &bytes),
                name: version.name,
                game_version: version.game_version,
                file_name: version.file_name,
                path: canonical.to_string_lossy().into_owned(),
                source: source.into(),
                profile_id: version.profile_id,
                preview: version.preview,
                legacy_patch: version.legacy_patch,
            });
            if let Ok(mut cache) = cache.lock() {
                if cache.len() >= 512 {
                    cache.clear();
                }
                cache.insert(
                    canonical,
                    CacheEntry {
                        size: metadata.len(),
                        modified,
                        candidate: candidate.clone(),
                    },
                );
            }
            if let Some(candidate) = candidate {
                result.push(candidate);
            }
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name).then(a.path.cmp(&b.path)));
    result
}

#[tauri::command]
pub async fn discover_optifine_files() -> Result<Vec<OptiFineCandidate>, String> {
    let root = crate::root_dir()?;
    tokio::task::spawn_blocking(move || discover(&root, dirs::download_dir()))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_discovered_optifine(
    app: AppHandle,
    candidate_id: String,
) -> Result<String, String> {
    let candidate = discover_optifine_files().await?.into_iter().find(|item| item.id == candidate_id)
        .ok_or("El archivo detectado cambió o ya no está disponible. Actualiza la lista o impórtalo manualmente")?;
    let path = PathBuf::from(&candidate.path);
    let bytes = tokio::task::spawn_blocking(move || bounded_read(&path))
        .await
        .map_err(|e| e.to_string())??;
    if fingerprint(Path::new(&candidate.path), &bytes) != candidate_id {
        return Err(
            "El JAR cambió después de detectarlo. Actualiza la lista antes de importarlo".into(),
        );
    }
    validate(&candidate.file_name, &bytes)?;
    crate::import_optifine(
        app,
        candidate.file_name,
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixture(build: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("optifine/OptiFineTweaker.class", options)
            .unwrap();
        zip.write_all(&vec![0; 4096]).unwrap();
        zip.start_file("net/optifine/Config.class", options)
            .unwrap();
        zip.write_all(build.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn discovers_only_matching_original_installer_without_mutating_files() {
        let root =
            std::env::temp_dir().join(format!("tp-optifine-discovery-{}", uuid::Uuid::new_v4()));
        let downloads = root.join("Relocated Downloads");
        fs::create_dir_all(&downloads).unwrap();
        fs::create_dir_all(root.join("mods")).unwrap();
        let filename = "OptiFine_1.8.9_HD_U_M5.jar";
        let data = fixture("OptiFine_1.8.9_HD_U_M5");
        assert!(validate(filename, &data).is_ok());
        fs::write(downloads.join(filename), &data).unwrap();
        fs::write(root.join("mods/OptiFine_1.7.10_HD_U_E7.jar"), &data).unwrap();
        fs::write(downloads.join("not-optifine.jar"), &data).unwrap();
        let candidates = discover(&root, Some(downloads.clone()));
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].game_version, "1.8.9");
        assert_eq!(candidates[0].source, "downloads");
        assert_eq!(fs::read(downloads.join(filename)).unwrap(), data);
        assert!(!root.join("versions").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn candidate_id_changes_if_the_file_changes_and_scan_does_not_recurse_downloads() {
        let root =
            std::env::temp_dir().join(format!("tp-optifine-discovery-{}", uuid::Uuid::new_v4()));
        let downloads = root.join("Downloads");
        fs::create_dir_all(downloads.join("unrelated/nested")).unwrap();
        let filename = "OptiFine_1.8.9_HD_U_M5.jar";
        fs::write(downloads.join(filename), fixture("OptiFine_1.8.9_HD_U_M5")).unwrap();
        fs::write(
            downloads.join("unrelated/nested").join(filename),
            fixture("OptiFine_1.8.9_HD_U_M5"),
        )
        .unwrap();
        let first = discover(&root, Some(downloads.clone()));
        let mut data = fixture("OptiFine_1.8.9_HD_U_M5");
        data.extend_from_slice(b"changed");
        fs::write(downloads.join(filename), data).unwrap();
        let second = discover(&root, Some(downloads));
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_ne!(first[0].id, second[0].id);
        fs::remove_dir_all(root).unwrap();
    }
}
