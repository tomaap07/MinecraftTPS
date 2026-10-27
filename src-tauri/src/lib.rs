use futures_util::{StreamExt, stream};
#[cfg(test)]
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    env, fs,
    io::{BufRead, BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex, OnceLock},
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

mod account_licensing;
mod addon_compatibility;
mod export_policy;
mod game_session;
mod hud_bridge;
#[cfg(test)]
mod hud_compat_smoke;
mod hud_controls;
mod hud_crosshair;
mod hud_ping;
mod hud_preview;
mod hud_transport;
#[cfg(windows)]
mod hud_windows;
mod launcher_display;
mod launcher_overlay;
mod launcher_plugins;
mod launcher_settings;
mod launcher_social;
mod launcher_tools;
mod loader_catalogs;
mod managed_addons;
mod microsoft_oauth;
mod mod_studio;
mod optifine_discovery;
mod poser;
mod poser_assets;
mod private_integrations;
#[cfg(test)]
mod release_client_smoke;
mod server_directory;
mod server_extras;
mod servers;
mod skin_library;
mod studio_progress;

#[derive(Serialize)]
struct JavaInfo {
    installed: bool,
    major: u8,
    version: String,
    path: String,
    is_portable: bool,
}
#[derive(Clone, Serialize)]
struct ProgressPayload {
    task: String,
    percent: u8,
    status: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct LauncherLogPayload {
    timestamp: u64,
    stream: String,
    line: String,
}
#[derive(Serialize)]
struct VersionItem {
    id: String,
    url: String,
    kind: String,
}

#[derive(Clone, Serialize)]
struct FabricGameVersion {
    id: String,
    stable: bool,
}

#[derive(Clone, Serialize)]
struct ForgeGameVersion {
    id: String,
    forge_version: String,
    channel: String,
}

#[derive(Clone, Serialize)]
struct OptiFineVersion {
    id: String,
    profile_id: String,
    game_version: String,
    name: String,
    file_name: String,
    preview: bool,
    legacy_patch: bool,
}

#[cfg(test)]
mod version_catalog_contract_tests {
    use super::{
        VersionItem, legacy_optifine_catalog, local_optifine_version, merge_legacy_optifine_patch,
        parse_optifine_catalog,
    };
    use std::io::{Cursor, Read, Write};

    fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, bytes) in entries {
            archive
                .start_file(*name, options)
                .expect("entry should start");
            archive.write_all(bytes).expect("entry should write");
        }
        archive
            .finish()
            .expect("archive should finish")
            .into_inner()
    }
    #[test]
    fn optifine_local_import_validates_file_identity_without_a_download() {
        let contents = (0..512)
            .flat_map(|_| *uuid::Uuid::new_v4().as_bytes())
            .collect::<Vec<_>>();
        let jar = make_zip(&[("optifine/OptiFineTweaker.class", &contents)]);
        let version = local_optifine_version("OptiFine_1.8.9_HD_U_M5.jar", &jar).unwrap();
        assert_eq!(version.game_version, "1.8.9");
        assert_eq!(version.profile_id, "1.8.9-OptiFine-M5");
        for name in [
            "../../OptiFine_1.8.9_HD_U_M5.jar",
            "some-mod.jar",
            "OptiFine_1.8.9_HD_U_M5.zip",
        ] {
            assert!(local_optifine_version(name, &jar).is_err());
        }
        let unrelated = make_zip(&[("unrelated/Plugin.class", &contents)]);
        assert!(local_optifine_version("OptiFine_1.8.9_HD_U_M5.jar", &unrelated).is_err());
    }

    #[test]
    fn version_catalog_exposes_kind_field_expected_by_frontend() {
        let value = serde_json::to_value(VersionItem {
            id: "1.21.9".into(),
            url: "https://example.invalid/1.21.9.json".into(),
            kind: "release".into(),
        })
        .expect("version catalog item should serialize");

        assert_eq!(
            value.get("kind").and_then(serde_json::Value::as_str),
            Some("release")
        );
        assert!(value.get("type").is_none());
    }

    #[test]
    fn optifine_catalog_reads_official_mirror_rows() {
        let html = r#"
            <tr class='downloadLine downloadLineMain'>
              <td class='colFile'>OptiFine HD U M5</td>
              <td class='colMirror'><a href="http://optifine.net/adloadx?f=OptiFine_1.8.9_HD_U_M5.jar">(Mirror)</a></td>
            </tr>
            <tr class="downloadLine downloadLinePreview">
              <td class="colFile">OptiFine HD U M6 pre1</td>
              <td class="colMirror"><a href='http://optifine.net/adloadx?f=preview_OptiFine_1.8.9_HD_U_M6_pre1.jar'>(Mirror)</a></td>
            </tr>
        "#;
        let versions = parse_optifine_catalog(html).expect("official rows should parse");

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].game_version, "1.8.9");
        assert_eq!(versions[0].name, "OptiFine HD U M5");
        assert_eq!(versions[0].file_name, "OptiFine_1.8.9_HD_U_M5.jar");
        assert_eq!(versions[0].profile_id, "1.8.9-OptiFine-M5");
        assert!(versions[1].preview);
        assert_eq!(
            versions[1].file_name,
            "preview_OptiFine_1.8.9_HD_U_M6_pre1.jar"
        );
    }

    #[test]
    fn historical_optifine_catalog_contains_legacy_installers_and_patch_zips() {
        let versions = legacy_optifine_catalog();
        assert!(versions.iter().any(|version| {
            version.game_version == "1.6.4"
                && version.file_name == "OptiFine_1.6.4_HD_U_D1.jar"
                && !version.legacy_patch
        }));
        assert!(versions.iter().any(|version| {
            version.game_version == "1.5.2"
                && version.file_name == "OptiFine_1.5.2_HD_U_D5.zip"
                && version.legacy_patch
        }));
        assert!(
            versions
                .iter()
                .any(|version| version.game_version == "b1.7.3")
        );
    }

    #[test]
    fn historical_optifine_patch_overlays_classes_and_removes_vanilla_signatures() {
        let base = make_zip(&[
            ("net/minecraft/Client.class", b"vanilla"),
            ("META-INF/MOJANG.SF", b"signature"),
        ]);
        let patch = make_zip(&[
            ("net/minecraft/Client.class", b"optifine"),
            ("optifine.cfg", b"options"),
        ]);
        let merged = merge_legacy_optifine_patch(&base, &patch).expect("patch should merge");
        let mut archive =
            zip::ZipArchive::new(Cursor::new(merged)).expect("merged JAR should open");
        let mut client = String::new();
        archive
            .by_name("net/minecraft/Client.class")
            .unwrap()
            .read_to_string(&mut client)
            .unwrap();
        assert_eq!(client, "optifine");
        assert!(archive.by_name("optifine.cfg").is_ok());
        assert!(archive.by_name("META-INF/MOJANG.SF").is_err());
    }
}

#[derive(Serialize)]
struct InstalledVersion {
    id: String,
    jar: bool,
    json: bool,
    size_mb: f64,
    loader: String,
    game_version: String,
}
#[derive(Serialize)]
struct RuntimeStatus {
    major: u8,
    installed: bool,
    enabled: bool,
    path: String,
}
#[derive(Serialize)]
struct ModrinthProject {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    icon_url: String,
    downloads: u64,
}
#[derive(Serialize)]
struct ModrinthSearchPage {
    projects: Vec<ModrinthProject>,
    total_hits: u64,
    page: u32,
    page_size: u32,
}
#[derive(Clone, Serialize)]
struct MinecraftProfile {
    id: String,
    name: String,
    skins: Vec<serde_json::Value>,
    capes: Vec<serde_json::Value>,
}
#[derive(Clone)]
struct MicrosoftSession {
    profile: MinecraftProfile,
    access_token: String,
    refresh_token: String,
    xuid: String,
    expires_at: u64,
}
#[derive(Default)]
struct AuthState {
    session: Mutex<Option<MicrosoftSession>>,
    refresh_lock: tokio::sync::Mutex<()>,
}

#[cfg(windows)]
mod token_storage {
    use super::{fs, root_dir};
    use std::ptr::null_mut;

    #[repr(C)]
    struct DataBlob {
        size: u32,
        data: *mut u8,
    }

    #[link(name = "Crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            input: *const DataBlob,
            description: *const u16,
            entropy: *const DataBlob,
            reserved: *mut std::ffi::c_void,
            prompt: *mut std::ffi::c_void,
            flags: u32,
            output: *mut DataBlob,
        ) -> i32;
        fn CryptUnprotectData(
            input: *const DataBlob,
            description: *mut *mut u16,
            entropy: *const DataBlob,
            reserved: *mut std::ffi::c_void,
            prompt: *mut std::ffi::c_void,
            flags: u32,
            output: *mut DataBlob,
        ) -> i32;
    }
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn LocalFree(memory: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GetLastError() -> u32;
    }

    pub(crate) fn protect(input: &[u8]) -> Result<Vec<u8>, String> {
        let input_blob = DataBlob {
            size: input.len() as u32,
            data: input.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            size: 0,
            data: null_mut(),
        };
        let ok = unsafe {
            CryptProtectData(
                &input_blob,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0x1,
                &mut output,
            )
        };
        if ok == 0 {
            return Err(format!(
                "Windows no pudo proteger la sesión ({}).",
                unsafe { GetLastError() }
            ));
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }.to_vec();
        unsafe { LocalFree(output.data.cast()) };
        Ok(bytes)
    }

    pub(crate) fn unprotect(input: &[u8]) -> Result<Vec<u8>, String> {
        let input_blob = DataBlob {
            size: input.len() as u32,
            data: input.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            size: 0,
            data: null_mut(),
        };
        let ok = unsafe {
            CryptUnprotectData(
                &input_blob,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0x1,
                &mut output,
            )
        };
        if ok == 0 {
            return Err(format!(
                "Windows no pudo recuperar la sesión ({}).",
                unsafe { GetLastError() }
            ));
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }.to_vec();
        unsafe { LocalFree(output.data.cast()) };
        Ok(bytes)
    }

    pub fn save_refresh_token(token: &str) -> Result<(), String> {
        let encrypted = protect(token.as_bytes())?;
        let path = root_dir()?.join("account.dat");
        let staged = root_dir()?.join("account.dat.new");
        fs::write(&staged, encrypted).map_err(|e| format!("No se pudo guardar la sesión: {e}"))?;
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("No se pudo actualizar la sesión: {e}"))?;
        }
        fs::rename(staged, path).map_err(|e| format!("No se pudo confirmar la sesión: {e}"))
    }

    pub fn load_refresh_token() -> Result<Option<String>, String> {
        let path = root_dir()?.join("account.dat");
        if !path.exists() {
            return Ok(None);
        }
        let encrypted = fs::read(path).map_err(|e| format!("No se pudo leer la sesión: {e}"))?;
        let clear = unprotect(&encrypted)?;
        String::from_utf8(clear)
            .map(Some)
            .map_err(|e| format!("La sesión guardada no es válida: {e}"))
    }

    pub fn clear_saved_refresh_token() -> Result<(), String> {
        let path = root_dir()?.join("account.dat");
        if path.exists() {
            fs::remove_file(path).map_err(|e| format!("No se pudo borrar la sesión: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod token_storage {
    pub(crate) fn protect(_: &[u8]) -> Result<Vec<u8>, String> {
        Err("El almacenamiento protegido solo está disponible en Windows".into())
    }
    pub(crate) fn unprotect(_: &[u8]) -> Result<Vec<u8>, String> {
        Err("El almacenamiento protegido solo está disponible en Windows".into())
    }
    pub fn save_refresh_token(_: &str) -> Result<(), String> {
        Err("El almacenamiento seguro de sesiones solo está habilitado en Windows.".into())
    }
    pub fn load_refresh_token() -> Result<Option<String>, String> {
        Ok(None)
    }
    pub fn clear_saved_refresh_token() -> Result<(), String> {
        Ok(())
    }
}
#[derive(serde::Deserialize)]
struct VersionManifest {
    versions: Vec<ManifestVersion>,
}
#[derive(serde::Deserialize)]
struct ManifestVersion {
    id: String,
    url: String,
    #[serde(rename = "type")]
    kind: String,
}

async fn load_version_manifest() -> Result<VersionManifest, String> {
    const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    let cache_path = root_dir()?.join("version_manifest_v2.json");
    let downloaded = match reqwest::get(MANIFEST_URL).await {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response
                .bytes()
                .await
                .map(|bytes| bytes.to_vec())
                .map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        },
        Err(error) => Err(error.to_string()),
    };
    let (bytes, downloaded_online) = match downloaded {
        Ok(bytes) => (bytes, true),
        Err(network_error) => (
            fs::read(&cache_path).map_err(|cache_error| {
                format!(
                    "No se pudo consultar el catálogo de Mojang ({network_error}) y no existe una copia local ({cache_error})"
                )
            })?,
            false,
        ),
    };
    let manifest = match serde_json::from_slice::<VersionManifest>(&bytes) {
        Ok(manifest) => manifest,
        Err(error) if downloaded_online => {
            let cached = fs::read(&cache_path).map_err(|cache_error| {
                format!("El manifiesto descargado está dañado ({error}) y no se pudo leer la copia local ({cache_error})")
            })?;
            serde_json::from_slice(&cached).map_err(|cache_error| {
                format!("El manifiesto descargado y la copia local están dañados: {error}; {cache_error}")
            })?
        }
        Err(error) => {
            return Err(format!(
                "El manifiesto local de versiones de Minecraft está dañado: {error}"
            ));
        }
    };
    if downloaded_online {
        if let Err(error) = fs::write(&cache_path, &bytes) {
            eprintln!("No se pudo actualizar el manifiesto local: {error}");
        }
    }
    Ok(manifest)
}

fn root_dir() -> Result<PathBuf, String> {
    let root = launcher_settings::data_root()?;
    for name in [
        "runtime",
        "versions",
        "libraries",
        "assets",
        "mods",
        "instances",
        "logs",
    ] {
        fs::create_dir_all(root.join(name))
            .map_err(|e| format!("No se pudo crear {}: {e}", name))?;
    }
    Ok(root)
}
fn runtime_enabled(major: u8) -> bool {
    let Ok(root) = root_dir() else {
        return true;
    };
    let path = root.join("runtime").join("enabled.json");
    fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .and_then(|v| v[major.to_string()].as_bool())
        .unwrap_or(true)
}
fn normalized_icon(bytes: &[u8]) -> Result<image::RgbaImage, String> {
    let decoded = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let scaled = decoded.thumbnail(128, 128).to_rgba8();
    let mut canvas = image::RgbaImage::from_pixel(128, 128, image::Rgba([0, 0, 0, 0]));
    image::imageops::overlay(
        &mut canvas,
        &scaled,
        (128 - scaled.width()) as i64 / 2,
        (128 - scaled.height()) as i64 / 2,
    );
    Ok(canvas)
}
fn java_binary() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}
fn parse_java_major(text: &str) -> Option<u8> {
    let version = text
        .lines()
        .find_map(|line| line.split_once("java.specification.version"))
        .and_then(|(_, value)| value.split_once('=').map(|(_, v)| v.trim()));
    let version = version.or_else(|| {
        text.split('"')
            .find(|part| part.chars().next().is_some_and(|c| c.is_ascii_digit()))
    })?;
    let mut parts = version.split('.');
    let first = parts.next()?.parse::<u8>().ok()?;
    if first == 1 {
        parts.next()?.parse::<u8>().ok()
    } else {
        Some(first)
    }
}
fn unsupported_java_message(details: &str, running_java: u8, version_id: &str) -> Option<String> {
    if !details.contains("UnsupportedClassVersionError") && !details.contains("class file version")
    {
        return None;
    }
    let required = details
        .split("class file version")
        .nth(1)
        .and_then(|suffix| suffix.trim_start().split('.').next())
        .and_then(|number| number.trim().parse::<u16>().ok())
        .and_then(|class_version| class_version.checked_sub(44))
        .map(|major| major.to_string());
    Some(match required {
        Some(required) => format!(
            "Minecraft {version_id} requiere Java {required} (class file version {}), pero se inició con Java {running_java}. Instala o activa Java {required} en Ajustes → Gestor de Java.",
            details
                .split("class file version")
                .nth(1)
                .unwrap_or("")
                .trim_start()
                .split_whitespace()
                .next()
                .unwrap_or("desconocida")
        ),
        None => format!(
            "Minecraft {version_id} requiere un Java más reciente que Java {running_java}. Instala o activa el runtime compatible en Ajustes → Gestor de Java."
        ),
    })
}

fn diagnose_game_exit(version_id: &str, exit_code: Option<i32>, details: &str) -> String {
    let lower = details.to_ascii_lowercase();
    let (cause, solution) = if lower.contains("processing of -javaagent failed")
        || lower.contains("addtosystemclassloadersearch")
        || lower.contains("appendtoclasspathforinstrumentation")
    {
        (
            "Java rechazó un agente antes de iniciar Minecraft.",
            "Actualiza el launcher: TP Bridge debe recibir una ruta Windows sin el prefijo \\\\?\\ y pasar su comprobación previa. Si añadiste otros agentes Java, revisa su compatibilidad y ruta.",
        )
    } else if lower.contains("outofmemoryerror") {
        (
            "Java agotó la memoria disponible.",
            "Cierra otras aplicaciones o ajusta la memoria asignada en Inicio; no la subas por encima de la RAM libre del sistema.",
        )
    } else if lower.contains("resolutionexception") && lower.contains("export package") {
        (
            "El motor encontró módulos Java duplicados en el perfil.",
            "Reinstala el perfil del loader y comprueba que no se mezclen bibliotecas de otra versión. El cliente base debe estar excluido de la detección de módulos de Forge/NeoForge.",
        )
    } else if lower.contains("incompatible mods found")
        || lower.contains("missing mandatory dependencies")
        || lower.contains("modloadingexception")
    {
        (
            "Un mod o una dependencia no es compatible con esta instalación.",
            "Revisa el nombre y la versión que indica el registro. Usa los archivos para el Minecraft y el loader exactos del modpack; reinstala el pack si modificaste su selección original.",
        )
    } else if lower.contains("noclassdeffounderror") || lower.contains("classnotfoundexception") {
        (
            "Falta una clase de Minecraft o de una biblioteca del perfil.",
            "Vuelve a iniciar la instalación para que el launcher valide y descargue los archivos requeridos; confirma también que el loader coincida con los mods.",
        )
    } else if lower.contains("unsatisfiedlinkerror")
        || lower.contains("no lwjgl in java.library.path")
        || lower.contains("failed to load lwjgl")
        || lower.contains("could not initialize opengl")
    {
        (
            "Java no pudo cargar una biblioteca nativa o gráfica.",
            "Repara la instalación para volver a descargar los natives del sistema y actualiza el controlador de la tarjeta gráfica.",
        )
    } else if lower.contains("nullpointerexception")
        && (lower.contains("inputstreamreader") || lower.contains("java.io.reader"))
    {
        (
            "El cliente no encontró un recurso que esperaba al iniciar; en versiones antiguas suele ser un archivo de recursos incompleto.",
            "Vuelve a iniciar para reparar los assets y comprueba que la carpeta resources del perfil esté completa.",
        )
    } else {
        (
            "Minecraft se cerró inesperadamente.",
            "Abre la consola y el último crash-report para revisar la causa concreta; verifica Java recomendado, mods y archivos del perfil.",
        )
    };
    let excerpt = details
        .lines()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "[DIAGNÓSTICO TPS - CÓDIGO DE SALIDA: {}] Minecraft {version_id}: {cause}\nSolución sugerida: {solution}{}",
        exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "desconocido".into()),
        if excerpt.is_empty() {
            String::new()
        } else {
            format!("\nDetalle técnico reciente:\n{excerpt}")
        }
    )
}
fn inspect_java(path: &Path, portable: bool) -> Option<JavaInfo> {
    let output = Command::new(path)
        .args(["-XshowSettings:properties", "-version"])
        .output()
        .ok()?;
    let details = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let major = parse_java_major(&details)?;
    let text = details
        .lines()
        .find(|line| {
            let line = line.to_ascii_lowercase();
            line.contains("version") && (line.contains("openjdk") || line.contains("java version"))
        })
        .or_else(|| details.lines().find(|line| !line.trim().is_empty()))
        .unwrap_or("Java detectado")
        .to_string();
    Some(JavaInfo {
        installed: output.status.success(),
        major,
        version: text,
        path: path.to_string_lossy().into_owned(),
        is_portable: portable,
    })
}
#[derive(Clone)]
struct JavaInstallation {
    major: u8,
    path: PathBuf,
    portable: bool,
}
fn add_java_candidate(candidates: &mut Vec<JavaInstallation>, path: PathBuf, portable: bool) {
    if !path.is_file() {
        return;
    }
    let Some(info) = inspect_java(&path, portable) else {
        return;
    };
    if info.major < 8 {
        return;
    }
    let normalized = fs::canonicalize(&path).unwrap_or(path);
    if candidates.iter().any(|item| item.path == normalized) {
        return;
    }
    candidates.push(JavaInstallation {
        major: info.major,
        path: normalized,
        portable,
    });
}
fn discover_java_installations() -> Vec<JavaInstallation> {
    let mut candidates = Vec::new();
    let binary = java_binary();
    for variable in ["JAVA_HOME", "JDK_HOME"] {
        if let Some(home) = env::var_os(variable) {
            add_java_candidate(
                &mut candidates,
                PathBuf::from(home).join("bin").join(binary),
                false,
            );
        }
    }
    if let Some(path) = env::var_os("PATH") {
        for directory in env::split_paths(&path) {
            add_java_candidate(&mut candidates, directory.join(binary), false);
        }
    }
    let root = root_dir().ok();
    if let Some(root) = &root {
        if let Ok(entries) = fs::read_dir(root.join("runtime")) {
            for entry in entries.filter_map(Result::ok) {
                add_java_candidate(&mut candidates, entry.path().join("bin").join(binary), true);
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        for program_files in [
            env::var_os("ProgramFiles"),
            env::var_os("ProgramFiles(x86)"),
        ]
        .into_iter()
        .flatten()
        {
            let base = PathBuf::from(program_files);
            for vendor_dir in [
                "Java",
                "Eclipse Adoptium",
                "Microsoft",
                "Amazon Corretto",
                "Zulu",
                "BellSoft",
            ] {
                let vendor_root = base.join(vendor_dir);
                if let Ok(entries) = fs::read_dir(vendor_root) {
                    for entry in entries.filter_map(Result::ok) {
                        add_java_candidate(
                            &mut candidates,
                            entry.path().join("bin").join(binary),
                            false,
                        );
                    }
                }
            }
        }
    }
    candidates.sort_by_key(|item| (item.major, !item.portable));
    candidates
}
#[tauri::command]
fn check_java() -> Result<JavaInfo, String> {
    discover_java_installations()
        .into_iter()
        .max_by_key(|candidate| candidate.major)
        .and_then(|candidate| inspect_java(&candidate.path, candidate.portable))
        .ok_or_else(|| "No se detectó Java instalado en el sistema.".to_string())
}
fn emit_progress(app: &AppHandle, task: &str, percent: u8, status: &str) {
    emit_launcher_log(app, "launcher", &format!("{task}: {status} ({percent}%)"));
    let _ = app.emit(
        "download-progress",
        ProgressPayload {
            task: task.into(),
            percent,
            status: status.into(),
        },
    );
}
fn emit_launcher_log(app: &AppHandle, stream: &str, line: &str) {
    let payload = LauncherLogPayload {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        stream: stream.to_owned(),
        line: line.to_owned(),
    };
    persist_launcher_log(&payload);
    let _ = app.emit("launcher-log", payload);
}

fn launcher_log_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn persist_launcher_log(payload: &LauncherLogPayload) {
    let Ok(_guard) = launcher_log_lock().lock() else {
        return;
    };
    let Ok(root) = root_dir() else { return };
    let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("logs").join("launcher.log"))
    else {
        return;
    };
    if serde_json::to_writer(&mut file, payload).is_ok() {
        let _ = file.write_all(b"\n");
    }
}

#[tauri::command]
fn read_launcher_logs() -> Result<Vec<LauncherLogPayload>, String> {
    let _guard = launcher_log_lock()
        .lock()
        .map_err(|_| "Registro del launcher bloqueado")?;
    let path = root_dir()?.join("logs").join("launcher.log");
    let Ok(file) = fs::File::open(path) else {
        return Ok(Vec::new());
    };
    let mut recent = VecDeque::with_capacity(1_500);
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Ok(entry) = serde_json::from_str::<LauncherLogPayload>(&line) {
            recent.push_back(entry);
            if recent.len() > 1_500 {
                recent.pop_front();
            }
        }
    }
    Ok(recent.into_iter().collect())
}

