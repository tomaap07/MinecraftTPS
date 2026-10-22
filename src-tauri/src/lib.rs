use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env, fs,
    io::{BufRead, BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex, OnceLock},
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

const MICROSOFT_CLIENT_ID: &str = "00000000402b5328";
const MICROSOFT_REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
const MICROSOFT_AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf?client_id=00000000402b5328&response_type=code&scope=service%3A%3Auser.auth.xboxlive.com%3A%3AMBI_SSL&redirect_uri=https%3A%2F%2Flogin.live.com%2Foauth20_desktop.srf";

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
    #[serde(rename = "type")]
    kind: String,
}
#[derive(Serialize)]
struct InstalledVersion {
    id: String,
    jar: bool,
    json: bool,
    size_mb: f64,
}
#[derive(Serialize)]
struct RuntimeStatus {
    major: u8,
    installed: bool,
    enabled: bool,
    path: String,
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

    fn protect(input: &[u8]) -> Result<Vec<u8>, String> {
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
            return Err(format!("Windows no pudo proteger la sesión ({}).", unsafe {
                GetLastError()
            }));
        }
        let bytes = unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }
            .to_vec();
        unsafe { LocalFree(output.data.cast()) };
        Ok(bytes)
    }

    fn unprotect(input: &[u8]) -> Result<Vec<u8>, String> {
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
            return Err(format!("Windows no pudo recuperar la sesión ({}).", unsafe {
                GetLastError()
            }));
        }
        let bytes = unsafe { std::slice::from_raw_parts(output.data, output.size as usize) }
            .to_vec();
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

