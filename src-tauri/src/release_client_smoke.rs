use crate::*;
use serde::Serialize;
use std::collections::BTreeSet;
use std::io::{Seek, SeekFrom};
use std::time::{Duration, Instant};
use tauri::Listener;

const BATCH_LIMIT: usize = 32;
const CLIENT_LIMIT: Duration = Duration::from_secs(180);
const STABLE_PERIOD: Duration = Duration::from_secs(18);

#[derive(Default, Serialize)]
struct ClientResult {
    id: String,
    runtime: String,
    started: bool,
    sound: String,
    renderer: String,
    menu: String,
    errors: Vec<String>,
    elapsed: f64,
    logrelative: String,
}

#[derive(Default)]
struct Evidence {
    lines: VecDeque<String>,
    runtime: String,
    sound: Option<String>,
    renderer: Option<String>,
    errors: BTreeSet<String>,
}

fn fatal_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    [
        "minecraft has crashed",
        "---- minecraft crash report ----",
        "failed to load required shader programs",
        "unable to find shader defined uniform",
        "couldn't compile pipeline",
        "exception in thread",
        "could not find or load main class",
        "unsupportedclassversionerror",
        "java.lang.noclassdeffounderror",
        "java.lang.classnotfoundexception",
        "a fatal error has been detected by the java runtime",
        "failed to initialize glfw",
        "failed to create window",
        "failed to initialize graphics",
        "starting integrated minecraft server",
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}

impl Evidence {
    fn observe(&mut self, line: &str) {
        let line: String = line.chars().take(2048).collect();
        if fatal_line(&line) && self.errors.len() < 24 {
            self.errors.insert(line.clone());
        }
        if let Some((_, runtime)) = line.split_once(" con Java ") {
            if line.contains("Iniciando ") {
                let major = runtime.split_whitespace().next().unwrap_or_default();
                if major.bytes().all(|b| b.is_ascii_digit()) && !major.is_empty() {
                    self.runtime = format!("Java {major}");
                }
            }
        }
        let lower = line.to_ascii_lowercase();
        if [
            "sound engine started",
            "soundsystem started",
            "openal initialized",
        ]
        .iter()
        .any(|pattern| lower.contains(pattern))
        {
            self.sound = Some(line.clone());
        }
        let strong_renderer = [
            "opengl renderer:",
            "opengl version:",
            "using graphics backend",
            "using graphics device:",
        ]
        .iter()
        .any(|pattern| lower.contains(pattern));
        if strong_renderer
            || (self.renderer.is_none()
                && ["backend library:", "lwjgl version:", "created:"]
                    .iter()
                    .any(|p| lower.contains(p)))
        {
            self.renderer = Some(line.clone());
        }
        self.lines.push_back(line);
        while self.lines.len() > 4000 {
            self.lines.pop_front();
        }
    }
}

fn selected_versions(raw: &str, allowed: &HashSet<String>) -> Result<Vec<String>, String> {
    let rows: Vec<_> = raw.split(',').map(str::trim).collect();
    if rows.is_empty() || rows.len() > BATCH_LIMIT {
        return Err(format!(
            "Indica entre 1 y {BATCH_LIMIT} IDs exactos por lote"
        ));
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for id in rows {
        if id.is_empty()
            || id.len() > 160
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b))
            || !allowed.contains(id)
        {
            return Err(format!("ID no publicado por Mojang ni instalado: {id}"));
        }
        if !seen.insert(id) {
            return Err(format!("ID duplicado en la prueba: {id}"));
        }
        result.push(id.to_owned());
    }
    Ok(result)
}

fn sound_required(base: &str) -> bool {
    if ["rd-", "c0.", "a1.", "b1."]
        .iter()
        .any(|p| base.starts_with(p))
    {
        return false;
    }
    if let Some(release) = base.strip_prefix("1.") {
        if let Some(minor) = release
            .split('.')
            .next()
            .and_then(|p| p.parse::<u32>().ok())
        {
            return minor >= 6;
        }
    }
    if base.get(2..3) == Some("w") {
        return base
            .get(..2)
            .and_then(|p| p.parse::<u32>().ok())
            .is_none_or(|year| year >= 14);
    }
    true
}