#[tauri::command]
fn clear_launcher_logs() -> Result<(), String> {
    let _guard = launcher_log_lock()
        .lock()
        .map_err(|_| "Registro del launcher bloqueado")?;
    fs::write(root_dir()?.join("logs").join("launcher.log"), [])
        .map_err(|e| format!("No se pudo limpiar el registro: {e}"))
}

#[tauri::command]
fn record_launcher_log(app: AppHandle, stream: String, line: String) {
    emit_launcher_log(&app, &stream, &line);
}
fn capture_process_output<R: Read + Send + 'static>(
    reader: R,
    app: AppHandle,
    stream: &'static str,
    recent_errors: Option<Arc<Mutex<VecDeque<String>>>>,
    game_dir: PathBuf,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            servers::observe_game_log(&game_dir, &line);
            emit_launcher_log(&app, stream, &line);
            if let Some(recent_errors) = &recent_errors {
                if let Ok(mut lines) = recent_errors.lock() {
                    lines.push_back(line);
                    while lines.len() > 100 {
                        lines.pop_front();
                    }
                }
            }
        }
    })
}

fn latest_crash_report_since(
    instance: &Path,
    launched_at: std::time::SystemTime,
) -> Option<String> {
    let reports = instance.join("crash-reports");
    let (_modified, path) = fs::read_dir(reports)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| {
            let metadata = fs::metadata(&path).ok()?;
            let modified = metadata.modified().ok()?;
            (path.is_file() && modified >= launched_at).then_some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)?;
    let report = fs::read_to_string(&path).ok()?;
    let excerpt = report.chars().take(20_000).collect::<String>();
    let truncated = report.chars().count() > 20_000;
    Some(format!(
        "Crash report {}:\n{}{}",
        path.display(),
        excerpt,
        if truncated {
            "\n[Informe truncado a 20.000 caracteres]"
        } else {
            ""
        }
    ))
}

#[tauri::command]
async fn install_portable_java(app: AppHandle, major: u8) -> Result<String, String> {
    if !(8..=40).contains(&major) {
        return Err("La versión de Java solicitada está fuera del rango admitido (8–40).".into());
    }
    let root = root_dir()?;
    #[cfg(not(target_os = "windows"))]
    return Err(
        "La instalación automática configurada actualmente solo está disponible para Windows x64."
            .into(),
    );
    #[cfg(target_os = "windows")]
    {
        let url = format!(
            "https://api.adoptium.net/v3/binary/latest/{major}/ga/windows/x64/jre/hotspot/normal/eclipse"
        );
        let task = format!("Descargando Java {major} Portable");
        emit_progress(&app, &task, 0, "Conectando con Eclipse Temurin…");
        let response = reqwest::get(url)
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let total = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            bytes.extend_from_slice(&chunk);
            let pct = if total > 0 {
                ((bytes.len() as u64 * 75 / total).min(75)) as u8
            } else {
                10
            };
            emit_progress(&app, &task, pct, "Descargando JRE…");
        }
        emit_progress(&app, &task, 78, "Extrayendo runtime…");
        let target = root.join("runtime").join(format!("jdk-{major}"));
        let staging = root.join("runtime").join(format!("jdk-{major}-installing"));
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
        let mut archive =
            zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("ZIP inválido: {e}"))?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let Some(name) = file.enclosed_name() else {
                continue;
            };
            let dest = staging.join(name.components().skip(1).collect::<PathBuf>());
            if file.is_dir() {
                fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut out = fs::File::create(&dest).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
            }
        }
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
        }
        fs::rename(&staging, &target).map_err(|e| e.to_string())?;
        emit_progress(&app, &task, 100, &format!("Java {major} instalado"));
        Ok(format!("Java {major} Portable instalado correctamente."))
    }
}