fn root_dir() -> Result<PathBuf, String> {
    let root = dirs::data_dir()
        .ok_or("No se pudo localizar AppData")?
        .join("MinecraftTPS");
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
    let _ = app.emit(
        "launcher-log",
        payload,
    );
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
    let mut recent = VecDeque::with_capacity(10_000);
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Ok(entry) = serde_json::from_str::<LauncherLogPayload>(&line) {
            recent.push_back(entry);
            if recent.len() > 10_000 {
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
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
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

fn latest_crash_report_since(instance: &Path, launched_at: std::time::SystemTime) -> Option<String> {
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
        if truncated { "\n[Informe truncado a 20.000 caracteres]" } else { "" }
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
    for entry in fs::read_dir(root)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
    {
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        let jar = entry.path().join(format!("{id}.jar")).exists();
        let json = entry.path().join(format!("{id}.json")).exists();
        let size_mb = directory_size(&entry.path()) as f64 / 1_048_576.0;
        result.push(InstalledVersion {
            id,
            jar,
            json,
            size_mb,
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
        microsoft_token.to_owned(),
        format!("t={microsoft_token}"),
        format!("d={microsoft_token}"),
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
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            continue;
        }
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Xbox Live rechazó la cuenta ({status}): {body}"));
        }
        xbl = Some(response.json::<serde_json::Value>().await.map_err(|e| e.to_string())?);
        break;
    }
    let xbl = xbl.ok_or("Xbox Live rechazó el access_token con los formatos puro, t= y d=.")?;
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
    let xsts_token = xsts["Token"].as_str().ok_or("Xbox no devolvió token XSTS")?;
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
    let profile = minecraft_profile(client, &minecraft_token).await?;
    Ok((profile, minecraft_token, xuid))
}

#[tauri::command]
async fn begin_microsoft_login(
    app: AppHandle,
    auth: State<'_, AuthState>,
) -> Result<MinecraftProfile, String> {
    let label = "microsoft-oauth-login";
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.set_focus();
        return Err("Ya hay una ventana de inicio de sesión abierta.".into());
    }
    let (callback_tx, mut callback_rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<String, String>>();
    let navigation_tx = callback_tx.clone();
    let auth_url = url::Url::parse(MICROSOFT_AUTH_URL).map_err(|e| e.to_string())?;
    let login_window =
        tauri::WebviewWindowBuilder::new(&app, label, tauri::WebviewUrl::External(auth_url))
            .title("Iniciar sesión con Microsoft")
            .inner_size(520.0, 720.0)
            .center()
            .on_navigation(move |url| {
                let is_callback = url.scheme() == "https"
                    && url.host_str() == Some("login.live.com")
                    && url.path() == "/oauth20_desktop.srf";
                if is_callback {
                    let result = url
                        .query_pairs()
                        .find(|(key, _)| key == "code")
                        .map(|(_, value)| value.into_owned())
                        .ok_or_else(|| {
                            url.query_pairs()
                                .find(|(key, _)| key == "error_description" || key == "error")
                                .map(|(_, value)| {
                                    format!("Microsoft rechazó el inicio de sesión: {value}")
                                })
                                .unwrap_or_else(|| {
                                    "Microsoft no devolvió un código de autorización".into()
                                })
                        });
                    let _ = navigation_tx.send(result);
                    return false;
                }
                true
            })
            .build()
            .map_err(|e| format!("No se pudo abrir la ventana de Microsoft: {e}"))?;
    login_window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
            let _ = callback_tx.send(Err("Inicio de sesión cancelado.".into()));
        }
    });
    let code = tokio::time::timeout(std::time::Duration::from_secs(300), callback_rx.recv())
        .await
        .map_err(|_| "Se agotó el tiempo para iniciar sesión con Microsoft.".to_string())?
        .ok_or("La ventana de inicio de sesión se cerró")??;
    let _ = login_window.close();
    let client = reqwest::Client::new();
    let token_response: serde_json::Value = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&[
            ("client_id", MICROSOFT_CLIENT_ID),
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", MICROSOFT_REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(error) = token_response["error_description"]
        .as_str()
        .or_else(|| token_response["error"].as_str())
    {
        return Err(format!("Microsoft no pudo completar OAuth: {error}"));
    }
    let microsoft_token = token_response["access_token"]
        .as_str()
        .ok_or("Microsoft no devolvió access_token")?;
    let refresh_token = token_response["refresh_token"]
        .as_str()
        .ok_or("Microsoft no devolvió refresh_token; vuelve a autorizar la cuenta para mantener la sesión entre reinicios.")?;
    let (profile, minecraft_token, xuid) =
        minecraft_session_from_microsoft_token(&client, microsoft_token).await?;
    let _auth_guard = auth.refresh_lock.lock().await;
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
    Ok(profile)
}

fn unix_time_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn ensure_microsoft_session(
    auth: &AuthState,
) -> Result<Option<MinecraftProfile>, String> {
    let _refresh_guard = auth.refresh_lock.lock().await;
    let current = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone();
    if let Some(session) = &current {
        if session.expires_at > unix_time_seconds() + 60 {
            return Ok(Some(session.profile.clone()));
        }
    }
    let refresh_token = match current {
        Some(session) => session.refresh_token,
        None => match token_storage::load_refresh_token()? {
            Some(token) => token,
            None => return Ok(None),
        },
    };
    let client = reqwest::Client::new();
    let token_response: serde_json::Value = client
        .post("https://login.live.com/oauth20_token.srf")
        .form(&[
            ("client_id", MICROSOFT_CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("redirect_uri", MICROSOFT_REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|e| format!("No se pudo renovar la sesión Microsoft: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Microsoft devolvió una respuesta inválida: {e}"))?;
    if let Some(error) = token_response["error_description"]
        .as_str()
        .or_else(|| token_response["error"].as_str())
    {
        if token_response["error"].as_str() == Some("invalid_grant") {
            let _ = token_storage::clear_saved_refresh_token();
            if let Ok(mut state) = auth.session.lock() {
                *state = None;
            }
        }
        return Err(format!("La sesión Microsoft necesita volver a autenticarse: {error}"));
    }
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
    Ok(Some(profile))
}
#[tauri::command]
async fn get_microsoft_profile(auth: State<'_, AuthState>) -> Result<Option<MinecraftProfile>, String> {
    ensure_microsoft_session(&auth).await
}
#[tauri::command]
async fn logout_microsoft(auth: State<'_, AuthState>) -> Result<(), String> {
    let _auth_guard = auth.refresh_lock.lock().await;
    *auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")? = None;
    token_storage::clear_saved_refresh_token()
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
    let manifest: VersionManifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
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
async fn get_version_java_requirement(version: String) -> Result<u8, String> {
    let root = root_dir()?;
    let cached = root
        .join("versions")
        .join(&version)
        .join(format!("{version}.json"));
    if let Ok(bytes) = fs::read(cached) {
        if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(major) = meta.pointer("/javaVersion/majorVersion").and_then(|v| v.as_u64()) {
                return Ok(major as u8);
            }
        }
    }
    let manifest: VersionManifest = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .await
        .map_err(|e| format!("No se pudo consultar el manifiesto de Java: {e}"))?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let entry = manifest.versions.iter().find(|entry| entry.id == version)
        .ok_or_else(|| format!("No se encontró el requisito de Java para {version}"))?;
    let meta: serde_json::Value = reqwest::get(&entry.url)
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let major = meta.pointer("/javaVersion/majorVersion")
        .and_then(|value| value.as_u64())
        .unwrap_or(8) as u8;
    Ok(major)
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
    if !index_path.exists() {
        let url = index["url"].as_str().ok_or("assetIndex.url ausente")?;
        let bytes = reqwest::get(url)
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&index_path, bytes).map_err(|e| e.to_string())?;
    }
    let index_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&index_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let objects = index_json["objects"]
        .as_object()
        .ok_or("Índice de assets sin objetos")?;
    let total = objects.len().max(1);
    let client = reqwest::Client::new();
    let downloads = objects
        .iter()
        .filter_map(|(logical, value)| {
            let hash = value["hash"].as_str()?;
            let object_path = root
                .join("assets")
                .join("objects")
                .join(&hash[..2])
                .join(hash);
            if object_path.exists() {
                return None;
            }
            Some((logical.clone(), hash.to_owned(), object_path))
        })
        .collect::<Vec<_>>();
    let count = downloads.len();
    let results = stream::iter(downloads.into_iter().map(|(logical, hash, path)| {
        let client = client.clone();
        async move {
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                &hash[..2],
                hash
            );
            let bytes = client
                .get(url)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .bytes()
                .await
                .map_err(|e| e.to_string())?;
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&path, &bytes).map_err(|e| e.to_string())?;
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
    // Los índices legacy esperan también los objetos proyectados a rutas tradicionales.
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
            if enabled && !target.exists() {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::copy(&source, &target).map_err(|e| e.to_string())?;
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
fn list_mods() -> Result<Vec<String>, String> {
    let dir = root_dir()?.join("mods");
    fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("jar"))
        })
        .map(|e| Ok(e.file_name().to_string_lossy().into_owned()))
        .collect()
}
#[tauri::command]
fn open_mods_folder(app: AppHandle) -> Result<(), String> {
    let path = root_dir()?.join("mods");
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_game(
    app: AppHandle,
    auth: State<'_, AuthState>,
    username: String,
    version: String,
    ram_gb: u32,
) -> Result<String, String> {
    ensure_microsoft_session(&auth).await?;
    let session = auth
        .session
        .lock()
        .map_err(|_| "Estado de cuenta bloqueado")?
        .clone();
    let username = session
        .as_ref()
        .map(|s| s.profile.name.clone())
        .unwrap_or_else(|| username.trim().to_owned());
    if username.is_empty() {
        return Err("Debes ingresar un nombre de usuario (Nick).".into());
    }
    if !(2..=16).contains(&ram_gb) {
        return Err("La memoria debe estar entre 2 y 16 GB.".into());
    }
    let id = version.split_whitespace().next().unwrap_or(&version);
    let root = root_dir()?;
    let instance = root.join("instances").join(&username);
    fs::create_dir_all(&instance).map_err(|e| e.to_string())?;
    let version_dir = root.join("versions").join(id);
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
    let manifest: VersionManifest =
        reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == id)
        .ok_or_else(|| format!("Versión Mojang desconocida: {id}"))?;
    let meta: serde_json::Value = reqwest::get(&entry.url)
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    fs::write(
        version_dir.join(format!("{id}.json")),
        serde_json::to_vec_pretty(&meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("No se pudo guardar el manifiesto de {id}: {e}"))?;
    install_assets(&app, &root, &meta, &instance).await?;
    let client = meta["downloads"]["client"]["url"]
        .as_str()
        .ok_or("La versión no publica client.jar")?;
    let jar = version_dir.join(format!("{id}.jar"));
    if !jar.exists() {
        emit_progress(&app, "Descargando Minecraft", 0, "Descargando cliente…");
        let response = reqwest::get(client)
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
        fs::write(&jar, data).map_err(|e| e.to_string())?;
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
    // Legacy clients (including betas) are more reliable on Java 8 than on a much newer JVM.
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
    // Hasta resolver los artefactos y reglas del JSON de cada versión no se puede formar
    // un classpath válido; el cliente Mojang por sí solo no es ejecutable con Java.
    let libraries = meta["libraries"]
        .as_array()
        .ok_or("Metadatos sin bibliotecas")?;
    let mut classpath = Vec::new();
    let natives_dir = version_dir.join("natives");
    fs::create_dir_all(&natives_dir).map_err(|e| e.to_string())?;
    for lib in libraries {
        if !launcher_rules_allow(lib) {
            continue;
        }
        if let Some(artifact) = lib.pointer("/downloads/artifact") {
            if let (Some(url), Some(path)) = (artifact["url"].as_str(), artifact["path"].as_str()) {
                let dest = root.join("libraries").join(path);
                if !dest.exists() {
                    emit_launcher_log(
                        &app,
                        "launcher",
                        &format!("Descargando biblioteca {}", dest.display()),
                    );
                    if let Some(p) = dest.parent() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                    let b = reqwest::get(url)
                        .await
                        .map_err(|e| e.to_string())?
                        .error_for_status()
                        .map_err(|e| e.to_string())?
                        .bytes()
                        .await
                        .map_err(|e| e.to_string())?;
                    fs::write(&dest, b).map_err(|e| e.to_string())?;
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
            let native = classifier
                .as_ref()
                .and_then(|name| classifiers.get(name))
                .or_else(|| classifiers.get("natives-windows-64"))
                .or_else(|| classifiers.get("natives-windows"));
            if let Some(native) = native {
                if let (Some(url), Some(path)) = (native["url"].as_str(), native["path"].as_str()) {
                    let archive_path = root.join("libraries").join(path);
                    if !archive_path.exists() {
                        emit_launcher_log(
                            &app,
                            "launcher",
                            &format!("Descargando nativos {}", archive_path.display()),
                        );
                        if let Some(parent) = archive_path.parent() {
                            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                        }
                        let data = reqwest::get(url)
                            .await
                            .map_err(|e| e.to_string())?
                            .error_for_status()
                            .map_err(|e| e.to_string())?
                            .bytes()
                            .await
                            .map_err(|e| e.to_string())?;
                        fs::write(&archive_path, data).map_err(|e| e.to_string())?;
                    }
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
    let asset_index_metadata = fs::read(
        assets.join("indexes").join(format!("{asset_index}.json")),
    )
    .ok()
    .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
    .unwrap_or_default();
    let game_assets = if meta["assets"].as_str() == Some("pre-1.6")
        || asset_index == "pre-1.6"
    {
        instance.join("resources")
    } else if asset_index_metadata["virtual"].as_bool().unwrap_or(false) {
        assets.join("virtual").join(
            meta["assets"]
                .as_str()
                .unwrap_or("legacy"),
        )
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
        ("${launcher_name}".into(), "MinecraftTPS".into()),
        (
            "${launcher_version}".into(),
            env!("CARGO_PKG_VERSION").into(),
        ),
        ("${resolution_width}".into(), "854".into()),
        ("${resolution_height}".into(), "480".into()),
        ("${clientid}".into(), MICROSOFT_CLIENT_ID.into()),
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
    // Pre-1.13 metadata defines its complete positional/flag contract in
    // minecraftArguments. Injecting modern flags into these clients produces
    // ignored arguments and can break legacy argument parsing.
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
    let mut jvm_args = Vec::new();
    append_launch_arguments(
        meta.pointer("/arguments/jvm")
            .unwrap_or(&serde_json::Value::Null),
        &replacements,
        &mut jvm_args,
    );
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
    let mut process_args = jvm_args;
    process_args.push(main.into());
    process_args.extend(game_args);
    let mut command = Command::new(&java_exec);
    command.args(&process_args);
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
    command.stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let game_started_at = std::time::SystemTime::now();
    let mut child = command
        .spawn()
        .map_err(|e| format!("No se pudo iniciar Java: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or("No se pudo capturar stdout de Minecraft")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("No se pudo capturar la salida de Java")?;
    let recent_errors = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let stdout_reader = capture_process_output(stdout, app.clone(), "stdout", None);
    let stderr_reader = capture_process_output(
        stderr,
        app.clone(),
        "stderr",
        Some(Arc::clone(&recent_errors)),
    );
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(2500);
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
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
            return Err(if details.trim().is_empty() {
                format!("Minecraft se cerró al iniciar (código {}).", status)
            } else {
                format!("Minecraft se cerró al iniciar:\n{details}")
            });
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
        let status = child.wait();
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
                    let details = recent_errors
                        .lock()
                        .map(|lines| lines.iter().cloned().collect::<Vec<_>>().join("\n"))
                        .unwrap_or_default();
                    let message = unsupported_java_message(&details, running_java, &game_id)
                        .unwrap_or_else(|| {
                            if crash_report.is_some() {
                                format!("Minecraft {game_id} generó un informe de crash. Revisa la entrada crash-report en la consola.")
                            } else {
                                format!("Minecraft {game_id} finalizó con error ({status}).")
                            }
                        });
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
        .setup(|app| {
            app.manage(AuthState::default());
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
            get_custom_app_icon,
            begin_microsoft_login,
            get_microsoft_profile,
            logout_microsoft,
            upload_minecraft_skin,
            activate_minecraft_cape,
            hide_minecraft_cape,
            fetch_mojang_versions,
            get_version_java_requirement,
            get_root_path,
            list_mods,
            open_mods_folder,
            launch_game
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar MinecraftTPS");
}