fn reserve_profile(root: &Path, id: &str) -> Result<(String, PathBuf), String> {
    let parent = root.join("instances");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    for a in alphabet {
        for b in alphabet {
            let name = format!("TPReleaseCheck{}{}", *a as char, *b as char);
            match fs::create_dir(parent.join(&name)) {
                Ok(()) => {
                    let dir = version_game_dir(root, &name, id)?;
                    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                    fs::write(
                        dir.join("options.txt"),
                        "fullscreen:false\npauseOnLostFocus:false\nresourcePacks:[]\n",
                    )
                    .map_err(|e| e.to_string())?;
                    return Ok((name, dir));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    Err("No quedan nombres libres TPReleaseCheck para perfiles aislados".into())
}

#[derive(Default)]
struct ClientLogCursor {
    offset: u64,
    lines: u64,
    pending: Vec<u8>,
}

impl ClientLogCursor {
    fn observe_new_lines(&mut self, path: &Path, evidence: &mut Evidence) {
        let Ok(mut file) = fs::File::open(path) else {
            return;
        };
        let Ok(meta) = file.metadata() else {
            return;
        };
        if meta.len() < self.offset {
            self.offset = 0;
            self.lines = 0;
            self.pending.clear();
        }
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return;
        }
        let mut bytes = Vec::new();
        if file.take(4 * 1024 * 1024).read_to_end(&mut bytes).is_err() {
            return;
        }
        self.offset += bytes.len() as u64;
        self.pending.extend_from_slice(&bytes);
        let mut consumed = 0;
        for (index, byte) in self.pending.iter().enumerate() {
            if *byte == b'\n' {
                let line = String::from_utf8_lossy(&self.pending[consumed..index]);
                evidence.observe(line.trim_end_matches('\r'));
                self.lines += 1;
                consumed = index + 1;
            }
        }
        self.pending.drain(..consumed);
        if self.pending.len() > 16 * 1024 {
            self.pending.drain(..self.pending.len() - 16 * 1024);
        }
    }
}

#[cfg(windows)]
struct TestProcesses {
    username: String,
    version: String,
    directory: PathBuf,
    handles: BTreeMap<u32, windows_sys::Win32::Foundation::HANDLE>,
}

#[cfg(windows)]
impl TestProcesses {
    fn capture(&mut self) {
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
        };
        for pid in launcher_tools::test_session_pids(&self.username) {
            if self.handles.contains_key(&pid) {
                continue;
            }
            if game_session::identity(pid) != Some((self.username.clone(), self.version.clone()))
                || game_session::directory(pid).and_then(|p| p.canonicalize().ok())
                    != self.directory.canonicalize().ok()
            {
                continue;
            }
            let handle = unsafe { OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, 0, pid) };
            if !handle.is_null() {
                self.handles.insert(pid, handle);
            }
        }
    }
    fn pids(&self) -> Vec<u32> {
        use windows_sys::Win32::{
            Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject,
        };
        self.handles
            .iter()
            .filter_map(|(pid, handle)| {
                (unsafe { WaitForSingleObject(*handle, 0) } == WAIT_TIMEOUT).then_some(*pid)
            })
            .collect()
    }
}

#[cfg(windows)]
impl Drop for TestProcesses {
    fn drop(&mut self) {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_TIMEOUT},
            System::Threading::{TerminateProcess, WaitForSingleObject},
        };
        self.capture();
        for handle in self.handles.values() {
            unsafe {
                if WaitForSingleObject(*handle, 0) == WAIT_TIMEOUT {
                    TerminateProcess(*handle, 0);
                    WaitForSingleObject(*handle, 5000);
                }
                CloseHandle(*handle);
            }
        }
    }
}