#[tauri::command]
fn list_java_runtimes() -> Result<Vec<RuntimeStatus>, String> {
    let mut list: Vec<_> = discover_java_installations()
        .into_iter()
        .map(|java| RuntimeStatus {
            major: java.major,
            installed: true,
            enabled: runtime_enabled(java.major),
            path: java.path.to_string_lossy().into_owned(),
        })
        .collect();
    for major in [8u8, 17, 21, 22, 25] {
        if !list.iter().any(|runtime| runtime.major == major) {
            list.push(RuntimeStatus {
                major,
                installed: false,
                enabled: runtime_enabled(major),
                path: root_dir()?
                    .join("runtime")
                    .join(format!("jdk-{major}"))
                    .join("bin")
                    .join(java_binary())
                    .to_string_lossy()
                    .into_owned(),
            });
        }
    }
    list.sort_by_key(|runtime| (runtime.major, runtime.installed));
    Ok(list)
}
#[tauri::command]
fn set_java_runtime_enabled(major: u8, enabled: bool) -> Result<(), String> {
    if !(8..=40).contains(&major) {
        return Err("Versión de Java no válida".into());
    }
    let root = root_dir()?;
    let path = root.join("runtime").join("enabled.json");
    let mut values = fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    values[major.to_string()] = serde_json::Value::Bool(enabled);
    fs::write(
        path,
        serde_json::to_vec_pretty(&values).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let p = entry.path();
            if p.is_dir() {
                directory_size(&p)
            } else {
                entry.metadata().map(|m| m.len()).unwrap_or(0)
            }
        })
        .sum()
}
fn launcher_os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}
fn launcher_rule_matches(rule: &serde_json::Value) -> bool {
    let os = &rule["os"];
    if let Some(name) = os["name"].as_str() {
        if name != launcher_os_name() {
            return false;
        }
    }
    if let Some(arch) = os["arch"].as_str() {
        let current = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86"
        };
        if arch != current && !(arch == "x86" && current == "x86_64") {
            return false;
        }
    }
    if let Some(features) = rule["features"].as_object() {
        for (name, required) in features {
            let available = match name.as_str() {
                "is_demo_user" => false,
                "has_custom_resolution" | "has_quick_plays_support" => true,
                "is_quick_play_singleplayer"
                | "is_quick_play_multiplayer"
                | "is_quick_play_realms" => false,
                _ => false,
            };
            if required.as_bool() != Some(available) {
                return false;
            }
        }
    }
    true
}
fn launcher_rules_allow(entry: &serde_json::Value) -> bool {
    let Some(rules) = entry["rules"].as_array() else {
        return true;
    };
    if rules.is_empty() {
        return true;
    }
    let mut allowed = !rules
        .iter()
        .any(|rule| rule["action"].as_str() == Some("allow"));
    for rule in rules {
        if launcher_rule_matches(rule) {
            allowed = rule["action"].as_str() == Some("allow");
        }
    }
    allowed
}
fn substitute_launch_arg(argument: &str, replacements: &[(String, String)]) -> String {
    replacements
        .iter()
        .fold(argument.to_owned(), |value, (key, replacement)| {
            value.replace(key, replacement)
        })
}
fn append_launch_arguments(
    entries: &serde_json::Value,
    replacements: &[(String, String)],
    output: &mut Vec<String>,
) {
    let Some(entries) = entries.as_array() else {
        return;
    };
    for entry in entries {
        match entry {
            serde_json::Value::String(argument) => {
                output.push(substitute_launch_arg(argument, replacements));
            }
            serde_json::Value::Object(_) if launcher_rules_allow(entry) => match &entry["value"] {
                serde_json::Value::String(argument) => {
                    output.push(substitute_launch_arg(argument, replacements));
                }
                serde_json::Value::Array(values) => {
                    for argument in values.iter().filter_map(|v| v.as_str()) {
                        output.push(substitute_launch_arg(argument, replacements));
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
#[tauri::command]
fn list_installed_versions() -> Result<Vec<InstalledVersion>, String> {
    let root = root_dir()?.join("versions");
    let mut result = Vec::new();
    for entry in fs::read_dir(&root)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
    {
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        let metadata_path = entry.path().join(format!("{id}.json"));
        let version_meta = fs::read(&metadata_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .unwrap_or_default();
        let base_version = version_meta["inheritsFrom"]
            .as_str()
            .or_else(|| version_meta["jar"].as_str())
            .unwrap_or(&id);
        let jar = entry.path().join(format!("{id}.jar")).exists()
            || root
                .join(base_version)
                .join(format!("{base_version}.jar"))
                .exists();
        let json = metadata_path.exists();
        let normalized_id = id.to_ascii_lowercase();
        let loader = if normalized_id.starts_with("neoforge-")
            || normalized_id.contains("-neoforge-")
            || version_meta["libraries"]
                .as_array()
                .is_some_and(|libraries| {
                    libraries.iter().any(|library| {
                        library["name"].as_str().is_some_and(|name| {
                            name.starts_with("net.neoforged:")
                                || name.starts_with("net.neoforged.fancymodloader:")
                        })
                    })
                }) {
            "neoforge"
        } else if normalized_id.starts_with("quilt-loader-") {
            "quilt"
        } else if normalized_id.starts_with("fabric-loader-") {
            "fabric"
        } else if normalized_id.starts_with("forge-") || normalized_id.contains("-forge-") {
            "forge"
        } else if normalized_id.starts_with("optifine-") || normalized_id.contains("-optifine-") {
            "optifine"
        } else if base_version != id {
            "custom"
        } else {
            "vanilla"
        }
        .to_owned();
        let game_version = base_version.to_owned();
        let size_mb = directory_size(&entry.path()) as f64 / 1_048_576.0;
        result.push(InstalledVersion {
            id,
            jar,
            json,
            size_mb,
            loader,
            game_version,
        });
    }
    result.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(result)
}
#[tauri::command]
fn open_version_folder(app: AppHandle, version: String) -> Result<(), String> {
    let path = root_dir()?.join("versions").join(&version);
    if !path.starts_with(root_dir()?.join("versions")) || !path.is_dir() {
        return Err("No se encontró esa instalación".into());
    }
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn delete_installed_version(version: String) -> Result<(), String> {
    if version.is_empty() || version.contains(['/', '\\', ':']) || version == "." || version == ".."
    {
        return Err("Identificador de versión no válido".into());
    }
    let root = root_dir()?.join("versions");
    let path = root.join(version);
    if !path.starts_with(&root) || !path.is_dir() {
        return Err("No se encontró esa instalación".into());
    }
    fs::remove_dir_all(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_custom_app_icon(app: AppHandle, image_base64: String) -> Result<(), String> {
    let encoded = image_base64
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&image_base64);
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|e| format!("Imagen inválida: {e}"))?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("El icono no puede superar 8 MB".into());
    }
    let canvas = normalized_icon(&bytes).map_err(|e| format!("No se pudo leer la imagen: {e}"))?;
    let mut normalized = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(canvas.clone())
        .write_to(&mut normalized, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let path = root_dir()?.join("custom_icon.png");
    fs::write(&path, normalized.into_inner()).map_err(|e| e.to_string())?;
    let window = app
        .get_webview_window("main")
        .ok_or("Ventana principal no encontrada")?;
    window
        .set_icon(tauri::image::Image::new_owned(canvas.into_raw(), 128, 128))
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn remove_custom_app_icon(app: AppHandle) -> Result<(), String> {
    let path = root_dir()?.join("custom_icon.png");
    let window = app
        .get_webview_window("main")
        .ok_or("Ventana principal no encontrada")?;
    let default_icon = app
        .default_window_icon()
        .ok_or("Icono predeterminado no disponible")?;

    window
        .set_icon(default_icon.clone())
        .map_err(|e| e.to_string())?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("No se pudo quitar el logo personalizado: {error}")),
    }
}

#[tauri::command]
fn get_custom_app_icon() -> Result<Option<String>, String> {
    let path = root_dir()?.join("custom_icon.png");
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(Some(format!(
        "data:image/png;base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
    )))
}

async fn minecraft_profile(
    client: &reqwest::Client,
    token: &str,
) -> Result<MinecraftProfile, String> {
    let response = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = response.status();
    let body: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(body["errorMessage"]
            .as_str()
            .unwrap_or("La cuenta no tiene un perfil Java de Minecraft o el token expiró.")
            .to_owned());
    }
    Ok(MinecraftProfile {
        id: body["id"].as_str().ok_or("Perfil sin UUID")?.to_owned(),
        name: body["name"].as_str().ok_or("Perfil sin nombre")?.to_owned(),
        skins: body["skins"].as_array().cloned().unwrap_or_default(),
        capes: body["capes"].as_array().cloned().unwrap_or_default(),
    })
}

async fn minecraft_session_from_microsoft_token(
    client: &reqwest::Client,
    microsoft_token: &str,
) -> Result<(MinecraftProfile, String, String), String> {
    let tickets = [

        format!("d={microsoft_token}"),
        microsoft_token.to_owned(),
        format!("t={microsoft_token}"),
    ];
    let mut xbl = None;
    for ticket in tickets {
        let response = client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "application/json")
            .header("x-xbl-contract-version", "1")
            .json(&serde_json::json!({
                "Properties": {
                    "AuthMethod": "RPS",
                    "SiteName": "user.auth.xboxlive.com",
                    "RpsTicket": ticket
                },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if matches!(
            response.status(),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::BAD_REQUEST
        ) {
            continue;
        }
        if !response.status().is_success() {
            let status = response.status();
            return Err(format!(
                "Xbox Live rechazó la cuenta (HTTP {}). Revisa el acceso Xbox de tu propia aplicación y las restricciones de la cuenta. La respuesta privada no se registra.",
                status.as_u16()
            ));
        }
        xbl = Some(
            response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| e.to_string())?,
        );
        break;
    }
    let xbl = xbl.ok_or("Xbox Live rechazó la solicitud con los formatos d=, token puro y t=. Revisa los permisos de Xbox de la aplicación Microsoft propia.")?;
    let xbl_token = xbl["Token"].as_str().ok_or("Xbox Live no devolvió token")?;
    let user_hash = xbl
        .pointer("/DisplayClaims/xui/0/uhs")
        .and_then(|value| value.as_str())
        .ok_or("Xbox Live no devolvió user hash")?;
    let xuid = xbl
        .pointer("/DisplayClaims/xui/0/xid")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();
    let xsts: serde_json::Value = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&serde_json::json!({"Properties":{"SandboxId":"RETAIL","UserTokens":[xbl_token]},"RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"}))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Xbox XSTS rechazó la cuenta: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let xsts_token = xsts["Token"]
        .as_str()
        .ok_or("Xbox no devolvió token XSTS")?;
    let minecraft: serde_json::Value = client
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&serde_json::json!({
            "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Minecraft no aceptó la cuenta: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let minecraft_token = minecraft["access_token"]
        .as_str()
        .ok_or("Minecraft no devolvió access_token")?
        .to_owned();
    account_licensing::verify(client, &minecraft_token).await?;
    let profile = minecraft_profile(client, &minecraft_token).await?;
    account_licensing::remember_verified(&profile)?;
    Ok((profile, minecraft_token, xuid))
}

#[tauri::command]
async fn begin_microsoft_login(
    app: AppHandle,
    auth: State<'_, AuthState>,
) -> Result<MinecraftProfile, String> {
    let token_response = microsoft_oauth::login(&app).await?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let microsoft_token = token_response["access_token"]
        .as_str()
        .ok_or("Microsoft no devolvió access_token")?;
    let refresh_token = token_response["refresh_token"]
        .as_str()
        .ok_or("Microsoft no devolvió refresh_token; vuelve a autorizar la cuenta para mantener la sesión entre reinicios.")?;
    let (profile, minecraft_token, xuid) =
        minecraft_session_from_microsoft_token(&client, microsoft_token).await?;
    let _auth_guard = auth.refresh_lock.lock().await;
    launcher_settings::remember_current(&auth)?;
    token_storage::save_refresh_token(refresh_token)?;
    let expires_at = unix_time_seconds() + token_response["expires_in"].as_u64().unwrap_or(86400);
    *auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")? = Some(MicrosoftSession {
        profile: profile.clone(),
        access_token: minecraft_token,
        refresh_token: refresh_token.to_owned(),
        xuid,
        expires_at,
    });
    launcher_settings::remember_profile(&profile)?;
    Ok(profile)
}

fn unix_time_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn ensure_microsoft_session(auth: &AuthState) -> Result<Option<MinecraftProfile>, String> {
    let _refresh_guard = auth.refresh_lock.lock().await;
    if launcher_settings::offline_profile_selected()? {
        *auth
            .session
            .lock()
            .map_err(|_| "Estado de cuenta bloqueado")? = None;
        return Ok(None);
    }
    ensure_microsoft_session_locked(auth).await
}

async fn ensure_microsoft_session_locked(
    auth: &AuthState,
) -> Result<Option<MinecraftProfile>, String> {
    let current = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone();
    if let Some(session) = &current {
        if session.expires_at > unix_time_seconds() + 60 {
            if account_licensing::verified_at(&session.profile.id).is_none() {
                account_licensing::verify(&reqwest::Client::new(), &session.access_token).await?;
                account_licensing::remember_verified(&session.profile)?;
            }
            return Ok(Some(session.profile.clone()));
        }
    }
    let refresh_token = match current {
        Some(session) => session.refresh_token,
        None => {
            launcher_settings::restore_active_account_token()?;
            match token_storage::load_refresh_token()? {
                Some(token) => token,
                None => return Ok(None),
            }
        }
    };
    let client = reqwest::Client::new();
    let token_response = microsoft_oauth::refresh(&refresh_token).await?;
    let microsoft_token = token_response["access_token"]
        .as_str()
        .ok_or("Microsoft no devolvió access_token al renovar la sesión")?;
    let rotated_refresh = token_response["refresh_token"]
        .as_str()
        .unwrap_or(&refresh_token)
        .to_owned();
    let (profile, access_token, xuid) =
        minecraft_session_from_microsoft_token(&client, microsoft_token).await?;
    token_storage::save_refresh_token(&rotated_refresh)?;
    let session = MicrosoftSession {
        profile: profile.clone(),
        access_token,
        refresh_token: rotated_refresh,
        xuid,
        expires_at: unix_time_seconds() + token_response["expires_in"].as_u64().unwrap_or(86400),
    };
    *auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")? = Some(session);
    launcher_settings::remember_profile(&profile)?;
    Ok(Some(profile))
}
#[tauri::command]
async fn get_microsoft_profile(
    auth: State<'_, AuthState>,
) -> Result<Option<MinecraftProfile>, String> {
    ensure_microsoft_session(&auth).await
}
#[tauri::command]
async fn logout_microsoft(auth: State<'_, AuthState>) -> Result<(), String> {
    let _auth_guard = auth.refresh_lock.lock().await;
    launcher_settings::remember_current(&auth)?;
    *auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")? = None;
    token_storage::clear_saved_refresh_token()?;
    launcher_settings::deactivate_account()
}
#[tauri::command]
async fn upload_minecraft_skin(
    auth: State<'_, AuthState>,
    skin_base64: String,
    variant: String,
) -> Result<MinecraftProfile, String> {
    if variant != "classic" && variant != "slim" {
        return Err("Elige el modelo clásico o slim".into());
    }
    let session = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone()
        .ok_or("Inicia sesión con Microsoft antes de cambiar tu skin")?;
    let encoded = skin_base64
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&skin_base64);
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|e| e.to_string())?;
    let decoded = image::load_from_memory(&bytes).map_err(|e| format!("PNG no válido: {e}"))?;
    let (width, height) = (decoded.width(), decoded.height());
    if width != 64 || (height != 64 && height != 32) {
        return Err("La skin debe medir 64×64 o 64×32 píxeles".into());
    }
    let form = reqwest::multipart::Form::new()
        .text("variant", variant)
        .part(
            "file",
            reqwest::multipart::Part::bytes(bytes)
                .file_name("skin.png")
                .mime_str("image/png")
                .map_err(|e| e.to_string())?,
        );
    let response = reqwest::Client::new()
        .post("https://api.minecraftservices.com/minecraft/profile/skins")
        .bearer_auth(&session.access_token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Minecraft Services rechazó la skin: {} {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }
    let profile = minecraft_profile(&reqwest::Client::new(), &session.access_token).await?;
    auth.session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .as_mut()
        .ok_or("La sesión expiró")?
        .profile = profile.clone();
    Ok(profile)
}
#[tauri::command]
async fn activate_minecraft_cape(
    auth: State<'_, AuthState>,
    cape_id: String,
) -> Result<MinecraftProfile, String> {
    let session = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone()
        .ok_or("Inicia sesión con Microsoft antes de cambiar capas")?;
    if !session
        .profile
        .capes
        .iter()
        .any(|c| c["id"].as_str() == Some(&cape_id))
    {
        return Err("Esa capa no pertenece a la cuenta autenticada".into());
    }
    reqwest::Client::new()
        .put("https://api.minecraftservices.com/minecraft/profile/capes/active")
        .bearer_auth(&session.access_token)
        .json(&serde_json::json!({"capeId":cape_id}))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("No se pudo activar la capa: {e}"))?;
    let profile = minecraft_profile(&reqwest::Client::new(), &session.access_token).await?;
    auth.session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .as_mut()
        .ok_or("La sesión expiró")?
        .profile = profile.clone();
    Ok(profile)
}

#[tauri::command]
async fn hide_minecraft_cape(auth: State<'_, AuthState>) -> Result<MinecraftProfile, String> {
    let session = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone()
        .ok_or("Inicia sesión con Microsoft antes de cambiar capas")?;
    reqwest::Client::new()
        .delete("https://api.minecraftservices.com/minecraft/profile/capes/active")
        .bearer_auth(&session.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("No se pudo ocultar la capa: {e}"))?;
    let profile = minecraft_profile(&reqwest::Client::new(), &session.access_token).await?;
    auth.session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .as_mut()
        .ok_or("La sesión expiró")?
        .profile = profile.clone();
    Ok(profile)
}

#[tauri::command]
async fn fetch_mojang_versions(include_old: bool) -> Result<Vec<VersionItem>, String> {
    let manifest = load_version_manifest().await?;
    Ok(manifest
        .versions
        .into_iter()
        .filter(|v| {
            v.kind == "release"
                || v.kind == "snapshot"
                || (include_old && (v.kind == "old_beta" || v.kind == "old_alpha"))
        })
        .map(|v| VersionItem {
            id: v.id,
            url: v.url,
            kind: v.kind,
        })
        .collect())
}

#[tauri::command]
async fn fetch_fabric_versions() -> Result<Vec<FabricGameVersion>, String> {
    let versions: Vec<serde_json::Value> = reqwest::Client::new()
        .get("https://meta.fabricmc.net/v2/versions/game")
        .send()
        .await
        .map_err(|error| format!("No se pudo consultar el catálogo de Fabric: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Fabric devolvió un error de catálogo: {error}"))?
        .json()
        .await
        .map_err(|error| format!("El catálogo de Fabric no es válido: {error}"))?;
    Ok(versions
        .into_iter()
        .filter_map(|entry| {
            let id = entry["version"].as_str()?.to_owned();
            if !id.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
            }) {
                return None;
            }
            Some(FabricGameVersion {
                id,
                stable: entry["stable"].as_bool().unwrap_or(false),
            })
        })
        .collect())
}

#[tauri::command]
async fn fetch_forge_versions() -> Result<Vec<ForgeGameVersion>, String> {
    let promotions: serde_json::Value = reqwest::Client::new()
        .get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
        .send()
        .await
        .map_err(|error| format!("No se pudo consultar el catálogo de Forge: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Forge devolvió un error de catálogo: {error}"))?
        .json()
        .await
        .map_err(|error| format!("El catálogo de Forge no es válido: {error}"))?;
    let promos = promotions["promos"]
        .as_object()
        .ok_or("El catálogo oficial de Forge no contiene builds")?;
    let mut versions = BTreeMap::<String, ForgeGameVersion>::new();
    for (key, value) in promos {
        let Some((game_version, channel)) = key
            .strip_suffix("-recommended")
            .map(|version| (version, "Recomendada"))
            .or_else(|| {
                key.strip_suffix("-latest")
                    .map(|version| (version, "Latest"))
            })
        else {
            continue;
        };
        let Some(forge_version) = value.as_str() else {
            continue;
        };
        if !game_version
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
        {
            continue;
        }
        let replace = versions
            .get(game_version)
            .is_none_or(|current| channel == "Recomendada" && current.channel != "Recomendada");
        if replace {
            versions.insert(
                game_version.to_owned(),
                ForgeGameVersion {
                    id: game_version.to_owned(),
                    forge_version: forge_version.to_owned(),
                    channel: channel.to_owned(),
                },
            );
        }
    }
    Ok(versions.into_values().collect())
}

#[cfg(test)]
fn parse_optifine_catalog(html: &str) -> Result<Vec<OptiFineVersion>, String> {
    let rows = Regex::new(
        r#"(?is)<tr\b[^>]*class\s*=\s*['\"][^'\"]*downloadLine[^'\"]*['\"][^>]*>(.*?)</tr>"#,
    )
    .map_err(|error| error.to_string())?;
    let file_cell = Regex::new(r#"(?is)<td\b[^>]*class\s*=\s*['\"]colFile['\"][^>]*>(.*?)</td>"#)
        .map_err(|error| error.to_string())?;
    let mirror_cell = Regex::new(
        r#"(?is)<td\b[^>]*class\s*=\s*['\"]colMirror['\"][^>]*>\s*<a\b[^>]*href\s*=\s*['\"]([^'\"]+)['\"]"#,
    )
    .map_err(|error| error.to_string())?;
    let strip_tags = Regex::new(r"(?is)<[^>]+>").map_err(|error| error.to_string())?;
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for row in rows
        .captures_iter(html)
        .filter_map(|captures| captures.get(1))
    {
        let row_html = row.as_str();
        let Some(file_html) = file_cell
            .captures(row_html)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str())
        else {
            continue;
        };
        let Some(mirror_html) = mirror_cell
            .captures(row_html)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str())
        else {
            continue;
        };
        let mirror = mirror_html
            .replace("&amp;", "&")
            .replace("&#39;", "'")
            .replace("&quot;", "\"");
        let Ok(url) = url::Url::parse(&mirror)
            .or_else(|_| url::Url::parse("https://optifine.net/")?.join(&mirror))
        else {
            continue;
        };
        if !matches!(url.host_str(), Some("optifine.net" | "www.optifine.net"))
            || !url.path().ends_with("/adloadx")
        {
            continue;
        }
        let Some(file_name) = url
            .query_pairs()
            .find(|(key, _)| key == "f")
            .map(|(_, value)| value.into_owned())
        else {
            continue;
        };
        if !file_name.ends_with(".jar")
            || !file_name.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
            })
            || !seen.insert(file_name.clone())
        {
            continue;
        }
        let normal_name = file_name.strip_prefix("preview_").unwrap_or(&file_name);
        let Some(version_part) = normal_name.strip_prefix("OptiFine_") else {
            continue;
        };
        let Some((game_version, _)) = version_part.split_once("_HD_U_") else {
            continue;
        };
        if !game_version
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
        {
            continue;
        }
        let name = strip_tags
            .replace_all(file_html, "")
            .replace("&nbsp;", " ")
            .replace("&#160;", " ")
            .replace("&amp;", "&")
            .trim()
            .to_owned();
        let profile_id = optifine_profile_id(&game_version, &file_name);
        result.push(OptiFineVersion {
            id: profile_id.clone(),
            profile_id,
            game_version: game_version.to_owned(),
            name,
            file_name: file_name.clone(),
            preview: file_name.starts_with("preview_"),
            legacy_patch: false,
        });
    }
    Ok(result)
}

fn optifine_profile_id(game_version: &str, file_name: &str) -> String {
    let stem = file_name.strip_prefix("preview_").unwrap_or(file_name);
    let normal = stem
        .strip_suffix(".jar")
        .or_else(|| stem.strip_suffix(".zip"))
        .unwrap_or(stem);
    let revision = normal
        .strip_prefix(&format!("OptiFine_{game_version}_HD_U_"))
        .or_else(|| normal.strip_prefix("OptiFine_"))
        .or_else(|| normal.strip_prefix("beta_OptiFine_"))
        .unwrap_or(normal)
        .replace('_', "-")
        .replace('.', "-");
    format!("{game_version}-OptiFine-{revision}")
}

#[cfg(test)]
fn legacy_optifine_catalog() -> Vec<OptiFineVersion> {



    const ENTRIES: &[(&str, &str, &str, bool)] = &[
        (
            "1.6.4",
            "OptiFine HD U D1",
            "OptiFine_1.6.4_HD_U_D1.jar",
            false,
        ),
        (
            "1.6.2",
            "OptiFine HD U C4",
            "OptiFine_1.6.2_HD_U_C4.jar",
            false,
        ),
        (
            "1.5.2",
            "OptiFine HD U D5",
            "OptiFine_1.5.2_HD_U_D5.zip",
            true,
        ),
        (
            "1.5.1",
            "OptiFine HD U D1",
            "OptiFine_1.5.1_HD_U_D1.zip",
            true,
        ),
        (
            "1.5",
            "OptiFine HD U A7",
            "OptiFine_1.5.0_HD_U_A7.zip",
            true,
        ),
        (
            "1.4.6",
            "OptiFine HD U D5",
            "OptiFine_1.4.6_HD_U_D5.zip",
            true,
        ),
        (
            "1.4.7",
            "OptiFine HD U D5 · compatible desde 1.4.6",
            "OptiFine_1.4.6_HD_U_D5.zip",
            true,
        ),
        (
            "1.4.5",
            "OptiFine HD U D8",
            "OptiFine_1.4.5_HD_U_D8.zip",
            true,
        ),
        (
            "1.4.4",
            "OptiFine HD U D2",
            "OptiFine_1.4.4_HD_U_D2.zip",
            true,
        ),
        (
            "1.4.3",
            "OptiFine HD U C1",
            "OptiFine_1.4.3_HD_U_C1.zip",
            true,
        ),
        (
            "1.4.2",
            "OptiFine HD U B5",
            "OptiFine_1.4.2_HD_U_B5.zip",
            true,
        ),
        (
            "1.4.1",
            "OptiFine HD U A5",
            "OptiFine_1.4.1_HD_U_A5.zip",
            true,
        ),
        (
            "1.4",
            "OptiFine HD U A3",
            "OptiFine_1.4.0_HD_U_A3.zip",
            true,
        ),
        (
            "1.3.2",
            "OptiFine HD U B4",
            "OptiFine_1.3.2_HD_U_B4.zip",
            true,
        ),
        (
            "1.3.1",
            "OptiFine HD U B1",
            "OptiFine_1.3.1_HD_U_B1.zip",
            true,
        ),
        ("1.2.5", "OptiFine HD C6", "OptiFine_1.2.5_HD_C6.zip", true),
        (
            "1.2.4",
            "OptiFine HD A2 (compatible)",
            "OptiFine_1.2.5_HD_A2.zip",
            true,
        ),
        ("1.2.3", "OptiFine HD B", "OptiFine_1.2.3_HD_B.zip", true),
        ("1.1", "OptiFine HD D6", "OptiFine_1.1_HD_D6.zip", true),
        ("1.0", "OptiFine HD D3", "OptiFine_1.0.0_HD_D3.zip", true),
        (
            "b1.8.1",
            "OptiFine Beta 1.8.1 HD D2",
            "beta_OptiFine_1.8.1_HD_D2.zip",
            true,
        ),
        (
            "b1.7.3",
            "OptiFine Beta 1.7.3 HD G",
            "beta_OptiFine_1.7.3_HD_G.zip",
            true,
        ),
    ];
    ENTRIES
        .iter()
        .map(
            |(game_version, name, file_name, legacy_patch)| OptiFineVersion {
                id: optifine_profile_id(game_version, file_name),
                profile_id: optifine_profile_id(game_version, file_name),
                game_version: (*game_version).to_owned(),
                name: format!("{name} · Histórico"),
                file_name: (*file_name).to_owned(),
                preview: false,
                legacy_patch: *legacy_patch,
            },
        )
        .collect()
}

