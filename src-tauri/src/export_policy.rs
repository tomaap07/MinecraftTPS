
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::{Cursor, Read, Write},
    path::{Component, Path, PathBuf},
};

const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_METADATA_BYTES: u64 = 256 * 1024;
const MAX_EXPORT_FILES: usize = 20_000;
const MAX_EXPORT_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportOmission {
    pub name: String,
    pub version: String,
    pub filename: String,
    pub reason: String,
    pub project_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExportResult {
    pub path: String,
    pub format: String,
    pub included_files: usize,
    pub referenced_files: usize,
    pub omitted: Vec<ExportOmission>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct DownloadRecord {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub version_id: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub sha1: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub size: u64,
}

#[derive(Clone, Default, Debug)]
struct JarIdentity {
    name: String,
    version: String,
    license: String,
    embedded_license: String,
    project_url: Option<String>,
    forbidden: Option<String>,
}

pub(crate) struct ExportPlan {
    pub files: Vec<(PathBuf, String)>,
    pub references: Vec<Value>,
    pub omitted: Vec<ExportOmission>,
}

fn safe_relative(name: &str) -> bool {
    !name.is_empty()
        && !name.contains(['\\', ':'])
        && !name.chars().any(char::is_control)
        && Path::new(name)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        && name.split('/').all(|part| {
            let device=part.split('.').next().unwrap_or_default().to_ascii_lowercase();
            !part.starts_with('.') && !part.ends_with([' ', '.']) && part.len()<=240
                && !["con","prn","aux","nul","com1","com2","com3","com4","com5","com6","com7","com8","com9","lpt1","lpt2","lpt3","lpt4","lpt5","lpt6","lpt7","lpt8","lpt9"].contains(&device.as_str())
        })
}

fn read_entry<R: Read + std::io::Seek>(jar: &mut zip::ZipArchive<R>, name: &str) -> Option<String> {
    let mut entry = jar.by_name(name).ok()?;
    if entry.size() > MAX_METADATA_BYTES {
        return None;
    }
    let mut bytes = Vec::new();
    entry
        .by_ref()
        .take(MAX_METADATA_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_METADATA_BYTES {
        return None;
    }
    String::from_utf8(bytes).ok()
}

fn unambiguous_license(value: &Value) -> String {
    match value {
        Value::String(text) => text.trim().into(),
        Value::Array(items) if items.len() == 1 => {
            items[0].as_str().unwrap_or_default().trim().into()
        }
        _ => String::new(),
    }
}

fn inspect_archive(path: &Path, filename: &str) -> JarIdentity {
    let mut identity = JarIdentity {
        name: filename.into(),
        version: "No declarada".into(),
        ..Default::default()
    };
    if filename.to_ascii_lowercase().contains("optifine") {
        identity.name = filename
            .trim_end_matches(".jar")
            .trim_end_matches(".zip")
            .replace('_', " ");
        identity.version = filename
            .trim_end_matches(".jar")
            .trim_end_matches(".zip")
            .into();
        identity.project_url = Some("https://optifine.net/downloads".into());
        identity.forbidden = Some(
            "OptiFine no se redistribuye con packs; descárgalo por separado desde la web oficial"
                .into(),
        );
    }
    let Ok(file) = fs::File::open(path) else {
        return identity;
    };
    let Ok(mut jar) = zip::ZipArchive::new(file) else {
        return identity;
    };
    if jar.len() > 100_000 {
        identity.forbidden =
            Some("El archivo excede los límites de inspección de seguridad".into());
        return identity;
    }
    let mut licenses = Vec::new();
    for name in jar.file_names() {
        let lower = name.to_ascii_lowercase();
        if lower == "optifine/optifinetweaker.class" || lower.starts_with("net/optifine/") {
            identity.forbidden = Some(
                "Contiene código de OptiFine: no se incluye su binario en packs compartidos".into(),
            );
            identity.project_url = Some("https://optifine.net/downloads".into());
        }
        if [
            "net/minecraft/client/main/main.class",
            "net/minecraft/client/minecraft.class",
            "com/mojang/minecraft/minecraft.class",
        ]
        .contains(&lower.as_str())
            || lower.starts_with("assets/minecraft/textures/")
            || lower.starts_with("assets/minecraft/sounds/")
        {
            identity.forbidden = Some("Contiene cliente o recursos de Minecraft; deben obtenerse de la instalación del destinatario".into());
        }
        if lower == "license"
            || lower == "license.txt"
            || lower == "license.md"
            || lower.starts_with("meta-inf/license")
        {
            licenses.push(name.to_owned());
        }
    }
    if let Some(text) = read_entry(&mut jar, "fabric.mod.json")
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    {
        identity.name = text["name"]
            .as_str()
            .or_else(|| text["id"].as_str())
            .unwrap_or(filename)
            .into();
        identity.version = text["version"].as_str().unwrap_or("No declarada").into();
        identity.license = unambiguous_license(&text["license"]);
        identity.project_url = text
            .pointer("/contact/homepage")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or(identity.project_url);
    } else if let Some(text) = read_entry(&mut jar, "quilt.mod.json")
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    {
        identity.name = text
            .pointer("/quilt_loader/metadata/name")
            .and_then(Value::as_str)
            .unwrap_or(filename)
            .into();
        identity.version = text
            .pointer("/quilt_loader/version")
            .and_then(Value::as_str)
            .unwrap_or("No declarada")
            .into();
        identity.license = unambiguous_license(&text["quilt_loader"]["metadata"]["license"]);
        identity.project_url = text
            .pointer("/quilt_loader/metadata/contact/homepage")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or(identity.project_url);
    } else {
        for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
            if let Some(text) =
                read_entry(&mut jar, name).and_then(|text| text.parse::<toml::Value>().ok())
            {
                identity.license = text
                    .get("license")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .into();
                if let Some(info) = text
                    .get("mods")
                    .and_then(toml::Value::as_array)
                    .filter(|mods| mods.len() == 1)
                    .and_then(|mods| mods.first())
                {
                    identity.name = info
                        .get("displayName")
                        .or_else(|| info.get("modId"))
                        .and_then(toml::Value::as_str)
                        .unwrap_or(filename)
                        .into();
                    identity.version = info
                        .get("version")
                        .and_then(toml::Value::as_str)
                        .unwrap_or("No declarada")
                        .into();
                    identity.project_url = info
                        .get("displayURL")
                        .and_then(toml::Value::as_str)
                        .map(str::to_owned)
                        .or(identity.project_url);
                }
                break;
            }
        }
        if identity.name == filename {
            if let Some(info) = read_entry(&mut jar, "mcmod.info")
                .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            {
                let entries = info.as_array().or_else(|| info["modList"].as_array());
                if let Some(info) = entries
                    .filter(|items| items.len() == 1)
                    .and_then(|items| items.first())
                {
                    identity.name = info["name"].as_str().unwrap_or(filename).into();
                    identity.version = info["version"].as_str().unwrap_or("No declarada").into();
                    identity.project_url = info["url"]
                        .as_str()
                        .map(str::to_owned)
                        .or(identity.project_url);
                }
            }
        }
    }
    if identity.version == "No declarada" || identity.version.contains("${") {
        if let Some(manifest) = read_entry(&mut jar, "META-INF/MANIFEST.MF") {
            if let Some(version) = manifest.lines().find_map(|line| {
                line.strip_prefix("Implementation-Version: ")
                    .or_else(|| line.strip_prefix("Specification-Version: "))
            }) {
                identity.version = version.trim().into();
            }
        }
    }
    for name in licenses.into_iter().take(8) {
        if let Some(text) = read_entry(&mut jar, &name) {
            identity.embedded_license.push_str(&text);
            identity.embedded_license.push('\n');
        }
    }
    identity.project_url = identity.project_url.filter(|url| {
        url::Url::parse(url).is_ok_and(|url| url.scheme() == "https" && url.host_str().is_some())
    });
    identity
}



fn explicit_redistribution(identity: &JarIdentity) -> bool {
    let text = identity.embedded_license.to_ascii_lowercase();
    match identity.license.as_str() {
        "MIT" => {
            text.contains("permission is hereby granted, free of charge")
                && text.contains("copyright notice")
        }
        "Apache-2.0" => {
            text.contains("apache license")
                && text.contains("version 2.0")
                && text.contains("http://www.apache.org/licenses/")
        }
        "BSD-2-Clause" | "BSD-3-Clause" => {
            text.contains("redistribution and use in source and binary forms")
                && text.contains("copyright notice")
        }
        "ISC" => {
            text.contains("permission to use, copy, modify, and/or distribute this software")
                && text.contains("copyright notice")
        }
        "CC0-1.0" => {
            text.contains("creativecommons.org/publicdomain/zero/1.0")
                || text.contains("cc0 1.0 universal")
        }
        _ => false,
    }
}

pub(crate) fn valid_modrinth_download(raw: &str) -> bool {
    url::Url::parse(raw).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("cdn.modrinth.com")
            && url.username().is_empty()
            && url.password().is_none()
            && url.port_or_known_default() == Some(443)
            && url.path().starts_with("/data/")
            && !url.path().contains(['\\', '\0'])
            && url.fragment().is_none()
    })
}

