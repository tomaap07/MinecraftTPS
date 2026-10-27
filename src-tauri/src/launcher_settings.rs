use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherSettings {
    pub language: String,
    pub launch_behavior: String,
    pub data_root: String,
    pub discord_presence: bool,
    pub discord_application_id: String,
    pub automatic_updates: bool,
    pub first_run_completed: bool,
    pub ui_scale_percent: u32,
    pub launcher_width: u32,
    pub launcher_height: u32,
    pub launcher_maximized: bool,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub fullscreen: bool,
    pub ram_gb: u32,
    pub fps_boost: bool,
    pub gc_mode: String,
}
impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            language: "es-419".into(),
            launch_behavior: "keep".into(),
            data_root: String::new(),
            discord_presence: false,
            discord_application_id: String::new(),
            automatic_updates: true,
            first_run_completed: false,
            ui_scale_percent: 100,
            launcher_width: 1280,
            launcher_height: 820,
            launcher_maximized: false,
            resolution_width: 1280,
            resolution_height: 720,
            fullscreen: false,
            ram_gb: 4,
            fps_boost: false,
            gc_mode: "g1gc".into(),
        }
    }
}
pub(crate) fn config_root() -> Result<PathBuf, String> {
    let path = dirs::data_dir()
        .ok_or("No se pudo localizar AppData")?
        .join("LauncherTPS");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}
pub fn load() -> Result<LauncherSettings, String> {
    let file = config_root()?.join("launcher-settings.json");
    if !file.exists() {
        return Ok(LauncherSettings::default());
    }
    let mut bytes = Vec::new();
    fs::File::open(file)
        .map_err(|e| e.to_string())?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("La configuración supera el límite de 64 KiB".into());
    }
    let mut settings: LauncherSettings = serde_json::from_slice(&bytes)
        .map_err(|e| format!("La configuración local está dañada: {e}"))?;

    settings.fps_boost = false;
    settings.gc_mode = "g1gc".into();


    settings.ui_scale_percent = settings.ui_scale_percent.clamp(80, 200);
    settings.launcher_width = settings.launcher_width.clamp(320, 7680);
    settings.launcher_height = settings.launcher_height.clamp(240, 4320);
    Ok(settings)
}
pub fn data_root() -> Result<PathBuf, String> {
    let prefs = load()?;
    if !prefs.data_root.is_empty() {
        let root = PathBuf::from(prefs.data_root);
        if !root.is_absolute() {
            return Err("La carpeta de datos debe tener una ruta absoluta".into());
        }
        return Ok(root);
    }
    let base = dirs::data_dir().ok_or("No se pudo localizar AppData")?;
    let legacy = base.join("MinecraftTPS");

    Ok(if legacy.exists() {
        legacy
    } else {
        base.join("LauncherTPS")
    })
}
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!("{}.pending", uuid::Uuid::new_v4()));
    let mut staged = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    if let Err(error) = staged.write_all(bytes).and_then(|_| staged.sync_all()) {
        drop(staged);
        let _ = fs::remove_file(&temp);
        return Err(error.to_string());
    }
    drop(staged);
    fs::rename(&temp, path).map_err(|e| {
        let _ = fs::remove_file(&temp);
        e.to_string()
    })
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    write_atomic(
        path,
        &serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
}
#[tauri::command]
pub fn get_launcher_settings() -> Result<LauncherSettings, String> {
    let mut settings = load()?;
    if settings.data_root.is_empty() {
        settings.data_root = data_root()?.to_string_lossy().into_owned();
    }
    Ok(settings)
}
#[tauri::command]
pub fn save_launcher_settings(mut settings: LauncherSettings) -> Result<LauncherSettings, String> {
    settings.fps_boost = false;
    settings.gc_mode = "g1gc".into();
    if !["keep", "minimize", "tray", "close"].contains(&settings.launch_behavior.as_str())
        || !(2..=128).contains(&settings.ram_gb)
        || !(320..=7680).contains(&settings.resolution_width)
        || !(240..=4320).contains(&settings.resolution_height)
        || !(80..=200).contains(&settings.ui_scale_percent)
        || !(320..=7680).contains(&settings.launcher_width)
        || !(240..=4320).contains(&settings.launcher_height)
        || settings.language.len() > 32
        || !["g1gc", "zgc"].contains(&settings.gc_mode.as_str())
    {
        return Err("Los valores de configuración no son válidos".into());
    }
    if !settings.discord_application_id.is_empty()
        && !valid_discord_id(&settings.discord_application_id)
    {
        return Err("El ID de aplicación de Discord debe tener entre 15 y 22 dígitos y ser un identificador válido".into());
    }
    if !settings.data_root.is_empty() {
        let path = Path::new(&settings.data_root);
        if !path.is_absolute() {
            return Err("La carpeta de datos debe ser una ruta absoluta".into());
        }
        fs::create_dir_all(path).map_err(|e| format!("No se puede usar esa carpeta: {e}"))?;
    }
    write_json(&config_root()?.join("launcher-settings.json"), &settings)?;
    Ok(settings)
}

fn bounded_file(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > max {
        return Err("El archivo local supera el límite permitido".into());
    }
    Ok(bytes)
}

