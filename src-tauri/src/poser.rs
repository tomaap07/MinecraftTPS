
use base64::Engine;
use serde_json::Value;
use std::{fs, io::Cursor, path::PathBuf, sync::Mutex};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

static LIBRARY_LOCK: Mutex<()> = Mutex::new(());

fn library_path() -> Result<PathBuf, String> {
    Ok(crate::root_dir()?.join("custom_poses.json"))
}
fn read_poses() -> Result<Vec<Value>, String> {
    let path = library_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path).map_err(|e| format!("No se pudo leer el catálogo de poses: {e}"))?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("El catálogo de poses supera el límite de 8 MB.".into());
    }
    let poses: Vec<Value> = serde_json::from_slice(&bytes)
        .map_err(|e| format!("El catálogo de poses está dañado: {e}"))?;
    for pose in &poses {
        validate_pose(pose)
            .map_err(|e| format!("Hay una pose no válida en custom_poses.json: {e}"))?;
    }
    Ok(poses)
}
fn write_json(path: &std::path::Path, value: &impl serde::Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let temp = path.with_extension("json.tmp");
    fs::write(&temp, bytes).map_err(|e| format!("No se pudo guardar la biblioteca: {e}"))?;
    fs::rename(&temp, path).map_err(|e| format!("No se pudo confirmar el guardado: {e}"))
}
fn validate_vector(value: &Value, limit: f64) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() == 3
            && items.iter().all(|v| {
                v.as_f64()
                    .is_some_and(|n| n.is_finite() && n.abs() <= limit)
            })
    })
}
fn validate_pose(pose: &Value) -> Result<(), String> {
    let id = pose["id"]
        .as_str()
        .ok_or("La pose no tiene identificador.")?;
    if !id.starts_with("custom-")
        || id.len() > 100
        || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err("Identificador de pose no válido.".into());
    }
    let name = pose["name"].as_str().ok_or("La pose no tiene nombre.")?;
    if name.trim().is_empty() || name.chars().count() > 80 {
        return Err("El nombre debe tener de 1 a 80 caracteres.".into());
    }
    for joint in [
        "head", "body", "rightArm", "leftArm", "rightLeg", "leftLeg", "cape",
    ] {
        if !validate_vector(&pose["joints"][joint], 180.0) {
            return Err(format!(
                "Ángulos no válidos en {joint}; usa valores entre −180 y 180 grados."
            ));
        }
    }
    if !validate_vector(&pose["position"], 100.0) || !validate_vector(&pose["rotation"], 180.0) {
        return Err("La posición o inclinación del personaje no es válida.".into());
    }
    Ok(())
}

#[tauri::command]
pub fn list_custom_poses() -> Result<Vec<Value>, String> {
    let _guard = LIBRARY_LOCK
        .lock()
        .map_err(|_| "La biblioteca de poses está ocupada.")?;
    read_poses()
}

#[tauri::command]
pub fn save_custom_pose(mut pose: Value) -> Result<Value, String> {
    validate_pose(&pose)?;
    let _guard = LIBRARY_LOCK
        .lock()
        .map_err(|_| "La biblioteca de poses está ocupada.")?;
    let mut poses = read_poses()?;
    let id = pose["id"].as_str().unwrap_or_default().to_owned();
    poses.retain(|entry| entry["id"].as_str() != Some(&id));
    if poses.len() >= 2000 {
        return Err("La biblioteca admite hasta 2000 poses personalizadas.".into());
    }
    pose["name"] = Value::String(pose["name"].as_str().unwrap_or_default().trim().to_owned());
    pose["category"] = Value::String("Personalizadas".into());
    poses.push(pose.clone());
    write_json(&library_path()?, &poses)?;
    Ok(pose)
}

#[tauri::command]
pub fn delete_custom_pose(id: String) -> Result<(), String> {
    let _guard = LIBRARY_LOCK
        .lock()
        .map_err(|_| "La biblioteca de poses está ocupada.")?;
    let mut poses = read_poses()?;
    let old_len = poses.len();
    poses.retain(|entry| entry["id"].as_str() != Some(&id));
    if old_len == poses.len() {
        return Err("La pose ya no existe en la biblioteca.".into());
    }
    write_json(&library_path()?, &poses)?;
    Ok(())
}

#[tauri::command]
pub fn save_pose_render(image_base64: String, name: String) -> Result<String, String> {
    if image_base64.len() > 48 * 1024 * 1024 {
        return Err("El render supera el límite de 48 MB.".into());
    }
    let encoded = image_base64
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(&image_base64);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| "El PNG del render no es válido.")?;
    let reader = image::ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| "No se pudo leer el PNG del render.")?;
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
    {
        return Err("El render debe ser un PNG de hasta 4096 × 4096 píxeles.".into());
    }
    let safe_name: String = name
        .chars()
        .take(60)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let directory = crate::root_dir()?.join("renders");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join(format!(
        "{}-{}.png",
        if safe_name.is_empty() {
            "Poser"
        } else {
            &safe_name
        },
        uuid::Uuid::new_v4()
    ));
    fs::write(&path, bytes).map_err(|e| format!("No se pudo guardar el render: {e}"))?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn open_pose_renders(app: AppHandle) -> Result<(), String> {
    let path = crate::root_dir()?.join("renders");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(path.to_string_lossy().as_ref(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incomplete_or_nonfinite_joint_data() {
        let mut pose = serde_json::json!({"id":"custom-demo","name":"Demo","position":[0,0,0],"rotation":[0,0,0],"joints":{}});
        assert!(validate_pose(&pose).is_err());
        for joint in [
            "head", "body", "rightArm", "leftArm", "rightLeg", "leftLeg", "cape",
        ] {
            pose["joints"][joint] = serde_json::json!([0, 0, 0]);
        }
        assert!(validate_pose(&pose).is_ok());
        pose["joints"]["head"] = serde_json::json!([181, 0, 0]);
        assert!(validate_pose(&pose).is_err());
    }
}