#[tauri::command]
async fn fetch_optifine_versions() -> Result<Vec<OptiFineVersion>, String> {
    let root = root_dir()?;
    Ok(list_installed_versions()?
        .into_iter()
        .filter(|v| v.loader == "optifine")
        .map(|v| {
            let metadata = root
                .join("versions")
                .join(&v.id)
                .join(format!("{}.json", v.id));
            let profile = fs::metadata(&metadata)
                .ok()
                .filter(|m| m.len() <= 2 * 1024 * 1024)
                .and_then(|_| fs::read(metadata).ok())
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
            let file_name = profile
                .as_ref()
                .and_then(|profile| profile["libraries"].as_array())
                .and_then(|libraries| {
                    libraries
                        .iter()
                        .find_map(|lib| lib["optifineFileName"].as_str())
                })
                .unwrap_or_default()
                .to_owned();
            let name = if file_name.is_empty() {
                v.id.clone()
            } else {
                file_name.trim_end_matches(".jar").replace('_', " ")
            };
            OptiFineVersion {
                id: v.id.clone(),
                profile_id: v.id,
                game_version: v.game_version,
                name,
                preview: file_name.starts_with("preview_"),
                legacy_patch: false,
                file_name,
            }
        })
        .collect())
}
fn local_optifine_version(file_name: &str, bytes: &[u8]) -> Result<OptiFineVersion, String> {
    optifine_discovery::validate(file_name, bytes)
}

fn extract_optifine_launchwrapper(bytes: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("El JAR de OptiFine no es válido: {error}"))?;
    let mut nested = match archive.by_name("launchwrapper-of-2.3.jar") {
        Ok(file) => file,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => {
            return Err(format!(
                "No se pudo leer el LaunchWrapper de OptiFine: {error}"
            ));
        }
    };
    let mut wrapper_bytes = Vec::new();
    nested
        .read_to_end(&mut wrapper_bytes)
        .map_err(|error| format!("No se pudo extraer LaunchWrapper OF: {error}"))?;
    drop(nested);
    let mut wrapper = zip::ZipArchive::new(Cursor::new(&wrapper_bytes))
        .map_err(|error| format!("El LaunchWrapper incluido por OptiFine está dañado: {error}"))?;
    if wrapper
        .by_name("net/minecraft/launchwrapper/Launch.class")
        .is_err()
    {
        return Err(
            "El LaunchWrapper incluido por OptiFine no contiene su clase de inicio.".into(),
        );
    }
    Ok(Some(wrapper_bytes))
}

fn merge_legacy_optifine_patch(base_bytes: &[u8], patch_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut merged = BTreeMap::<String, Vec<u8>>::new();
    for (bytes, label) in [
        (base_bytes, "cliente base"),
        (patch_bytes, "parche OptiFine"),
    ] {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| format!("El {label} no es un ZIP/JAR válido: {error}"))?;
        for index in 0..archive.len() {
            let mut entry = archive
                .by_index(index)
                .map_err(|error| format!("No se pudo leer {label}: {error}"))?;
            if entry.is_dir() {
                continue;
            }
            let Some(path) = entry.enclosed_name() else {
                return Err(format!("El {label} contiene una ruta de archivo insegura."));
            };
            let name = path.to_string_lossy().replace('\\', "/");

            if name.eq_ignore_ascii_case("META-INF")
                || name.to_ascii_uppercase().starts_with("META-INF/")
            {
                continue;
            }
            let mut contents = Vec::new();
            entry
                .read_to_end(&mut contents)
                .map_err(|error| format!("No se pudo extraer {name}: {error}"))?;
            merged.insert(name, contents);
        }
    }
    if !merged.keys().any(|name| name.ends_with(".class")) {
        return Err("El parche histórico no contiene clases ejecutables.".into());
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, contents) in merged {
        writer
            .start_file(name, options)
            .map_err(|error| format!("No se pudo crear el JAR parcheado: {error}"))?;
        writer
            .write_all(&contents)
            .map_err(|error| format!("No se pudo escribir el JAR parcheado: {error}"))?;
    }
    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| format!("No se pudo finalizar el JAR parcheado: {error}"))
}

fn ensure_legacy_optifine_launchwrapper(meta: &mut serde_json::Value) {
    if meta["mainClass"].as_str() != Some("net.minecraft.launchwrapper.Launch")
        || meta.pointer("/javaVersion/majorVersion").and_then(|v|v.as_u64()).unwrap_or(8)>8 {
        return;
    }
    let is_optifine = meta["minecraftArguments"].as_str().is_some_and(|args|args.contains("optifine.OptiFineTweaker"))
        || meta["libraries"].as_array().is_some_and(|libs|libs.iter().any(|lib|lib["name"].as_str().is_some_and(|name|name.starts_with("optifine:OptiFine:"))));
    if !is_optifine{return;}
    if !meta["libraries"].is_array(){meta["libraries"]=serde_json::json!([]);}
    let libraries=meta["libraries"].as_array_mut().unwrap();
    if libraries.iter().any(|lib|lib["name"].as_str().and_then(|name|name.split(':').nth(1)).is_some_and(|name|name=="launchwrapper"||name=="launchwrapper-of")){return;}
    libraries.push(serde_json::json!({"name":"net.minecraft:launchwrapper:1.12","downloads":{"artifact":{"path":"net/minecraft/launchwrapper/1.12/launchwrapper-1.12.jar","url":"https://libraries.minecraft.net/net/minecraft/launchwrapper/1.12/launchwrapper-1.12.jar","sha1":"111e7bea9c968cdb3d06ef4632bf7ff0824d0f36","size":32999}}}));
}

#[tauri::command]
async fn import_optifine(
    app: AppHandle,
    file_name: String,
    jar_base64: String,
) -> Result<String, String> {
    if jar_base64.len() > 108 * 1024 * 1024 {
        return Err("El JAR supera el límite de 80 MB".into());
    }
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, jar_base64)
        .map_err(|_| "Archivo local inválido")?;
    let optifine = local_optifine_version(&file_name, &bytes)?;
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 TP LAUNCHER/0.1.0")
        .build()
        .map_err(|error| error.to_string())?;
    emit_progress(
        &app,
        "Instalador OptiFine",
        5,
        &format!("Verificando Minecraft {}…", optifine.game_version),
    );
    let manifest = load_version_manifest().await?;
    let base = manifest
        .versions
        .iter()
        .find(|entry| entry.id == optifine.game_version)
        .ok_or_else(|| {
            format!(
                "Minecraft {} no está en el manifiesto oficial",
                optifine.game_version
            )
        })?;
    let base_metadata: serde_json::Value = client
        .get(&base.url)
        .send()
        .await
        .map_err(|error| format!("No se pudo consultar Minecraft {}: {error}", base.id))?
        .error_for_status()
        .map_err(|error| format!("El manifiesto de Minecraft {} falló: {error}", base.id))?
        .json()
        .await
        .map_err(|error| {
            format!(
                "El manifiesto de Minecraft {} no es válido: {error}",
                base.id
            )
        })?;

    emit_progress(
        &app,
        "Instalador OptiFine",
        20,
        &format!("Importando el archivo local {}…", optifine.name),
    );
    let normal_file = optifine
        .file_name
        .strip_prefix("preview_")
        .unwrap_or(&optifine.file_name);
    let is_patch = optifine.legacy_patch;
    let profile_id = optifine.profile_id.clone();
    if !profile_id
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_'))
    {
        return Err("OptiFine devolvió un identificador de perfil no válido".into());
    }
    let root = root_dir()?;
    let profile_dir = root.join("versions").join(&profile_id);
    fs::create_dir_all(&profile_dir).map_err(|error| error.to_string())?;
    let mut profile = serde_json::json!({
        "id": profile_id,
        "inheritsFrom": optifine.game_version,
        "type": "release"
    });
    if is_patch {
        let base_client_metadata = base_metadata
            .pointer("/downloads/client")
            .ok_or("La versión base no publica el cliente vanilla")?;
        let client_url = base_client_metadata["url"]
            .as_str()
            .ok_or("La versión base no contiene URL de cliente")?;
        let client_hash = base_client_metadata["sha1"].as_str();
        let client_size = base_client_metadata["size"].as_u64();
        let base_jar = root
            .join("versions")
            .join(&optifine.game_version)
            .join(format!("{}.jar", optifine.game_version));
        let base_bytes = if file_matches(&base_jar, client_hash, client_size) {
            fs::read(&base_jar).map_err(|error| error.to_string())?
        } else {
            emit_progress(
                &app,
                "Instalador OptiFine",
                35,
                "Descargando cliente base de Minecraft…",
            );
            let bytes = client
                .get(client_url)
                .send()
                .await
                .map_err(|error| format!("No se pudo descargar el cliente base: {error}"))?
                .error_for_status()
                .map_err(|error| format!("La descarga del cliente base falló: {error}"))?
                .bytes()
                .await
                .map_err(|error| format!("Descarga incompleta del cliente base: {error}"))?
                .to_vec();
            if client_size.is_some_and(|size| bytes.len() as u64 != size)
                || !sha1_matches(&bytes, client_hash)
            {
                return Err(format!(
                    "El cliente base de Minecraft {} no pasó la verificación.",
                    optifine.game_version
                ));
            }
            if let Some(parent) = base_jar.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let temporary = base_jar.with_extension(format!("jar.download-{}", std::process::id()));
            fs::write(&temporary, &bytes).map_err(|error| error.to_string())?;
            if base_jar.exists() {
                fs::remove_file(&base_jar).map_err(|error| error.to_string())?;
            }
            fs::rename(temporary, &base_jar).map_err(|error| error.to_string())?;
            bytes
        };
        let patched = merge_legacy_optifine_patch(&base_bytes, &bytes)?;
        let patched_jar = profile_dir.join(format!("{profile_id}.jar"));
        let temporary_patched_jar =
            patched_jar.with_extension(format!("jar.tmp-{}", std::process::id()));
        fs::write(&temporary_patched_jar, patched)
            .map_err(|error| format!("No se pudo escribir la instalación parcheada: {error}"))?;
        if patched_jar.exists() {
            fs::remove_file(&patched_jar).map_err(|error| error.to_string())?;
        }
        fs::rename(&temporary_patched_jar, &patched_jar)
            .map_err(|error| format!("No se pudo guardar el cliente parcheado: {error}"))?;
        profile["patchedClientJar"] = serde_json::Value::String(
            patched_jar
                .strip_prefix(&root)
                .map_err(|_| "La instalación OptiFine salió de la carpeta del juego")?
                .to_string_lossy()
                .replace('\\', "/"),
        );
        if let Some(main_class) = base_metadata["mainClass"].as_str() {
            profile["mainClass"] = serde_json::Value::String(main_class.to_owned());
        }
    } else {
        let library_version = normal_file
            .strip_prefix("OptiFine_")
            .and_then(|name| name.strip_suffix(".jar"))
            .ok_or("El nombre oficial del JAR OptiFine no es válido")?;
        let coordinate = format!("optifine:OptiFine:{library_version}");
        let relative_path = maven_library_path(&coordinate)
            .ok_or("No se pudo generar la ruta de la biblioteca OptiFine")?;
        let library_path = root.join("libraries").join(&relative_path);
        if let Some(parent) = library_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let temporary_library =
            library_path.with_extension(format!("jar.download-{}", std::process::id()));
        fs::write(&temporary_library, &bytes).map_err(|error| error.to_string())?;
        if library_path.exists() {
            fs::remove_file(&library_path).map_err(|error| error.to_string())?;
        }
        fs::rename(&temporary_library, &library_path)
            .map_err(|error| format!("No se pudo guardar la biblioteca OptiFine: {error}"))?;
        let sha1 = format!("{:x}", Sha1::digest(&bytes));
        let optifine_library = serde_json::json!({
            "name": coordinate,
            "optifineFileName": optifine.file_name,
            "downloads": { "artifact": { "path": relative_path, "url": "", "sha1": sha1, "size": bytes.len() } }
        });
        profile["mainClass"] =
            serde_json::Value::String("net.minecraft.launchwrapper.Launch".into());
        profile["libraries"] = serde_json::json!([optifine_library]);
        if let Some(legacy_arguments) = base_metadata["minecraftArguments"].as_str() {
            profile["minecraftArguments"] = serde_json::Value::String(format!(
                "{legacy_arguments} --tweakClass optifine.OptiFineTweaker"
            ));
        } else {
            profile["arguments"] = serde_json::json!({
                "game": ["--tweakClass", "optifine.OptiFineTweaker"]
            });
        }
    }
    if base_metadata.pointer("/javaVersion/majorVersion").and_then(|v|v.as_u64()).unwrap_or(8)<=8 {
        ensure_legacy_optifine_launchwrapper(&mut profile);
    }
    let profile_path = profile_dir.join(format!("{profile_id}.json"));
    let temporary_profile = profile_dir.join(format!("{profile_id}.json.tmp"));
    fs::write(
        &temporary_profile,
        serde_json::to_vec_pretty(&profile).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if profile_path.exists() {
        fs::remove_file(&profile_path).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary_profile, profile_path).map_err(|error| error.to_string())?;
    emit_progress(
        &app,
        "Instalador OptiFine",
        100,
        &format!("Perfil {profile_id} listo para iniciar"),
    );
    Ok(profile_id)
}

#[tauri::command]
async fn install_fabric(game_version: String) -> Result<String, String> {
    if game_version.is_empty()
        || !game_version.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '.' || character == '-'
        })
    {
        return Err("Versión de Minecraft no válida".into());
    }
    let client = reqwest::Client::builder()
        .user_agent("TP LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)")
        .build()
        .map_err(|error| error.to_string())?;
    let loader_versions: Vec<serde_json::Value> = client
        .get(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{game_version}"
        ))
        .send()
        .await
        .map_err(|error| format!("No se pudo consultar Fabric Meta: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Fabric no ofrece una versión para {game_version}: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Respuesta de Fabric inválida: {error}"))?;
    let loader_version = loader_versions
        .iter()
        .find(|candidate| {
            candidate
                .pointer("/loader/stable")
                .and_then(|value| value.as_bool())
                == Some(true)
        })
        .or_else(|| loader_versions.first())
        .and_then(|candidate| {
            candidate
                .pointer("/loader/version")
                .and_then(|value| value.as_str())
        })
        .ok_or_else(|| format!("Fabric no tiene un loader compatible con {game_version}"))?;
    let profile: serde_json::Value = client
        .get(format!(
            "https://meta.fabricmc.net/v2/versions/loader/{game_version}/{loader_version}/profile/json"
        ))
        .send()
        .await
        .map_err(|error| format!("No se pudo descargar el perfil Fabric: {error}"))?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| format!("Perfil Fabric inválido: {error}"))?;
    let id = profile["id"]
        .as_str()
        .filter(|id| {
            !id.is_empty()
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
        })
        .ok_or("Fabric devolvió un identificador de perfil no válido")?;
    if profile["inheritsFrom"].as_str() != Some(game_version.as_str()) {
        return Err("El perfil Fabric no coincide con la versión de Minecraft solicitada".into());
    }
    let profile_dir = root_dir()?.join("versions").join(id);
    fs::create_dir_all(&profile_dir).map_err(|error| error.to_string())?;
    let target = profile_dir.join(format!("{id}.json"));
    let temporary = profile_dir.join(format!("{id}.json.tmp"));
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&profile).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    if target.exists() {
        fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, target).map_err(|error| error.to_string())?;
    Ok(id.to_owned())
}