fn reference(path: &Path, name: &str, record: Option<&DownloadRecord>) -> Option<Value> {
    let record = record?;
    if record.provider != "modrinth"
        || record.project_id.is_empty()
        || record.version_id.is_empty()
        || !valid_modrinth_download(&record.url)
        || record.sha1.len() != 40
        || !record.sha1.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES || record.size != bytes.len() as u64 {
        return None;
    }
    let hash = format!("{:x}", Sha1::digest(&bytes));
    if !hash.eq_ignore_ascii_case(&record.sha1) {
        return None;
    }
    Some(
        json!({"path":name, "hashes":{"sha1":hash,"sha512":format!("{:x}",Sha512::digest(&bytes))},"env":{"client":"required","server":"optional"},"downloads":[record.url],"fileSize":bytes.len()}),
    )
}

pub(crate) fn load_records(dir: &Path) -> Vec<DownloadRecord> {
    let path = dir.join("tps-content.json");
    if !fs::metadata(&path).is_ok_and(|m| m.is_file() && m.len() <= 8 * 1024 * 1024) {
        return Vec::new();
    }
    fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<Vec<DownloadRecord>>(&b).ok())
        .unwrap_or_default()
}

fn omission(
    name: &str,
    identity: JarIdentity,
    reason: String,
    record: Option<&DownloadRecord>,
) -> ExportOmission {
    ExportOmission {
        name: identity.name,
        version: if identity.version.is_empty() {
            "No declarada".into()
        } else {
            identity.version
        },
        filename: name.into(),
        reason,
        project_url: identity.project_url.or_else(|| {
            record
                .filter(|r| r.provider == "modrinth" && !r.project_id.is_empty())
                .map(|r| format!("https://modrinth.com/mod/{}", r.project_id))
        }),
    }
}

