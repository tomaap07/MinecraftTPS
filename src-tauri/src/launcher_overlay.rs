
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    io::Read,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
pub const LABEL: &str = "tp-hud-overlay";
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OverlayState {
    pub running: bool,
    pub username: String,
    pub version: String,
    pub server: String,
    pub active_games: u32,
}
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HudKeys {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub lmb: bool,
    pub rmb: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Armor {
    pub name: String,
    pub item_id:String,
    pub slot:String,
    pub remaining: u32,
    pub maximum: u32,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Telemetry {
    pub schema_version: u32,
    pub pid: u32,
    pub timestamp_ms: u64,
    pub has_world: bool,
    pub game_fps: Option<f64>,
    pub frame_time_ms:Option<f64>,
    pub fps_low:Option<f64>,
    pub metrics:std::collections::BTreeMap<String,f64>,
    pub world_metrics:std::collections::BTreeMap<String,f64>,
    pub labels:std::collections::BTreeMap<String,String>,
    pub singleplayer:bool,
    pub world_time:Option<f64>,
    pub world_time_source:String,
    pub game_time:Option<f64>,
    pub biome:String,
    pub dimension:String,
    pub server_address:String,
    pub attack_cooldown:Option<f64>,pub target_distance:Option<f64>,pub speed:Option<f64>,pub target_kind:String,pub hit_sequence:Option<u64>,
    pub source: String,
    pub status: String,
    pub capabilities: Vec<String>,
    pub coordinates_restricted: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub direction: String,
    pub ping: Option<u32>,
    pub ping_source:String,
    pub armor: Vec<Armor>,
    pub map: Option<MiniMap>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct MiniMap {
    pub pixels: String,
    pub origin_x: i32,
    pub origin_z: i32,
}
#[derive(Clone, Default, Serialize)]
pub struct OverlaySnapshot {
    #[serde(flatten)]
    pub state: OverlayState,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub updated_at: u64,
    pub game_fps: Option<f64>,
    pub fps_source: String,
    pub keys: HudKeys,
    pub left_cps: usize,
    pub right_cps: usize,
    pub editing: bool,
    pub attached: bool,
    pub focused: bool,
    pub pid: u32,
    pub telemetry: Option<Telemetry>,
    pub message: String,
}
fn frame() -> &'static Mutex<OverlaySnapshot> {
    static FRAME: OnceLock<Mutex<OverlaySnapshot>> = OnceLock::new();
    FRAME.get_or_init(|| Mutex::new(OverlaySnapshot::default()))
}
static EPOCH: AtomicU64 = AtomicU64::new(0);
static EDITING: AtomicBool = AtomicBool::new(false);
static CREATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static WORKER: Mutex<()> = Mutex::new(());
fn lease(pid: u32) {
    if let Ok(path) = crate::launcher_plugins::profile_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::write(
                parent.join(format!("overlay-{pid}.active")),
                now().to_string(),
            );
        }
    }
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn memory() -> (f64, f64) {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct Memory {
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
            fn GlobalMemoryStatusEx(value: *mut Memory) -> i32;
        }
        let mut m = Memory {
            length: std::mem::size_of::<Memory>() as u32,
            load: 0,
            total: 0,
            available: 0,
            page_total: 0,
            page_available: 0,
            virtual_total: 0,
            virtual_available: 0,
            extended: 0,
        };
        if unsafe { GlobalMemoryStatusEx(&mut m) } > 0 {
            return (
                m.total as f64 / 1073741824.,
                m.available as f64 / 1073741824.,
            );
        }
    }
    (0., 0.)
}
fn telemetry_file(path: std::path::PathBuf, pid: u32) -> Option<Telemetry> {
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > 16384 { return None; }
    let mut bytes=Vec::new();file.take(16385).read_to_end(&mut bytes).ok()?;
    decode_telemetry(&bytes,pid,now())
}
pub fn telemetry(pid: u32) -> Option<Telemetry> {
    let folder=crate::launcher_plugins::profile_path().ok()?.parent()?.to_path_buf();
    let companion=telemetry_file(folder.join(format!("telemetry-{pid}.json")),pid);
    let bridge=crate::hud_transport::latest(pid).and_then(|bytes|decode_telemetry(&bytes,pid,now())).or_else(||telemetry_file(folder.join(format!("bridge-{pid}.json")),pid));


    let mut combined=if let Some(mut companion)=companion.filter(|data|data.game_fps.is_some()||data.has_world){
        if let Some(bridge)=bridge{for(key,value)in bridge.labels{companion.labels.insert(key,value);}if !bridge.armor.is_empty(){companion.armor=bridge.armor.clone();}for(key,value)in bridge.metrics{companion.metrics.entry(key).or_insert(value);}for(key,value)in bridge.world_metrics{companion.world_metrics.entry(key).or_insert(value);}companion.frame_time_ms=bridge.frame_time_ms;companion.fps_low=bridge.fps_low;
            if companion.has_world&&bridge.has_world{if bridge.world_time.is_some(){companion.world_time=bridge.world_time;companion.world_time_source=bridge.world_time_source;}if bridge.game_time.is_some(){companion.game_time=bridge.game_time;}if !bridge.biome.is_empty(){companion.biome=bridge.biome;}if !bridge.dimension.is_empty(){companion.dimension=bridge.dimension;}companion.singleplayer|=bridge.singleplayer;companion.server_address=bridge.server_address;}
            for capability in bridge.capabilities{if !companion.capabilities.contains(&capability){companion.capabilities.push(capability);}}
        }Some(companion)
    }else{bridge};
    if let Some(data)=combined.as_mut(){crate::hud_ping::decorate(data);}
    combined
}
fn decode_telemetry(bytes: &[u8], pid: u32, time: u64) -> Option<Telemetry> {
    if bytes.len() > 16384 {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    if [
        "schema_version",
        "pid",
        "timestamp_ms",
        "has_world",
        "game_fps",
        "armor",
    ]
    .iter()
    .any(|k| value.get(k).is_none())
    {
        return None;
    }
    if value["has_world"] == true && ["x", "y", "z"].iter().any(|k| !value[k].is_number()) {
        return None;
    }
    let legacy_capabilities=value.get("capabilities").is_none();
    let mut v: Telemetry = serde_json::from_value(value).ok()?;
    if legacy_capabilities {
        v.capabilities=["fps","coordinates","direction","armor","ping"].into_iter().map(str::to_string).collect();
        if v.map.is_some(){v.capabilities.push("minimap".into());}
    }
    if v.source.is_empty(){v.source="TP Companion".into();}
    if v.coordinates_restricted {for key in ["pitch","yaw","block_light","sky_light","spawn_x","spawn_y","spawn_z","border_distance","border_size","block_entities_chunk"]{v.world_metrics.remove(key);}v.capabilities.retain(|cap|!["coordinates","direction","minimap","biome"].contains(&cap.as_str()));}
    if let Some(map) = &v.map {
        use base64::Engine;
        if map.pixels.len() != 2304
            || base64::engine::general_purpose::STANDARD
                .decode(&map.pixels)
                .ok()?
                .len()
                != 1728
            || map.origin_x.unsigned_abs() > 100_000_000
            || map.origin_z.unsigned_abs() > 100_000_000
        {
            return None;
        }
    }
    if v.metrics.len()>160||v.world_metrics.len()>160||v.labels.len()>32||v.metrics.iter().chain(v.world_metrics.iter()).any(|(k,v)|k.len()>40||!k.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')||!v.is_finite()||v.abs()>1e16)||v.labels.iter().any(|(k,v)|k.len()>40||v.len()>512){return None;}
    if v.schema_version != 1
        || v.pid != pid
        || v.timestamp_ms > time + 1000
        || time.saturating_sub(v.timestamp_ms) > 3500
        || v.game_fps.is_some_and(|fps|!fps.is_finite()||!(0.0..=100_000.).contains(&fps))
        || v.source.len()>64 || v.status.len()>512 || v.capabilities.len()>128
        || v.capabilities.iter().any(|cap|!["fps","frame_time","fps_low","days","biome","coordinates","direction","armor","ping","minimap","combat","controls","no_fog","optical_zoom"].contains(&cap.as_str()))
        || [v.frame_time_ms,v.fps_low,v.world_time,v.game_time].iter().flatten().any(|n|!n.is_finite()||*n<0.0||*n>1e15)
        || v.attack_cooldown.is_some_and(|n|!n.is_finite()||!(0.0..=1.).contains(&n))
        || [v.target_distance,v.speed].iter().flatten().any(|n|!n.is_finite()||!(0.0..=10000.).contains(n))
        || !["","hostile","passive","other"].contains(&v.target_kind.as_str())
        || v.world_time_source.len()>64
        || v.biome.len()>128||v.dimension.len()>128||v.server_address.len()>512
        || ![v.x, v.y, v.z]
            .iter()
            .all(|n| n.is_finite() && n.abs() < 100_000_000.)
        || v.armor.len() > 6
        || v.direction.len() > 24
        || v.armor
            .iter()
            .any(|a| a.name.len() > 256 || a.remaining > a.maximum)
    {
        return None;
    }
    if v.has_world{if let Some(distance)=v.target_distance{v.world_metrics.insert("target_distance".into(),distance);}}
    Some(v)
}
#[tauri::command]
pub fn get_overlay_snapshot() -> Result<OverlaySnapshot, String> {
    let mut snapshot=frame().lock().map(|f|f.clone()).map_err(|_|"HUD ocupado".to_string())?;


    let pids=crate::game_session::tracked_pids();
    let pid=if pids.contains(&snapshot.pid){Some(snapshot.pid)}else{pids.first().copied()};
    if let Some(pid)=pid{if let Some(data)=telemetry(pid){
        snapshot.pid=pid;snapshot.attached=true;snapshot.updated_at=now();
        snapshot.game_fps=data.game_fps;snapshot.fps_source=data.source.clone();snapshot.message=data.status.clone();snapshot.telemetry=Some(data);
    }}
    Ok(snapshot)
}
#[tauri::command]
pub fn get_launcher_overlay(app: AppHandle) -> bool {
    app.get_webview_window(LABEL).is_some()
}
#[tauri::command]
pub fn publish_launcher_overlay(app: AppHandle, state: OverlayState) -> Result<(), String> {
    frame().lock().map_err(|_| "HUD ocupado")?.state = state.clone();
    if state.running && !get_launcher_overlay(app.clone()) {
        let auto = crate::launcher_plugins::get_launcher_plugins(app.clone())
            .is_ok_and(|w| w.profile.native_overlay);
        if auto {
            tauri::async_runtime::spawn(async move {
                let _ = set_launcher_overlay(app, true).await;
            });
        }
    }
    Ok(())
}
#[tauri::command]
pub fn set_hud_interactive(app: AppHandle, editing: bool) -> Result<(), String> {
    let window = app
        .get_webview_window(LABEL)
        .ok_or("Activa el HUD desde Plugins.")?;
    #[cfg(windows)]
    {
        let current = frame().lock().map(|f| f.pid).unwrap_or_default();
        let mut pids = crate::game_session::tracked_pids();
        if pids.contains(&current) {
            pids.retain(|p| *p == current);
        }
        let target =
            crate::hud_windows::find(&pids).ok_or("No hay una ventana Minecraft disponible.")?;
        let overlay = window.hwnd().map_err(|e| e.to_string())?.0 as usize;
        let fg = crate::hud_windows::foreground();
        if fg != target.hwnd && fg != overlay {
            return Err("Abre Minecraft antes de editar su HUD.".into());
        }
        let title=crate::launcher_plugins::get_launcher_plugins(app.clone()).is_ok_and(|p|p.profile.title_theme)&&frame().lock().is_ok_and(|f|f.telemetry.as_ref().is_some_and(|v|v.labels.get("screen").is_some_and(|s|s=="title")));
        window
            .set_ignore_cursor_events(!(editing||title))
            .map_err(|e| e.to_string())?;
        window.set_focusable(editing||title).map_err(|e| e.to_string())?;
        EDITING.store(editing, Ordering::Release);
        if editing {
            crate::hud_windows::release_cursor();
            let _ = window.set_focus();
            crate::hud_windows::focus(overlay);
        } else {
            crate::hud_windows::focus(target.hwnd);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (window, editing);
        return Err("El HUD universal está disponible en Windows.".into());
    }
    Ok(())
}
#[tauri::command]
pub async fn hud_debug_shortcut(app:AppHandle,shortcut:String)->Result<(),String>{
    let letter=match shortcut.as_str(){"hitboxes"=>0x42,"chunks"=>0x47,"clear_chat"=>0x44,"tooltips"=>0x48,"reload"=>0x54,_=>return Err("Atajo no permitido".into())};
    #[cfg(windows)]{
        let pid=frame().lock().map(|v|v.pid).unwrap_or_default();if !crate::game_session::tracked_pids().contains(&pid){return Err("No hay una partida activa".into());}
        set_hud_interactive(app,false)?;
        tokio::time::sleep(Duration::from_millis(60)).await;
        let target=crate::hud_windows::find(&[pid]).ok_or("Minecraft ya no está disponible")?;
        crate::hud_windows::debug_shortcut(target,letter)
    }
    #[cfg(not(windows))]{let _=(app,letter);Err("Este atajo requiere Windows".into())}
}
#[tauri::command]
pub async fn set_launcher_overlay(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let _lock = CREATE.lock().await;
    if !enabled {
        #[cfg(windows)]
        if let Some(w) = app.get_webview_window(LABEL) {
            if w.hwnd()
                .is_ok_and(|h| h.0 as usize == crate::hud_windows::foreground())
            {
                let _ = set_hud_interactive(app.clone(), false);
            }
        }
        EPOCH.fetch_add(1, Ordering::AcqRel);
        EDITING.store(false, Ordering::Release);
        if let Some(w) = app.get_webview_window(LABEL) {
            let _ = w.set_ignore_cursor_events(true);
            w.close().map_err(|e| e.to_string())?;
        }
        return Ok(false);
    }
    if !cfg!(windows) {
        return Err("El HUD universal necesita Windows.".into());
    }
    if get_launcher_overlay(app.clone()) {
        return Ok(true);
    }
    let w = WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("/overlay".into()))
        .title("TP LAUNCHER · HUD universal")
        .inner_size(960., 540.)
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(false)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;
    w.set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())?;
    w.set_focusable(false).map_err(|e| e.to_string())?;
    let epoch = EPOCH.fetch_add(1, Ordering::AcqRel) + 1;
    #[cfg(windows)]
    std::thread::spawn(move || run(app, epoch));
    #[cfg(not(windows))]
    let _ = (app, epoch);
    Ok(true)
}
#[derive(Default)]
struct Clicks {
    last_left: bool,
    last_right: bool,
    left: VecDeque<Instant>,
    right: VecDeque<Instant>,
}
impl Clicks {
    fn update(&mut self, k: &HudKeys, t: Instant) {
        if k.lmb && !self.last_left {
            self.left.push_back(t);
        }
        if k.rmb && !self.last_right {
            self.right.push_back(t);
        }
        self.last_left = k.lmb;
        self.last_right = k.rmb;
        while self
            .left
            .front()
            .is_some_and(|v| t.duration_since(*v) >= Duration::from_secs(1))
        {
            self.left.pop_front();
        }
        while self
            .right
            .front()
            .is_some_and(|v| t.duration_since(*v) >= Duration::from_secs(1))
        {
            self.right.pop_front();
        }
    }
}
#[cfg(windows)]
fn run(app: AppHandle, epoch: u64) {
    use crate::hud_windows as win;
    let Ok(_worker) = WORKER.lock() else {
        return;
    };
    if EPOCH.load(Ordering::Acquire) != epoch {
        return;
    }
    let mut target = None;
    let mut borderless: Option<win::Borderless> = None;
    let mut last_bounds = None;
    let mut visible = false;
    let mut title_input=false;
    let mut last_hotkey = false;
    let mut shortcut: Option<win::HudHotkey> = None;
    let mut clicks = Clicks::default();
    let mut profile = crate::launcher_plugins::get_launcher_plugins(app.clone())
        .map(|w| w.profile)
        .unwrap_or_default();
    let mut profile_json = serde_json::to_string(&profile).unwrap_or_default();
    let mut config_tick = Instant::now() - Duration::from_secs(2);
    let mut emit_tick = config_tick;
    let mut discover_tick = config_tick;
    let mut metrics_tick = config_tick;
    let mut lease_tick = config_tick;
    let mut input_ready = false;
    let mut data = None;
    let mut ram = (0., 0.);
    while EPOCH.load(Ordering::Acquire) == epoch {
        let Some(w) = app.get_webview_window(LABEL) else {
            break;
        };
        let Ok(hwnd) = w.hwnd() else {
            break;
        };
        let overlay = hwnd.0 as usize;
        let time = Instant::now();
        if config_tick.elapsed() > Duration::from_millis(500) {
            config_tick = time;
            if let Ok(ws) = crate::launcher_plugins::get_launcher_plugins(app.clone()) {
                let json = serde_json::to_string(&ws.profile).unwrap_or_default();
                if json != profile_json {
                    profile = ws.profile;
                    profile_json = json;
                    let _ = app.emit_to(LABEL, "tp-hud-profile", &profile);
                }
            }
            ram = memory();
        }
        let pids = crate::game_session::tracked_pids();
        if discover_tick.elapsed() > Duration::from_millis(250)
            || target.is_some_and(|t: win::GameWindow| !win::valid(t) || !pids.contains(&t.pid))
        {
            discover_tick = time;
            let fg = win::foreground();
            let next = if let Some(t) = target.filter(|t| win::valid(*t) && pids.contains(&t.pid)) {
                if fg == t.hwnd || fg == overlay {
                    Some(t)
                } else {
                    win::find(&pids)
                }
            } else {
                win::find(&pids)
            };
            if next != target {
                borderless = None;
                target = next;
                last_bounds = None;
                data = None;
                EDITING.store(false, Ordering::Release);
                let _ = w.set_ignore_cursor_events(true);
                let _ = w.set_focusable(false);
            }
        }
        let fg = win::foreground();

        if let Some(t)=target.filter(|t|fg==t.hwnd){if let Ok(profile)=crate::launcher_plugins::profile_path(){if let Some(folder)=profile.parent(){let path=folder.join(format!("editor-{}.request",t.pid));if let Ok(meta)=std::fs::symlink_metadata(&path){if meta.is_file()&&!meta.file_type().is_symlink()&&meta.len()<32{let fresh=meta.modified().ok().and_then(|time|time.elapsed().ok()).is_some_and(|age|age<Duration::from_secs(3));let _=std::fs::remove_file(path);if fresh{let _=w.set_ignore_cursor_events(false);let _=w.set_focusable(true);EDITING.store(true,Ordering::Release);win::release_cursor();let _=w.set_focus();}}}}}}
        let editing = EDITING.load(Ordering::Acquire);
        let title=profile.title_theme&&data.as_ref().is_some_and(|v: &Telemetry|v.labels.get("screen").is_some_and(|v|v=="title"));
        if title!=title_input{title_input=title;let _=w.set_ignore_cursor_events(!(title||editing));let _=w.set_focusable(title||editing);if !title&&fg==overlay&&!editing{if let Some(t)=target{win::focus(t.hwnd);}}}
        let active = target.is_some_and(|t| fg == t.hwnd) || ((editing||title) && fg == overlay);
        let shown = active && target.and_then(win::bounds).is_some();
        if !shown {
            let notify = visible || editing;
            shortcut = None;
            if visible {
                let _ = w.hide();
                visible = false;
            }
            if editing {
                EDITING.store(false, Ordering::Release);
                let _ = w.set_ignore_cursor_events(true);
                let _ = w.set_focusable(false);
            }
            clicks = Clicks::default();
            input_ready = false;
            last_hotkey = false;
            if let Ok(mut f) = frame().lock() {
                f.keys = HudKeys::default();
                f.left_cps = 0;
                f.right_cps = 0;
                f.focused = false;
                f.editing = false;
                f.game_fps = None;
                f.telemetry = None;
                f.attached = target.is_some();
                if notify {
                    let _ = app.emit_to(LABEL, "tp-hud-frame", &*f);
                }
            }
            std::thread::sleep(Duration::from_millis(120));
            continue;
        }
        let Some(t) = target else {
            continue;
        };
        let wants_borderless=profile.auto_borderless||data.as_ref().is_some_and(|v|v.labels.get("fullscreen_borderless").is_some_and(|v|v=="true"));
        if wants_borderless && borderless.as_ref().map(win::Borderless::target) != Some(t) {
            borderless = win::Borderless::apply(t);
            last_bounds = None;
        } else if !wants_borderless && borderless.is_some() {
            borderless = None;
            last_bounds = None;
        }
        if let Some(mode)=&borderless{mode.maintain();}
        if let Some(r) = win::bounds(t) {
            if last_bounds != Some(r) {
                win::fit(overlay, r);
                last_bounds = Some(r);
            }
        }
        if !visible {
            let _ = w.show();
            visible = true;
        }
        if shortcut.is_none() {
            shortcut = Some(win::HudHotkey::new());
        }
        let hotkey = win::key(0xA1)
            || (!shortcut.as_ref().is_some_and(|s| s.registered()) && win::key(0x77));
        let triggered = shortcut.as_ref().is_some_and(|s| s.pressed());
        if triggered || (hotkey && !last_hotkey) {
            let _ = set_hud_interactive(app.clone(), !editing);
        }
        last_hotkey = hotkey;
        if EDITING.load(Ordering::Acquire) && win::key(0x1b) {
            let _ = set_hud_interactive(app.clone(), false);
        }
        let editing = EDITING.load(Ordering::Acquire);
        let keys = if editing {
            HudKeys::default()
        } else {
            let (left, right) = win::mouse();
            HudKeys {
                w: win::key(0x57),
                a: win::key(0x41),
                s: win::key(0x53),
                d: win::key(0x44),
                lmb: left,
                rmb: right,
            }
        };
        if editing {
            clicks = Clicks::default();
        } else {
            if !input_ready {
                clicks.last_left = keys.lmb;
                clicks.last_right = keys.rmb;
                input_ready = true;
            }
            clicks.update(&keys, time);
        }
        if lease_tick.elapsed() > Duration::from_secs(1) {
            lease_tick = time;
            lease(t.pid);
        }
        if metrics_tick.elapsed() > Duration::from_millis(250) {
            metrics_tick = time;
            data = telemetry(t.pid);
        }
        let internal_fps=data.as_ref().and_then(|v|v.game_fps);
        if emit_tick.elapsed() > Duration::from_millis(33) {
            emit_tick = time;
            let internal = internal_fps;
            if let Ok(mut f) = frame().lock() {
                f.total_memory_gb = ram.0;
                f.available_memory_gb = ram.1;
                f.updated_at = now();
                f.game_fps = internal;
                f.fps_source = data.as_ref().map(|d|d.source.as_str()).unwrap_or("Conectando con Minecraft").into();
                f.message = data.as_ref().map(|d|d.status.clone()).filter(|s|!s.is_empty()).unwrap_or_else(||
                    "Sin datos internos: si activaste el HUD con el juego abierto, reinicia Minecraft desde TP LAUNCHER.".into());
                f.keys = keys;
                f.left_cps = clicks.left.len();
                f.right_cps = clicks.right.len();
                f.editing = editing;
                f.attached = true;
                f.focused = true;
                f.pid = t.pid;
                f.telemetry = data.clone();
                let _ = app.emit_to(LABEL, "tp-hud-frame", &*f);
            }
        }
        std::thread::sleep(Duration::from_millis(8));
    }
    drop(borderless);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hud_telemetry_rejects_stale_wrong_pid_and_missing_coordinates() {
        let mut v = serde_json::json!({"schema_version":1,"pid":42,"timestamp_ms":10000,"has_world":true,"game_fps":144,"x":12,"y":64,"z":-8,"direction":"north","armor":[]});
        let bytes = serde_json::to_vec(&v).unwrap();
        assert!(decode_telemetry(&bytes, 42, 10001).is_some());
        assert!(decode_telemetry(&bytes, 43, 10001).is_none());
        assert!(decode_telemetry(&bytes, 42, 15000).is_none());
        v.as_object_mut().unwrap().remove("x");
        assert!(decode_telemetry(&serde_json::to_vec(&v).unwrap(), 42, 10001).is_none());
    }
    #[test]
    fn hud_telemetry_bounds_map_pixels_and_origin() {
        use base64::Engine;
        let pixels = base64::engine::general_purpose::STANDARD.encode([0u8; 1728]);
        let mut v = serde_json::json!({"schema_version":1,"pid":42,"timestamp_ms":10000,"has_world":false,"game_fps":60,"armor":[],"map":{"pixels":pixels,"origin_x":0,"origin_z":0}});
        assert!(decode_telemetry(&serde_json::to_vec(&v).unwrap(), 42, 10001).is_some());
        v["map"]["origin_x"] = serde_json::json!(i32::MIN);
        assert!(decode_telemetry(&serde_json::to_vec(&v).unwrap(), 42, 10001).is_none());
        v["map"]["origin_x"] = serde_json::json!(0);
        v["map"]["pixels"] = serde_json::json!("invalid");
        assert!(decode_telemetry(&serde_json::to_vec(&v).unwrap(), 42, 10001).is_none());
    }
    #[test]
    fn hud_cps_counts_only_edges() {
        let mut c = Clicks::default();
        let t = Instant::now();
        let mut k = HudKeys {
            lmb: true,
            ..Default::default()
        };
        c.update(&k, t);
        c.update(&k, t + Duration::from_millis(10));
        assert_eq!(c.left.len(), 1);
        k.lmb = false;
        c.update(&k, t + Duration::from_millis(20));
        k.lmb = true;
        c.update(&k, t + Duration::from_millis(40));
        assert_eq!(c.left.len(), 2);
        k.lmb = false;
        c.update(&k, t + Duration::from_millis(1100));
        assert!(c.left.is_empty());
    }
    #[test]fn hud_bridge_accepts_optional_fps_and_respects_reduced_coordinates(){
        let v=serde_json::json!({"schema_version":1,"pid":7,"timestamp_ms":10000,"has_world":true,"game_fps":null,"source":"TP Bridge","capabilities":["coordinates","direction"],"coordinates_restricted":true,"x":0,"y":0,"z":0,"armor":[]});
        let data=decode_telemetry(&serde_json::to_vec(&v).unwrap(),7,10001).unwrap();assert!(data.game_fps.is_none());assert!(!data.capabilities.contains(&"coordinates".into()));assert_eq!(data.source,"TP Bridge");
    }
}