#[tauri::command]
async fn install_forge(app: AppHandle, game_version: String) -> Result<String, String> {
    if game_version.is_empty()
        || !game_version.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '.' || character == '-'
        })
    {
        return Err("Versión de Minecraft no válida".into());
    }
    emit_progress(
        &app,
        "Instalador Forge",
        0,
        &format!("Consultando Forge para Minecraft {game_version}…"),
    );
    let client = reqwest::Client::builder()
        .user_agent("TP LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)")
        .build()
        .map_err(|error| error.to_string())?;
    let promotions: serde_json::Value = client
        .get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
        .send()
        .await
        .map_err(|error| format!("No se pudo consultar Forge: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Forge devolvió un error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Catálogo de Forge inválido: {error}"))?;
    let promos = promotions["promos"]
        .as_object()
        .ok_or("El catálogo oficial de Forge no contiene versiones")?;
    let forge_version = promos
        .get(&format!("{game_version}-recommended"))
        .or_else(|| promos.get(&format!("{game_version}-latest")))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            format!("Forge todavía no publica una build para Minecraft {game_version}")
        })?;
    let forge_id = format!("{game_version}-forge-{forge_version}");
    let root = root_dir()?;
    let profile_json = root
        .join("versions")
        .join(&forge_id)
        .join(format!("{forge_id}.json"));
    if let Ok(bytes) = fs::read(&profile_json) {
        if serde_json::from_slice::<serde_json::Value>(&bytes)
            .is_ok_and(|profile| profile["inheritsFrom"].as_str() == Some(game_version.as_str()))
        {
            return Ok(forge_id);
        }
    }
    let installer_dir = root.join("runtime").join("forge-installers");
    fs::create_dir_all(&installer_dir).map_err(|error| error.to_string())?;
    let installer = installer_dir.join(format!(
        "forge-{game_version}-{forge_version}-installer.jar"
    ));
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{game_version}-{forge_version}/forge-{game_version}-{forge_version}-installer.jar"
    );
    let valid_cached_installer =
        fs::read(&installer).is_ok_and(|bytes| zip::ZipArchive::new(Cursor::new(bytes)).is_ok());
    if !valid_cached_installer {
        emit_progress(
            &app,
            "Instalador Forge",
            18,
            "Descargando el instalador oficial…",
        );
        if installer.exists() {
            fs::remove_file(&installer).map_err(|error| error.to_string())?;
        }
        download_verified(&client, &installer_url, &installer, None, None).await?;
    }
    let installer_bytes = fs::read(&installer).map_err(|error| error.to_string())?;
    let mut archive = zip::ZipArchive::new(Cursor::new(installer_bytes))
        .map_err(|error| format!("El instalador descargado no es un JAR válido: {error}"))?;
    if archive.by_name("install_profile.json").is_err() && archive.by_name("version.json").is_err()
    {
        return Err("El JAR oficial de Forge no contiene un perfil de instalación válido".into());
    }
    drop(archive);

    let java_requirement = get_version_java_requirement(game_version.clone())
        .await?
        .max(8);
    let mut runtime = discover_java_installations()
        .into_iter()
        .filter(|runtime| runtime.major == java_requirement && runtime_enabled(runtime.major))
        .min_by_key(|runtime| !runtime.portable);
    if runtime.is_none() && java_requirement != 8 {
        runtime = discover_java_installations()
            .into_iter()
            .filter(|runtime| runtime.major > java_requirement && runtime_enabled(runtime.major))
            .min_by_key(|runtime| runtime.major);
    }
    if runtime.is_none() {
        if !runtime_enabled(java_requirement) {
            return Err(format!(
                "Java {java_requirement} está desactivado en Ajustes"
            ));
        }
        install_portable_java(app.clone(), java_requirement).await?;
        runtime = discover_java_installations()
            .into_iter()
            .find(|runtime| runtime.major == java_requirement && runtime_enabled(runtime.major));
    }
    let java_path = runtime
        .ok_or_else(|| format!("No se encontró Java {java_requirement} para instalar Forge"))?
        .path;
    emit_progress(
        &app,
        "Instalador Forge",
        38,
        &format!("Instalando Forge {forge_version} para Minecraft {game_version}…"),
    );
    let target_dir = root.clone();
    let working_dir = installer_dir.clone();
    let install_result = tokio::task::spawn_blocking(move || {
        let mut command = Command::new(java_path);
        command
            .arg("-jar")
            .arg(installer)
            .arg("--installClient")
            .arg(target_dir);
        command.current_dir(working_dir);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        command.output().map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("El instalador Forge terminó inesperadamente: {error}"))??;
    let stdout = String::from_utf8_lossy(&install_result.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&install_result.stderr).into_owned();
    for line in stdout
        .lines()
        .chain(stderr.lines())
        .filter(|line| !line.trim().is_empty())
        .take(120)
    {
        emit_launcher_log(&app, "forge-installer", line);
    }
    if !install_result.status.success() {
        return Err(format!(
            "El instalador oficial de Forge terminó con {}. Revisa la consola para ver su registro.",
            install_result.status
        ));
    }
    let profile_installed = fs::read(&profile_json)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_some_and(|profile| profile["inheritsFrom"].as_str() == Some(game_version.as_str()));
    if !profile_installed {
        return Err(format!(
            "Forge terminó, pero no se encontró el perfil {forge_id}; revisa la salida del instalador en Consola."
        ));
    }
    emit_progress(&app, "Instalador Forge", 100, "Perfil Forge instalado");
    Ok(forge_id)
}

fn resolve_mod_game_version(root: &Path, version_id: &str) -> Result<(String, String), String> {
    if version_id.is_empty()
        || !version_id.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || character == '-'
                || character == '.'
                || character == '_'
                || character == '+'
        })
    {
        return Err("Identificador de versión no válido".into());
    }
    let version_path = root
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"));
    if let Ok(bytes) = fs::read(version_path) {
        if let Ok(profile) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(game_version) = profile["inheritsFrom"]
                .as_str()
                .or_else(|| profile["jar"].as_str())
            {
                let normalized_id = version_id.to_ascii_lowercase();
                let loader = if normalized_id.starts_with("neoforge-")
                    || normalized_id.contains("-neoforge-")
                    || profile["libraries"].as_array().is_some_and(|libraries| {
                        libraries.iter().any(|library| {
                            library["name"].as_str().is_some_and(|name| {
                                name.starts_with("net.neoforged:")
                                    || name.starts_with("net.neoforged.fancymodloader:")
                            })
                        })
                    }) {
                    "neoforge"
                } else if normalized_id.starts_with("quilt-loader-") {
                    "quilt"
                } else if normalized_id.starts_with("fabric-loader-") {
                    "fabric"
                } else if normalized_id.starts_with("forge-") || normalized_id.contains("-forge-") {
                    "forge"
                } else if normalized_id.starts_with("optifine-")
                    || normalized_id.contains("-optifine-")
                {
                    "optifine"
                } else {
                    "custom"
                };
                return Ok((game_version.to_owned(), loader.to_owned()));
            }
        }
    }
    Ok((version_id.to_owned(), "vanilla".to_owned()))
}

#[tauri::command]
async fn search_modrinth_mods(
    query: String,
    game_version: String,
    loader: String,
    project_type: String,
    page: u32,
) -> Result<ModrinthSearchPage, String> {
    let allowed_loaders = ["fabric", "forge", "neoforge", "quilt"];
    if !allowed_loaders.contains(&loader.as_str()) {
        return Err("Elige Fabric, Forge, NeoForge o Quilt para buscar mods".into());
    }
    if !["mod", "modpack"].contains(&project_type.as_str()) {
        return Err("Tipo de proyecto Modrinth no válido".into());
    }
    if game_version != "all"
        && (game_version.is_empty()
            || !game_version.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '+')
            }))
    {
        return Err("Versión de Minecraft no válida para la búsqueda".into());
    }
    let client = reqwest::Client::builder()
        .user_agent("TP LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)")
        .build()
        .map_err(|error| error.to_string())?;
    let mut facets = vec![
        vec![format!("project_type:{project_type}")],
        vec![format!("categories:{loader}")],
    ];
    if game_version != "all" {
        facets.push(vec![format!("versions:{game_version}")]);
    }
    let facets_query = serde_json::to_string(&facets).map_err(|error| error.to_string())?;
    let page = page.max(1);
    let offset = (page - 1).saturating_mul(24);
    let offset_query = offset.to_string();
    let response: serde_json::Value = client
        .get("https://api.modrinth.com/v2/search")
        .query(&[
            ("query", query.trim()),
            ("facets", facets_query.as_str()),
            ("limit", "24"),
            ("offset", offset_query.as_str()),
        ])
        .send()
        .await
        .map_err(|error| format!("No se pudo buscar en Modrinth: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Modrinth devolvió un error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Respuesta de búsqueda Modrinth inválida: {error}"))?;
    let projects = response["hits"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|hit| {
            Some(ModrinthProject {
                project_id: hit["project_id"].as_str()?.to_owned(),
                slug: hit["slug"].as_str()?.to_owned(),
                title: hit["title"].as_str()?.to_owned(),
                description: hit["description"].as_str().unwrap_or_default().to_owned(),
                icon_url: hit["icon_url"].as_str().unwrap_or_default().to_owned(),
                downloads: hit["downloads"].as_u64().unwrap_or_default(),
            })
        })
        .collect();
    Ok(ModrinthSearchPage {
        projects,
        total_hits: response["total_hits"].as_u64().unwrap_or_default(),
        page,
        page_size: 24,
    })
}

#[tauri::command]
async fn install_modrinth_mod(
    project_id: String,
    version_id: String,
    loader: String,
    username: String,
) -> Result<String, String> {
    let allowed_loaders = ["fabric", "forge", "neoforge", "quilt"];
    if !allowed_loaders.contains(&loader.as_str()) {
        return Err("Tipo de loader no válido".into());
    }
    if project_id.is_empty()
        || !project_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err("Identificador de proyecto Modrinth no válido".into());
    }
    let root = root_dir()?;
    let (game_version, installed_loader) = resolve_mod_game_version(&root, &version_id)?;
    if installed_loader != loader {
        return Err(format!(
            "La instalación seleccionada usa {installed_loader}; instala o selecciona una instancia {loader} compatible antes de agregar mods"
        ));
    }
    let mods_dir = version_game_dir(&root, &username, &version_id)?.join("mods");
    fs::create_dir_all(&mods_dir).map_err(|error| error.to_string())?;
    let client = reqwest::Client::builder()
        .user_agent("TP LAUNCHER/0.1.0 (+https://github.com/tomaap07/MinecraftTPS)")
        .build()
        .map_err(|error| error.to_string())?;
    let versions_url = format!("https://api.modrinth.com/v2/project/{project_id}/version");
    let response: Vec<serde_json::Value> = client
        .get(&versions_url)
        .query(&[
            (
                "game_versions",
                serde_json::json!([game_version]).to_string(),
            ),
            ("loaders", serde_json::json!([loader]).to_string()),
            ("include_changelog", "false".to_owned()),
        ])
        .send()
        .await
        .map_err(|error| format!("No se pudieron consultar versiones del mod: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Modrinth devolvió un error: {error}"))?
        .json()
        .await
        .map_err(|error| format!("Versiones del mod inválidas: {error}"))?;
    let selected = response.first().cloned().ok_or_else(|| {
        format!("Este mod no tiene una versión compatible con Minecraft {game_version} y {loader}")
    })?;
    let mut pending = VecDeque::from([selected]);
    let mut installed_projects = HashSet::new();
    let mut installed_files = Vec::new();
    while let Some(mod_version) = pending.pop_front() {
        let current_project = mod_version["project_id"].as_str().unwrap_or(&project_id);
        if !installed_projects.insert(current_project.to_owned()) {
            continue;
        }
        let file = mod_version["files"]
            .as_array()
            .and_then(|files| {
                files
                    .iter()
                    .find(|file| file["primary"].as_bool() == Some(true))
                    .or(files.first())
            })
            .ok_or("Modrinth no devolvió un archivo para el mod")?;
        let url = file["url"]
            .as_str()
            .ok_or("El archivo del mod no tiene URL")?;
        let filename = file["filename"]
            .as_str()
            .ok_or("El archivo del mod no tiene nombre")?;
        if !filename.to_ascii_lowercase().ends_with(".jar")
            || filename.contains(['/', '\\', ':'])
            || filename == ".jar"
        {
            return Err("Modrinth devolvió un nombre de archivo no válido".into());
        }
        let hash = file.pointer("/hashes/sha1").and_then(|hash| hash.as_str());
        download_verified(
            &client,
            url,
            &mods_dir.join(filename),
            hash,
            file["size"].as_u64(),
        )
        .await?;
        installed_files.push(filename.to_owned());

        for dependency in mod_version["dependencies"].as_array().into_iter().flatten() {
            if dependency["dependency_type"].as_str() != Some("required") {
                continue;
            }
            let dependency_version = if let Some(dependency_id) = dependency["version_id"].as_str()
            {
                client
                    .get(format!(
                        "https://api.modrinth.com/v2/version/{dependency_id}"
                    ))
                    .send()
                    .await
                    .map_err(|error| {
                        format!("No se pudo consultar una dependencia obligatoria: {error}")
                    })?
                    .error_for_status()
                    .map_err(|error| {
                        format!("No se pudo obtener una dependencia obligatoria: {error}")
                    })?
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|error| error.to_string())?
            } else if let Some(dependency_project) = dependency["project_id"].as_str() {
                let versions: Vec<serde_json::Value> = client
                    .get(format!(
                        "https://api.modrinth.com/v2/project/{dependency_project}/version"
                    ))
                    .query(&[
                        (
                            "game_versions",
                            serde_json::json!([game_version]).to_string(),
                        ),
                        ("loaders", serde_json::json!([loader]).to_string()),
                        ("include_changelog", "false".to_owned()),
                    ])
                    .send()
                    .await
                    .map_err(|error| {
                        format!("No se pudo buscar una dependencia obligatoria: {error}")
                    })?
                    .error_for_status()
                    .map_err(|error| {
                        format!("No se pudo descargar una dependencia obligatoria: {error}")
                    })?
                    .json()
                    .await
                    .map_err(|error| error.to_string())?;
                let Some(version) = versions.into_iter().next() else {
                    return Err(format!(
                        "Una dependencia requerida no tiene versión compatible: {dependency_project}"
                    ));
                };
                version
            } else {
                continue;
            };
            pending.push_back(dependency_version);
        }
    }
    Ok(format!(
        "Se instaló {} desde Modrinth para Minecraft {game_version} ({loader}), con sus dependencias requeridas.",
        installed_files.join(", ")
    ))
}

#[tauri::command]
fn export_modpack_zip(
    username: String,
    version_id: String,
) -> Result<export_policy::ExportResult, String> {
    if version_id.is_empty()
        || version_id == "."
        || version_id == ".."
        || !version_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err("Identificador de versión no válido".into());
    }
    let root = root_dir()?;
    let (game_version, loader) = resolve_mod_game_version(&root, &version_id)?;
    let dir = version_game_dir(&root, &username, &version_id)?;
    let mut mods = fs::read_dir(dir.join("mods"))
        .ok()
        .into_iter()
        .flatten()
        .take(20_000)
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_ok_and(|kind| kind.is_file() && !kind.is_symlink())
        })
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
                && !entry
                    .file_name()
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .ends_with(".disabled.jar")
        })
        .map(|entry| {
            (
                entry.path(),
                format!("mods/{}", entry.file_name().to_string_lossy()),
            )
        })
        .collect::<Vec<_>>();
    mods.sort_by(|a, b| a.1.cmp(&b.1));
    if mods.is_empty() {
        return Err("La instalación no tiene mods .jar activos para exportar".into());
    }
    let mut plan = export_policy::plan(mods, &export_policy::load_records(&dir));
    if loader == "optifine" {
        plan.omitted.push(export_policy::ExportOmission { name:"OptiFine".into(),version:version_id.clone(),filename:format!("versions/{version_id}/{version_id}.json"),reason:"El motor OptiFine se importa por separado desde el sitio del autor; el pack no contiene su binario".into(),project_url:Some("https://optifine.net/downloads".into()) });
    }
    let exports = root.join("exports");
    fs::create_dir_all(&exports).map_err(|e| e.to_string())?;
    let path = exports.join(format!(
        "TP-LAUNCHER-{game_version}-{loader}-{}-{}.zip",
        unix_time_seconds(),
        Uuid::new_v4().simple()
    ));
    let mut manifests = BTreeMap::new();
    manifests.insert("manifest-tps.json".into(),serde_json::json!({
        "format":"minecrafttps-modpack", "formatVersion":1, "minecraftVersion":game_version, "loader":loader,
        "mods":plan.files.len() + plan.references.len(), "sharingPolicy":1,
    }));
    let dependencies =
        mod_studio::export_loader_dependencies(&root, &version_id, &game_version, &loader)?;
    export_policy::write_archive(
        &path,
        "zip",
        plan,
        manifests,
        dependencies,
        &format!("TP LAUNCHER · {game_version} · {loader}"),
    )
}

#[tauri::command]
fn open_modpack_exports_folder(app: AppHandle) -> Result<(), String> {
    let path = root_dir()?.join("exports");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn import_modpack_zip(
    app: AppHandle,
    username: String,
    version_id: String,
    archive_base64: String,
) -> Result<usize, String> {
    const MAX_ARCHIVE_BYTES: usize = 512 * 1024 * 1024;
    if archive_base64.len() > MAX_ARCHIVE_BYTES * 4 / 3 + 8 {
        return Err("El ZIP supera el límite de importación de 512 MB".into());
    }
    if version_id.is_empty()
        || version_id == "."
        || version_id == ".."
        || !version_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return Err("Identificador de versión no válido".into());
    }
    let encoded = archive_base64
        .split_once(',')
        .filter(|(header, _)| header.starts_with("data:") && header.ends_with(";base64"))
        .map(|(_, data)| data)
        .unwrap_or(&archive_base64);
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|e| format!("ZIP no válido: {e}"))?;
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err("El ZIP supera el límite de importación de 512 MB".into());
    }
    let root = root_dir()?;
    let (game, loader) = resolve_mod_game_version(&root, &version_id)?;
    let dir = version_game_dir(&root, &username, &version_id)?;
    export_policy::import_quick_archive(app, dir, bytes, game, loader).await
}

#[tauri::command]
async fn get_version_java_requirement(version: String) -> Result<u8, String> {
    let root = root_dir()?;
    let cached = root
        .join("versions")
        .join(&version)
        .join(format!("{version}.json"));
    let mut requirement_version = version.clone();
    if let Ok(bytes) = fs::read(cached) {
        if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(major) = meta
                .pointer("/javaVersion/majorVersion")
                .and_then(|v| v.as_u64())
            {
                return Ok(major as u8);
            }
            if let Some(base) = meta["inheritsFrom"]
                .as_str()
                .or_else(|| meta["jar"].as_str())
            {
                requirement_version = base.to_owned();
                let base_meta = root
                    .join("versions")
                    .join(base)
                    .join(format!("{base}.json"));
                if let Ok(bytes) = fs::read(base_meta) {
                    if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        if let Some(major) = meta
                            .pointer("/javaVersion/majorVersion")
                            .and_then(|v| v.as_u64())
                        {
                            return Ok(major as u8);
                        }
                    }
                }
            }
        }
    }
    let manifest: VersionManifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .map_err(|e| format!("No se pudo consultar el manifiesto de Java: {e}"))?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == requirement_version)
        .ok_or_else(|| format!("No se encontró el requisito de Java para {requirement_version}"))?;
    let meta: serde_json::Value = reqwest::get(&entry.url)
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let major = meta
        .pointer("/javaVersion/majorVersion")
        .and_then(|value| value.as_u64())
        .unwrap_or(8) as u8;
    Ok(major)
}

fn sha1_matches(bytes: &[u8], expected: Option<&str>) -> bool {
    expected
        .is_none_or(|expected| format!("{:x}", Sha1::digest(bytes)).eq_ignore_ascii_case(expected))
}