pub(crate) fn plan(tree: Vec<(PathBuf, String)>, records: &[DownloadRecord]) -> ExportPlan {
    let index = records
        .iter()
        .filter(|r| !r.filename.contains(['/', '\\', ':']))
        .map(|r| (format!("mods/{}", r.filename), r))
        .collect::<HashMap<_, _>>();
    let mut plan = ExportPlan {
        files: Vec::new(),
        references: Vec::new(),
        omitted: Vec::new(),
    };
    let mut total = 0u64;
    for (path, name) in tree {
        let record = index.get(&name).copied();
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&name);
        let lower = name.to_ascii_lowercase();
        let archive =
            lower.ends_with(".jar") || lower.ends_with(".zip") || lower.ends_with(".jar.disabled");
        let regular_file = fs::symlink_metadata(&path).is_ok_and(|m| {
            m.is_file() && !m.file_type().is_symlink() && m.len() <= MAX_ARCHIVE_BYTES
        });
        let mut identity = if archive && regular_file {
            inspect_archive(&path, filename)
        } else {
            JarIdentity {
                name: filename.into(),
                version: "No declarada".into(),
                ..Default::default()
            }
        };
        let denied_root = [
            "versions/",
            "libraries/",
            "assets/",
            "resources/",
            "runtime/",
            "natives/",
            "cache/",
        ]
        .iter()
        .any(|prefix| lower.starts_with(prefix));
        let reason = if !safe_relative(&name) {
            Some("Ruta no segura o reservada para compartir".into())
        } else if denied_root {
            Some(
                "Cliente, bibliotecas o recursos de una instalación local; no se redistribuyen"
                    .into(),
            )
        } else if let Some(reason) = identity.forbidden.clone() {
            Some(reason)
        } else if !regular_file {
            Some("Archivo no regular o superior al límite de 256 MB".into())
        } else if archive {
            if let Some(reference) = reference(&path, &name, record) {
                plan.references.push(reference);
                continue;
            }
            if record.is_some() && !record.is_some_and(|r| r.url.is_empty()) {

                Some("No se pudo verificar la referencia de descarga y su hash; revisa el archivo o su procedencia".into())
            } else if !lower.starts_with("mods/")
                || !lower.ends_with(".jar")
                || !explicit_redistribution(&identity)
            {
                let license = if identity.license.is_empty() {
                    "sin licencia verificable"
                } else {
                    identity.license.as_str()
                };
                Some(format!(
                    "Permiso de redistribución no verificado ({license}); se comparte solo configuración y referencias autorizadas"
                ))
            } else {
                None
            }
        } else {
            let config = lower.starts_with("config/")
                || lower.starts_with("defaultconfigs/")
                || lower.starts_with("kubejs/")
                || lower.starts_with("scripts/")
                || ["options.txt", "optionsof.txt", "optionsshaders.txt"].contains(&lower.as_str());
            let text = [
                "json",
                "json5",
                "toml",
                "cfg",
                "conf",
                "properties",
                "txt",
                "js",
                "zs",
                "yml",
                "yaml",
                "lang",
                "xml",
                "snbt",
            ]
            .contains(
                &path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .as_str(),
            );
            if config
                && text
                && !filename.to_ascii_lowercase().contains("secret")
                && !filename.to_ascii_lowercase().contains("token")
            {
                None
            } else {
                Some(
                    "Archivo ajeno a la configuración compartible o binario sin permiso verificado"
                        .into(),
                )
            }
        };
        if let Some(reason) = reason {
            if identity.name.is_empty() {
                identity.name = filename.into();
            }
            plan.omitted.push(omission(&name, identity, reason, record));
            continue;
        }
        let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        total = total.saturating_add(size);
        if plan.files.len() >= MAX_EXPORT_FILES || total > MAX_EXPORT_BYTES {
            plan.omitted.push(omission(
                &name,
                identity,
                "Supera los límites del pack compartido (20.000 archivos / 1 GB)".into(),
                record,
            ));
            continue;
        }
        plan.files.push((path, name));
    }
    plan.files.sort_by(|a, b| a.1.cmp(&b.1));
    plan.references
        .sort_by_key(|v| v["path"].as_str().unwrap_or_default().to_owned());
    plan.omitted.sort_by(|a, b| a.filename.cmp(&b.filename));
    plan
}