fn background_png(encoded: &str) -> Result<Vec<u8>, String> {
    const MAX: usize = 16 * 1024 * 1024;
    if encoded.len() > MAX * 4 / 3 + 128 {
        return Err("El fondo supera 16 MiB".into());
    }
    let data = if let Some((mime, value)) = encoded.split_once(',') {
        if ![
            "data:image/png;base64",
            "data:image/jpeg;base64",
            "data:image/jpg;base64",
            "data:image/webp;base64",
        ]
        .contains(&mime)
        {
            return Err("Selecciona una imagen PNG, JPEG o WebP".into());
        }
        value
    } else {
        encoded
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| "El fondo no contiene una imagen Base64 válida")?;
    if bytes.len() > MAX {
        return Err("El fondo supera 16 MiB".into());
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "El fondo no es una imagen válida")?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::WebP)
    ) {
        return Err("Selecciona una imagen PNG, JPEG o WebP".into());
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16000);
    limits.max_image_height = Some(16000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| "No se pudo leer la imagen; revisa su formato y dimensiones")?
        .thumbnail(2560, 2560);
    let mut png = Cursor::new(Vec::new());
    image
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    if png.get_ref().len() > MAX {
        return Err("El fondo procesado supera 16 MiB; utiliza una imagen más pequeña".into());
    }
    Ok(png.into_inner())
}