fn file_matches(path: &Path, expected_sha1: Option<&str>, expected_size: Option<u64>) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if expected_size.is_some_and(|size| metadata.len() != size) {
        return false;
    }
    if let Some(expected) = expected_sha1 {
        let Ok(bytes) = fs::read(path) else {
            return false;
        };
        return sha1_matches(&bytes, Some(expected));
    }
    metadata.len() > 0
}

fn http_status_explanation(status: reqwest::StatusCode) -> &'static str {
    match status.as_u16() {
        401 | 403 => "el servidor rechazó el acceso al recurso",
        404 => "el servidor no encontró el recurso solicitado",
        408 | 504 => "el servidor agotó el tiempo de espera",
        429 => "el servidor limitó temporalmente las descargas",
        500..=599 => "el servidor remoto informó un fallo temporal",
        _ => "el servidor rechazó la descarga",
    }
}

async fn download_verified(
    client: &reqwest::Client,
    url: &str,
    path: &Path,
    expected_sha1: Option<&str>,
    expected_size: Option<u64>,
) -> Result<(), String> {
    if file_matches(path, expected_sha1, expected_size) {
        return Ok(());
    }
    let response = client.get(url).send().await.map_err(|error| {
        format!(
            "No se pudo conectar para descargar el recurso. URL: {url}. Detalle técnico: {error}"
        )
    })?;
    if !response.status().is_success() {
        let status = response.status();
        return Err(format!(
            "Falló la descarga del recurso (HTTP {}): {}. URL: {url}",
            status.as_u16(),
            http_status_explanation(status)
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Descarga incompleta desde {url}: {error}"))?;
    if expected_size.is_some_and(|size| bytes.len() as u64 != size) {
        return Err(format!(
            "Tamaño incorrecto al descargar {url}: se esperaban {} bytes y llegaron {}",
            expected_size.unwrap_or_default(),
            bytes.len()
        ));
    }
    if !sha1_matches(&bytes, expected_sha1) {
        return Err(format!("La verificación SHA-1 falló para {url}"));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension(format!("download-{}", std::process::id()));
    fs::write(&temporary, &bytes).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })
}

fn version_game_dir(root: &Path, username: &str, version_id: &str) -> Result<PathBuf, String> {
    if username.is_empty()
        || username.contains(['/', '\\', ':'])
        || username == "."
        || username == ".."
    {
        return Err("Nombre de jugador no válido para la carpeta de la instancia".into());
    }
    let player_dir = root.join("instances").join(username);
    let (_, loader) = resolve_mod_game_version(root, version_id)?;
    if loader == "vanilla" {

        Ok(player_dir)
    } else {
        Ok(player_dir.join("profiles").join(version_id))
    }
}

fn maven_library_path(coordinate: &str) -> Option<String> {
    let (coordinate, extension) = coordinate
        .split_once('@')
        .map_or((coordinate, "jar"), |(coordinate, extension)| {
            (coordinate, extension)
        });
    let parts = coordinate.split(':').collect::<Vec<_>>();
    if parts.len() < 3 || parts.len() > 4 || parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    let valid_maven_segment = |segment: &str| {
        !segment.is_empty()
            && segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '+' | '-')
            })
            && !segment.contains("..")
    };
    if !parts[0].split('.').all(valid_maven_segment)
        || !parts[1..].iter().all(|part| valid_maven_segment(part))
        || !extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = parts
        .get(3)
        .map(|value| format!("-{value}"))
        .unwrap_or_default();
    Some(format!(
        "{group}/{artifact}/{version}/{artifact}-{version}{classifier}.{extension}"
    ))
}

fn maven_module_key(coordinate: &str) -> Option<String> {
    let coordinate = coordinate.split('@').next().unwrap_or(coordinate);
    let mut parts = coordinate.split(':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    if group.is_empty() || artifact.is_empty() {
        return None;
    }
    Some(format!("{group}:{artifact}"))
}

async fn install_assets(
    app: &AppHandle,
    root: &Path,
    meta: &serde_json::Value,
    game_dir: &Path,
) -> Result<(), String> {
    let Some(index) = meta.get("assetIndex") else {
        return Ok(());
    };
    let id = index["id"]
        .as_str()
        .ok_or("assetIndex.id ausente en el manifiesto")?;
    let index_path = root
        .join("assets")
        .join("indexes")
        .join(format!("{id}.json"));
    let index_url = index["url"].as_str().ok_or("assetIndex.url ausente")?;
    let client = reqwest::Client::new();
    let index_sha1 = index["sha1"].as_str();
    let index_size = index["size"].as_u64();
    download_verified(&client, index_url, &index_path, index_sha1, index_size).await?;
    let index_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let objects = index_json["objects"]
        .as_object()
        .ok_or("Índice de assets sin objetos")?;
    let total = objects.len().max(1);
    let downloads = objects
        .iter()
        .filter_map(|(logical, value)| {
            let hash = value["hash"].as_str()?;
            let size = value["size"].as_u64();
            let object_path = root
                .join("assets")
                .join("objects")
                .join(&hash[..2])
                .join(hash);
            if file_matches(&object_path, Some(hash), size) {
                return None;
            }
            Some((logical.clone(), hash.to_owned(), size, object_path))
        })
        .collect::<Vec<_>>();
    let count = downloads.len();
    let results = stream::iter(downloads.into_iter().map(|(logical, hash, size, path)| {
        let client = client.clone();
        async move {
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                &hash[..2],
                hash
            );
            download_verified(&client, &url, &path, Some(&hash), size).await?;
            Ok::<_, String>((logical, path))
        }
    }))
    .buffer_unordered(64)
    .collect::<Vec<_>>()
    .await;
    let mut complete = total - count;
    let mut last_percent = 0;
    for result in results {
        let _ = result?;
        complete += 1;
        let percent = (complete * 100 / total) as u8;
        if percent != last_percent {
            last_percent = percent;
            emit_progress(
                app,
                "Descargando recursos de Minecraft",
                percent,
                &format!("Descargando assets {complete}/{total}"),
            );
        }
    }

    for (logical, value) in objects {
        let hash = value["hash"].as_str().ok_or("Objeto sin hash")?;
        let source = root
            .join("assets")
            .join("objects")
            .join(&hash[..2])
            .join(hash);
        for (enabled, target) in [
            (
                index_json["virtual"].as_bool().unwrap_or(false),
                root.join("assets")
                    .join("virtual")
                    .join("legacy")
                    .join(logical),
            ),
            (
                index_json["map_to_resources"].as_bool().unwrap_or(false),
                game_dir.join("resources").join(logical),
            ),
        ] {
            if enabled && !file_matches(&target, Some(hash), value["size"].as_u64()) {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let temporary = target.with_extension(format!("asset-{}", std::process::id()));
                if temporary.exists() {
                    fs::remove_file(&temporary).map_err(|e| e.to_string())?;
                }
                fs::copy(&source, &temporary).map_err(|e| e.to_string())?;
                if target.exists() {
                    fs::remove_file(&target).map_err(|e| e.to_string())?;
                }
                fs::rename(temporary, &target).map_err(|e| e.to_string())?;
            }
        }
    }
    if count == 0 {
        emit_progress(
            app,
            "Descargando recursos de Minecraft",
            100,
            "Todos los idiomas y sonidos están listos",
        );
    }
    Ok(())
}

#[tauri::command]
fn get_root_path() -> Result<String, String> {
    Ok(root_dir()?.to_string_lossy().into_owned())
}
#[tauri::command]
fn list_mods(username: String, version_id: String) -> Result<Vec<String>, String> {
    let dir = version_game_dir(&root_dir()?, &username, &version_id)?.join("mods");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            name.ends_with(".jar") || name.ends_with(".jar.disabled")
        })
        .map(|e| Ok(e.file_name().to_string_lossy().into_owned()))
        .collect()
}

#[tauri::command]
fn list_shared_mods() -> Result<Vec<String>, String> {
    let dir = root_dir()?.join("mods");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    Ok(fs::read_dir(dir)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|kind| kind.is_file())
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
        })
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect())
}

