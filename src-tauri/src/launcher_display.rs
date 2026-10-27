
use serde::Serialize;
use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

#[derive(Serialize)]
pub struct LauncherDisplayInfo {
    pub width: u32,
    pub height: u32,
    pub work_width: u32,
    pub work_height: u32,
    pub scale_factor: f64,
    pub maximized: bool,
}

fn main_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main").ok_or_else(|| "No se encontró la ventana del launcher.".into())
}

fn dpi(window: &WebviewWindow) -> Result<f64, String> {
    let factor = window.scale_factor().map_err(|e| format!("No se pudo consultar la escala del monitor: {e}"))?;
    Ok(if factor.is_finite() && factor > 0.0 { factor } else { 1.0 })
}

struct WorkArea {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    frame_width: u32,
    frame_height: u32,
}

fn work_area(window: &WebviewWindow) -> Result<WorkArea, String> {
    let monitor = match window.current_monitor().map_err(|e| e.to_string())? {
        Some(monitor) => monitor,
        None => window.primary_monitor().map_err(|e| e.to_string())?
            .ok_or("No se pudo determinar el área disponible del monitor.")?,
    };
    let area = monitor.work_area();
    if area.size.width == 0 || area.size.height == 0 {
        return Err("El monitor no informó un área de trabajo válida.".into());
    }
    let inner = window.inner_size().map_err(|e| e.to_string())?;
    let outer = window.outer_size().map_err(|e| e.to_string())?;
    Ok(WorkArea { x: area.position.x, y: area.position.y,
        width: area.size.width, height: area.size.height,
        frame_width: outer.width.saturating_sub(inner.width),
        frame_height: outer.height.saturating_sub(inner.height) })
}

fn info(window: &WebviewWindow) -> Result<LauncherDisplayInfo, String> {
    let scale_factor = dpi(window)?;
    let size = window.inner_size().map_err(|e| e.to_string())?.to_logical::<f64>(scale_factor);
    let area = work_area(window)?;
    Ok(LauncherDisplayInfo {
        width: size.width.round().max(1.0) as u32,
        height: size.height.round().max(1.0) as u32,
        work_width: ((area.width.saturating_sub(area.frame_width)) as f64 / scale_factor).floor().max(1.0) as u32,
        work_height: ((area.height.saturating_sub(area.frame_height)) as f64 / scale_factor).floor().max(1.0) as u32,
        scale_factor,
        maximized: window.is_maximized().map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub fn get_launcher_display_info(app: AppHandle) -> Result<LauncherDisplayInfo, String> {
    info(&main_window(&app)?)
}

#[tauri::command]
pub fn preview_launcher_scale(app: AppHandle, percent: u32) -> Result<(), String> {
    if !(80..=200).contains(&percent) {
        return Err("El tamaño de la interfaz debe estar entre 80 % y 200 %.".into());
    }
    main_window(&app)?.set_zoom(f64::from(percent) / 100.0)
        .map_err(|e| format!("No se pudo ajustar el tamaño de la interfaz: {e}"))
}

fn apply_window(window: &WebviewWindow, width: u32, height: u32, maximized: bool) -> Result<LauncherDisplayInfo, String> {
    if !(320..=7680).contains(&width) || !(240..=4320).contains(&height) {
        return Err("El tamaño del launcher debe estar entre 320 × 240 y 7680 × 4320.".into());
    }
    if window.is_fullscreen().map_err(|e| e.to_string())? {
        window.set_fullscreen(false).map_err(|e| e.to_string())?;
    }
    window.unmaximize().map_err(|e| format!("No se pudo restaurar la ventana: {e}"))?;
    let factor = dpi(window)?;
    let area = work_area(window)?;
    let max_width = ((area.width.saturating_sub(area.frame_width)) as f64 / factor).floor().max(1.0) as u32;
    let max_height = ((area.height.saturating_sub(area.frame_height)) as f64 / factor).floor().max(1.0) as u32;

    let min_width = 860.min(max_width);
    let min_height = 620.min(max_height);
    let width = width.clamp(min_width, max_width);
    let height = height.clamp(min_height, max_height);
    window.set_min_size(Some(LogicalSize::new(f64::from(min_width), f64::from(min_height))))
        .map_err(|e| format!("No se pudo ajustar el mínimo de la ventana: {e}"))?;
    window.set_size(LogicalSize::new(f64::from(width), f64::from(height)))
        .map_err(|e| format!("No se pudo cambiar el tamaño del launcher: {e}"))?;

    let physical_width = (f64::from(width) * factor).round() as u32 + area.frame_width;
    let physical_height = (f64::from(height) * factor).round() as u32 + area.frame_height;
    let x = i64::from(area.x) + i64::from(area.width.saturating_sub(physical_width) / 2);
    let y = i64::from(area.y) + i64::from(area.height.saturating_sub(physical_height) / 2);
    window.set_position(PhysicalPosition::new(x.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        y.clamp(i32::MIN as i64, i32::MAX as i64) as i32)).map_err(|e| e.to_string())?;
    if maximized { window.maximize().map_err(|e| format!("No se pudo maximizar la ventana: {e}"))?; }
    info(window)
}

#[tauri::command]
pub fn apply_launcher_window(app: AppHandle, width: u32, height: u32, maximized: bool) -> Result<LauncherDisplayInfo, String> {
    apply_window(&main_window(&app)?, width, height, maximized)
}


pub fn restore_display(app: &AppHandle) -> Result<(), String> {
    let settings = super::launcher_settings::load().unwrap_or_default();
    let window = main_window(app)?;
    window.set_zoom(f64::from(settings.ui_scale_percent) / 100.0)
        .map_err(|e| format!("No se pudo recuperar el tamaño de la interfaz: {e}"))?;
    apply_window(&window, settings.launcher_width, settings.launcher_height, settings.launcher_maximized)?;
    Ok(())
}