#[tauri::command]
pub fn get_custom_background() -> Result<String, String> {
    let path = config_root()?.join("custom-background.png");
    if !path.exists() {
        return Ok(String::new());
    }
    let bytes = bounded_file(&path, 16 * 1024 * 1024)?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

#[tauri::command]
pub fn set_custom_background(image_base64: String) -> Result<String, String> {
    let path = config_root()?.join("custom-background.png");
    if image_base64.trim().is_empty() {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
        return Ok(String::new());
    }
    let bytes = background_png(&image_base64)?;
    write_atomic(&path, &bytes)?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
#[tauri::command]
pub fn get_system_summary() -> Value {
    let mut total = 8u64 * 1024 * 1024 * 1024;
    #[cfg(windows)]
    {
        #[repr(C)]
        struct MemoryStatus {
            length: u32,
            load: u32,
            total: u64,
            available: u64,
            page_total: u64,
            page_available: u64,
            virtual_total: u64,
            virtual_available: u64,
            extended: u64,
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
        }
        let mut status = MemoryStatus {
            length: std::mem::size_of::<MemoryStatus>() as u32,
            load: 0,
            total: 0,
            available: 0,
            page_total: 0,
            page_available: 0,
            virtual_total: 0,
            virtual_available: 0,
            extended: 0,
        };
        if unsafe { GlobalMemoryStatusEx(&mut status) } > 0 {
            total = status.total;
        }
    }
    json!({"total_memory_gb":total/(1024*1024*1024),"recommended_ram_gb":((total/(1024*1024*1024))/2).clamp(2,8),"os_language":system_language()})
}
fn system_language() -> String {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetUserDefaultLocaleName(name: *mut u16, count: i32) -> i32;
        }
        let mut buffer = [0u16; 85];
        let length = unsafe { GetUserDefaultLocaleName(buffer.as_mut_ptr(), buffer.len() as i32) };
        if length > 1 {
            return String::from_utf16_lossy(&buffer[..(length as usize - 1).min(buffer.len())]);
        }
    }
    std::env::var("LANG")
        .unwrap_or_default()
        .split('.')
        .next()
        .unwrap_or_default()
        .replace('_', "-")
}
#[tauri::command]
pub async fn test_java_executable(path: String) -> Result<String, String> {
    let path = java_executable_path(&path)?;
    let mut command = tokio::process::Command::new(&path);
    command.arg("-version").kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let output = tokio::time::timeout(Duration::from_secs(8), command.output())
        .await
        .map_err(|_| "Java no respondió en 8 segundos")?
        .map_err(|e| format!("No se pudo ejecutar Java: {e}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if !output.status.success() {
        return Err(format!(
            "El ejecutable falló: {}",
            text.chars().take(700).collect::<String>()
        ));
    }
    Ok(text.chars().take(1000).collect())
}

fn java_executable_path(value: &str) -> Result<PathBuf, String> {
    let value = value.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value);
    if value.is_empty() || value.len() > 4096 || value.contains(['\0', '\r', '\n', '"']) {
        return Err("Selecciona un ejecutable Java válido".into());
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err("Selecciona la ruta completa de java.exe o javaw.exe".into());
    }
    let path = if path.is_dir() {
        #[cfg(windows)]
        let candidate = path.join("bin").join("java.exe");
        #[cfg(not(windows))]
        let candidate = path.join("bin").join("java");
        candidate
    } else {
        path
    };
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if !["java.exe", "javaw.exe", "java"].contains(&name.to_ascii_lowercase().as_str()) {
        return Err("Selecciona el ejecutable Java de la instalación".into());
    }
    let path =
        if name.eq_ignore_ascii_case("javaw.exe") && path.with_file_name("java.exe").is_file() {
            path.with_file_name("java.exe")
        } else {
            path
        };
    if !path.is_file() {
        return Err("No se encontró el ejecutable Java en esa ruta".into());
    }
    Ok(path)
}
#[tauri::command]
pub fn open_launcher_folder(app: AppHandle, folder: String) -> Result<(), String> {
    let root = crate::root_dir()?;
    let path = match folder.as_str() {
        "root" => root,
        "screenshots" | "saves" | "resourcepacks" | "shaderpacks" => root.join(folder),
        _ => return Err("Carpeta no permitida".into()),
    };
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub status: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Accounts {
    active: String,
    entries: Vec<AccountSummary>,
}
fn account_lock() -> &'static Mutex<()> {
    static LOCK: Mutex<()> = Mutex::new(());
    &LOCK
}
fn valid_account_id(id: &str) -> bool {
    let suffix = id
        .strip_prefix("microsoft-")
        .or_else(|| id.strip_prefix("offline-"));
    suffix.is_some_and(|suffix| {
        suffix.len() == 32 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}
fn accounts_from(bytes: &[u8]) -> Result<Accounts, String> {
    let mut accounts: Accounts = serde_json::from_slice(bytes)
        .map_err(|e| format!("No se pudo leer la biblioteca de cuentas: {e}"))?;
    if accounts.entries.len() > 128 {
        return Err("La biblioteca admite hasta 128 cuentas".into());
    }
    let mut ids = std::collections::HashSet::new();
    for entry in &mut accounts.entries {
        if !valid_account_id(&entry.id)
            || !["microsoft", "offline"].contains(&entry.kind.as_str())
            || !entry.id.starts_with(&format!("{}-", entry.kind))
            || !ids.insert(entry.id.clone())
            || !(3..=16).contains(&entry.name.len())
            || !entry
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("La biblioteca contiene una cuenta inválida".into());
        }
        entry.active = entry.id == accounts.active;
    }
    if !accounts.active.is_empty() && !ids.contains(&accounts.active) {
        accounts.active.clear();
    }
    Ok(accounts)
}
fn read_accounts() -> Result<Accounts, String> {
    let path = config_root()?.join("accounts.json");
    if path.exists() {
        return accounts_from(&bounded_file(&path, 1024 * 1024)?);
    }


    let legacy_root = crate::root_dir()?;
    let legacy = legacy_root.join("accounts.json");
    if legacy == path || !legacy.exists() {
        return Ok(Accounts::default());
    }
    let accounts = accounts_from(&bounded_file(&legacy, 1024 * 1024)?)?;
    for entry in accounts
        .entries
        .iter()
        .filter(|entry| entry.kind == "microsoft")
    {
        let old = legacy_root
            .join("accounts")
            .join(format!("{}.dat", entry.id));
        let new = token_file(&entry.id)?;
        if old.exists() && !new.exists() {
            write_atomic(&new, &bounded_file(&old, 64 * 1024)?)?;
        }
    }
    save_accounts(&accounts)?;
    Ok(accounts)
}
fn token_file(id: &str) -> Result<PathBuf, String> {
    if !valid_account_id(id) {
        return Err("Identificador de cuenta inválido".into());
    }
    let dir = config_root()?.join("accounts");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{id}.dat")))
}
fn save_accounts(accounts: &Accounts) -> Result<(), String> {
    write_json(&config_root()?.join("accounts.json"), accounts)
}
pub fn deactivate_account() -> Result<(), String> {
    let _guard = account_lock()
        .lock()
        .map_err(|_| "Biblioteca de cuentas ocupada")?;
    let mut accounts = read_accounts()?;
    accounts.active.clear();
    for entry in &mut accounts.entries {
        entry.active = false;
    }
    save_accounts(&accounts)
}
pub fn restore_active_account_token() -> Result<(), String> {
    let _guard = account_lock()
        .lock()
        .map_err(|_| "Biblioteca de cuentas ocupada")?;
    let target = crate::root_dir()?.join("account.dat");
    if target.exists() {
        return Ok(());
    }
    let accounts = read_accounts()?;
    if accounts
        .entries
        .iter()
        .any(|entry| entry.id == accounts.active && entry.kind == "microsoft")
    {
        let saved = token_file(&accounts.active)?;
        if saved.exists() {
            write_atomic(&target, &bounded_file(&saved, 64 * 1024)?)?;
        }
    }
    Ok(())
}
pub fn remember_profile(profile: &crate::MinecraftProfile) -> Result<(), String> {
    let _guard = account_lock()
        .lock()
        .map_err(|_| "Biblioteca de cuentas ocupada")?;
    if profile.id.len() != 32 || !profile.id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("El perfil Microsoft no tiene un UUID válido".into());
    }
    let mut accounts = read_accounts()?;
    let id = format!("microsoft-{}", profile.id.to_ascii_lowercase());
    let current = crate::root_dir()?.join("account.dat");
    if current.exists() {
        write_atomic(&token_file(&id)?, &bounded_file(&current, 64 * 1024)?)?;
    }
    if !accounts.entries.iter().any(|entry| entry.id == id) && accounts.entries.len() >= 128 {
        return Err("La biblioteca admite hasta 128 cuentas".into());
    }
    accounts.entries.retain(|entry| entry.id != id);
    accounts.entries.push(AccountSummary {
        id: id.clone(),
        name: profile.name.clone(),
        kind: "microsoft".into(),
        active: true,
        avatar_url: format!("https://mc-heads.net/avatar/{}/64", profile.id),
        status: "Sesión guardada".into(),
    });
    accounts.active = id;
    save_accounts(&accounts)
}
pub fn remember_current(auth: &crate::AuthState) -> Result<(), String> {
    let profile = auth
        .session
        .lock()
        .map_err(|_| "Cuenta ocupada")?
        .as_ref()
        .map(|session| session.profile.clone());
    if let Some(profile) = profile {
        remember_profile(&profile)?;
    }
    Ok(())
}
#[tauri::command]
pub fn list_launcher_accounts() -> Result<Vec<AccountSummary>, String> {
    let _guard = account_lock()
        .lock()
        .map_err(|_| "Biblioteca de cuentas ocupada")?;
    let mut accounts = read_accounts()?;
    for entry in &mut accounts.entries {
        entry.active = entry.id == accounts.active;
    }
    Ok(accounts.entries)
}
#[derive(Serialize)]pub struct OfflineEligibility{pub allowed:bool,pub verified_at:Option<u64>,pub expires_at:Option<u64>,pub message:String}
fn offline_eligibility_for(_accounts:&Accounts)->OfflineEligibility{
 OfflineEligibility{allowed:true,verified_at:None,expires_at:None,message:"Perfiles locales disponibles sin iniciar sesión con Microsoft. No autentican tu identidad ni habilitan servidores que requieren una sesión oficial. Usa una copia del juego que tengas derecho a utilizar.".into()}
}
#[tauri::command]pub fn get_offline_eligibility()->Result<OfflineEligibility,String>{let _guard=account_lock().lock().map_err(|_|"Biblioteca de cuentas ocupada")?;Ok(offline_eligibility_for(&read_accounts()?))}
fn offline_selected(accounts:&Accounts)->bool{accounts.entries.iter().any(|entry|entry.id==accounts.active&&entry.kind=="offline")}
pub(crate) fn offline_profile_selected()->Result<bool,String>{let _guard=account_lock().lock().map_err(|_|"Biblioteca de cuentas ocupada")?;Ok(offline_selected(&read_accounts()?))}
pub(crate) fn validate_offline_name(name:&str)->Result<(),String>{if !(3..=16).contains(&name.len())||!name.bytes().all(|byte|byte.is_ascii_alphanumeric()||byte==b'_'){return Err("El nick debe tener de 3 a 16 letras, números o guiones bajos".into());}Ok(())}
#[tauri::command]
pub fn add_offline_account(name: String) -> Result<AccountSummary, String> {
    let name = name.trim();
    validate_offline_name(name)?;
    let _guard = account_lock()
        .lock()
        .map_err(|_| "Biblioteca de cuentas ocupada")?;
    let mut accounts = read_accounts()?;
    let id = format!(
        "offline-{}",
        uuid::Uuid::new_v3(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("OfflinePlayer:{name}").as_bytes()
        )
        .simple()
    );
    if let Some(existing) = accounts.entries.iter().find(|entry| entry.id == id) {
        return Ok(existing.clone());
    }
    if accounts.entries.len() >= 128 {
        return Err("La biblioteca admite hasta 128 cuentas".into());
    }
    let account = AccountSummary {
        id,
        name: name.into(),
        kind: "offline".into(),
        active: false,
        avatar_url: format!("https://mc-heads.net/avatar/{name}/64"),
        status: "Perfil local sin autenticación".into(),
    };
    accounts.entries.push(account.clone());
    save_accounts(&accounts)?;
    Ok(account)
}
#[derive(Serialize)]
pub struct ActivatedAccount {
    account: AccountSummary,
    profile: Option<crate::MinecraftProfile>,
}
fn restore_file(path: &Path, previous: Option<&[u8]>) -> Result<(), String> {
    if let Some(bytes) = previous {
        write_atomic(path, bytes)
    } else {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}
fn optional_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
    if !path.exists() {
        Ok(None)
    } else {
        bounded_file(path, 64 * 1024).map(Some)
    }
}
#[tauri::command]
pub async fn activate_launcher_account(
    auth: State<'_, crate::AuthState>,
    id: String,
) -> Result<ActivatedAccount, String> {
    let _auth_guard = auth.refresh_lock.lock().await;
    remember_current(&auth)?;
    let previous_accounts = {
        let _guard = account_lock()
            .lock()
            .map_err(|_| "Biblioteca de cuentas ocupada")?;
        read_accounts()?
    };
    let mut account = previous_accounts
        .entries
        .iter()
        .find(|entry| entry.id == id)
        .cloned()
        .ok_or("Cuenta no encontrada")?;
    let old_session = auth.session.lock().map_err(|_| "Cuenta ocupada")?.clone();
    let active_file = crate::root_dir()?.join("account.dat");
    let old_file = optional_file(&active_file)?;
    let result = async {
        if account.kind == "microsoft" {
            let bytes = bounded_file(&token_file(&id)?, 64 * 1024)
                .map_err(|_| "La cuenta necesita iniciar sesión de nuevo")?;
            write_atomic(&active_file, &bytes)?;
        } else {
            crate::token_storage::clear_saved_refresh_token()?;
        }
        *auth.session.lock().map_err(|_| "Cuenta ocupada")? = None;
        let profile = if account.kind == "microsoft" {
            let profile = crate::ensure_microsoft_session_locked(&auth)
                .await?
                .ok_or("La cuenta necesita iniciar sesión de nuevo")?;
            if format!("microsoft-{}", profile.id.to_ascii_lowercase()) != id {
                return Err(
                    "La sesión guardada pertenece a otra cuenta; se restauró la cuenta anterior"
                        .into(),
                );
            }
            Some(profile)
        } else {
            None
        };
        {
            let _guard = account_lock()
                .lock()
                .map_err(|_| "Biblioteca de cuentas ocupada")?;
            let mut accounts = read_accounts()?;
            accounts.active = id.clone();
            save_accounts(&accounts)?;
        }
        Ok::<_, String>(profile)
    }
    .await;
    match result {
        Ok(profile) => {
            account.active = true;
            Ok(ActivatedAccount { account, profile })
        }
        Err(error) => {
            let mut rollback = Vec::new();
            if let Err(reason) = restore_file(&active_file, old_file.as_deref()) {
                rollback.push(reason);
            }
            match auth.session.lock() {
                Ok(mut session) => *session = old_session,
                Err(_) => rollback.push("No se pudo restaurar la memoria de la sesión".into()),
            }
            {
                let _guard = account_lock()
                    .lock()
                    .map_err(|_| "Biblioteca de cuentas ocupada")?;
                if let Err(reason) = save_accounts(&previous_accounts) {
                    rollback.push(reason);
                }
            }
            if rollback.is_empty() {
                Err(error)
            } else {
                Err(format!(
                    "{error}. La restauración de la cuenta anterior requiere atención: {}",
                    rollback.join("; ")
                ))
            }
        }
    }
}
#[tauri::command]
pub async fn remove_launcher_account(
    auth: State<'_, crate::AuthState>,
    id: String,
) -> Result<(), String> {
    let _auth_guard = auth.refresh_lock.lock().await;
    let _guard = account_lock()
        .lock()
        .map_err(|_| "Biblioteca de cuentas ocupada")?;
    let mut accounts = read_accounts()?;
    if !accounts.entries.iter().any(|entry| entry.id == id) {
        return Err("Cuenta no encontrada".into());
    }
    let previous_accounts = accounts.clone();
    let active = accounts.active == id;
    let active_file = crate::root_dir()?.join("account.dat");
    let old_active = if active {
        optional_file(&active_file)?
    } else {
        None
    };
    let vault = token_file(&id)?;
    let old_vault = optional_file(&vault)?;
    let result = (|| {
        if active {
            crate::token_storage::clear_saved_refresh_token()?;
            accounts.active.clear();
        }
        restore_file(&vault, None)?;
        accounts.entries.retain(|entry| entry.id != id);
        save_accounts(&accounts)
    })();
    if let Err(error) = result {
        let mut failures = Vec::new();
        if active {
            if let Err(reason) = restore_file(&active_file, old_active.as_deref()) {
                failures.push(reason);
            }
        }
        if let Err(reason) = restore_file(&vault, old_vault.as_deref()) {
            failures.push(reason);
        }
        if let Err(reason) = save_accounts(&previous_accounts) {
            failures.push(reason);
        }
        return Err(if failures.is_empty() {
            error
        } else {
            format!(
                "{error}. No se pudo restaurar toda la cuenta: {}",
                failures.join("; ")
            )
        });
    }
    if active {
        *auth.session.lock().map_err(|_| "Cuenta ocupada")? = None;
    }
    Ok(())
}