#[cfg(windows)]
async fn run_client(app: &AppHandle, root: &Path, output: &Path, id: &str) -> ClientResult {
    let started_at = Instant::now();
    let mut result = ClientResult {
        id: id.into(),
        sound: "unverified".into(),
        renderer: "unverified".into(),
        runtime: "unverified".into(),
        menu: "unverified".into(),
        ..Default::default()
    };
    let (name, dir) = match reserve_profile(root, id) {
        Ok(value) => value,
        Err(error) => {
            result.errors.push(error);
            return result;
        }
    };
    let base = resolve_mod_game_version(root, id)
        .map(|(game, _)| game)
        .unwrap_or_else(|_| id.to_owned());
    let mut processes = TestProcesses {
        username: name.clone(),
        version: id.into(),
        directory: dir.clone(),
        handles: BTreeMap::new(),
    };
    let evidence = Arc::new(Mutex::new(Evidence::default()));
    let capture = Arc::clone(&evidence);
    let listener = app.listen("launcher-log", move |event| {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(event.payload()) {
            if let Some(line) = value["line"].as_str() {
                if let Ok(mut evidence) = capture.lock() {
                    evidence.observe(line);
                }
            }
        }
    });
    let mut launch = Box::pin(launch_game_session(
        app.clone(),
        None,
        name.clone(),
        id.into(),
        4,
        None,
        Some(false),
        None,
    ));
    let launch_result = loop {
        tokio::select! {
            value = &mut launch => break value,
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                processes.capture();
                if started_at.elapsed() >= CLIENT_LIMIT { break Err("Tiempo de arranque agotado (180 s)".into()); }
            }
        }
    };
    processes.capture();
    drop(launch);
    if let Err(error) = launch_result {
        result.errors.push(error);
    }
    let mut ready_since = None;
    let mut log_cursor = ClientLogCursor::default();
    if result.errors.is_empty() {
        while started_at.elapsed() < CLIENT_LIMIT {
            processes.capture();
            let pids = processes.pids();
            if pids.is_empty() {
                result
                    .errors
                    .push("El proceso terminó antes de completar la prueba de estabilidad".into());
                break;
            }
            if let Ok(mut state) = evidence.lock() {
                log_cursor.observe_new_lines(&dir.join("logs/latest.log"), &mut state);
                if !state.errors.is_empty() {
                    break;
                }
                let window = crate::hud_windows::find(&pids).is_some();
                let audio = state.sound.is_some() || !sound_required(&base);
                if window && state.renderer.is_some() && audio {
                    let since = ready_since.get_or_insert_with(Instant::now);
                    if since.elapsed() >= STABLE_PERIOD {
                        result.started = true;
                        break;
                    }
                } else {
                    ready_since = None;
                }
            }
            if let Ok(snapshot) = launcher_overlay::get_overlay_snapshot() {
                if pids.contains(&snapshot.pid)
                    && snapshot
                        .telemetry
                        .as_ref()
                        .is_some_and(|t| t.labels.get("screen").is_some_and(|s| s == "title"))
                {
                    result.menu = "observed_title_screen".into();
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        if !result.started && result.errors.is_empty() {
            result.errors.push("No se verificaron ventana, renderizado y estabilidad durante 18 s dentro del límite de 180 s".into());
        }
    }
    app.unlisten(listener);
    if let Ok(state) = evidence.lock() {
        if !state.runtime.is_empty() {
            result.runtime = state.runtime.clone();
        }
        if let Some(sound) = &state.sound {
            result.sound = format!("initialized: {sound}");
        }
        if let Some(renderer) = &state.renderer {
            result.renderer = renderer.clone();
        }
        result.errors.extend(state.errors.iter().cloned());
        let log_name = format!("release-client-smoke-logs/{name}.log");
        let log_path = output.join(&log_name);
        let log_text = state.lines.iter().cloned().collect::<Vec<_>>().join("\n");
        if let Err(error) = fs::write(&log_path, log_text) {
            result
                .errors
                .push(format!("No se pudo guardar el log: {error}"));
        }
        result.logrelative = format!(".codex-tmp/{log_name}");
    }
    if sound_required(&base) && result.sound == "unverified" {
        result
            .errors
            .push("No se observó la inicialización del audio".into());
    }
    if fs::read_dir(dir.join("crash-reports")).is_ok_and(|mut reports| reports.next().is_some()) {
        result
            .errors
            .push("Minecraft generó un crash-report en el perfil aislado".into());
    }
    result.started &= result.errors.is_empty();
    result.elapsed = started_at.elapsed().as_secs_f64();
    drop(processes);
    tokio::time::sleep(Duration::from_millis(500)).await;
    result
}

#[cfg(windows)]
#[test]
#[ignore = "Launches explicitly selected isolated Minecraft clients; set TP_RELEASE_TEST_VERSIONS, at most 32 exact IDs"]
fn selected_release_clients_real_smoke() {
    let raw = std::env::var("TP_RELEASE_TEST_VERSIONS").expect("Define TP_RELEASE_TEST_VERSIONS con IDs exactos separados por comas; no hay lote automático");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let root = root_dir().unwrap();
    assert!(
        game_session::tracked_pids().is_empty(),
        "Otra prueba ya está ejecutando Minecraft en este proceso"
    );
    let manifest = runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(30), load_version_manifest()).await
        })
        .expect("Timeout al cargar el catálogo permitido")
        .expect("No se pudo validar el catálogo de Mojang");
    let mut allowed: HashSet<String> = manifest.versions.into_iter().map(|v| v.id).collect();
    allowed.extend(
        list_installed_versions()
            .unwrap()
            .into_iter()
            .filter(|v| v.json)
            .map(|v| v.id),
    );
    let versions = selected_versions(&raw, &allowed).expect("Selección de versiones inválida");
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".codex-tmp");
    fs::create_dir_all(output.join("release-client-smoke-logs")).unwrap();
    let report_path = output.join("release-client-smoke.json");
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    let app = tauri::Builder::default()
        .any_thread()
        .build(context)
        .unwrap();
    let mut results = Vec::new();
    for id in versions {
        let result = runtime.block_on(run_client(app.handle(), &root, &output, &id));
        println!(
            "{}: started={}, sound={}, errors={}, elapsed={:.1}s",
            result.id,
            result.started,
            result.sound,
            result.errors.len(),
            result.elapsed
        );
        results.push(result);
        fs::write(&report_path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
    let failures: Vec<_> = results
        .iter()
        .filter(|r| !r.started || !r.errors.is_empty())
        .map(|r| format!("{}: {}", r.id, r.errors.join("; ")))
        .collect();
    assert!(
        failures.is_empty(),
        "Pruebas fallidas (informe {}):\n{}",
        report_path.display(),
        failures.join("\n")
    );
}

#[test]
fn release_smoke_rejects_implicit_batches_duplicates_and_unpublished_ids() {
    let allowed = [
        "1.12.2",
        "1.21.1",
        "26.2",
        "26.3",
        "fabric-loader-0.20.1-26.3",
    ]
    .map(str::to_owned)
    .into_iter()
    .collect();
    assert_eq!(
        selected_versions("1.12.2, 26.3", &allowed).unwrap(),
        ["1.12.2", "26.3"]
    );
    for raw in ["", "1.12.2,", "1.21.1,1.21.1", "26.9", "../26.3"] {
        assert!(selected_versions(raw, &allowed).is_err());
    }
    assert!(selected_versions(&vec!["26.3"; 33].join(","), &allowed).is_err());
}

#[test]
fn release_smoke_distinguishes_graphics_failure_audio_and_offline_realms() {
    assert!(fatal_line("Failed to load required shader programs"));
    assert!(fatal_line(
        "ShaderCompileException: Unable to find shader defined uniform (Sampler1)"
    ));
    assert!(!fatal_line(
        "Couldn't connect to realms: Realms authentication error"
    ));
    assert!(!fatal_line("[Server thread/INFO]: routine information"));
    let mut evidence = Evidence::default();
    evidence.observe("Starting up SoundSystem...");
    assert!(evidence.sound.is_none());
    evidence.observe("OpenAL initialized.");
    assert!(evidence.sound.is_some());
    evidence.observe("Minecraft: Iniciando 26.3 con Java 25 (runtime) (0%)");
    assert_eq!(evidence.runtime, "Java 25");
    assert!(!sound_required("1.5.2"));
    assert!(!sound_required("b1.7.3"));
    assert!(sound_required("1.12.2"));
    assert!(sound_required("26.3"));
}

#[test]
fn release_smoke_reads_neoforge_sound_log_incrementally_and_resets_after_truncation() {
    let path = std::env::temp_dir().join(format!("tp-release-log-{}.txt", Uuid::new_v4()));
    let backend = "[27oct2026 06:53:48.393] [Render thread/INFO] [net.minecraft.client.Minecraft/]: Backend library: LWJGL version 3.3.3+5\n";
    let openal = "[27oct2026 06:53:52.101] [Render thread/INFO] [com.mojang.blaze3d.audio.Library/]: OpenAL initialized on device OpenAL Soft\n";
    let sound = "[27oct2026 06:53:52.103] [Render thread/INFO] [net.minecraft.client.sounds.SoundEngine/SOUNDS]: Sound engine started";
    fs::write(&path, backend).unwrap();
    let mut cursor = ClientLogCursor::default();
    let mut evidence = Evidence::default();
    cursor.observe_new_lines(&path, &mut evidence);
    assert_eq!(cursor.lines, 1);
    assert!(evidence.renderer.is_some());
    assert!(evidence.sound.is_none());
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(format!("{openal}{sound}").as_bytes())
        .unwrap();
    cursor.observe_new_lines(&path, &mut evidence);
    assert_eq!(cursor.lines, 2);
    assert!(
        evidence
            .sound
            .as_ref()
            .unwrap()
            .contains("OpenAL initialized")
    );
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"\r\n")
        .unwrap();
    cursor.observe_new_lines(&path, &mut evidence);
    assert_eq!(cursor.lines, 3);
    assert_eq!(evidence.sound.as_deref(), Some(sound));
    let observed = evidence.lines.len();
    cursor.observe_new_lines(&path, &mut evidence);
    assert_eq!(evidence.lines.len(), observed);
    fs::write(&path, "log restarted\n").unwrap();
    cursor.observe_new_lines(&path, &mut evidence);
    assert_eq!(cursor.lines, 1);
    assert_eq!(
        evidence.lines.back().map(String::as_str),
        Some("log restarted")
    );
    fs::remove_file(path).unwrap();
}