#[tauri::command]
fn copy_shared_mods(username: String, version_id: String) -> Result<usize, String> {
    let root = root_dir()?;
    let (_, loader) = resolve_mod_game_version(&root, &version_id)?;
    if loader == "vanilla" {
        return Err("Instala y selecciona un perfil Fabric o Forge antes de copiar mods".into());
    }
    let source_dir = root.join("mods");
    let target_dir = version_game_dir(&root, &username, &version_id)?.join("mods");
    fs::create_dir_all(&target_dir).map_err(|error| error.to_string())?;
    let mut copied = 0;
    for entry in fs::read_dir(source_dir)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_ok_and(|kind| kind.is_file())
            || !entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
        {
            continue;
        }
        let target = target_dir.join(entry.file_name());
        if !target.exists() {
            fs::copy(entry.path(), target).map_err(|error| error.to_string())?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn mod_file_path(
    root: &Path,
    username: &str,
    version_id: &str,
    file_name: &str,
) -> Result<PathBuf, String> {
    if file_name.is_empty()
        || file_name.contains(['/', '\\', ':'])
        || !(file_name.to_ascii_lowercase().ends_with(".jar")
            || file_name.to_ascii_lowercase().ends_with(".jar.disabled"))
    {
        return Err("Nombre de archivo de mod no válido".into());
    }
    Ok(version_game_dir(root, username, version_id)?
        .join("mods")
        .join(file_name))
}

#[tauri::command]
fn set_mod_enabled(
    username: String,
    version_id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {
    let root = root_dir()?;
    let source = mod_file_path(&root, &username, &version_id, &file_name)?;
    if !source.is_file() {
        return Err("No se encontró el mod en esta instalación".into());
    }
    let lower = file_name.to_ascii_lowercase();
    let target_name = if enabled {
        file_name
            .strip_suffix(".disabled")
            .or_else(|| file_name.strip_suffix(".DISABLED"))
            .unwrap_or(&file_name)
            .to_owned()
    } else if lower.ends_with(".jar") {
        format!("{file_name}.disabled")
    } else {
        return Ok(());
    };
    let target = mod_file_path(&root, &username, &version_id, &target_name)?;
    if target.exists() {
        return Err("Ya existe un archivo con ese nombre de mod".into());
    }
    fs::rename(source, target)
        .map_err(|error| format!("No se pudo cambiar el estado del mod: {error}"))
}

#[tauri::command]
fn remove_mod(username: String, version_id: String, file_name: String) -> Result<(), String> {
    let path = mod_file_path(&root_dir()?, &username, &version_id, &file_name)?;
    fs::remove_file(path).map_err(|error| format!("No se pudo quitar el mod: {error}"))
}
#[tauri::command]
fn open_mods_folder(app: AppHandle, username: String, version_id: String) -> Result<(), String> {
    let path = version_game_dir(&root_dir()?, &username, &version_id)?.join("mods");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

fn normalize_bootstrap_ignore_list(main: &str, client_jar: &Path, args: &mut Vec<String>) {
    if !matches!(
        main,
        "cpw.mods.bootstraplauncher.BootstrapLauncher" | "cpw.mods.modlauncher.Launcher"
    ) {
        return;
    }



    let Some(name) = client_jar.file_name().and_then(|n| n.to_str()) else {
        return;
    };
    if let Some(arg) = args.iter_mut().find(|arg| arg.starts_with("-DignoreList=")) {
        if !arg
            .trim_start_matches("-DignoreList=")
            .split(',')
            .any(|item| item == name)
        {
            arg.push(',');
            arg.push_str(name);
        }
    } else {
        args.push(format!("-DignoreList={name}"));
    }
}

fn append_game_preferences(
    args: &mut Vec<String>,
    main: &str,
    game_id: &str,
    preferences: &launcher_settings::LauncherSettings,
    quick_play_manifest: bool,
    server_address: Option<&str>,
) -> Result<(), String> {
    let version = game_id
        .split('.')
        .take(2)
        .map(|n| n.parse::<u32>().unwrap_or(0))
        .collect::<Vec<_>>();
    let modern = version.first().is_some_and(|major| *major > 1);
    let since = |minor| {
        modern
            || (version.first() == Some(&1)
                && version.get(1).is_some_and(|current| *current >= minor))
    };
    let supports_options = main == "net.minecraft.client.main.Main" || since(6);
    if supports_options {

        let mut index = 0;
        while index < args.len() {
            if ["--width", "--height"].contains(&args[index].as_str()) {
                args.remove(index);
                if index < args.len() {
                    args.remove(index);
                }
            } else if args[index] == "--fullscreen" {
                args.remove(index);
            } else {
                index += 1;
            }
        }
        args.extend([
            "--width".into(),
            preferences.resolution_width.to_string(),
            "--height".into(),
            preferences.resolution_height.to_string(),
        ]);
        if preferences.fullscreen {
            args.push("--fullscreen".into());
        }
    }
    if let Some(address) = server_address.filter(|value| !value.is_empty()) {
        let server = servers::parse_address(address)?;
        if quick_play_manifest || since(20) {
            args.extend(["--quickPlayMultiplayer".into(), server.display()]);
        } else {
            args.extend([
                "--server".into(),
                server.host,
                "--port".into(),
                server.port.to_string(),
            ]);
        }
    }
    Ok(())
}
#[cfg(test)]
mod launch_preferences_tests {
    use super::*;
    #[test]
    fn hud_diagnostic_identifies_agent_bootstrap_failure() {
        let details = "Unexpected error (103) returned by AddToSystemClassLoaderSearch\nFATAL ERROR in native method: processing of -javaagent failed, appending to system class path failed";
        let message = diagnose_game_exit("26.3", Some(1), details);
        assert!(message.contains("Java rechazó un agente antes de iniciar Minecraft"));
        assert!(message.contains("comprobación previa"));
    }
    #[test]
    #[ignore = "Abre el cliente de un modpack Fabric de prueba hasta inicializar recursos y audio; luego cierra solo ese proceso"]
    fn fabric_real_client_smoke() {
        client_smoke("fabric", "TPSSmokeTest");
    }
    #[test]
    #[ignore = "Abre un cliente NeoForge de prueba y comprueba carga de recursos y audio"]
    fn neoforge_real_client_smoke() {
        client_smoke("neoforge", "TPSNeoTest");
    }
    fn client_smoke(loader: &str, test_name: &str) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let pack = mod_studio::studio_list_packs()
            .unwrap()
            .into_iter()
            .find(|p| {
                p.loader == loader && p.game_version == "1.21.1" && p.install_state == "ready"
            })
            .expect("Ejecuta primero modrinth_real_pack_installation");
        let dir = mod_studio::pack_game_dir(&pack.id).unwrap();
        let mut context = tauri::generate_context!();
        context.config_mut().app.windows.clear();
        let app = tauri::Builder::default()
            .any_thread()
            .build(context)
            .unwrap();
        let result = runtime.block_on(launch_game_session(
            app.handle().clone(),
            None,
            test_name.into(),
            pack.version_id,
            4,
            Some(pack.id.clone()),
            Some(false),
            None,
        ));
        println!("Inicio: {result:?}");
        result.unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
        let log = dir.join("logs/latest.log");
        let mut initialized = false;
        while std::time::Instant::now() < deadline {
            let text = fs::read_to_string(&log).unwrap_or_default();
            if text.contains("Sound engine started") && text.contains("Created:") {
                std::thread::sleep(std::time::Duration::from_secs(10));
                initialized = !launcher_tools::test_session_pids(test_name).is_empty();
                break;
            }
            if launcher_tools::test_session_pids(test_name).is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        for pid in launcher_tools::test_session_pids(test_name) {
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .output();
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
        let text = fs::read_to_string(&log).unwrap_or_default();
        println!("Log de la prueba: {}", log.display());
        assert!(
            initialized,
            "El cliente no completó la carga de audio/recursos. {}",
            text.lines().rev().take(35).collect::<Vec<_>>().join("\n")
        );
    }
    #[test]
    #[ignore = "Abre Vanilla 1.21.1 instalado con perfil local, verifica recursos/audio y cierra solo el PID de prueba"]
    fn local_vanilla_client_smoke() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let root = root_dir().unwrap();
        assert!(
            root.join("versions/1.21.1/1.21.1.jar").is_file(),
            "Se necesita una instalación local 1.21.1"
        );
        let name = "TPSOfflineTest";
        let dir = version_game_dir(&root, name, "1.21.1").unwrap();
        let mut context = tauri::generate_context!();
        context.config_mut().app.windows.clear();
        let app = tauri::Builder::default()
            .any_thread()
            .build(context)
            .unwrap();
        runtime
            .block_on(launch_game_session(
                app.handle().clone(),
                None,
                name.into(),
                "1.21.1".into(),
                4,
                None,
                Some(false),
                None,
            ))
            .expect("El perfil local debe arrancar sin Microsoft ni licencia cacheada");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(150);
        let log = dir.join("logs/latest.log");
        let mut ready = false;
        while std::time::Instant::now() < deadline {
            let text = fs::read_to_string(&log).unwrap_or_default();
            if text.contains("Sound engine started") && text.contains("Created:") {
                ready = !launcher_tools::test_session_pids(name).is_empty();
                break;
            }
            if launcher_tools::test_session_pids(name).is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        for pid in launcher_tools::test_session_pids(name) {
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .output();
            }
        }
        assert!(
            ready,
            "El cliente local no completó recursos/audio. Revisa el log privado de la instancia de prueba"
        );
        println!(
            "Vanilla 1.21.1: perfil local sin Microsoft, recursos y audio inicializados; cerrado solo el proceso de prueba."
        );
    }
    #[test]
    fn modern_forge_ignores_the_actual_inherited_client_jar() {
        let mut args = vec!["-DignoreList=client-extra,neoforge-21.1.248.jar".into()];
        normalize_bootstrap_ignore_list(
            "cpw.mods.bootstraplauncher.BootstrapLauncher",
            Path::new("1.21.1.jar"),
            &mut args,
        );
        assert_eq!(
            args[0],
            "-DignoreList=client-extra,neoforge-21.1.248.jar,1.21.1.jar"
        );
        normalize_bootstrap_ignore_list(
            "cpw.mods.bootstraplauncher.BootstrapLauncher",
            Path::new("1.21.1.jar"),
            &mut args,
        );
        assert_eq!(args[0].matches("1.21.1.jar").count(), 1);
        let original = args.clone();
        normalize_bootstrap_ignore_list(
            "net.minecraft.launchwrapper.Launch",
            Path::new("1.12.2.jar"),
            &mut args,
        );
        assert_eq!(args, original);
    }

    #[test]
    fn classic_clients_keep_their_original_window_argument_contract() {
        let mut args = vec!["Steve".into(), "token:0".into()];
        append_game_preferences(
            &mut args,
            "net.minecraft.launchwrapper.Launch",
            "1.5.2",
            &launcher_settings::LauncherSettings::default(),
            false,
            None,
        )
        .unwrap();
        assert_eq!(args, vec!["Steve", "token:0"]);
    }
    #[test]
    fn modern_and_snapshot_servers_use_quick_play_and_deduplicate_window_options() {
        for version in ["1.21.9", "26.3", "25w01a"] {
            let mut prefs = launcher_settings::LauncherSettings::default();
            prefs.fullscreen = true;
            let mut args = vec![
                "--width".into(),
                "854".into(),
                "--height".into(),
                "480".into(),
            ];
            append_game_preferences(
                &mut args,
                "net.minecraft.client.main.Main",
                version,
                &prefs,
                true,
                Some("[::1]:25566"),
            )
            .unwrap();
            assert_eq!(args.iter().filter(|a| a.as_str() == "--width").count(), 1);
            assert!(
                args.windows(2)
                    .any(|pair| pair == ["--quickPlayMultiplayer", "[::1]:25566"])
            );
            assert!(args.iter().any(|arg| arg == "--fullscreen"));
        }
    }
    #[test]
    fn legacy_server_uses_separate_host_and_port() {
        let mut args = Vec::new();
        append_game_preferences(
            &mut args,
            "net.minecraft.client.main.Main",
            "1.12.2",
            &launcher_settings::LauncherSettings::default(),
            false,
            Some("localhost:25566"),
        )
        .unwrap();
        assert!(
            args.windows(4)
                .any(|pair| pair == ["--server", "localhost", "--port", "25566"])
        );
        assert!(!args.iter().any(|arg| arg == "--quickPlayMultiplayer"));
    }
}

#[tauri::command]
async fn launch_game(
    app: AppHandle,
    auth: State<'_, AuthState>,
    username: String,
    version: String,
    ram_gb: u32,
    instance_id: Option<String>,
    fps_boost: Option<bool>,
    server_address: Option<String>,
) -> Result<String, String> {
    ensure_microsoft_session(&auth).await?;
    let session = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone();
    launch_game_session(
        app,
        session,
        username,
        version,
        ram_gb,
        instance_id,
        fps_boost,
        server_address,
    )
    .await
}
async fn launch_game_session(
    app: AppHandle,
    session: Option<MicrosoftSession>,
    username: String,
    version: String,
    ram_gb: u32,
    instance_id: Option<String>,
    _fps_boost: Option<bool>,
    server_address: Option<String>,
) -> Result<String, String> {
    let username = session
        .as_ref()
        .map(|s| s.profile.name.clone())
        .unwrap_or_else(|| username.trim().to_owned());
    if username.is_empty() {
        return Err("Debes ingresar un nombre de usuario (Nick).".into());
    }
    if session.is_none() {
        launcher_settings::validate_offline_name(&username)?;
    }
    if !(2..=128).contains(&ram_gb) {
        return Err("La memoria debe estar entre 2 y 128 GB.".into());
    }
    let id = version.split_whitespace().next().unwrap_or(&version);
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '+'))
    {
        return Err("Identificador de versión no válido".into());
    }
    let root = root_dir()?;
    let instance = if let Some(pack_id) = instance_id.as_deref().filter(|value| !value.is_empty()) {
        mod_studio::ensure_pack_ready(pack_id)?;
        let directory = mod_studio::pack_game_dir(pack_id)?;
        let pack: serde_json::Value = serde_json::from_slice(
            &fs::read(directory.join("tps-pack.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if pack["version_id"].as_str() != Some(id) {
            return Err("La versión seleccionada no coincide con la de este modpack".into());
        }
        directory
    } else {
        version_game_dir(&root, &username, id)?
    };
    fs::create_dir_all(&instance).map_err(|e| e.to_string())?;
    let manifest: VersionManifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
    let profile = if manifest.versions.iter().any(|entry| entry.id == id) {
        None
    } else {
        let profile_path = root.join("versions").join(id).join(format!("{id}.json"));
        let bytes = fs::read(&profile_path).map_err(|_| {
            format!(
                "No se encontró el perfil instalado {id}; instálalo de nuevo desde Instalaciones"
            )
        })?;
        let profile: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("El perfil {id} está dañado: {error}"))?;
        if !profile["inheritsFrom"].is_string() {
            return Err(format!("El perfil {id} no indica su versión base"));
        }
        Some(profile)
    };
    let patched_client_jar = profile
        .as_ref()
        .and_then(|profile| profile["patchedClientJar"].as_str())
        .map(|relative| {
            let path = Path::new(relative);
            if path.is_absolute()
                || path
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
            {
                return Err("El perfil contiene una ruta de cliente parcheado no válida".to_owned());
            }
            Ok(root.join(path))
        })
        .transpose()?;
    let base_id = profile
        .as_ref()
        .and_then(|profile| {
            profile["inheritsFrom"]
                .as_str()
                .or_else(|| profile["jar"].as_str())
        })
        .unwrap_or(id)
        .to_owned();
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == base_id)
        .ok_or_else(|| format!("Versión base de Minecraft desconocida: {base_id}"))?;
    let version_dir = root.join("versions").join(&base_id);
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
    let base_meta: serde_json::Value = reqwest::get(&entry.url)
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let mut meta = base_meta.clone();
    if let Some(profile) = profile {
        if let Some(main_class) = profile["mainClass"].as_str() {
            meta["mainClass"] = serde_json::Value::String(main_class.to_owned());
        }
        if let Some(legacy_arguments) = profile["minecraftArguments"].as_str() {
            meta["minecraftArguments"] = serde_json::Value::String(legacy_arguments.to_owned());
        }
        if let Some(profile_libraries) = profile["libraries"].as_array() {
            if !meta["libraries"].is_array() {
                meta["libraries"] = serde_json::Value::Array(Vec::new());
            }
            let overridden_modules = profile_libraries
                .iter()
                .filter_map(|library| library["name"].as_str().and_then(maven_module_key))
                .collect::<HashSet<_>>();
            if let Some(base_libraries) = meta["libraries"].as_array_mut() {
                base_libraries.retain(|library| {
                    library["name"]
                        .as_str()
                        .and_then(maven_module_key)
                        .is_none_or(|module| !overridden_modules.contains(&module))
                });
                base_libraries.extend(profile_libraries.iter().cloned());
            }
        }
        for argument_group in ["game", "jvm"] {
            if let Some(profile_args) = profile
                .pointer(&format!("/arguments/{argument_group}"))
                .and_then(|value| value.as_array())
            {
                if !meta
                    .pointer(&format!("/arguments/{argument_group}"))
                    .is_some_and(|value| value.is_array())
                {
                    if !meta["arguments"].is_object() {
                        meta["arguments"] = serde_json::json!({});
                    }
                    meta["arguments"][argument_group] = serde_json::json!([]);
                }
                if let Some(base_args) = meta
                    .pointer_mut(&format!("/arguments/{argument_group}"))
                    .and_then(|value| value.as_array_mut())
                {
                    base_args.extend(profile_args.iter().cloned());
                }
            }
        }
    }
    ensure_legacy_optifine_launchwrapper(&mut meta);
    fs::write(
        version_dir.join(format!("{base_id}.json")),
        serde_json::to_vec_pretty(&base_meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("No se pudo guardar el manifiesto de {base_id}: {e}"))?;
    install_assets(&app, &root, &meta, &instance).await?;
    let client_url = meta["downloads"]["client"]["url"]
        .as_str()
        .ok_or("La versión no publica client.jar")?;
    let base_jar = version_dir.join(format!("{base_id}.jar"));
    let client_hash = meta["downloads"]["client"]["sha1"].as_str();
    let client_size = meta["downloads"]["client"]["size"].as_u64();
    if !file_matches(&base_jar, client_hash, client_size) {
        emit_progress(&app, "Descargando Minecraft", 0, "Descargando cliente…");
        let response = reqwest::get(client_url)
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let total = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();
        let mut data = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            data.extend_from_slice(&chunk);
            let pct = if total > 0 {
                (data.len() as u64 * 100 / total).min(100) as u8
            } else {
                5
            };
            emit_progress(&app, "Descargando Minecraft", pct, "Descargando cliente…");
        }
        if client_size.is_some_and(|expected| expected != data.len() as u64) {
            return Err(format!(
                "El cliente de Minecraft {base_id} llegó incompleto; se volverá a descargar al próximo intento."
            ));
        }
        if !sha1_matches(&data, client_hash) {
            return Err(format!(
                "Falló la verificación del cliente de Minecraft {base_id}; vuelve a intentar la descarga."
            ));
        }
        let temporary = base_jar.with_extension(format!("jar.download-{}", std::process::id()));
        fs::write(&temporary, data).map_err(|e| e.to_string())?;
        if base_jar.exists() {
            fs::remove_file(&base_jar).map_err(|e| e.to_string())?;
        }
        fs::rename(temporary, &base_jar).map_err(|e| e.to_string())?;
    }
    let jar = patched_client_jar.unwrap_or(base_jar);
    if !jar.is_file() {
        return Err(format!(
            "No se encontró el cliente parcheado de OptiFine: {}. Reinstala esa versión.",
            jar.display()
        ));
    }
    let declared_java = meta
        .pointer("/javaVersion/majorVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(8) as u8;
    let java_major = [8u8, 17, 21, 22, 25]
        .into_iter()
        .find(|major| *major >= declared_java.max(8))
        .unwrap_or(declared_java.max(8));
    let mut runtimes = discover_java_installations();
    let mut selected_java = runtimes
        .iter()
        .filter(|java| java.major == java_major && runtime_enabled(java.major))
        .min_by_key(|java| !java.portable)
        .cloned();
    if selected_java.is_none() && java_major != 8 {
        selected_java = runtimes
            .iter()
            .filter(|java| java.major > java_major && runtime_enabled(java.major))
            .min_by_key(|java| java.major)
            .cloned();
    }

    if selected_java.is_none() {
        if !runtime_enabled(java_major) {
            return Err(format!(
                "Java {java_major} está desactivado en Ajustes y no hay otro runtime compatible para {id}."
            ));
        }
        install_portable_java(app.clone(), java_major)
            .await
            .map_err(|error| {
                format!(
                    "{id} requiere Java {java_major}. No se pudo instalar automáticamente: {error}"
                )
            })?;
        runtimes = discover_java_installations();
        selected_java = runtimes
            .into_iter()
            .find(|java| java.major == java_major && runtime_enabled(java.major));
    }
    let selected_java = selected_java.ok_or_else(|| {
        format!(
            "No se encontró un runtime Java {java_major} compatible con {id}. Instálalo o revisa el Gestor de Java."
        )
    })?;
    let running_java = selected_java.major;
    let java_exec = selected_java.path;
    emit_progress(
        &app,
        "Minecraft",
        0,
        &format!(
            "Iniciando {id} con Java {running_java} ({})",
            java_exec.display()
        ),
    );
    let assets = root.join("assets");
    fs::create_dir_all(&assets).map_err(|e| e.to_string())?;


    let libraries = meta["libraries"]
        .as_array()
        .ok_or("Metadatos sin bibliotecas")?;
    let mut classpath = Vec::new();
    let mut uses_embedded_optifine_launchwrapper = false;
    let download_client = reqwest::Client::new();
    let natives_dir = instance.join("natives").join(id);
    fs::create_dir_all(&natives_dir).map_err(|e| e.to_string())?;
    for lib in libraries {
        if !launcher_rules_allow(lib) {
            continue;
        }
        if uses_embedded_optifine_launchwrapper
            && lib["name"].as_str() == Some("net.minecraft:launchwrapper:1.12")
        {
            continue;
        }
        let artifact = lib.pointer("/downloads/artifact");
        let coordinate_path = lib["name"].as_str().and_then(maven_library_path);



        let has_native_metadata =
            lib.get("natives").is_some() || lib.pointer("/downloads/classifiers").is_some();
        let relative_path = artifact
            .and_then(|artifact| artifact["path"].as_str())
            .or_else(|| {
                (!has_native_metadata)
                    .then_some(coordinate_path.as_deref())
                    .flatten()
            });
        if let Some(relative_path) = relative_path {
            let relative = Path::new(relative_path);
            if relative.is_absolute()
                || relative
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
            {
                return Err("El manifiesto contiene una ruta de biblioteca no válida".into());
            }
            let dest = root.join("libraries").join(relative);
            let library_url = artifact
                .and_then(|artifact| artifact["url"].as_str())
                .map(str::to_owned)
                .or_else(|| {
                    let base = lib["url"]
                        .as_str()
                        .unwrap_or("https://libraries.minecraft.net/");
                    coordinate_path
                        .as_ref()
                        .map(|path| format!("{}/{path}", base.trim_end_matches('/')))
                });
            if let Some(url) = library_url {
                emit_launcher_log(
                    &app,
                    "launcher",
                    &format!("Comprobando biblioteca {}", dest.display()),
                );
                let expected_sha1 = artifact
                    .and_then(|artifact| artifact["sha1"].as_str())
                    .or_else(|| lib["sha1"].as_str());
                let expected_size = artifact
                    .and_then(|artifact| artifact["size"].as_u64())
                    .or_else(|| lib["size"].as_u64());
                if lib.get("optifineFileName").is_some()
                    || lib["name"]
                        .as_str()
                        .is_some_and(|name| name.starts_with("optifine:OptiFine:"))
                {
                    if !file_matches(&dest, expected_sha1, expected_size) {
                        return Err("Falta el JAR local de OptiFine o cambió su contenido. Impórtalo de nuevo desde Instalaciones; TP no descarga binarios de OptiFine".into());
                    }
                    let optifine_bytes = fs::read(&dest).map_err(|error| {
                        format!("No se pudo leer el JAR instalado de OptiFine: {error}")
                    })?;
                    if let Some(wrapper_bytes) = extract_optifine_launchwrapper(&optifine_bytes)? {
                        let wrapper_relative = maven_library_path("optifine:launchwrapper-of:2.3")
                            .ok_or("No se pudo construir la ruta de LaunchWrapper OF")?;
                        let wrapper_path = root.join("libraries").join(&wrapper_relative);
                        let wrapper_sha1 = format!("{:x}", Sha1::digest(&wrapper_bytes));
                        let wrapper_size = wrapper_bytes.len() as u64;
                        if !file_matches(&wrapper_path, Some(&wrapper_sha1), Some(wrapper_size)) {
                            if let Some(parent) = wrapper_path.parent() {
                                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                            }
                            let temporary = wrapper_path
                                .with_extension(format!("jar.download-{}", std::process::id()));
                            fs::write(&temporary, &wrapper_bytes)
                                .map_err(|error| error.to_string())?;
                            if wrapper_path.exists() {
                                fs::remove_file(&wrapper_path)
                                    .map_err(|error| error.to_string())?;
                            }
                            fs::rename(&temporary, &wrapper_path).map_err(|error| {
                                format!("No se pudo instalar LaunchWrapper OF: {error}")
                            })?;
                        }
                        emit_launcher_log(
                            &app,
                            "launcher",
                            "OptiFine incluye LaunchWrapper OF 2.3; se usará en lugar del LaunchWrapper clásico incompatible con Java moderno.",
                        );
                        classpath.push(wrapper_path.to_string_lossy().into_owned());
                        uses_embedded_optifine_launchwrapper = true;
                    }
                } else {
                    download_verified(&download_client, &url, &dest, expected_sha1, expected_size)
                        .await?;
                }
                classpath.push(dest.to_string_lossy().into_owned());
            }
        }
        #[cfg(target_os = "windows")]
        if let Some(classifiers) = lib.pointer("/downloads/classifiers") {
            let classifier = lib
                .pointer("/natives/windows")
                .and_then(|v| v.as_str())
                .map(|name| name.replace("${arch}", "64"));
            let native = classifier.as_ref().and_then(|name| classifiers.get(name));
            if let Some(native) = native {
                if let (Some(url), Some(path)) = (native["url"].as_str(), native["path"].as_str()) {
                    let archive_path = root.join("libraries").join(path);
                    download_verified(
                        &download_client,
                        url,
                        &archive_path,
                        native["sha1"].as_str(),
                        native["size"].as_u64(),
                    )
                    .await?;
                    let data = fs::read(&archive_path).map_err(|e| e.to_string())?;
                    if let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(data)) {
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
                            if file.enclosed_name().is_some_and(|p| {
                                p.extension().is_some_and(|x| x.eq_ignore_ascii_case("dll"))
                            }) {
                                let target = natives_dir
                                    .join(file.name().rsplit('/').next().unwrap_or("native.dll"));
                                let mut out =
                                    fs::File::create(target).map_err(|e| e.to_string())?;
                                std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
            }
        }
    }
    if uses_embedded_optifine_launchwrapper {
        classpath.retain(|entry| {
            Path::new(entry).file_name().and_then(|name| name.to_str())
                != Some("launchwrapper-1.12.jar")
        });
    }
    classpath.push(jar.to_string_lossy().into_owned());
    let uuid = session
        .as_ref()
        .map(|s| s.profile.id.clone())
        .unwrap_or_else(|| {
            Uuid::new_v3(
                &Uuid::NAMESPACE_DNS,
                format!("OfflinePlayer:{username}").as_bytes(),
            )
            .simple()
            .to_string()
        });
    let access_token = session
        .as_ref()
        .map(|s| s.access_token.as_str())
        .unwrap_or("0");
    let user_type = if session.is_some() { "msa" } else { "legacy" };
    let auth_session = format!("token:{access_token}");
    let asset_index = meta["assetIndex"]["id"].as_str().unwrap_or("");
    let asset_index_metadata = fs::read(assets.join("indexes").join(format!("{asset_index}.json")))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .unwrap_or_default();
    let game_assets = if meta["assets"].as_str() == Some("pre-1.6") || asset_index == "pre-1.6" {
        instance.join("resources")
    } else if asset_index_metadata["virtual"].as_bool().unwrap_or(false) {
        assets
            .join("virtual")
            .join(meta["assets"].as_str().unwrap_or("legacy"))
    } else {
        assets.clone()
    };
    let library_dir = root.join("libraries");
    let natives_path = natives_dir.to_string_lossy().into_owned();
    let classpath_string = classpath.join(if cfg!(windows) { ";" } else { ":" });
    let replacements = vec![
        ("${auth_player_name}".into(), username.clone()),
        ("${version_name}".into(), id.to_owned()),
        (
            "${game_directory}".into(),
            instance.to_string_lossy().into_owned(),
        ),
        (
            "${assets_root}".into(),
            assets.to_string_lossy().into_owned(),
        ),
        (
            "${game_assets}".into(),
            game_assets.to_string_lossy().into_owned(),
        ),
        ("${assets_index_name}".into(), asset_index.to_owned()),
        ("${auth_uuid}".into(), uuid.clone()),
        ("${auth_access_token}".into(), access_token.to_owned()),
        ("${user_properties}".into(), "{}".into()),
        ("${user_type}".into(), user_type.to_owned()),
        ("${version_type}".into(), entry.kind.clone()),
        ("${auth_session}".into(), auth_session),
        ("${natives_directory}".into(), natives_path.clone()),
        (
            "${library_directory}".into(),
            library_dir.to_string_lossy().into_owned(),
        ),
        ("${classpath}".into(), classpath_string.clone()),
        (
            "${classpath_separator}".into(),
            if cfg!(windows) { ";" } else { ":" }.into(),
        ),
        ("${launcher_name}".into(), "TP LAUNCHER".into()),
        (
            "${launcher_version}".into(),
            env!("CARGO_PKG_VERSION").into(),
        ),
        ("${resolution_width}".into(), "854".into()),
        ("${resolution_height}".into(), "480".into()),
        (
            "${clientid}".into(),
            microsoft_oauth::client_id().unwrap_or_default(),
        ),
        (
            "${auth_xuid}".into(),
            session.as_ref().map(|s| s.xuid.clone()).unwrap_or_default(),
        ),
    ];
    let mut game_args: Vec<String> = if let Some(legacy) = meta["minecraftArguments"].as_str() {
        legacy
            .split_whitespace()
            .map(|arg| substitute_launch_arg(arg, &replacements))
            .collect()
    } else {
        let mut args = Vec::new();
        append_launch_arguments(
            meta.pointer("/arguments/game")
                .unwrap_or(&serde_json::Value::Null),
            &replacements,
            &mut args,
        );
        args
    };



    if !meta["minecraftArguments"].is_string() {
        if !game_args.iter().any(|a| a == "--username") {
            game_args.extend(["--username".into(), username.clone()]);
        }
        if !game_args.iter().any(|a| a == "--version") {
            game_args.extend(["--version".into(), id.into()]);
        }
        if !game_args.iter().any(|a| a == "--gameDir") {
            game_args.extend(["--gameDir".into(), instance.to_string_lossy().into_owned()]);
        }
        if !game_args.iter().any(|a| a == "--assetsDir") {
            game_args.extend(["--assetsDir".into(), assets.to_string_lossy().into_owned()]);
        }
        if !game_args.iter().any(|a| a == "--assetIndex") {
            game_args.extend(["--assetIndex".into(), asset_index.into()]);
        }
        if !game_args.iter().any(|a| a == "--uuid") {
            game_args.extend(["--uuid".into(), uuid.clone()]);
        }
        if !game_args.iter().any(|a| a == "--accessToken") {
            game_args.extend(["--accessToken".into(), access_token.into()]);
        }
        if !game_args.iter().any(|a| a == "--userType") {
            game_args.extend(["--userType".into(), user_type.into()]);
        }
        if !game_args.iter().any(|a| a == "--userProperties") {
            game_args.extend(["--userProperties".into(), "{}".into()]);
        }
    }
    let main = meta["mainClass"]
        .as_str()
        .ok_or("Metadatos sin mainClass")?;
    let preferences = launcher_settings::load()?;
    append_game_preferences(
        &mut game_args,
        main,
        &base_id,
        &preferences,
        meta.pointer("/arguments/game")
            .is_some_and(|args| args.to_string().contains("quickPlayMultiplayer")),
        server_address.as_deref(),
    )?;
    let mut jvm_args = Vec::new();
    append_launch_arguments(
        meta.pointer("/arguments/jvm")
            .unwrap_or(&serde_json::Value::Null),
        &replacements,
        &mut jvm_args,
    );
    normalize_bootstrap_ignore_list(main, &jar, &mut jvm_args);
    if !jvm_args
        .iter()
        .any(|arg| arg == "-cp" || arg == "-classpath" || arg == "--class-path")
    {
        jvm_args.extend(["-cp".into(), classpath_string]);
    }
    if !jvm_args
        .iter()
        .any(|arg| arg.starts_with("-Djava.library.path="))
    {
        jvm_args.push(format!("-Djava.library.path={natives_path}"));
    }
    if !jvm_args.iter().any(|arg| arg.starts_with("-Xms")) {
        jvm_args.push("-Xms1G".into());
    }
    if !jvm_args.iter().any(|arg| arg.starts_with("-Xmx")) {
        jvm_args.push(format!("-Xmx{ram_gb}G"));
    }
    if java_major >= 8
        && launcher_plugins::get_launcher_plugins(app.clone()).is_ok_and(|w| w.profile.low_latency)
    {

        if !jvm_args
            .iter()
            .any(|a| a.starts_with("-XX:+Use") && a.ends_with("GC"))
        {
            jvm_args.extend(
                [
                    "-XX:+UseG1GC",
                    "-XX:+ParallelRefProcEnabled",
                    "-XX:MaxGCPauseMillis=30",
                    "-XX:+UnlockExperimentalVMOptions",
                    "-XX:+DisableExplicitGC",
                ]
                .into_iter()
                .map(str::to_owned),
            );
        }
    }
    let mut process_args = jvm_args;
    if launcher_plugins::get_launcher_plugins(app.clone()).is_ok_and(|w| w.profile.native_overlay) {
        if let Err(error) = launcher_plugins::disable_pause_on_focus_loss(&instance) {
            emit_launcher_log(
                &app,
                "launcher",
                &format!("HUD: no se pudo preparar el ajuste de foco: {error}"),
            );
        }
        emit_progress(&app, "HUD", 96, "Conectando FPS y datos del juego…");
        let forge = meta["libraries"].as_array().is_some_and(|libs| {
            libs.iter().any(|lib| {
                lib["name"]
                    .as_str()
                    .is_some_and(|name| name.starts_with("net.minecraftforge:"))
            })
        });
        match hud_bridge::launch_option(&app, &base_meta, &base_id, forge, &java_exec).await {
            Ok(option) => {
                process_args.push(option);
                emit_launcher_log(
                    &app,
                    "launcher",
                    "HUD: puente local preparado; no requiere permisos ETW.",
                );
            }
            Err(error) => emit_launcher_log(
                &app,
                "launcher",
                &format!(
                    "HUD: {error}. Minecraft se iniciará sin el conector de métricas; el editor y las teclas seguirán disponibles."
                ),
            ),
        }
    }
    process_args.push(format!(
        "-Dtp.launcher.profile={}",
        root_dir()?
            .join("plugins")
            .join("hud-profile.json")
            .to_string_lossy()
    ));
    process_args.push(main.into());
    process_args.extend(game_args);
    let mut command = Command::new(&java_exec);
    command.args(&process_args).current_dir(&instance);
    let mut safe_args = process_args.clone();
    if session.is_some() {
        for arg in &mut safe_args {
            *arg = arg.replace(access_token, "[TOKEN REDACTED]");
        }
    }
    let rendered_args = safe_args
        .iter()
        .map(|arg| format!("{arg:?}"))
        .collect::<Vec<_>>()
        .join(" ");
    emit_launcher_log(
        &app,
        "launcher",
        &format!("Comando: {:?} {rendered_args}", java_exec),
    );
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let game_started_at = std::time::SystemTime::now();
    let mut child = command
        .spawn()
        .map_err(|e| format!("No se pudo iniciar Java: {e}"))?;
    let game_pid = child.id();
    let game_session_guard = game_session::track(
        app.clone(),
        game_pid,
        &username,
        id,
        server_address.as_deref(),
        &instance,
    );
    if let Err(error) = launcher_tools::start_play_session(game_pid, &username, id) {
        emit_launcher_log(
            &app,
            "stderr",
            &format!("No se pudo registrar la sesión: {error}"),
        );
    }
    launcher_settings::after_game_started(&app);
    let stdout = child
        .stdout
        .take()
        .ok_or("No se pudo capturar stdout de Minecraft")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("No se pudo capturar la salida de Java")?;
    let recent_errors = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let stdout_reader = capture_process_output(
        stdout,
        app.clone(),
        "stdout",
        Some(Arc::clone(&recent_errors)),
        instance.clone(),
    );
    let stderr_reader = capture_process_output(
        stderr,
        app.clone(),
        "stderr",
        Some(Arc::clone(&recent_errors)),
        instance.clone(),
    );
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(2500);
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let _ = launcher_tools::checkpoint_play_session(game_pid, true);
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            let mut details = recent_errors
                .lock()
                .map(|lines| lines.iter().cloned().collect::<Vec<_>>().join("\n"))
                .unwrap_or_default();
            if let Some(report) = latest_crash_report_since(&instance, game_started_at) {
                details = if details.trim().is_empty() {
                    report
                } else {
                    format!("{details}\n{report}")
                };
            }
            if let Some(message) = unsupported_java_message(&details, running_java, id) {
                return Err(message);
            }
            return Err(diagnose_game_exit(id, status.code(), &details));
        }
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let monitor_app = app.clone();
    let game_id = id.to_owned();
    let game_instance = instance.clone();
    std::thread::spawn(move || {
        let _game_session_guard = game_session_guard;
        let mut last_checkpoint = std::time::Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Err(error) => break Err(error),
                Ok(None) => {
                    if last_checkpoint.elapsed() >= std::time::Duration::from_secs(15) {
                        if let Err(error) = launcher_tools::checkpoint_play_session(game_pid, false)
                        {
                            emit_launcher_log(
                                &monitor_app,
                                "stderr",
                                &format!("No se pudo guardar el tiempo de juego: {error}"),
                            );
                        }
                        last_checkpoint = std::time::Instant::now();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        };
        let _ = launcher_tools::checkpoint_play_session(game_pid, true);
        let _ = stdout_reader.join();
        let _ = stderr_reader.join();
        match status {
            Ok(status) => {
                let crash_report = latest_crash_report_since(&game_instance, game_started_at);
                if let Some(report) = &crash_report {
                    emit_launcher_log(&monitor_app, "crash-report", report);
                }
                if status.success() && crash_report.is_none() {
                    emit_launcher_log(
                        &monitor_app,
                        "launcher",
                        &format!("Minecraft {game_id} finalizó correctamente ({status})."),
                    );
                } else {
                    let mut details = recent_errors
                        .lock()
                        .map(|lines| lines.iter().cloned().collect::<Vec<_>>().join("\n"))
                        .unwrap_or_default();
                    if let Some(report) = crash_report {
                        details.push('\n');
                        details.push_str(&report);
                    }
                    let message = unsupported_java_message(&details, running_java, &game_id)
                        .unwrap_or_else(|| diagnose_game_exit(&game_id, status.code(), &details));
                    emit_launcher_log(&monitor_app, "stderr", &message);
                }
            }
            Err(error) => emit_launcher_log(
                &monitor_app,
                "stderr",
                &format!("No se pudo consultar el cierre de Minecraft {game_id}: {error}"),
            ),
        }
    });
    emit_progress(&app, "Minecraft", 100, "Cliente iniciado");
    Ok(format!("Minecraft {id} iniciado para {username}."))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            app.manage(AuthState::default());
            launcher_settings::start_presence_worker();
            let show = tauri::menu::MenuItem::with_id(
                app,
                "show",
                "Abrir TP LAUNCHER",
                true,
                None::<&str>,
            )?;
            let quit = tauri::menu::MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show, &quit])?;
            let mut tray = tauri::tray::TrayIconBuilder::new()
                .tooltip("TP LAUNCHER")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..
                        }
                    ) {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            let _ = tray.build(app)?;
            if let Ok(root) = root_dir() {
                let icon = root.join("custom_icon.png");
                if let (Ok(bytes), Some(window)) = (fs::read(icon), app.get_webview_window("main"))
                {
                    if let Ok(image) = normalized_icon(&bytes) {
                        let _ = window.set_icon(tauri::image::Image::new_owned(
                            image.into_raw(),
                            128,
                            128,
                        ));
                    }
                }
            }
            let _ = launcher_display::restore_display(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            launcher_settings::get_launcher_settings,
            game_session::get_game_session,
            launcher_display::get_launcher_display_info,
            launcher_display::preview_launcher_scale,
            launcher_display::apply_launcher_window,
            launcher_settings::get_custom_background,
            launcher_settings::set_custom_background,
            launcher_settings::save_launcher_settings,
            launcher_settings::get_system_summary,
            launcher_settings::test_java_executable,
            launcher_settings::open_launcher_folder,
            launcher_settings::list_launcher_accounts,
            launcher_settings::get_offline_eligibility,
            launcher_settings::add_offline_account,
            launcher_settings::activate_launcher_account,
            launcher_settings::remove_launcher_account,
            launcher_settings::launcher_news,
            poser_assets::get_poser_assets,
            launcher_settings::check_launcher_updates,
            launcher_settings::download_launcher_update,
            launcher_settings::install_launcher_update,
            servers::servers_list,
            servers::servers_save,
            servers::server_ping,
            server_directory::server_directory_search,
            server_directory::server_directory_config,
            server_directory::server_directory_submit,
            server_extras::measure_server_latency,
            server_extras::get_hosting_recommendation,
            servers::lan_status,
            servers::install_e4mc,
            launcher_plugins::get_launcher_plugins,
            launcher_plugins::save_launcher_plugins,
            launcher_plugins::open_launcher_plugins_folder,
            launcher_plugins::prepare_launcher_plugins,
            launcher_social::get_social_profile,
            launcher_social::save_social_profile,
            launcher_overlay::set_launcher_overlay,
            launcher_overlay::get_launcher_overlay,
            launcher_overlay::get_overlay_snapshot,
            launcher_overlay::set_hud_interactive,
            launcher_overlay::publish_launcher_overlay,
            launcher_tools::get_playtime,
            launcher_tools::resolve_player_appearance,
            launcher_tools::get_default_player_appearance,
            launcher_tools::get_player_appearance_by_uuid,
            launcher_tools::list_screenshots,
            launcher_overlay::hud_debug_shortcut,
            hud_controls::hud_menu_background,
            hud_controls::hud_game_content,
            hud_controls::hud_title_background,
            hud_controls::hud_item_icons,
            hud_controls::hud_game_command,
            hud_preview::get_hud_preview,
            hud_preview::set_hud_preview,
            launcher_tools::screenshots_page,
            launcher_tools::screenshot_data,
            launcher_tools::open_screenshot,
            launcher_tools::open_player_folder,
            mod_studio::studio_search,
            mod_studio::studio_categories,
            mod_studio::hud_install_content,
            mod_studio::studio_project,
            mod_studio::studio_dependency_projects,
            mod_studio::studio_loader_versions,
            mod_studio::studio_create_pack,
            mod_studio::studio_list_packs,
            mod_studio::studio_list_pack_mods,
            mod_studio::studio_enrich_mod_art,
            mod_studio::studio_enrich_library_art,
            mod_studio::studio_set_pack_mod_enabled,
            mod_studio::studio_remove_pack_mod,
            mod_studio::studio_open_pack_folder,
            mod_studio::studio_install_version,
            mod_studio::studio_export_pack,
            mod_studio::studio_import_pack,
            mod_studio::studio_retry_pack,
            mod_studio::studio_supply_pack_file,
            studio_progress::studio_current_operation,
            skin_library::list_saved_skins,
            skin_library::save_library_skin,
            skin_library::save_current_skin,
            skin_library::import_library_skin,
            skin_library::rename_library_skin,
            skin_library::delete_library_skin,
            skin_library::apply_library_skin,
            poser::list_custom_poses,
            poser::save_custom_pose,
            poser::delete_custom_pose,
            poser::save_pose_render,
            poser::open_pose_renders,
            check_java,
            install_portable_java,
            list_java_runtimes,
            set_java_runtime_enabled,
            list_installed_versions,
            read_launcher_logs,
            clear_launcher_logs,
            record_launcher_log,
            open_version_folder,
            delete_installed_version,
            set_custom_app_icon,
            remove_custom_app_icon,
            get_custom_app_icon,
            begin_microsoft_login,
            get_microsoft_profile,
            logout_microsoft,
            upload_minecraft_skin,
            activate_minecraft_cape,
            hide_minecraft_cape,
            fetch_mojang_versions,
            fetch_fabric_versions,
            fetch_forge_versions,
            fetch_optifine_versions,
            optifine_discovery::discover_optifine_files,
            optifine_discovery::import_discovered_optifine,
            loader_catalogs::fetch_extra_loader_catalog,
            loader_catalogs::install_extra_loader,
            install_fabric,
            install_forge,
            import_optifine,
            search_modrinth_mods,
            install_modrinth_mod,
            export_modpack_zip,
            open_modpack_exports_folder,
            import_modpack_zip,
            get_version_java_requirement,
            get_root_path,
            list_mods,
            list_shared_mods,
            copy_shared_mods,
            set_mod_enabled,
            remove_mod,
            open_mods_folder,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar TP LAUNCHER");
}

#[cfg(test)]mod optifine_launch_contract_tests{
 use super::*;
 #[test]fn repairs_legacy_profiles_without_duplicate_or_unrelated_libraries(){
 let mut meta=serde_json::json!({"mainClass":"net.minecraft.launchwrapper.Launch","minecraftArguments":"--tweakClass optifine.OptiFineTweaker","libraries":[{"name":"optifine:OptiFine:1.12.2_HD_U_G5"}]});
 ensure_legacy_optifine_launchwrapper(&mut meta);assert_eq!(meta["libraries"].as_array().unwrap().len(),2);assert_eq!(meta["libraries"][1]["downloads"]["artifact"]["sha1"],"111e7bea9c968cdb3d06ef4632bf7ff0824d0f36");ensure_legacy_optifine_launchwrapper(&mut meta);assert_eq!(meta["libraries"].as_array().unwrap().len(),2);
 let mut forge=serde_json::json!({"mainClass":"net.minecraft.launchwrapper.Launch","libraries":[]});ensure_legacy_optifine_launchwrapper(&mut forge);assert!(forge["libraries"].as_array().unwrap().is_empty());
 let mut modern=serde_json::json!({"mainClass":"net.minecraft.launchwrapper.Launch","javaVersion":{"majorVersion":21},"minecraftArguments":"--tweakClass optifine.OptiFineTweaker","libraries":[]});ensure_legacy_optifine_launchwrapper(&mut modern);assert!(modern["libraries"].as_array().unwrap().is_empty());
 }
}