#[path="launcher_news_feed.rs"]
mod news_feed;
#[derive(Clone, Serialize, Deserialize)]
pub struct NewsItem {
    title: String,
    summary: String,
    image: String,
    #[serde(default)]image_fallback: String,
    url: String,
    date: String,
    category: String,
}
#[tauri::command]
pub async fn launcher_news() -> Result<Vec<NewsItem>, String> {
    let cache = crate::root_dir()?.join("news-cache.json");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;
    let loaded = async {
        let response = client
            .get("https://launchercontent.mojang.com/news.json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        if response
            .content_length()
            .is_some_and(|len| len > 4 * 1024 * 1024)
        {
            return Err("El feed de noticias es demasiado grande".to_string());
        }
        let value = response.json::<Value>().await.map_err(|e| e.to_string())?;
        Ok::<Value, String>(value)
    }
    .await;
    let feed = match loaded {
        Ok(value) => {
            let _ = write_json(&cache, &value);
            value
        }
        Err(error) => serde_json::from_slice(
            &fs::read(cache).map_err(|_| format!("No se pudieron cargar las noticias: {error}"))?,
        )
        .map_err(|e| e.to_string())?,
    };
    Ok(news_feed::parse(&feed))
}
const RELEASES_API: &str = "https://api.github.com/repos/tomaap07/MinecraftTPS/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/tomaap07/MinecraftTPS/releases";
const MAX_UPDATE: u64 = 512 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateDownload {
    pub path: String,
    pub version: String,
    pub asset_id: u64,
    pub name: String,
    pub sha256: String,
    pub size: u64,
}
fn github_client(download: bool) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(if download { 300 } else { 15 }))
        .user_agent("TP LAUNCHER")
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            let allowed = attempt.url().scheme() == "https"
                && attempt.url().host_str().is_some_and(|host| {
                    if download {
                        [
                            "github.com",
                            "release-assets.githubusercontent.com",
                            "objects.githubusercontent.com",
                            "github-releases.githubusercontent.com",
                        ]
                        .contains(&host)
                    } else {
                        host == "api.github.com"
                    }
                });
            if !allowed {
                attempt
                    .error("La actualización intentó salir de los servidores oficiales de GitHub")
            } else if attempt.previous().len() >= 8 {
                attempt.error("Demasiadas redirecciones de GitHub")
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| e.to_string())
}
async fn latest_release() -> Result<Option<Value>, String> {
    use futures_util::StreamExt;
    let response = github_client(false)?
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("No se pudo consultar GitHub: {e}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = response
        .error_for_status()
        .map_err(|e| format!("GitHub no pudo consultar las versiones: {e}"))?;
    if response
        .content_length()
        .is_some_and(|length| length > 2 * 1024 * 1024)
    {
        return Err("La información de la versión supera 2 MiB".into());
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            return Err("La información de la versión supera 2 MiB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| format!("GitHub devolvió información inválida: {e}"))
}
fn valid_digest(value: &str) -> Option<String> {
    value
        .strip_prefix("sha256:")
        .filter(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
}
fn update_asset_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 180
        && !name.starts_with('.')
        && !name.contains("..")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b' '))
        && (name.to_ascii_lowercase().ends_with(".exe")
            || name.to_ascii_lowercase().ends_with(".msi"))
}
fn asset_download_url(value: &str) -> Option<url::Url> {
    url::Url::parse(value).ok().filter(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("github.com")
            && url
                .path()
                .starts_with("/tomaap07/MinecraftTPS/releases/download/")
    })
}
fn supported_asset(value: &Value) -> bool {
    let name = value["name"].as_str().unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    let arch_supported = if cfg!(target_arch = "x86_64") {
        !lower.contains("arm64")
            && !lower.contains("aarch64")
            && !lower.contains("_x86_")
            && !lower.contains("_i686_")
    } else {
        lower.contains("arm64") || lower.contains("aarch64")
    };
    cfg!(windows)
        && arch_supported
        && update_asset_name(name)
        && value["size"]
            .as_u64()
            .is_some_and(|size| size > 0 && size <= MAX_UPDATE)
        && valid_digest(value["digest"].as_str().unwrap_or_default()).is_some()
        && asset_download_url(value["browser_download_url"].as_str().unwrap_or_default()).is_some()
}
fn newer_release(tag: &str) -> bool {
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION"));
    let remote = semver::Version::parse(tag.trim_start_matches('v'));
    matches!((current,remote),(Ok(current),Ok(remote)) if remote>current)
}
#[tauri::command]
pub async fn check_launcher_updates() -> Result<Value, String> {
    let Some(release) = latest_release().await? else {
        return Ok(
            json!({"available":false,"verifiable":false,"message":"Todavía no hay una versión publicada en GitHub","current":env!("CARGO_PKG_VERSION"),"url":RELEASES_PAGE}),
        );
    };
    let tag = release["tag_name"].as_str().unwrap_or_default();
    let asset = release["assets"]
        .as_array()
        .and_then(|assets| assets.iter().find(|asset| supported_asset(asset)));
    let available = newer_release(tag);
    let url = release["html_url"]
        .as_str()
        .filter(|url| url.starts_with("https://github.com/tomaap07/MinecraftTPS/releases/tag/"))
        .unwrap_or(RELEASES_PAGE);
    Ok(
        json!({"available":available,"verifiable":asset.is_some(),"version":tag,"url":url,"asset_id":asset.and_then(|asset|asset["id"].as_u64()),"asset_name":asset.and_then(|asset|asset["name"].as_str()),"download_size":asset.and_then(|asset|asset["size"].as_u64()),"digest":asset.and_then(|asset|asset["digest"].as_str()),"message":if available{format!("Nueva versión disponible: {tag}")}else{format!("TP LAUNCHER está al día. Última versión publicada: {tag}")},"current":env!("CARGO_PKG_VERSION")}),
    )
}
#[tauri::command]
pub async fn download_launcher_update(asset_id: u64) -> Result<UpdateDownload, String> {
    use futures_util::StreamExt;
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncWriteExt;
    let release = latest_release()
        .await?
        .ok_or("Todavía no hay una versión publicada")?;
    let version = release["tag_name"].as_str().unwrap_or_default();
    if !newer_release(version) {
        return Err("No hay una versión posterior a la instalada".into());
    }
    let asset = release["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .find(|asset| asset["id"].as_u64() == Some(asset_id) && supported_asset(asset))
        })
        .ok_or("La versión no contiene ese instalador compatible con digest SHA-256 de GitHub")?;
    let name = asset["name"]
        .as_str()
        .ok_or("El instalador no tiene nombre")?;
    let expected = valid_digest(asset["digest"].as_str().unwrap_or_default())
        .ok_or("GitHub no publicó un digest SHA-256 válido")?;
    let size = asset["size"]
        .as_u64()
        .ok_or("GitHub no publicó el tamaño del instalador")?;
    let url = asset_download_url(asset["browser_download_url"].as_str().unwrap_or_default())
        .ok_or("El instalador no pertenece al repositorio oficial")?;
    let dir = config_root()?.join("updates");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join(format!("{asset_id}-{name}"));
    let pending = dir.join(format!("{}.pending", uuid::Uuid::new_v4()));
    let result = async {
        let response = github_client(true)?
            .get(url)
            .send()
            .await
            .map_err(|e| format!("No se pudo descargar la actualización: {e}"))?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        if response
            .content_length()
            .is_some_and(|length| length != size || length > MAX_UPDATE)
        {
            return Err(
                "El tamaño del instalador no coincide con el publicado por GitHub".to_string(),
            );
        }
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)
            .await
            .map_err(|e| e.to_string())?;
        let mut hash = Sha256::new();
        let mut received = 0u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| e.to_string())?;
            received += chunk.len() as u64;
            if received > size || received > MAX_UPDATE {
                return Err("La descarga supera el tamaño publicado".into());
            }
            hash.update(&chunk);
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        }
        file.flush().await.map_err(|e| e.to_string())?;
        file.sync_all().await.map_err(|e| e.to_string())?;
        drop(file);
        if received != size || format!("{:x}", hash.finalize()) != expected {
            return Err(
                "La verificación SHA-256 de la actualización falló; no se instalará ese archivo"
                    .into(),
            );
        }
        tokio::fs::rename(&pending, &target)
            .await
            .map_err(|e| e.to_string())?;
        let record = UpdateDownload {
            path: target.to_string_lossy().into_owned(),
            version: version.into(),
            asset_id,
            name: name.into(),
            sha256: expected,
            size,
        };
        write_json(&dir.join("download.json"), &record)?;
        Ok::<_, String>(record)
    }
    .await;
    if result.is_err() {
        let _ = fs::remove_file(&pending);
    }
    result
}
#[tauri::command]
pub async fn install_launcher_update(app: AppHandle, path: String) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncReadExt;
    if crate::launcher_tools::active_game_count() > 0 {
        return Err("Cierra Minecraft antes de instalar la actualización".into());
    }
    let dir = config_root()?.join("updates");
    let record: UpdateDownload =
        serde_json::from_slice(&bounded_file(&dir.join("download.json"), 16 * 1024)?)
            .map_err(|_| "No hay un instalador verificado disponible")?;
    if !update_asset_name(&record.name) || record.size > MAX_UPDATE {
        return Err("El registro de actualización no es válido".into());
    }
    let expected_path = dir.join(format!("{}-{}", record.asset_id, record.name));
    if Path::new(&path) != expected_path
        || fs::symlink_metadata(&expected_path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("El instalador debe estar dentro de la carpeta de actualizaciones".into());
    }
    let release = latest_release()
        .await?
        .ok_or("La versión ya no está publicada")?;
    let asset = release["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|asset| {
                asset["id"].as_u64() == Some(record.asset_id) && supported_asset(asset)
            })
        })
        .ok_or("GitHub ya no publica ese instalador compatible")?;
    let expected = valid_digest(asset["digest"].as_str().unwrap_or_default())
        .ok_or("GitHub no publicó un digest verificable")?;
    if expected != record.sha256
        || asset["size"].as_u64() != Some(record.size)
        || asset["name"].as_str() != Some(record.name.as_str())
    {
        return Err(
            "Los metadatos del instalador cambiaron; descarga de nuevo la actualización".into(),
        );
    }
    let mut file = tokio::fs::File::open(&expected_path)
        .await
        .map_err(|e| e.to_string())?;
    if file.metadata().await.map_err(|e| e.to_string())?.len() != record.size {
        return Err("El instalador está incompleto".into());
    }
    let mut buffer = vec![0; 128 * 1024];
    let mut hash = Sha256::new();
    let mut read = 0u64;
    loop {
        let count = file.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        read += count as u64;
        if read > MAX_UPDATE {
            return Err("El instalador supera el límite permitido".into());
        }
        hash.update(&buffer[..count]);
    }
    if format!("{:x}", hash.finalize()) != expected {
        return Err("El instalador cambió desde su descarga; se canceló la ejecución".into());
    }
    drop(file);
    if crate::launcher_tools::active_game_count() > 0 {
        return Err("Cierra Minecraft antes de instalar la actualización".into());
    }
    #[cfg(windows)]
    {
        let mut command = if record.name.to_ascii_lowercase().ends_with(".msi") {
            let mut command = std::process::Command::new("msiexec.exe");
            command.arg("/i").arg(&expected_path);
            command
        } else {
            std::process::Command::new(&expected_path)
        };
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
        command
            .spawn()
            .map_err(|e| format!("No se pudo abrir el instalador: {e}"))?;
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            app.exit(0);
        });
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("Abre la página de versiones para instalar la actualización en este sistema".into())
    }
}