pub(crate) fn write_archive(
    path: &Path,
    format: &str,
    plan: ExportPlan,
    mut manifests: BTreeMap<String, Value>,
    dependencies: Value,
    pack_name: &str,
) -> Result<ExportResult, String> {
    let mut result = ExportResult {
        path: path.to_string_lossy().into_owned(),
        format: format.into(),
        included_files: plan.files.len(),
        referenced_files: plan.references.len(),
        omitted: plan.omitted,
    };


    manifests.insert("modrinth.index.json".into(), json!({"formatVersion":1,"game":"minecraft","versionId":"1.0.0","name":pack_name,"summary":"Creado con TP LAUNCHER; consulta export-report.json","files":plan.references,"dependencies":dependencies}));
    if let Some(pack) = manifests.get_mut("tps-pack.json") {
        if let Some(pack) = pack.as_object_mut() {
            pack.remove("icon_data");
        }
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let writing = (|| -> Result<(), String> {
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, value) in manifests {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            zip.write_all(&serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        zip.start_file("export-report.json", options)
            .map_err(|e| e.to_string())?;

        let report = json!({"policyVersion":1,"format":format,"included_files":result.included_files,"referenced_files":result.referenced_files,"omitted":result.omitted,"notice":"Los archivos omitidos no viajan con el pack. Las referencias verificadas se descargan desde su proveedor. Revisa los permisos de tus configuraciones antes de publicar."});
        zip.write_all(&serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for (source, name) in plan.files {
            zip.start_file(format!("overrides/{name}"), options)
                .map_err(|e| e.to_string())?;
            std::io::copy(
                &mut fs::File::open(source).map_err(|e| e.to_string())?,
                &mut zip,
            )
            .map_err(|e| e.to_string())?;
        }
        zip.finish().map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = writing {
        let _ = fs::remove_file(path);
        return Err(error);
    }
    result.path = path.to_string_lossy().into_owned();
    Ok(result)
}

fn quick_mod_name(path: &str) -> Result<String, String> {
    if !safe_relative(path) {
        return Err("El pack contiene una ruta de mod insegura".into());
    }
    let path = path.strip_prefix("overrides/").unwrap_or(path);
    let Some(filename) = path.strip_prefix("mods/") else {
        return Err("El ZIP rápido solo admite archivos dentro de mods/; usa Studio para importar otras configuraciones".into());
    };
    if filename.contains('/')
        || !filename.to_ascii_lowercase().ends_with(".jar")
        || filename.len() > 240
    {
        return Err("El pack declara un nombre de mod no válido".into());
    }
    Ok(filename.into())
}


pub(crate) async fn import_quick_archive(
    app: tauri::AppHandle,
    dir: PathBuf,
    bytes: Vec<u8>,
    game: String,
    loader: String,
) -> Result<usize, String> {
    let _guard = crate::studio_progress::INSTALL_LOCK.lock().await;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("ZIP de pack no válido: {e}"))?;
    if archive.len() > MAX_EXPORT_FILES {
        return Err("El pack supera los 20.000 archivos permitidos".into());
    }
    let manifest:Value=read_entry(&mut archive,"manifest-tps.json").and_then(|text|serde_json::from_str(&text).ok()).ok_or("El ZIP rápido necesita manifest-tps.json de TP LAUNCHER; importa otros formatos desde Studio")?;
    if manifest["format"].as_str() != Some("minecrafttps-modpack")
        || manifest["formatVersion"].as_u64() != Some(1)
    {
        return Err("Formato de pack TP LAUNCHER no compatible".into());
    }
    if manifest["minecraftVersion"].as_str() != Some(game.as_str())
        || manifest["loader"].as_str() != Some(loader.as_str())
    {
        return Err(format!(
            "El pack requiere Minecraft {} con {}; la instalación seleccionada usa Minecraft {game} con {loader}",
            manifest["minecraftVersion"]
                .as_str()
                .unwrap_or("desconocido"),
            manifest["loader"].as_str().unwrap_or("desconocido")
        ));
    }
    let index = match archive.by_name("modrinth.index.json") {
        Ok(mut file) => {
            if file.size() > 8 * 1024 * 1024 {
                return Err("El índice de descargas supera 8 MB".into());
            }
            let mut text = String::new();
            file.by_ref()
                .take(8 * 1024 * 1024 + 1)
                .read_to_string(&mut text)
                .map_err(|e| e.to_string())?;
            Some(
                serde_json::from_str::<Value>(&text)
                    .map_err(|e| format!("Índice Modrinth inválido: {e}"))?,
            )
        }
        Err(zip::result::ZipError::FileNotFound) => None,
        Err(error) => return Err(error.to_string()),
    };
    if index.as_ref().is_some_and(|v| {
        v["formatVersion"].as_u64() != Some(1)
            || v["game"].as_str() != Some("minecraft")
            || v["dependencies"]["minecraft"].as_str() != Some(game.as_str())
    }) {
        return Err("El índice de descargas no corresponde a la versión del pack".into());
    }
    let references = index
        .as_ref()
        .and_then(|v| v["files"].as_array())
        .cloned()
        .unwrap_or_default();
    if references.len() > 10_000 {
        return Err("El pack declara demasiadas descargas".into());
    }
    let mods_dir = dir.join("mods");
    fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    let staging = mods_dir.join(format!(".tps-import-{}", uuid::Uuid::new_v4().simple()));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let result=async {
        let mut names=std::collections::HashSet::new();
        let mut expanded=0u64;
        for i in 0..archive.len() {
            let mut file=archive.by_index(i).map_err(|e|e.to_string())?;
            if file.is_dir() || !file.name().to_ascii_lowercase().ends_with(".jar") { continue; }
            if file.unix_mode().is_some_and(|mode|mode & 0o170000==0o120000) { return Err("El ZIP no puede incluir enlaces simbólicos".into()); }
            let name=quick_mod_name(file.name())?;
            if !names.insert(name.to_ascii_lowercase()) { return Err("El pack contiene nombres de mods repetidos".into()); }
            expanded=expanded.saturating_add(file.size());
            if file.size()>MAX_ARCHIVE_BYTES || expanded>MAX_EXPORT_BYTES { return Err("Los archivos del pack superan 256 MB por mod o 1 GB en total".into()); }
            let mut target=fs::OpenOptions::new().write(true).create_new(true).open(staging.join(&name)).map_err(|e|e.to_string())?;
            let expected=file.size();
            let copied=std::io::copy(&mut file.by_ref().take(MAX_ARCHIVE_BYTES+1),&mut target).map_err(|e|e.to_string())?;
            if copied!=expected { return Err(format!("El archivo {name} no tiene el tamaño declarado")); }
        }
        drop(archive);
        let client=reqwest::Client::builder().user_agent("TP LAUNCHER/0.1.0").timeout(std::time::Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::custom(|attempt|if valid_modrinth_download(attempt.url().as_str()) && attempt.previous().len()<5 { attempt.follow() } else { attempt.error("La descarga intentó salir del CDN oficial de Modrinth") }))
            .build().map_err(|e|e.to_string())?;
        let mut added=Vec::new();
        for (i,reference) in references.iter().enumerate() {
            let path=reference["path"].as_str().ok_or("El índice contiene una descarga sin ruta")?;
            let name=quick_mod_name(path)?;
            if !names.insert(name.to_ascii_lowercase()) { return Err(format!("El índice repite {name} o lo duplica con un archivo incluido")); }
            let expected=reference["fileSize"].as_u64().filter(|size|*size>0 && *size<=MAX_ARCHIVE_BYTES).ok_or("El índice declara un tamaño de descarga inválido")?;
            expanded=expanded.saturating_add(expected);
            if expanded>MAX_EXPORT_BYTES { return Err("Las descargas del pack superan el límite de 1 GB".into()); }
            let sha1=reference["hashes"]["sha1"].as_str().filter(|s|s.len()==40 && s.bytes().all(|b|b.is_ascii_hexdigit())).ok_or("Falta un hash SHA-1 válido en la descarga")?;
            let sha512=reference["hashes"]["sha512"].as_str().filter(|s|s.len()==128 && s.bytes().all(|b|b.is_ascii_hexdigit())).ok_or("Falta un hash SHA-512 válido en la descarga")?;
            let url=reference["downloads"].as_array().and_then(|urls|urls.iter().filter_map(Value::as_str).find(|url|valid_modrinth_download(url))).ok_or("La descarga no procede del CDN oficial de Modrinth")?;
            crate::emit_progress(&app,"Importando pack",((i*95)/references.len().max(1)) as u8,&format!("Descargando {name}"));
            let mut response=client.get(url).send().await.map_err(|e|format!("No se pudo descargar {name}: {e}"))?.error_for_status().map_err(|e|format!("La descarga de {name} falló: {e}"))?;
            if response.content_length().is_some_and(|size|size>expected) { return Err(format!("La descarga de {name} supera su tamaño declarado")); }
            let mut bytes=Vec::with_capacity(expected.min(8*1024*1024) as usize);
            while let Some(chunk)=response.chunk().await.map_err(|e|e.to_string())? {
                if bytes.len() as u64+chunk.len() as u64>expected { return Err(format!("La descarga de {name} supera su tamaño declarado")); }
                bytes.extend_from_slice(&chunk);
            }
            if bytes.len() as u64!=expected || !format!("{:x}",Sha1::digest(&bytes)).eq_ignore_ascii_case(sha1) || !format!("{:x}",Sha512::digest(&bytes)).eq_ignore_ascii_case(sha512) {
                return Err(format!("La descarga de {name} no coincide con sus hashes o tamaño; no se instaló"));
            }
            zip::ZipArchive::new(Cursor::new(&bytes)).map_err(|_|format!("{name} no es un JAR válido"))?;
            fs::write(staging.join(&name),&bytes).map_err(|e|e.to_string())?;
            let parsed=url::Url::parse(url).map_err(|e|e.to_string())?;
            let parts=parsed.path_segments().map(|parts|parts.collect::<Vec<_>>()).unwrap_or_default();
            if parts.len()>=5 && parts[0]=="data" && parts[2]=="versions" {
                added.push(DownloadRecord { provider:"modrinth".into(),project_id:parts[1].into(),version_id:parts[3].into(),filename:name,sha1:sha1.into(),url:url.into(),size:expected });
            }
        }
        let staged=fs::read_dir(&staging).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
        if staged.is_empty() { return Err("El pack no incluye mods redistribuibles ni referencias verificadas; consulta export-report.json para ver los omitidos".into()); }
        if let Some(entry)=staged.iter().find(|entry|mods_dir.join(entry.file_name()).exists()) {
            return Err(format!("Ya existe {} en la instancia. Desactívalo o quítalo antes de importar",entry.file_name().to_string_lossy()));
        }
        let mut created=Vec::new();
        for entry in &staged {
            let target=mods_dir.join(entry.file_name());
            let copying=(||->Result<(),String>{
                let mut dest=fs::OpenOptions::new().write(true).create_new(true).open(&target).map_err(|e|e.to_string())?;
                created.push(target.clone());
                std::io::copy(&mut fs::File::open(entry.path()).map_err(|e|e.to_string())?,&mut dest).map_err(|e|e.to_string())?;
                Ok(())
            })();
            if let Err(error)=copying { for target in created { let _=fs::remove_file(target); } return Err(error); }
        }
        if !added.is_empty() {
            let mut records=load_records(&dir);
            records.extend(added);
            let metadata=serde_json::to_vec_pretty(&records).map_err(|e|e.to_string())?;

            let temporary=dir.join(format!(".tps-content-{}.json",uuid::Uuid::new_v4().simple()));
            fs::write(&temporary,metadata).map_err(|e|e.to_string())?;
            if let Err(error)=fs::rename(&temporary,dir.join("tps-content.json")) { let _=fs::remove_file(temporary); return Err(error.to_string()); }
        }
        crate::emit_progress(&app,"Importando pack",100,"Mods verificados e importados");
        Ok(staged.len())
    }.await;

    let canonical_staging = staging.canonicalize().ok();
    let canonical_mods = mods_dir.canonicalize().ok();
    if canonical_staging
        .as_ref()
        .zip(canonical_mods.as_ref())
        .is_some_and(|(target, parent)| {
            target.is_absolute()
                && parent.is_absolute()
                && target.parent() == Some(parent.as_path())
        })
        && staging
            .file_name()
            .is_some_and(|s| s.to_string_lossy().starts_with(".tps-import-"))
    {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn jar(root: &Path, filename: &str, entries: &[(&str, &str)]) -> PathBuf {
        let path = root.join(filename);
        let mut writer = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        for (name, contents) in entries {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(contents.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
        path
    }
    #[test]
    fn zip_and_mrpack_never_include_optifine_clients_or_unknown_binary_licenses() {
        let root = std::env::temp_dir().join(format!("tp-export-policy-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let of = jar(
            &root,
            "renamed.jar",
            &[
                ("optifine/OptiFineTweaker.class", "binary"),
                (
                    "fabric.mod.json",
                    r#"{"name":"OptiFine bridge","version":"HD U M5","license":"MIT"}"#,
                ),
            ],
        );
        let client = jar(
            &root,
            "client.jar",
            &[("net/minecraft/client/main/Main.class", "binary")],
        );
        let unknown = jar(
            &root,
            "private.jar",
            &[(
                "fabric.mod.json",
                r#"{"name":"Private Mod","version":"2.3.7","license":"All Rights Reserved"}"#,
            )],
        );
        for extension in ["zip", "mrpack"] {
            let plan = plan(
                vec![
                    (of.clone(), "mods/renamed.jar".into()),
                    (client.clone(), "mods/client.jar".into()),
                    (unknown.clone(), "mods/private.jar".into()),
                ],
                &[],
            );
            let output = root.join(format!("pack.{extension}"));
            let report = write_archive(
                &output,
                extension,
                plan,
                BTreeMap::new(),
                json!({"minecraft":"1.21.1"}),
                "Test",
            )
            .unwrap();
            assert_eq!(report.omitted.len(), 3);
            assert!(
                report
                    .omitted
                    .iter()
                    .any(|f| f.name == "Private Mod" && f.version == "2.3.7")
            );
            let mut zip = zip::ZipArchive::new(fs::File::open(output).unwrap()).unwrap();
            assert!(!zip.file_names().any(|n| n.ends_with(".jar")));
            let report: Value =
                serde_json::from_reader(zip.by_name("export-report.json").unwrap()).unwrap();
            assert_eq!(report["omitted"].as_array().unwrap().len(), 3);
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn tracked_modrinth_download_is_referenced_by_exact_hash_but_tampering_is_omitted() {
        let root = std::env::temp_dir().join(format!("tp-export-policy-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let file = jar(
            &root,
            "example.jar",
            &[("fabric.mod.json", r#"{"name":"Example","version":"1.4.9"}"#)],
        );
        let bytes = fs::read(&file).unwrap();
        let mut record = DownloadRecord {
            provider: "modrinth".into(),
            project_id: "example".into(),
            version_id: "release".into(),
            filename: "example.jar".into(),
            sha1: format!("{:x}", Sha1::digest(&bytes)),
            url: "https://cdn.modrinth.com/data/example/versions/release/example.jar".into(),
            size: bytes.len() as u64,
        };
        let good = plan(
            vec![(file.clone(), "mods/example.jar".into())],
            &[record.clone()],
        );
        assert!(good.files.is_empty());
        assert!(good.omitted.is_empty());
        assert_eq!(good.references.len(), 1);
        assert_eq!(good.references[0]["hashes"]["sha1"], record.sha1);
        record.sha1 = "0".repeat(40);
        let bad = plan(vec![(file, "mods/example.jar".into())], &[record]);
        assert!(bad.references.is_empty());
        assert_eq!(bad.omitted[0].version, "1.4.9");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn ambiguous_license_is_not_a_grant_and_permissive_license_requires_bundled_notice() {
        let mut identity = JarIdentity {
            license: "MIT".into(),
            ..Default::default()
        };
        assert!(!explicit_redistribution(&identity));
        identity.embedded_license="Permission is hereby granted, free of charge. The above copyright notice shall be included.".into();
        assert!(explicit_redistribution(&identity));
        identity.license = "MIT OR Custom-License".into();
        assert!(!explicit_redistribution(&identity));
        assert!(!valid_modrinth_download(
            "https://cdn.modrinth.com.attacker.invalid/data/a.jar"
        ));
        assert!(!valid_modrinth_download(
            "https://user@cdn.modrinth.com/data/a.jar"
        ));
    }
}
