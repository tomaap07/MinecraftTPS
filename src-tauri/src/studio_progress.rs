
use serde::Serialize;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct StudioProgress {
    pub id: String,
    pub title: String,
    pub stage: String,
    pub detail: String,
    pub percent: f64,
    pub current: usize,
    pub total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub status: String,
}
static CURRENT: OnceLock<Mutex<Option<StudioProgress>>> = OnceLock::new();

pub static INSTALL_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Clone)]
pub struct Operation {
    pub app: Option<AppHandle>,
    state: Arc<Mutex<StudioProgress>>,
}
impl Operation {
    pub fn new(app: Option<AppHandle>, title: &str) -> Self {
        let result = Self {
            app,
            state: Arc::new(Mutex::new(StudioProgress {
                id: uuid::Uuid::new_v4().to_string(),
                title: title.into(),
                stage: "Preparando instalación".into(),
                detail: String::new(),
                percent: 0.0,
                current: 0,
                total: 0,
                bytes_done: 0,
                bytes_total: 0,
                status: "running".into(),
            })),
        };
        result.update(|_| {});
        result
    }
    fn update(&self, change: impl FnOnce(&mut StudioProgress)) {
        if let Ok(mut state) = self.state.lock() {
            change(&mut state);
            if let Ok(mut current) = CURRENT.get_or_init(Default::default).lock() {
                *current = Some(state.clone());
            }
            if let Some(app) = &self.app {
                let _ = app.emit("studio-progress", state.clone());
            }
        }
    }
    pub fn stage(&self, title: &str, detail: &str, percent: f64) {
        self.update(|s| {
            s.stage = title.into();
            s.detail = detail.into();
            s.percent = percent.clamp(0.0, 99.0);
            s.bytes_done = 0;
            s.bytes_total = 0;
        });
        self.log(&format!("{title}: {detail}"));
    }
    pub fn file(&self, name: &str, current: usize, total: usize) {
        self.update(|s| {
            s.stage = "Instalando archivos del modpack".into();
            s.detail = name.into();
            s.current = current;
            s.total = total;
            s.percent = 20.0 + 70.0 * current as f64 / total.max(1) as f64;
            s.bytes_done = 0;
            s.bytes_total = 0;
        });
    }
    pub fn bytes(&self, done: u64, total: u64) {
        self.update(|s| {
            s.bytes_done = done;
            s.bytes_total = total;
            if s.total == 0 && total > 0 {
                s.percent = 3.0 + 5.0 * done.min(total) as f64 / total as f64;
            }
            if s.total > 0 && total > 0 {
                s.percent = 20.0
                    + 70.0 * (s.current as f64 + done.min(total) as f64 / total as f64)
                        / s.total as f64;
            }
        });
    }
    pub fn log(&self, message: &str) {
        if let Some(app) = &self.app {
            crate::emit_launcher_log(app, "modpack", message);
        }
    }
    pub fn complete<T>(&self, result: Result<T, String>, success: &str) -> Result<T, String> {
        self.update(|s| match &result {
            Ok(_) => {
                s.status = "success".into();
                s.stage = "Instalación completada".into();
                s.detail = success.into();
                s.percent = 100.0;
                s.current = s.total;
            }
            Err(error) => {
                s.status = "error".into();
                s.stage = "No se pudo completar la instalación".into();
                s.detail = error.clone();
            }
        });
        self.log(result.as_ref().err().map(String::as_str).unwrap_or(success));
        result
    }
}
#[tauri::command]
pub fn studio_current_operation() -> Option<StudioProgress> {
    CURRENT.get_or_init(Default::default).lock().ok()?.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_and_failure_are_distinct_and_progress_counts_bytes() {
        let op = Operation::new(None, "Pack");
        op.file("example.jar", 1, 4);
        op.bytes(50, 100);
        assert_eq!(op.state.lock().unwrap().percent, 46.25);
        let _: Result<(), _> = op.complete(Err("Falta un archivo".into()), "OK");
        assert_eq!(op.state.lock().unwrap().status, "error");
        let _: Result<(), _> = op.complete(Ok(()), "OK");
        assert_eq!(op.state.lock().unwrap().percent, 100.0);
    }
}