pub fn after_game_started(app: &AppHandle) {
    let behavior = load().unwrap_or_default().launch_behavior;
    if let Some(window) = app.get_webview_window("main") {
        match behavior.as_str() {
            "minimize" => {
                let _ = window.minimize();
            }
            "tray" => {
                let _ = window.hide();
            }
            "close" => {
                app.exit(0);
            }
            _ => {}
        }
    }
}
pub fn start_presence_worker() {
    static STARTED: OnceLock<()> = OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }
    std::thread::spawn(|| {
        use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
        let mut connected: Option<(String, DiscordIpcClient)> = None;
        loop {
            let settings = load().unwrap_or_default();
            if settings.discord_presence && valid_discord_id(&settings.discord_application_id) {
                let id = settings.discord_application_id;
                if connected.as_ref().is_none_or(|(current, _)| current != &id) {
                    if let Some((_, mut client)) = connected.take() {
                        let _ = client.close();
                    }
                    if let Ok(mut client) = DiscordIpcClient::new(&id) {
                        if client.connect().is_ok() {
                            connected = Some((id, client));
                        }
                    }
                }
                if let Some((_, client)) = &mut connected {
                    let session = crate::game_session::get_game_session();
                    let state = if session.running {
                        format!("Minecraft {} · {}", session.version, if session.server.is_empty() { "Partida local" } else { &session.server })
                    } else { "Explorando el universo de Minecraft".into() };
                    if client
                        .set_activity(
                            activity::Activity::new()
                                .details(if session.running { "Jugando en TP LAUNCHER" } else { "TP LAUNCHER · Centro de Mando" })
                                .state(&state),
                        )
                        .is_err()
                    {
                        connected = None;
                    }
                }
            } else if let Some((_, mut client)) = connected.take() {
                let _ = client.clear_activity();
                let _ = client.close();
            }
            std::thread::sleep(Duration::from_secs(15));
        }
    });
}
fn valid_discord_id(value: &str) -> bool {
    (15..=22).contains(&value.len())
        && !value.starts_with('0')
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok_and(|id| id > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("launchertps-settings-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn local_profiles_do_not_require_or_claim_microsoft_entitlements(){
        let mut accounts=Accounts::default();let eligibility=offline_eligibility_for(&accounts);
        assert!(eligibility.allowed);assert!(eligibility.verified_at.is_none());assert!(eligibility.expires_at.is_none());assert!(!offline_selected(&accounts));
        accounts.active="offline-example".into();accounts.entries.push(AccountSummary{id:accounts.active.clone(),name:"LocalPlayer".into(),kind:"offline".into(),active:false,avatar_url:String::new(),status:String::new()});
        assert!(offline_selected(&accounts));accounts.entries[0].kind="microsoft".into();assert!(!offline_selected(&accounts));
        for name in ["Steve","Local_Player","User123"]{assert!(validate_offline_name(name).is_ok());}
        for name in ["","ab","with space","../profile","abcdefghijklmnopq"]{assert!(validate_offline_name(name).is_err());}
    }
    #[test]
    fn settings_default_gc_and_update_validation_are_consistent() {
        let settings: LauncherSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.gc_mode, "g1gc");
        assert_eq!(settings.launch_behavior, "keep");
        assert!(valid_discord_id("123456789012345678"));
        assert!(!valid_discord_id("123"));
        assert!(!valid_discord_id("18446744073709551616"));
        assert!(valid_digest(&format!("sha256:{}", "ab".repeat(32))).is_some());
        assert!(valid_digest("sha256:abc").is_none());
        assert!(update_asset_name("LauncherTPS_1.0_x64-setup.exe"));
        assert!(!update_asset_name("../update.exe"));
        assert!(!update_asset_name("update.ps1"));
        assert!(
            asset_download_url(
                "https://github.com/tomaap07/MinecraftTPS/releases/download/v1.0/setup.exe"
            )
            .is_some()
        );
        assert!(
            asset_download_url(
                "https://github.com/example/MinecraftTPS/releases/download/v1.0/setup.exe"
            )
            .is_none()
        );
        assert!(!newer_release(env!("CARGO_PKG_VERSION")));
        assert!(newer_release("999.0.0"));
        assert!(!newer_release("invalid"));
    }
    #[test]
    fn background_is_scaled_and_bounded_and_files_replace_atomically() {
        let image = image::DynamicImage::new_rgba8(3000, 4);
        let mut source = Cursor::new(Vec::new());
        image
            .write_to(&mut source, image::ImageFormat::Png)
            .unwrap();
        let encoded = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(source.get_ref())
        );
        let png = background_png(&encoded).unwrap();
        let decoded = image::load_from_memory(&png).unwrap();
        assert_eq!(decoded.width(), 2560);
        assert!(decoded.height() <= 4);
        assert!(background_png("data:text/html;base64,PGh0bWw+").is_err());
        assert!(background_png("not an image").is_err());
        let scratch = Scratch::new();
        let target = scratch.0.join("state.json");
        write_atomic(&target, b"first").unwrap();
        write_atomic(&target, b"second").unwrap();
        assert_eq!(bounded_file(&target, 8).unwrap(), b"second");
        assert!(bounded_file(&target, 2).is_err());
    }
    #[test]
    fn account_library_rejects_invalid_and_duplicate_entries_without_tokens() {
        let id = format!("offline-{}", "a".repeat(32));
        let one = json!({"id":id,"name":"Steve","kind":"offline"});
        let valid = json!({"active":id,"entries":[one.clone()]});
        let accounts = accounts_from(&serde_json::to_vec(&valid).unwrap()).unwrap();
        assert!(accounts.entries[0].active);
        let duplicate = json!({"active":id,"entries":[one.clone(),one]});
        assert!(accounts_from(&serde_json::to_vec(&duplicate).unwrap()).is_err());
        assert!(!valid_account_id(""));
        assert!(!valid_account_id("../../account"));
        assert!(!valid_account_id("microsoft-token"));
    }
    #[cfg(windows)]
    #[test]
    fn java_8_paths_accept_spaces_and_switch_javaw_to_console_binary() {
        let scratch = Scratch::new();
        let root = scratch.0.join("Java 8 Portable");
        let bin = root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let java = bin.join("java.exe");
        let javaw = bin.join("javaw.exe");
        fs::write(&java, b"fixture").unwrap();
        fs::write(&javaw, b"fixture").unwrap();
        assert_eq!(
            java_executable_path(&format!("\"{}\"", javaw.display())).unwrap(),
            java
        );
        assert_eq!(java_executable_path(root.to_str().unwrap()).unwrap(), java);
        assert!(java_executable_path("java.exe").is_err());
    }
    #[cfg(windows)]
    #[tokio::test]
    #[ignore = "Requiere el runtime Java 8 Portable instalado en AppData"]
    async fn installed_java_8_reports_its_version() {
        let java = dirs::data_dir()
            .unwrap()
            .join("MinecraftTPS/runtime/jdk-8/bin/java.exe");
        let text = test_java_executable(java.to_string_lossy().into_owned())
            .await
            .unwrap();
        assert!(
            text.contains("1.8.") || text.contains("version \"8"),
            "El ejecutable no corresponde a Java 8: {text}"
        );
    }
}
