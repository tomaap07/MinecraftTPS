
use crate::studio_progress::{INSTALL_LOCK, Operation};
use base64::Engine;
use futures_util::{StreamExt, stream};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    io::{Cursor, Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

const MR: &str = "https://api.modrinth.com/v2";
const CF: &str = "https://api.curseforge.com/v1";
const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;
const MAX_PACK_EXPANDED: u64 = 16 * 1024 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
pub struct MissingPackFile {
    pub filename: String,
    pub project_id: String,
    pub file_id: String,
    pub sha1: String,
    pub size: u64,
    pub website_url: String,
}
fn ready_state() -> String {
    "ready".into()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StudioPack {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub loader_version: String,
    pub version_id: String,
    pub icon_data: String,
    #[serde(default)]
    pub icon_url: String,
    #[serde(default)]
    pub source_provider: String,
    #[serde(default)]
    pub source_project_id: String,
    pub created_at: u64,
    #[serde(default = "ready_state")]
    pub install_state: String,
    #[serde(default)]
    pub install_error: String,
    #[serde(default)]
    pub archive_sha1: String,
    #[serde(default)]
    pub missing_files: Vec<MissingPackFile>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct StudioProject {
    provider: String,
    id: String,
    slug: String,
    title: String,
    description: String,
    icon_url: String,
    #[serde(default)]
    preview_url: String,
    author: String,
    downloads: u64,
    categories: Vec<String>,
    loaders: Vec<String>,
    url: String,
    project_type: String,
    updated: String,
}
#[derive(Serialize)]
pub struct StudioSearch {
    projects: Vec<StudioProject>,
    total_hits: u64,
    page: u32,
    total_pages: u64,
    warnings: Vec<String>,
}
#[derive(Clone, Serialize)]
pub struct StudioCategory {
    pub id: String,
    pub name: String,
    pub provider: String,
}
#[tauri::command]
pub async fn studio_categories(
    provider: String,
    project_type: String,
) -> Result<Vec<StudioCategory>, String> {
    if !["modrinth", "curseforge", "both"].contains(&provider.as_str())
        || !["mod", "modpack", "resourcepack", "shader"].contains(&project_type.as_str())
    {
        return Err("Catálogo no válido".into());
    }
    let client = client()?;
    let class = match project_type.as_str() {
        "modpack" => "4471",
        "resourcepack" => "12",
        "shader" => "6552",
        _ => "6",
    };
    let (mr, cf) = tokio::join!(
        async {
            if provider == "curseforge" {
                None
            } else {
                Some(get_json(&client, &format!("{MR}/tag/category"), &[], false).await)
            }
        },
        async {
            if provider == "modrinth" {
                None
            } else {
                Some(
                    get_json(
                        &client,
                        &format!("{CF}/categories"),
                        &[
                            ("gameId".into(), "432".into()),
                            ("classId".into(), class.into()),
                        ],
                        true,
                    )
                    .await,
                )
            }
        }
    );
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for (source, result) in [("modrinth", mr), ("curseforge", cf)] {
        if let Some(result) = result {
            match result {
                Ok(value) => {
                    let values = if source == "modrinth" {
                        value.as_array()
                    } else {
                        value["data"].as_array()
                    };
                    for item in values.into_iter().flatten() {
                        if source == "modrinth"
                            && item["project_type"].as_str() != Some(&project_type)
                        {
                            continue;
                        }
                        let id = if source == "modrinth" {
                            text(item, "name")
                        } else {
                            identifier(item, "id")
                        };
                        if !id.is_empty() {
                            rows.push(StudioCategory {
                                id: format!("{source}:{id}"),
                                name: text(item, "name"),
                                provider: source.into(),
                            });
                        }
                    }
                }
                Err(e) => errors.push(e),
            }
        }
    }
    rows.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.name.cmp(&b.name))
    });
    if rows.is_empty() && !errors.is_empty() {
        return Err(errors.join(" · "));
    }
    Ok(rows)
}

#[tauri::command]
pub async fn hud_install_content(
    app: AppHandle,
    pid: u32,
    provider: String,
    project_id: String,
    file_id: String,
    project_type: String,
) -> Result<String, String> {
    if !["resourcepack", "shader"].contains(&project_type.as_str()) {
        return Err("F8 solo instala recursos y shaders durante la partida".into());
    }
    let dir = crate::game_session::directory(pid).ok_or("La partida ya no está abierta")?;
    let (username, version) = crate::game_session::identity(pid).ok_or("Partida no disponible")?;
    let (game, loader) = crate::resolve_mod_game_version(&crate::root_dir()?, &version)?;
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Ya hay una instalación en curso")?;
    let operation = Operation::new(Some(app), "Instalar contenido desde Minecraft");
    let result = install_version_inner(
        &operation,
        provider,
        project_id,
        file_id,
        project_type,
        None,
        username,
        version,
        Some((dir, game, loader)),
    )
    .await;
    let success = result.as_ref().cloned().unwrap_or_default();
    operation.complete(result, &success)
}
#[derive(Clone, Serialize, Deserialize)]
pub struct StudioDependency {
    pub id: String,
    pub version_id: String,
    pub kind: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct StudioVersion {
    pub id: String,
    pub name: String,
    pub number: String,
    pub release_type: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub size: u64,
    pub published: String,
    pub filename: String,
    pub dependencies: Vec<StudioDependency>,
}
#[derive(Serialize)]
pub struct StudioDetail {
    project: StudioProject,
    body: String,
    gallery: Vec<String>,
    versions: Vec<StudioVersion>,
}
#[derive(Serialize)]
pub struct LoaderVersion {
    version: String,
    stable: bool,
}
#[derive(Serialize)]
pub struct PackMod {
    filename: String,
    enabled: bool,
    size: u64,
    provider: String,
    project_id: String,
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
    icon_data: String,
    loader: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct JarMetadata {
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
    icon_data: String,
    loader: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct ContentFile {
    provider: String,
    project_id: String,
    version_id: String,
    filename: String,
    sha1: String,
    url: String,
    size: u64,
}

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent("MinecraftTPS/0.1.0 (https://github.com/tomaap07/MinecraftTPS)")
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| e.to_string())
}
fn curseforge_client() -> Result<Client, String> {
    static CLIENT: std::sync::OnceLock<Result<Client, String>> = std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            Client::builder()
                .timeout(Duration::from_secs(90))
                .redirect(reqwest::redirect::Policy::custom(|attempt| {
                    if attempt.url().scheme() != "https"
                        || attempt.url().host_str() != Some("api.curseforge.com")
                    {
                        attempt.error(
                            "CurseForge intentó redirigir la solicitud fuera de su API oficial",
                        )
                    } else if attempt.previous().len() >= 10 {
                        attempt.error("CurseForge superó el límite de redirecciones")
                    } else {
                        attempt.follow()
                    }
                }))
                .build()
                .map_err(|e| e.to_string())
        })
        .clone()
}
fn safe_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 180
        || id == "."
        || id == ".."
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'))
    {
        return Err("Identificador no válido".into());
    }
    Ok(())
}
fn safe_file(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 240
        || name.contains(['/', '\\', ':'])
        || name.starts_with('.')
    {
        return Err("Nombre de archivo no válido".into());
    }
    Ok(())
}
fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn text(value: &Value, field: &str) -> String {
    value[field].as_str().unwrap_or_default().to_owned()
}
fn identifier(value: &Value, field: &str) -> String {
    value[field]
        .as_str()
        .map(str::to_owned)
        .or_else(|| value[field].as_u64().map(|n| n.to_string()))
        .unwrap_or_default()
}
fn loaders(value: &Value) -> Vec<String> {
    strings(value)
        .into_iter()
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| ["forge", "fabric", "neoforge", "quilt", "vanilla", "iris", "optifine", "oculus", "canvas"].contains(&s.as_str()))
        .collect()
}
fn cf_loader(loader: &str) -> Option<u32> {
    match loader {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}
async fn get_json(
    client: &Client,
    url: &str,
    params: &[(String, String)],
    cf: bool,
) -> Result<Value, String> {
    let mut request = if cf {
        let parsed = url::Url::parse(url).map_err(|_| "URL de CurseForge inválida")?;
        if parsed.scheme() != "https" || parsed.host_str() != Some("api.curseforge.com") {
            return Err("La solicitud de CurseForge debe usar su API HTTPS oficial".into());
        }
        curseforge_client()?.get(parsed)
    } else {
        client.get(url)
    }
    .query(params)
    .header("Accept", "application/json");
    if cf {
        request = request
            .header(
                "x-api-key",
                crate::private_integrations::curseforge_api_key()?,
            )
            .header(
                "User-Agent",
                "TP-LAUNCHER/0.1.0 (https://github.com/tomaap07/MinecraftTPS)",
            );
    }
    request_json(request, cf).await
}
async fn request_json(request: reqwest::RequestBuilder, cf: bool) -> Result<Value, String> {
    let mut retries = 0u32;
    loop {
        let response = request
            .try_clone()
            .ok_or("No se pudo preparar la solicitud al proveedor")?
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(_) if retries < 2 => {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(600 * retries as u64)).await;
                continue;
            }
            Err(error) => return Err(format!("No se pudo conectar con el proveedor: {error}")),
        };
        let status = response.status();
        if status.is_success() {
            return response
                .json()
                .await
                .map_err(|e| format!("El proveedor envió una respuesta inválida: {e}"));
        }
        if (status.as_u16() == 429 || status.is_server_error() || (cf && status.as_u16() == 403))
            && retries < 2
        {
            retries += 1;
            let delay = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(retries as u64)
                .min(10);
            tokio::time::sleep(Duration::from_secs(delay)).await;
            continue;
        }
        let detail = if cf && status.as_u16() == 403 {


            let mut stream = response.bytes_stream();
            let mut prefix = Vec::new();
            while prefix.len() < 4096 {
                let Some(Ok(chunk)) = stream.next().await else {
                    break;
                };
                let remaining = 4096 - prefix.len();
                prefix.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
            }
            let message = String::from_utf8_lossy(&prefix).to_ascii_lowercase();
            if message.contains("api key")
                && (message.contains("missing") || message.contains("invalid"))
            {
                ": el servicio no acepta la clave API integrada (clave ausente o inválida)"
            } else {
                ": el servicio rechazó la solicitud; comprueba los permisos de la aplicación en CurseForge"
            }
        } else {
            ""
        };
        return Err(format!(
            "{} respondió HTTP {}{detail}",
            if cf { "CurseForge" } else { "Modrinth" },
            status.as_u16(),
        ));
    }
}

fn mr_project(v: &Value) -> StudioProject {
    let id = identifier(v, "project_id");
    let id = if id.is_empty() {
        identifier(v, "id")
    } else {
        id
    };
    let slug = text(v, "slug");
    let mut engines = loaders(&v["loaders"]);
    if engines.is_empty() {
        engines = loaders(&v["categories"]);
    }
    StudioProject {
        provider: "modrinth".into(),
        id,
        slug: slug.clone(),
        title: text(v, "title"),
        description: text(v, "description"),
        preview_url: v["featured_gallery"]
            .as_str()
            .or_else(|| {
                v["gallery"]
                    .as_array()
                    .and_then(|g| g.first())
                    .and_then(Value::as_str)
            })
            .or_else(|| {
                v["gallery"]
                    .as_array()
                    .and_then(|g| g.first())
                    .and_then(|g| g["url"].as_str())
            })
            .unwrap_or_default()
            .into(),
        icon_url: text(v, "icon_url"),
        author: text(v, "author"),
        downloads: v["downloads"].as_u64().unwrap_or(0),
        categories: strings(&v["categories"]),
        loaders: engines,
        url: format!(
            "https://modrinth.com/{}/{}",
            v["project_type"].as_str().unwrap_or("mod"),
            slug
        ),
        project_type: text(v, "project_type"),
        updated: text(v, "date_modified"),
    }
}
fn cf_logo(v: &Value) -> Option<&str> {
    ["/logo/thumbnailUrl", "/logo/url"].iter().find_map(|key| {
        v.pointer(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
    })
}
fn cf_project(v: &Value, kind: &str) -> StudioProject {
    StudioProject {
        provider: "curseforge".into(),
        id: identifier(v, "id"),
        slug: text(v, "slug"),
        title: text(v, "name"),
        description: text(v, "summary"),
        icon_url: cf_logo(v).unwrap_or_default().into(),
        preview_url: v["screenshots"]
            .as_array()
            .and_then(|g| g.first())
            .and_then(|g| g["url"].as_str())
            .unwrap_or_default()
            .into(),
        author: v["authors"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|a| a["name"].as_str())
            .collect::<Vec<_>>()
            .join(", "),
        downloads: v["downloadCount"].as_f64().unwrap_or(0.0) as u64,
        categories: v["categories"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|a| a["name"].as_str())
            .map(str::to_owned)
            .collect(),
        loaders: Vec::new(),
        url: v
            .pointer("/links/websiteUrl")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .into(),
        project_type: match v["classId"].as_u64() {
            Some(4471) => "modpack",
            Some(12) => "resourcepack",
            Some(6552) => "shader",
            _ => kind,
        }
        .into(),
        updated: text(v, "dateModified"),
    }
}
async fn search_mr(
    client: &Client,
    query: &str,
    kind: &str,
    game: &str,
    loader: &str,
    sort: &str,
    category: &str,
    offset: u64,
    limit: u64,
) -> Result<(Vec<StudioProject>, u64), String> {
    let mut facets = vec![vec![format!("project_type:{kind}")]];
    if game != "all" && !game.is_empty() {
        facets.push(vec![format!("versions:{game}")]);
    }
    if loader != "all" && !loader.is_empty() && matches!(kind, "mod" | "modpack") {
        facets.push(vec![format!("categories:{loader}")]);
    }
    if !category.is_empty() {
        facets.push(vec![format!("categories:{category}")]);
    }
    let value = get_json(
        client,
        &format!("{MR}/search"),
        &[
            ("query".into(), query.into()),
            ("facets".into(), json!(facets).to_string()),
            (
                "index".into(),
                match sort {
                    "downloads" => "downloads",
                    "recent" => "updated",
                    _ => "relevance",
                }
                .into(),
            ),
            ("limit".into(), limit.to_string()),
            ("offset".into(), offset.to_string()),
        ],
        false,
    )
    .await?;
    Ok((
        value["hits"]
            .as_array()
            .into_iter()
            .flatten()
            .map(mr_project)
            .collect(),
        value["total_hits"].as_u64().unwrap_or(0),
    ))
}
async fn search_cf(
    client: &Client,
    query: &str,
    kind: &str,
    game: &str,
    loader: &str,
    sort: &str,
    category: &str,
    offset: u64,
    limit: u64,
) -> Result<(Vec<StudioProject>, u64), String> {
    if offset >= 10_000 {
        return Err("CurseForge permite recorrer hasta 10.000 resultados por consulta. Usa los filtros para acceder a otros proyectos.".into());
    }
    let limit = limit.min(10_000 - offset);
    let mut params = vec![
        ("gameId".into(), "432".into()),
        (
            "classId".into(),
            match kind {
                "modpack" => "4471",
                "resourcepack" => "12",
                "shader" => "6552",
                _ => "6",
            }
            .into(),
        ),
        ("searchFilter".into(), query.into()),
        (
            "sortField".into(),
            match sort {
                "downloads" => "6",
                "recent" => "3",
                _ => "2",
            }
            .into(),
        ),
        ("sortOrder".into(), "desc".into()),
        ("index".into(), offset.to_string()),
        ("pageSize".into(), limit.to_string()),
    ];
    if game != "all" && !game.is_empty() {
        params.push(("gameVersion".into(), game.into()));
    }
    if let Some(engine) = cf_loader(loader).filter(|_| matches!(kind, "mod" | "modpack")) {
        params.push(("modLoaderType".into(), engine.to_string()));
    }
    if !category.is_empty() {
        params.push(("categoryId".into(), category.into()));
    }
    let value = get_json(client, &format!("{CF}/mods/search"), &params, true).await?;
    Ok((
        value["data"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|v| cf_project(v, kind))
            .collect(),
        value
            .pointer("/pagination/totalCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    ))
}
#[tauri::command]
pub async fn studio_search(
    provider: String,
    query: String,
    project_type: String,
    game_version: String,
    loader: String,
    sort: String,
    page: u32,
    category: Option<String>,
) -> Result<StudioSearch, String> {
    if !["modrinth", "curseforge", "both"].contains(&provider.as_str())
        || !["mod", "modpack", "resourcepack", "shader"].contains(&project_type.as_str())
    {
        return Err("Proveedor o tipo de contenido no válido".into());
    }
    let category = category.unwrap_or_default();
    let (category_provider, category_value) = category.split_once(':').unwrap_or(("", ""));
    if !category.is_empty()
        && (!["modrinth", "curseforge"].contains(&category_provider)
            || category_value.is_empty()
            || category_value.len() > 80
            || !category_value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || (provider != "both" && provider != category_provider))
    {
        return Err("Categoría no válida para este proveedor".into());
    }
    let page = page.max(1);
    let size = if provider == "both" { 12 } else { 24 };
    let offset = (page as u64 - 1) * size;
    let client = client()?;
    let mut warnings = Vec::new();
    let mut all = Vec::new();
    let mut total = 0;
    let mut pages = 0;
    let (mr, cf) = tokio::join!(
        async {
            if provider != "curseforge" && category_provider != "curseforge" {
                Some(
                    search_mr(
                        &client,
                        &query,
                        &project_type,
                        &game_version,
                        &loader,
                        &sort,
                        category_value,
                        offset,
                        size,
                    )
                    .await,
                )
            } else {
                None
            }
        },
        async {
            if provider != "modrinth" && category_provider != "modrinth" {
                Some(
                    search_cf(
                        &client,
                        &query,
                        &project_type,
                        &game_version,
                        &loader,
                        &sort,
                        category_value,
                        offset,
                        size,
                    )
                    .await,
                )
            } else {
                None
            }
        }
    );
    for result in [mr, cf].into_iter().flatten() {
        match result {
            Ok((projects, count)) => {
                let count = if projects.first().is_some_and(|p| p.provider == "curseforge")
                    && count > 10_000
                {
                    warnings.push("CurseForge permite recorrer los primeros 10.000 resultados. Afina los filtros para consultar otros proyectos.".into());
                    10_000
                } else {
                    count
                };
                total += count;
                pages = pages.max(count.div_ceil(size));
                all.push(projects)
            }
            Err(e) => warnings.push(e),
        }
    }
    if all.is_empty() {
        return Err(warnings.join(" · "));
    }
    let mut projects = Vec::new();
    let max = all.iter().map(Vec::len).max().unwrap_or(0);
    for i in 0..max {
        for group in &all {
            if let Some(p) = group.get(i) {
                projects.push(p.clone());
            }
        }
    }
    if sort == "downloads" {
        projects.sort_by_key(|p| std::cmp::Reverse(p.downloads));
    } else if sort == "recent" {
        projects.sort_by(|a, b| b.updated.cmp(&a.updated));
    }
    Ok(StudioSearch {
        projects,
        total_hits: total,
        page,
        total_pages: pages.max(1),
        warnings,
    })
}
fn mr_version(v: &Value) -> StudioVersion {
    let file = v["files"]
        .as_array()
        .and_then(|files| {
            files
                .iter()
                .find(|f| f["primary"].as_bool() == Some(true))
                .or_else(|| files.first())
        })
        .cloned()
        .unwrap_or(Value::Null);
    StudioVersion {
        id: identifier(v, "id"),
        name: text(v, "name"),
        number: text(v, "version_number"),
        release_type: text(v, "version_type"),
        game_versions: strings(&v["game_versions"]),
        loaders: strings(&v["loaders"]),
        size: file["size"].as_u64().unwrap_or(0),
        published: text(v, "date_published"),
        filename: text(&file, "filename"),
        dependencies: v["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|d| StudioDependency {
                id: identifier(d, "project_id"),
                version_id: identifier(d, "version_id"),
                kind: text(d, "dependency_type"),
            })
            .collect(),
    }
}
fn cf_version(v: &Value) -> StudioVersion {
    StudioVersion {
        id: identifier(v, "id"),
        name: text(v, "displayName"),
        number: identifier(v, "id"),
        release_type: match v["releaseType"].as_u64() {
            Some(2) => "beta",
            Some(3) => "alpha",
            _ => "release",
        }
        .into(),
        game_versions: strings(&v["gameVersions"])
            .into_iter()
            .filter(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
            .collect(),
        loaders: loaders(&v["gameVersions"]),
        size: v["fileLength"].as_u64().unwrap_or(0),
        published: text(v, "fileDate"),
        filename: text(v, "fileName"),
        dependencies: v["dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|d| StudioDependency {
                id: identifier(d, "modId"),
                version_id: String::new(),
                kind: match d["relationType"].as_u64() {
                    Some(3) => "required",
                    Some(2) => "optional",
                    Some(5) => "incompatible",
                    _ => "embedded",
                }
                .into(),
            })
            .collect(),
    }
}
async fn cf_files(
    client: &Client,
    project: &str,
    game: &str,
    loader: &str,
) -> Result<Vec<Value>, String> {
    safe_id(project)?;
    let mut result = Vec::new();
    let mut index = 0u64;
    loop {
        let mut params = vec![
            ("index".into(), index.to_string()),
            ("pageSize".into(), "50".into()),
        ];
        if !game.is_empty() && game != "all" {
            params.push(("gameVersion".into(), game.into()));
        }
        if let Some(l) = cf_loader(loader) {
            params.push(("modLoaderType".into(), l.to_string()));
        }
        let response =
            get_json(client, &format!("{CF}/mods/{project}/files"), &params, true).await?;
        let chunk = response["data"].as_array().cloned().unwrap_or_default();
        let n = chunk.len();
        result.extend(chunk);
        index += n as u64;
        let total = response
            .pointer("/pagination/totalCount")
            .and_then(Value::as_u64)
            .unwrap_or(index);
        if total > 10_000 {
            return Err("El proyecto supera las 10.000 versiones que CurseForge permite consultar; usa su web oficial para archivos adicionales.".into());
        }
        if n == 0 || index >= total {
            break;
        }
    }
    Ok(result)
}
#[tauri::command]
pub async fn studio_dependency_projects(
    provider: String,
    project_ids: Vec<String>,
) -> Result<Vec<StudioProject>, String> {
    if !["modrinth", "curseforge"].contains(&provider.as_str()) {
        return Err("Proveedor no válido".into());
    }
    if project_ids.len() > 50 {
        return Err("Consulta hasta 50 dependencias a la vez".into());
    }
    let mut unique = HashSet::new();
    let ids = project_ids
        .into_iter()
        .filter(|id| unique.insert(id.clone()))
        .collect::<Vec<_>>();
    for id in &ids {
        safe_id(id)?;
    }
    let client = client()?;
    let is_cf = provider == "curseforge";
    let results = stream::iter(ids.into_iter().map(|id| {
        let client = client.clone();
        async move {
            let url = if is_cf {
                format!("{CF}/mods/{id}")
            } else {
                format!("{MR}/project/{id}")
            };
            let value = get_json(&client, &url, &[], is_cf).await?;
            Ok::<_, String>(if is_cf {
                cf_project(&value["data"], "mod")
            } else {
                mr_project(&value)
            })
        }
    }))
    .buffer_unordered(5)
    .collect::<Vec<_>>()
    .await;
    let mut projects = Vec::new();
    let mut errors = Vec::new();
    for result in results {
        match result {
            Ok(project) => projects.push(project),
            Err(error) => errors.push(error),
        }
    }
    if projects.is_empty() && !errors.is_empty() {
        return Err(errors.remove(0));
    }
    Ok(projects)
}
#[tauri::command]
pub async fn studio_project(provider: String, project_id: String) -> Result<StudioDetail, String> {
    safe_id(&project_id)?;
    let client = client()?;
    if provider == "modrinth" {
        let detail_url = format!("{MR}/project/{project_id}");
        let versions_url = format!("{MR}/project/{project_id}/version");
        let (detail, versions) = tokio::join!(
            get_json(&client, &detail_url, &[], false),
            get_json(&client, &versions_url, &[], false)
        );
        let value = detail?;
        let mut project = mr_project(&value);
        if let Some(team) = value["team"].as_str() {
            if let Ok(members) =
                get_json(&client, &format!("{MR}/team/{team}/members"), &[], false).await
            {
                project.author = members
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|m| m.pointer("/user/username").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join(", ");
            }
        }
        let versions = versions?
            .as_array()
            .into_iter()
            .flatten()
            .map(mr_version)
            .collect();
        Ok(StudioDetail {
            project,
            body: text(&value, "body"),
            gallery: value["gallery"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|g| g["url"].as_str())
                .map(str::to_owned)
                .collect(),
            versions,
        })
    } else if provider == "curseforge" {
        let value = get_json(&client, &format!("{CF}/mods/{project_id}"), &[], true).await?;
        let data = &value["data"];
        let body = get_json(
            &client,
            &format!("{CF}/mods/{project_id}/description"),
            &[],
            true,
        )
        .await?;
        let versions = cf_files(&client, &project_id, "all", "all")
            .await?
            .iter()
            .filter(|file|file["isAvailable"].as_bool()!=Some(false))
            .map(cf_version)
            .collect::<Vec<_>>();
        let mut project = cf_project(data, "mod");
        project.loaders = versions
            .iter()
            .flat_map(|v| v.loaders.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        Ok(StudioDetail {
            project,

            body: text(&body, "data"),
            gallery: data["screenshots"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|g| g["url"].as_str())
                .map(str::to_owned)
                .collect(),
            versions,
        })
    } else {
        Err("Proveedor no válido".into())
    }
}



pub(crate) fn neoforge_version_matches(game: &str, version: &str) -> bool {
    let (game_release, game_stage) = game
        .split_once('-')
        .map(|(release, stage)| (release, Some(stage)))
        .unwrap_or((game, None));
    if game_release == "1.20.1" {
        return game_stage.is_none() && version.starts_with("1.20.1-");
    }
    let (neo_release, neo_stage) = version
        .split_once('+')
        .map(|(release, stage)| (release, Some(stage)))
        .unwrap_or((version, None));
    if game_stage != neo_stage {
        return false;
    }
    let Some(mc) = game_release
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
    else {
        return false;
    };
    if !(2..=3).contains(&mc.len()) {
        return false;
    }
    let Some(neo) = neo_release
        .split('-')
        .next()
        .unwrap_or_default()
        .split('.')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()
    else {
        return false;
    };
    let patch = mc.get(2).copied().unwrap_or(0);
    if mc[0] == 1 {
        neo.len() == 3 && neo[0] == mc[1] && neo[1] == patch
    } else if mc[0] >= 26 {
        neo.len() == 4 && neo[0] == mc[0] && neo[1] == mc[1] && neo[2] == patch
    } else {
        false
    }
}
#[tauri::command]
pub async fn studio_loader_versions(
    loader: String,
    game_version: String,
) -> Result<Vec<LoaderVersion>, String> {
    safe_id(&game_version)?;
    let client = client()?;
    match loader.as_str() {
        "vanilla" => Ok(vec![LoaderVersion {
            version: game_version,
            stable: true,
        }]),
        "fabric" | "quilt" => {
            let host = if loader == "fabric" {
                "https://meta.fabricmc.net/v2"
            } else {
                "https://meta.quiltmc.org/v3"
            };
            let value = get_json(
                &client,
                &format!("{host}/versions/loader/{game_version}"),
                &[],
                false,
            )
            .await?;
            Ok(value
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| {
                    v.pointer("/loader/version")
                        .and_then(Value::as_str)
                        .map(|version| LoaderVersion {
                            version: version.into(),
                            stable: v
                                .pointer("/loader/stable")
                                .and_then(Value::as_bool)
                                .unwrap_or(true),
                        })
                })
                .collect())
        }
        "optifine" => Ok(crate::fetch_optifine_versions()
            .await?
            .into_iter()
            .filter(|v| v.game_version == game_version)
            .map(|v| LoaderVersion {
                version: v.id,
                stable: !v.preview,
            })
            .collect()),
        "forge" | "neoforge" => {
            let url = if loader == "forge" {
                "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml"
            } else if game_version == "1.20.1" {
                "https://maven.neoforged.net/releases/net/neoforged/forge/maven-metadata.xml"
            } else {
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml"
            };
            let xml = client
                .get(url)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())?;
            let regex = regex::Regex::new(r"<version>([^<]+)</version>").unwrap();
            let mut versions = Vec::new();
            for cap in regex.captures_iter(&xml) {
                let v = &cap[1];
                let matches = if loader == "forge" {
                    v.starts_with(&format!("{game_version}-"))
                } else {
                    neoforge_version_matches(&game_version, v)
                };
                if matches {
                    versions.push(LoaderVersion {
                        version: v.into(),
                        stable: !v.contains("beta") && !v.contains("alpha"),
                    });
                }
            }
            versions.reverse();
            Ok(versions)
        }
        _ => Err("Motor no compatible".into()),
    }
}
fn loader_profile_matches(root: &Path, id: &str, game: &str, loader: &str, version: &str) -> bool {
    let profile: Value = match fs::read(root.join("versions").join(id).join(format!("{id}.json")))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
    {
        Some(value) => value,
        None => return false,
    };
    if profile["inheritsFrom"].as_str() != Some(game) {
        return false;
    }
    let revision = version.strip_prefix(&format!("{game}-")).unwrap_or(version);
    if loader == "neoforge" && game == "1.20.1" {


        return id == format!("{game}-forge-{revision}")
            && profile["libraries"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v["name"].as_str())
                .any(|name| name.starts_with("net.neoforged.fancymodloader:"));
    }
    let exact_name = if loader == "forge" {
        format!("net.minecraftforge:forge:{version}")
    } else {
        format!(
            "net.neoforged:{}:{version}",
            if game == "1.20.1" {
                "forge"
            } else {
                "neoforge"
            }
        )
    };
    profile["libraries"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v["name"].as_str())
        .any(|name| {
            name == exact_name
                || name
                    .strip_prefix(&exact_name)
                    .is_some_and(|suffix| suffix.starts_with(':'))
        })
        || if loader == "forge" {
            id == format!("{game}-forge-{revision}")
                || id == format!("{game}-forge{revision}")
                || id == format!("{game}-Forge{revision}")
        } else {
            id == format!("neoforge-{version}")
        }
}
pub(crate) async fn install_loader(
    operation: &Operation,
    game: &str,
    loader: &str,
    version: &str,
) -> Result<String, String> {
    safe_id(game)?;
    safe_id(version)?;
    let normalized_version;
    let version = if (loader == "forge" || (loader == "neoforge" && game == "1.20.1"))
        && !version.starts_with(&format!("{game}-"))
    {
        normalized_version = format!("{game}-{version}");
        normalized_version.as_str()
    } else {
        version
    };
    if loader == "vanilla" {
        return Ok(game.into());
    }
    if loader == "optifine" {
        return Err("OptiFine requiere un JAR local elegido por el usuario. Impórtalo desde Instalaciones antes de usarlo en un pack; TP LAUNCHER no lo descarga ni redistribuye.".into());
    }
    let client = client()?;
    if loader == "fabric" || loader == "quilt" {
        let host = if loader == "fabric" {
            "https://meta.fabricmc.net/v2"
        } else {
            "https://meta.quiltmc.org/v3"
        };
        let value = get_json(
            &client,
            &format!("{host}/versions/loader/{game}/{version}/profile/json"),
            &[],
            false,
        )
        .await?;
        let id = value["id"]
            .as_str()
            .ok_or("El proveedor no devolvió el perfil del motor")?;
        safe_id(id)?;
        if value["inheritsFrom"].as_str() != Some(game) {
            return Err("El perfil del motor no corresponde a esta versión".into());
        }
        let dir = crate::root_dir()?.join("versions").join(id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::write(
            dir.join(format!("{id}.json")),
            serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        return Ok(id.into());
    }
    if loader != "forge" && loader != "neoforge" {
        return Err("Motor no compatible".into());
    }
    let artifact = if loader == "forge" || (loader == "neoforge" && game == "1.20.1") {
        "forge"
    } else {
        "neoforge"
    };
    let maven_base = if loader == "forge" {
        "https://maven.minecraftforge.net/net/minecraftforge/forge"
    } else if game == "1.20.1" {
        "https://maven.neoforged.net/releases/net/neoforged/forge"
    } else {
        "https://maven.neoforged.net/releases/net/neoforged/neoforge"
    };
    let root = crate::root_dir()?;
    let existing = crate::list_installed_versions_inner()?
        .into_iter()
        .find(|v| loader_profile_matches(&root, &v.id, game, loader, version));
    if let Some(existing) = existing {
        return Ok(existing.id);
    }
    let manifest = crate::load_version_manifest().await?;
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == game)
        .ok_or("No se encontró la versión Vanilla base del pack")?;
    let base = get_json(&client, &entry.url, &[], false).await?;
    let base_dir = root.join("versions").join(game);
    fs::create_dir_all(&base_dir).map_err(|e| e.to_string())?;
    save_json(&base_dir.join(format!("{game}.json")), &base)?;
    if let Some(url) = base
        .pointer("/downloads/client/url")
        .and_then(Value::as_str)
    {
        crate::download_verified(
            &client,
            url,
            &base_dir.join(format!("{game}.jar")),
            base.pointer("/downloads/client/sha1")
                .and_then(Value::as_str),
            base.pointer("/downloads/client/size")
                .and_then(Value::as_u64),
        )
        .await?;
    }
    let launcher_profiles = root.join("launcher_profiles.json");
    if !launcher_profiles.exists() {
        save_json(&launcher_profiles, &json!({"profiles":{},"settings":{}}))?;
    }
    let installers = root.join("runtime").join("studio-installers");
    fs::create_dir_all(&installers).map_err(|e| e.to_string())?;
    let jar = installers.join(format!("{artifact}-{version}-installer.jar"));
    crate::download_verified(
        &client,
        &format!("{maven_base}/{version}/{artifact}-{version}-installer.jar"),
        &jar,
        None,
        None,
    )
    .await?;
    let requirement = crate::get_version_java_requirement(game.into())
        .await?
        .max(8);
    let mut runtime = crate::discover_java_installations()
        .into_iter()
        .filter(|v| {
            (if requirement == 8 {
                v.major == 8
            } else {
                v.major >= requirement
            }) && crate::runtime_enabled(v.major)
        })
        .min_by_key(|v| v.major);
    if runtime.is_none() {
        crate::install_portable_java(
            operation
                .app
                .clone()
                .ok_or("Instala el runtime Java requerido desde Ajustes")?,
            requirement,
        )
        .await?;
        runtime = crate::discover_java_installations()
            .into_iter()
            .find(|v| v.major == requirement);
    }
    let java = runtime
        .ok_or("No hay un Java compatible para instalar el motor")?
        .path;
    operation.stage(
        "Instalando motor del modpack",
        &format!("{loader} {version} · Java {requirement}"),
        12.0,
    );
    let target = root.clone();
    let installer_operation = operation.clone();
    let output = tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new(java);
        command
            .arg("-jar")
            .arg(jar)
            .arg("--installClient")
            .arg(target)
            .current_dir(installers);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        use std::io::{BufRead, BufReader};
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = command.spawn().map_err(|e| e.to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or("No se pudo capturar el instalador")?;
        let stderr = child
            .stderr
            .take()
            .ok_or("No se pudo capturar el instalador")?;
        let out_op = installer_operation.clone();
        let out = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                out_op.log(&line);
            }
        });
        let err = std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                installer_operation.log(&line);
            }
        });
        let status = child.wait().map_err(|e| e.to_string())?;
        let _ = out.join();
        let _ = err.join();
        Ok::<_, String>(status)
    })
    .await
    .map_err(|e| e.to_string())??;
    if !output.success() {
        return Err(format!(
            "El instalador {artifact} terminó con {output}. Revisa la consola."
        ));
    }
    crate::list_installed_versions_inner()?
        .into_iter()
        .find(|v| loader_profile_matches(&root, &v.id, game, loader, version))
        .map(|v| v.id)
        .ok_or_else(|| "El instalador terminó sin crear un perfil válido".into())
}
pub fn pack_game_dir(id: &str) -> Result<PathBuf, String> {
    safe_id(id)?;
    let path = crate::root_dir()?.join("instances").join(id);
    let pack = read_pack(id)?;
    if pack.id != id {
        return Err("El identificador de la instancia no coincide".into());
    }
    Ok(path)
}
pub fn read_pack(id: &str) -> Result<StudioPack, String> {
    safe_id(id)?;
    serde_json::from_slice(
        &fs::read(
            crate::root_dir()?
                .join("instances")
                .join(id)
                .join("tps-pack.json"),
        )
        .map_err(|_| "No se encontró el modpack seleccionado")?,
    )
    .map_err(|e| format!("Metadatos del pack inválidos: {e}"))
}
pub fn ensure_pack_ready(id: &str) -> Result<(), String> {
    let pack = read_pack(id)?;
    if pack.install_state != "ready" {
        return Err(format!(
            "{} tiene una instalación incompleta. Abre Mis modpacks y pulsa Reintentar instalación. {}",
            pack.name, pack.install_error
        ));
    }
    Ok(())
}

fn curseforge_content_path(file: &Value, filename: &str) -> Result<PathBuf, String> {
    safe_file(filename)?;
    let folder = match file["classId"].as_u64() {
        Some(12) => "resourcepacks",
        Some(6552) => "shaderpacks",
        _ if filename.to_ascii_lowercase().ends_with(".jar") => "mods",
        _ => {
            return Err(format!(
                "No se pudo determinar el tipo de contenido de {filename}; no se instalará como un mod JAR."
            ));
        }
    };
    Ok(Path::new(folder).join(filename))
}

#[tauri::command]
pub async fn studio_retry_pack(app: AppHandle, pack_id: String) -> Result<StudioPack, String> {
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Ya hay una instalación en curso. Espera a que termine.")?;
    let pack = read_pack(&pack_id)?;
    if pack.install_state == "ready" {
        return Err("Esta instalación ya está completa".into());
    }
    if pack.archive_sha1.len() != 40 || !pack.archive_sha1.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(
            "No hay un archivo de importación guardado; selecciona otra vez el archivo original."
                .into(),
        );
    }
    let bytes = fs::read(
        crate::root_dir()?
            .join("cache/modpacks")
            .join(format!("{}.zip", pack.archive_sha1)),
    )
    .map_err(|_| "No se encontró el archivo de importación; selecciona otra vez el original.")?;
    verify_download(&bytes, None, Some(&pack.archive_sha1))?;
    let operation = Operation::new(Some(app), "Reintentar instalación del modpack");
    let result = import_bytes(&operation, bytes, pack.name).await;
    operation.complete(
        result,
        "El modpack se instaló correctamente y está listo para jugar.",
    )
}
fn supplied_pack_file_at(
    cache: &Path,
    pack: &StudioPack,
    filename: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let required = pack
        .missing_files
        .iter()
        .find(|file| file.filename == filename)
        .ok_or("El archivo no figura entre los archivos pendientes de este modpack")?;
    if required.sha1.len() != 40 || !required.sha1.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(
            "El proveedor no publicó un hash válido; no se puede verificar el archivo".into(),
        );
    }
    verify_download(bytes, Some(required.size), Some(&required.sha1))?;
    fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    fs::write(cache.join(required.sha1.to_ascii_lowercase()), bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn studio_supply_pack_file(
    pack_id: String,
    file_name: String,
    file_base64: String,
) -> Result<usize, String> {
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Espera a que termine la instalación actual.")?;
    let mut pack = read_pack(&pack_id)?;
    let bytes = decode_archive(&file_base64, MAX_DOWNLOAD as usize)?;
    supplied_pack_file_at(
        &crate::root_dir()?.join("cache/content"),
        &pack,
        &file_name,
        &bytes,
    )?;
    pack.missing_files.retain(|file| file.filename != file_name);
    save_json(&pack_game_dir(&pack_id)?.join("tps-pack.json"), &pack)?;
    Ok(pack.missing_files.len())
}

fn save_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let temporary = path.with_extension(format!("{}.pending", Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    if let Err(error) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(error.to_string());
    }
    drop(file);
    fs::rename(&temporary, path).map_err(|e| {
        let _ = fs::remove_file(temporary);
        e.to_string()
    })
}

fn content(dir: &Path) -> Vec<ContentFile> {
    fs::read(dir.join("tps-content.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Vec<ContentFile>>(&b).ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|file| safe_file(&file.filename).is_ok() && file.filename.ends_with(".jar"))
        .collect()
}
#[tauri::command]
pub fn studio_list_packs() -> Result<Vec<StudioPack>, String> {
    let mut packs = Vec::new();
    for entry in fs::read_dir(crate::root_dir()?.join("instances"))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
    {
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            if let Ok(pack) = read_pack(&entry.file_name().to_string_lossy()) {
                packs.push(pack);
            }
        }
    }
    packs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(packs)
}
#[tauri::command]
pub async fn studio_create_pack(
    app: AppHandle,
    name: String,
    game_version: String,
    loader: String,
    loader_version: String,
    icon_base64: Option<String>,
) -> Result<StudioPack, String> {
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Ya hay una instalación en curso. Espera a que termine.")?;
    let operation = Operation::new(Some(app), "Crear modpack");
    let result = create_pack_inner(
        &operation,
        name,
        game_version,
        loader,
        loader_version,
        icon_base64,
    )
    .await;
    operation.complete(result, "El modpack está listo para añadir contenido.")
}
async fn create_pack_inner(
    operation: &Operation,
    name: String,
    game_version: String,
    loader: String,
    loader_version: String,
    icon_base64: Option<String>,
) -> Result<StudioPack, String> {
    if name.trim().is_empty() || name.chars().count() > 80 {
        return Err("El nombre del modpack debe tener entre 1 y 80 caracteres".into());
    }
    safe_id(&game_version)?;
    let chosen = if loader_version.is_empty() {
        studio_loader_versions(loader.clone(), game_version.clone())
            .await?
            .into_iter()
            .next()
            .ok_or("No hay versiones de este motor para el juego elegido")?
            .version
    } else {
        loader_version
    };
    operation.stage(
        "Preparando motor",
        &format!("Minecraft {game_version} / {loader} {chosen}"),
        8.0,
    );
    let version_id = install_loader(operation, &game_version, &loader, &chosen).await?;
    let slug = name
        .trim()
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .take(60)
        .collect::<String>();
    let id = format!(
        "{}-{}",
        if slug.is_empty() { "pack" } else { &slug },
        &Uuid::new_v4().simple().to_string()[..8]
    );
    let dir = crate::root_dir()?.join("instances").join(&id);
    fs::create_dir(&dir).map_err(|e| e.to_string())?;
    for folder in [
        "mods",
        "config",
        "resourcepacks",
        "shaderpacks",
        "screenshots",
        "saves",
    ] {
        fs::create_dir(dir.join(folder)).map_err(|e| e.to_string())?;
    }
    let mut icon_data = String::new();
    if let Some(icon) = icon_base64.filter(|s| !s.is_empty()) {
        let bytes = decode_archive(&icon, 8 * 1024 * 1024)?;
        let png = icon_png(&bytes, 128)
            .ok_or("El icono no es una imagen PNG/JPEG válida o supera sus límites")?;
        fs::write(dir.join("icon.png"), &png).map_err(|e| e.to_string())?;
        icon_data = image_data(&png);
    }
    let pack = StudioPack {
        id,
        name: name.trim().into(),
        game_version,
        loader,
        loader_version: chosen,
        version_id,
        icon_data,
        icon_url: String::new(),
        source_provider: String::new(),
        source_project_id: String::new(),
        created_at: crate::unix_time_seconds(),
        install_state: ready_state(),
        install_error: String::new(),
        archive_sha1: String::new(),
        missing_files: Vec::new(),
    };
    save_json(&dir.join("tps-pack.json"), &pack)?;
    Ok(pack)
}

fn short_text(value: &str, limit: usize) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(limit)
        .collect()
}

fn jar_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
    limit: usize,
) -> Option<Vec<u8>> {
    if name.is_empty()
        || name.len() > 512
        || name.contains('\\')
        || Path::new(name)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let entry = archive.by_name(name).ok()?;
    if entry.is_dir() || entry.size() > limit as u64 {
        return None;
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry.take(limit as u64 + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() <= limit).then_some(bytes)
}

fn icon_png(bytes: &[u8], edge: u32) -> Option<Vec<u8>> {
    if bytes.len() > 2 * 1024 * 1024 {
        return None;
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let decoded = reader.decode().ok()?.thumbnail(edge, edge);
    let mut png = Cursor::new(Vec::new());
    decoded.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(png.into_inner())
}

fn image_data(bytes: &[u8]) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

fn official_cover_url(value: &str) -> Option<url::Url> {
    let url = url::Url::parse(value).ok()?;
    (url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none_or(|p| p == 443)
        && url.host_str().is_some_and(|host| {
            [
                "cdn.modrinth.com",
                "media.forgecdn.net",
                "mediafilez.forgecdn.net",
                "edge.forgecdn.net",
            ]
            .contains(&host)
        }))
    .then_some(url)
}

async fn save_pack_cover(client: &Client, pack: &mut StudioPack, source: &str) {
    let Some(url) = official_cover_url(source) else {
        return;
    };
    pack.icon_url = url.to_string();
    let png = async {
        let response = client
            .get(url)
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .ok()?;
        if !response.status().is_success()
            || official_cover_url(response.url().as_str()).is_none()
            || response
                .content_length()
                .is_some_and(|length| length > 2 * 1024 * 1024)
        {
            return None;
        }
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.ok()?;
            if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
                return None;
            }
            bytes.extend_from_slice(&chunk);
        }
        icon_png(&bytes, 128)
    }
    .await;
    if let Ok(dir) = pack_game_dir(&pack.id) {
        if let Some(png) = png {
            if fs::write(dir.join("icon.png"), &png).is_ok() {
                pack.icon_data = image_data(&png);
            }
        }
    }
}

fn metadata_authors(value: &Value) -> Vec<String> {
    let authors = if let Some(values) = value.as_array() {
        values
            .iter()
            .filter_map(|v| v.as_str().or_else(|| v["name"].as_str()))
            .map(str::to_owned)
            .collect()
    } else if let Some(values) = value.as_object() {
        values.keys().cloned().collect()
    } else {
        value
            .as_str()
            .unwrap_or_default()
            .split(',')
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    authors
        .into_iter()
        .map(|name| short_text(&name, 100))
        .filter(|name| !name.is_empty())
        .take(12)
        .collect()
}

fn json_icon(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .or_else(|| {
            value
                .as_object()
                .and_then(|icons| {
                    icons
                        .get("128")
                        .or_else(|| icons.get("64"))
                        .or_else(|| icons.values().next())
                })
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_default()
}



fn mod_icon_path(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/");
    let normalized = normalized.trim_start_matches('/');
    let normalized = normalized.strip_prefix("./").unwrap_or(normalized);
    if normalized.is_empty()
        || normalized.len() > 512
        || normalized.contains(':')
        || normalized.chars().any(char::is_control)
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(normalized.to_owned())
}
fn jar_icon<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    path: &str,
) -> Option<Vec<u8>> {
    let path = mod_icon_path(path)?;
    if let Some(bytes) = jar_entry(archive, &path, 2 * 1024 * 1024) {
        if let Some(png) = icon_png(&bytes, 96) {
            return Some(png);
        }
    }
    let matches = archive
        .file_names()
        .filter(|entry| entry.eq_ignore_ascii_case(&path))
        .take(2)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return None;
    }
    jar_entry(archive, &matches[0], 2 * 1024 * 1024).and_then(|bytes| icon_png(&bytes, 96))
}

fn jar_metadata_uncached(path: &Path) -> JarMetadata {
    let fallback = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Mod local")
        .trim_end_matches(".disabled")
        .trim_end_matches(".jar")
        .replace('_', " ");
    let mut result = JarMetadata {
        name: short_text(&fallback, 160),
        ..Default::default()
    };
    let Some(file) = fs::File::open(path)
        .ok()
        .filter(|file| file.metadata().is_ok_and(|meta| meta.len() <= MAX_DOWNLOAD))
    else {
        return result;
    };
    let Ok(mut archive) = zip::ZipArchive::new(file) else {
        return result;
    };
    if archive.len() > 50_000 {
        return result;
    }
    let mut icon = String::new();
    let mut declared_icons = Vec::new();
    for (name, loader) in [
        ("fabric.mod.json", "fabric"),
        ("quilt.mod.json", "quilt"),
        ("mcmod.info", "forge"),
    ] {
        let Some(bytes) = jar_entry(&mut archive, name, 1024 * 1024) else {
            continue;
        };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        let metadata = if loader == "quilt" {
            &value["quilt_loader"]["metadata"]
        } else if name == "mcmod.info" {
            value
                .as_array()
                .and_then(|values| values.first())
                .or_else(|| {
                    value["modList"]
                        .as_array()
                        .and_then(|values| values.first())
                })
                .unwrap_or(&value)
        } else {
            &value
        };
        let display_name = metadata["name"]
            .as_str()
            .or_else(|| metadata["modid"].as_str())
            .or_else(|| metadata["id"].as_str())
            .unwrap_or(&result.name);
        result.name = short_text(display_name, 160);
        result.version = short_text(
            if loader == "quilt" {
                value["quilt_loader"]["version"].as_str()
            } else {
                metadata["version"].as_str()
            }
            .unwrap_or_default(),
            100,
        );
        result.description = short_text(metadata["description"].as_str().unwrap_or_default(), 512);
        result.authors = metadata_authors(if loader == "quilt" {
            &metadata["contributors"]
        } else if name == "mcmod.info" {
            &metadata["authorList"]
        } else {
            &metadata["authors"]
        });
        if result.authors.is_empty() {
            result.authors = metadata_authors(&metadata["authorList"]);
        }
        icon = json_icon(&metadata["icon"]);
        if icon.is_empty() {
            icon = text(metadata, "logoFile");
        }


        if name == "mcmod.info" {
            let entries = value.as_array().or_else(|| value["modList"].as_array());
            for row in entries.into_iter().flatten().take(64) {
                if let Some(path) = row["logoFile"].as_str() {
                    declared_icons.push(path.to_string());
                }
            }
        }
        result.loader = loader.into();
        break;
    }
    if result.loader.is_empty() {
        for (name, loader) in [
            ("META-INF/neoforge.mods.toml", "neoforge"),
            ("META-INF/mods.toml", "forge"),
        ] {
            let Some(bytes) = jar_entry(&mut archive, name, 1024 * 1024) else {
                continue;
            };
            let Some(value) = std::str::from_utf8(&bytes)
                .ok()
                .and_then(|text| toml::from_str::<toml::Value>(text).ok())
            else {
                continue;
            };
            let Some(metadata) = value
                .get("mods")
                .and_then(toml::Value::as_array)
                .and_then(|mods| mods.first())
            else {
                continue;
            };
            result.name = short_text(
                metadata
                    .get("displayName")
                    .or_else(|| metadata.get("modId"))
                    .and_then(toml::Value::as_str)
                    .unwrap_or(&result.name),
                160,
            );
            result.version = short_text(
                metadata
                    .get("version")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default(),
                100,
            );
            result.description = short_text(
                metadata
                    .get("description")
                    .and_then(toml::Value::as_str)
                    .unwrap_or_default(),
                512,
            );
            result.authors = metadata_authors(
                &serde_json::to_value(metadata.get("authors").or_else(|| value.get("authors")))
                    .unwrap_or(Value::Null),
            );
            icon = metadata
                .get("logoFile")
                .or_else(|| value.get("logoFile"))
                .and_then(toml::Value::as_str)
                .unwrap_or_default()
                .into();
            for row in value
                .get("mods")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .take(64)
            {
                if let Some(path) = row.get("logoFile").and_then(toml::Value::as_str) {
                    declared_icons.push(path.to_string());
                }
            }
            result.loader = loader.into();
            break;
        }
    }
    if result.version.contains("${") {
        result.version = jar_entry(&mut archive, "META-INF/MANIFEST.MF", 64 * 1024)
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|manifest| {
                manifest.lines().find_map(|line| {
                    line.strip_prefix("Implementation-Version:")
                        .map(|version| short_text(version, 100))
                })
            })
            .unwrap_or_default();
    }


    let mut candidates = mod_icon_path(&icon).into_iter().collect::<Vec<_>>();
    candidates.extend(declared_icons.into_iter().filter_map(|s| mod_icon_path(&s)));
    for conventional in ["icon.png", "logo.png", "pack.png"] {
        if !candidates.iter().any(|s| s == conventional) {
            candidates.push(conventional.into());
        }
    }
    for icon in candidates {
        result.icon_data = jar_icon(&mut archive, &icon)
            .map(|bytes| image_data(&bytes))
            .unwrap_or_default();
        if !result.icon_data.is_empty() {
            break;
        }
    }
    result
}

fn jar_metadata(path: &Path) -> JarMetadata {
    let cache = (|| {
        let info = fs::metadata(path).ok()?;
        let modified = info
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos();
        let identity = format!(
            "v3|{}|{}|{modified}",
            path.to_string_lossy().trim_end_matches(".disabled"),
            info.len()
        );
        Some(
            crate::root_dir()
                .ok()?
                .join("cache")
                .join("mod-metadata")
                .join(format!("{:x}.json", Sha1::digest(identity.as_bytes()))),
        )
    })();
    if let Some(path) = &cache {
        if let Ok(file) = fs::File::open(path) {
            let mut bytes = Vec::new();
            if file.take(256 * 1024 + 1).read_to_end(&mut bytes).is_ok()
                && bytes.len() <= 256 * 1024
            {
                if let Ok(metadata) = serde_json::from_slice(&bytes) {
                    return metadata;
                }
            }
        }
    }
    let metadata = jar_metadata_uncached(path);
    if let Some(path) = cache {
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_ok() {
                let _ = save_json(&path, &metadata);
            }
        }
    }
    metadata
}

#[tauri::command]
pub fn studio_list_pack_mods(pack_id: String) -> Result<Vec<PackMod>, String> {
    let dir = pack_game_dir(&pack_id)?;
    let index = content(&dir);
    let mut mods = Vec::new();
    for entry in fs::read_dir(dir.join("mods"))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().into_owned();
        if !filename.ends_with(".jar") && !filename.ends_with(".jar.disabled") {
            continue;
        }
        let source = index
            .iter()
            .find(|f| f.filename == filename.trim_end_matches(".disabled"));
        let metadata = jar_metadata(&entry.path());
        mods.push(PackMod {
            enabled: !filename.ends_with(".disabled"),
            size: entry.metadata().map(|m| m.len()).unwrap_or(0),
            provider: source.map(|f| f.provider.clone()).unwrap_or_default(),
            project_id: source.map(|f| f.project_id.clone()).unwrap_or_default(),
            filename,
            name: metadata.name,
            version: metadata.version,
            authors: metadata.authors,
            description: metadata.description,
            icon_data: metadata.icon_data,
            loader: metadata.loader,
        });
    }
    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(mods)
}
fn art_name(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
fn exact_art_candidates(rows: Vec<StudioProject>, wanted: &str) -> Vec<StudioProject> {
    rows.into_iter()
        .filter(|p| art_name(&p.title) == wanted && official_cover_url(&p.icon_url).is_some())
        .collect()
}
async fn lookup_art(
    client: &Client,
    provider: &str,
    id: &str,
    name: &str,
    kind: &str,
) -> Option<String> {
    if ["curseforge", "modrinth"].contains(&provider)
        && !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        let (url, cf) = if provider == "curseforge" {
            (format!("{CF}/mods/{id}"), true)
        } else {
            (format!("{MR}/project/{id}"), false)
        };
        if let Ok(value) = get_json(client, &url, &[], cf).await {
            let info = if cf { &value["data"] } else { &value };
            let actual = if cf {
                cf_project(info, kind)
            } else {
                mr_project(info)
            };
            if actual.project_type != kind {
                return None;
            }
            let icon = if cf {
                cf_logo(info)
            } else {
                info["icon_url"].as_str()
            };
            if let Some(icon) = icon.filter(|s| official_cover_url(s).is_some()) {
                return Some(icon.into());
            }
        }


        return None;
    }
    let wanted = art_name(name);
    if wanted.len() < 3 {
        return None;
    }
    let mut candidates = Vec::new();
    if let Ok((rows, _)) = search_mr(client, name, kind, "all", "all", "relevance", "", 0, 24).await
    {
        candidates.extend(exact_art_candidates(rows, &wanted));
    }
    if crate::private_integrations::curseforge_api_key().is_ok() {
        if let Ok((rows, _)) =
            search_cf(client, name, kind, "all", "all", "relevance", "", 0, 24).await
        {
            candidates.extend(exact_art_candidates(rows, &wanted));
        }
    }
    if candidates.len() == 1 {
        return Some(candidates.remove(0).icon_url);
    }


    let author = candidates
        .first()
        .map(|p| art_name(&p.author))
        .unwrap_or_default();
    if !author.is_empty() && candidates.iter().all(|p| art_name(&p.author) == author) {
        return candidates.first().map(|p| p.icon_url.clone());
    }
    None
}
async fn cached_mod_art(client: &Client, provider: &str, id: &str, name: &str) -> Option<String> {
    let folder = crate::root_dir().ok()?.join("cache/mod-art");
    fs::create_dir_all(&folder).ok()?;
    let key = format!(
        "{:x}",
        Sha1::digest(format!("v2|{provider}|{id}|{name}").as_bytes())
    );
    let file = folder.join(format!("{key}.png"));
    let missing = folder.join(format!("{key}.none"));
    if fs::symlink_metadata(&file)
        .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink() && m.len() < 512 * 1024)
    {
        if let Ok(source) = fs::File::open(&file) {
            let mut bytes = Vec::new();
            if source.take(512 * 1024).read_to_end(&mut bytes).is_ok() {
                if let Some(png) = icon_png(&bytes, 96) {
                    return Some(image_data(&png));
                }
            }
        }
    }
    if fs::metadata(&missing)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|t| t < Duration::from_secs(900))
    {
        return None;
    }
    let Some(source) = lookup_art(client, provider, id, name, "mod").await else {
        let _ = fs::write(missing, b"");
        return None;
    };
    let response = client
        .get(official_cover_url(&source)?)
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() || official_cover_url(response.url().as_str()).is_none() {
        return None;
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            return None;
        }
        bytes.extend_from_slice(&chunk);
    }
    let png = icon_png(&bytes, 96)?;
    let pending = folder.join(format!("{key}-{}.pending", uuid::Uuid::new_v4()));
    fs::write(&pending, &png)
        .and_then(|_| fs::rename(&pending, file))
        .map_err(|_| {
            let _ = fs::remove_file(&pending);
        })
        .ok()?;
    Some(image_data(&png))
}
#[tauri::command]
pub async fn studio_enrich_mod_art(
    app: AppHandle,
    pack_id: String,
) -> Result<Vec<PackMod>, String> {
    let mods = studio_list_pack_mods(pack_id.clone())?;
    let client = Client::builder()
        .user_agent("TP-LAUNCHER/0.1.0 (https://github.com/tomaap07/MinecraftTPS)")
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|e| e.to_string())?;
    Ok(futures_util::stream::iter(mods).map(|mut item|{let client=client.clone();let app=app.clone();let pack_id=pack_id.clone();async move{if item.icon_data.is_empty(){if let Some(icon)=cached_mod_art(&client,&item.provider,&item.project_id,&item.name).await{item.icon_data=icon;let _=app.emit("studio-mod-art",json!({"pack_id":pack_id,"filename":item.filename,"icon_data":item.icon_data}));}}item}}).buffered(4).collect().await)
}
#[tauri::command]
pub async fn studio_enrich_library_art() -> Result<Vec<StudioPack>, String> {
    let mut packs = studio_list_packs()?;
    let client = client()?;
    for pack in &mut packs {
        if !pack.icon_data.is_empty() {
            continue;
        }


        if official_cover_url(&pack.icon_url).is_some() {
            let source = pack.icon_url.clone();
            save_pack_cover(&client, pack, &source).await;
        }
        if pack.icon_data.is_empty() {
            if let Some(icon) = lookup_art(
                &client,
                &pack.source_provider,
                &pack.source_project_id,
                &pack.name,
                "modpack",
            )
            .await
            {
                save_pack_cover(&client, pack, &icon).await;
            }
        }


        if let Ok(mut current) = read_pack(&pack.id) {
            if current.icon_data.is_empty() {
                current.icon_data = pack.icon_data.clone();
                current.icon_url = pack.icon_url.clone();
                if let Ok(dir) = pack_game_dir(&current.id) {
                    let _ = save_json(&dir.join("tps-pack.json"), &current);
                }
            }
            *pack = current;
        }
    }
    Ok(packs)
}
fn pack_mod_path(pack: &str, file: &str) -> Result<PathBuf, String> {
    safe_file(file)?;
    if !file.ends_with(".jar") && !file.ends_with(".jar.disabled") {
        return Err("Solo se admiten mods JAR".into());
    }
    Ok(pack_game_dir(pack)?.join("mods").join(file))
}
#[tauri::command]
pub fn studio_set_pack_mod_enabled(
    pack_id: String,
    file_name: String,
    enabled: bool,
) -> Result<(), String> {
    let source = pack_mod_path(&pack_id, &file_name)?;
    let target_name = if enabled {
        file_name.trim_end_matches(".disabled").to_string()
    } else if file_name.ends_with(".disabled") {
        file_name.clone()
    } else {
        format!("{file_name}.disabled")
    };
    let target = pack_mod_path(&pack_id, &target_name)?;
    if source == target {
        return Ok(());
    }
    if target.exists() {
        return Err("Ya existe otro mod con ese nombre en el estado de destino".into());
    }
    fs::rename(source, target).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn studio_remove_pack_mod(pack_id: String, file_name: String) -> Result<(), String> {
    let path = pack_mod_path(&pack_id, &file_name)?;
    fs::remove_file(path).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn studio_open_pack_folder(app: AppHandle, pack_id: String) -> Result<(), String> {
    app.opener()
        .open_path(
            pack_game_dir(&pack_id)?.to_string_lossy().as_ref(),
            None::<&str>,
        )
        .map_err(|e| e.to_string())
}

async fn raw_version(
    client: &Client,
    provider: &str,
    project: &str,
    id: &str,
) -> Result<Value, String> {
    safe_id(id)?;
    if provider == "modrinth" {
        get_json(client, &format!("{MR}/version/{id}"), &[], false).await
    } else if provider == "curseforge" {
        safe_id(project)?;
        Ok(get_json(
            client,
            &format!("{CF}/mods/{project}/files/{id}"),
            &[],
            true,
        )
        .await?["data"]
            .clone())
    } else {
        Err("Proveedor no válido".into())
    }
}
async fn cf_pack_metadata(manifest: &Value) -> Result<HashMap<String, Value>, String> {
    let files = manifest["files"]
        .as_array()
        .ok_or("El manifiesto de CurseForge no contiene una lista de archivos")?;
    let cache = crate::root_dir()?.join("cache/curseforge-files");
    fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let mut result = HashMap::new();
    let mut pending = Vec::new();
    for file in files
        .iter()
        .filter(|f| f["required"].as_bool() != Some(false))
    {
        let project = identifier(file, "projectID");
        let id = identifier(file, "fileID");
        safe_id(&project)?;
        safe_id(&id)?;
        let path = cache.join(format!("{project}-{id}.json"));
        let cached = fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .filter(|v| identifier(v, "id") == id && identifier(v, "modId") == project);
        if let Some(value) = cached {
            result.insert(id, value);
        } else {
            pending.push((project, id));
        }
    }
    for group in pending.chunks(50) {
        let ids = group
            .iter()
            .map(|(_, id)| {
                id.parse::<u64>()
                    .map_err(|_| "El manifiesto contiene un ID de archivo no numérico")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let response = request_json(
            curseforge_client()?
                .post(format!("{CF}/mods/files"))
                .header(
                    "x-api-key",
                    crate::private_integrations::curseforge_api_key()?,
                )
                .header("Accept", "application/json")
                .header(
                    "User-Agent",
                    "TP-LAUNCHER/0.1.0 (https://github.com/tomaap07/MinecraftTPS)",
                )
                .json(&json!({"fileIds":ids})),
            true,
        )
        .await?;
        for (project, id) in group {
            let value = response["data"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|v| identifier(v, "id") == *id && identifier(v, "modId") == *project)
                .ok_or_else(|| {
                    format!("CurseForge no publicó el archivo {id} del proyecto {project}")
                })?
                .clone();
            save_json(&cache.join(format!("{project}-{id}.json")), &value)?;
            result.insert(id.clone(), value);
        }
    }
    Ok(result)
}

fn verify_download(bytes: &[u8], size: Option<u64>, hash: Option<&str>) -> Result<(), String> {
    if size.is_some_and(|s| s > 0 && s != bytes.len() as u64) {
        return Err("La descarga está incompleta: tamaño distinto al publicado".into());
    }
    if let Some(hash) = hash.filter(|s| !s.is_empty()) {
        if format!("{:x}", Sha1::digest(bytes)) != hash.to_ascii_lowercase() {
            return Err(
                "El archivo no coincide con su hash SHA-1; se canceló la instalación".into(),
            );
        }
    }
    Ok(())
}
async fn file_bytes_progress(
    client: &Client,
    source: &str,
    size: Option<u64>,
    hash: Option<&str>,
    operation: Option<&Operation>,
) -> Result<Vec<u8>, String> {
    let url = url::Url::parse(source).map_err(|_| "URL de descarga inválida")?;
    if url.scheme() != "https" {
        return Err("La descarga debe usar HTTPS".into());
    }
    let cache = hash
        .filter(|h| h.len() == 40 && h.chars().all(|c| c.is_ascii_hexdigit()))
        .and_then(|h| {
            crate::root_dir()
                .ok()
                .map(|root| root.join("cache/content").join(h.to_ascii_lowercase()))
        });
    if let Some(path) = &cache {
        if let Ok(bytes) = fs::read(path) {
            if verify_download(&bytes, size, hash).is_ok() {
                if let Some(op) = operation {
                    op.bytes(bytes.len() as u64, bytes.len() as u64);
                }
                return Ok(bytes);
            }
        }
    }
    let mut last_error = String::new();
    for attempt in 0..3 {
        let result = async {
            let response = client
                .get(url.clone())
                .send()
                .await
                .map_err(|e| format!("No se pudo descargar el archivo: {e}"))?
                .error_for_status()
                .map_err(|e| e.to_string())?;
            let total = size
                .filter(|s| *s > 0)
                .or(response.content_length())
                .unwrap_or(0);
            if total > MAX_DOWNLOAD {
                return Err("El archivo supera 512 MB".into());
            }
            let mut stream = response.bytes_stream();
            let mut bytes = Vec::new();
            let mut notified = std::time::Instant::now();
            while let Some(chunk) = stream.next().await {
                bytes.extend_from_slice(&chunk.map_err(|e| e.to_string())?);
                if bytes.len() as u64 > MAX_DOWNLOAD {
                    return Err("El archivo supera 512 MB".into());
                }
                if notified.elapsed() >= Duration::from_millis(120) {
                    if let Some(op) = operation {
                        op.bytes(bytes.len() as u64, total);
                    }
                    notified = std::time::Instant::now();
                }
            }
            verify_download(&bytes, size, hash)?;
            if let Some(op) = operation {
                op.bytes(bytes.len() as u64, total);
            }
            Ok::<_, String>(bytes)
        }
        .await;
        match result {
            Ok(bytes) => {
                if let Some(path) = &cache {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
                    fs::write(&temporary, &bytes).map_err(|e| e.to_string())?;
                    if path.exists() {
                        fs::remove_file(path).map_err(|e| e.to_string())?;
                    }
                    fs::rename(temporary, path).map_err(|e| e.to_string())?;
                }
                return Ok(bytes);
            }
            Err(error) => {
                last_error = error;
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(400 * (attempt + 1))).await;
                }
            }
        }
    }
    Err(last_error)
}
struct DownloadedFile {
    size: u64,
    sha1: String,
}
fn verified_content_file(
    path: &Path,
    size: Option<u64>,
    sha1: Option<&str>,
    sha512: Option<&str>,
) -> Option<DownloadedFile> {
    let file = fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    if length > 2 * 1024 * 1024 * 1024 || size.is_some_and(|s| s > 0 && s != length) {
        return None;
    }
    let mut one = Sha1::new();
    let mut two = Sha512::new();
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = [0u8; 65536];
    loop {
        let count = reader.read(&mut buffer).ok()?;
        if count == 0 {
            break;
        }
        one.update(&buffer[..count]);
        two.update(&buffer[..count]);
    }
    let actual = format!("{:x}", one.finalize());
    if sha1
        .filter(|h| !h.is_empty())
        .is_some_and(|h| !h.eq_ignore_ascii_case(&actual))
        || sha512
            .filter(|h| !h.is_empty())
            .is_some_and(|h| !h.eq_ignore_ascii_case(&format!("{:x}", two.finalize())))
    {
        return None;
    }
    Some(DownloadedFile {
        size: length,
        sha1: actual,
    })
}
async fn download_content_file(
    client: &Client,
    source: &str,
    target: &Path,
    size: Option<u64>,
    sha1: Option<&str>,
    sha512: Option<&str>,
    operation: &Operation,
) -> Result<DownloadedFile, String> {
    const MAX_CONTENT: u64 = 2 * 1024 * 1024 * 1024;
    if let Some(info) = verified_content_file(target, size, sha1, sha512) {
        operation.bytes(info.size, info.size);
        return Ok(info);
    }
    let key = sha1
        .filter(|h| h.len() == 40 && h.bytes().all(|c| c.is_ascii_hexdigit()))
        .map(|h| h.to_ascii_lowercase())
        .unwrap_or_else(|| format!("url-{:x}", Sha1::digest(source.as_bytes())));
    let cache = crate::root_dir()?.join("cache/content").join(key);
    if let Some(parent) = cache.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if let Some(info) = verified_content_file(&cache, size, sha1, sha512) {
        fs::copy(&cache, target).map_err(|e| e.to_string())?;
        operation.bytes(info.size, info.size);
        return Ok(info);
    }
    let url = url::Url::parse(source).map_err(|_| "URL de contenido inválida")?;
    if url.scheme() != "https" {
        return Err("La descarga debe usar HTTPS".into());
    }
    let mut last = String::new();
    for attempt in 0..3 {
        let temporary = cache.with_extension(format!("{}.download", Uuid::new_v4()));
        let result = async {
            use tokio::io::AsyncWriteExt;
            let response = client.get(url.clone()).timeout(Duration::from_secs(900)).send().await.map_err(|e|e.to_string())?.error_for_status().map_err(|e|e.to_string())?;
            let total = size.filter(|s| *s>0).or(response.content_length()).unwrap_or(0);
            if total > MAX_CONTENT { return Err("Un archivo del pack supera el límite de 2 GB".into()); }
            let mut output = tokio::fs::File::create(&temporary).await.map_err(|e|e.to_string())?;
            let mut stream = response.bytes_stream(); let mut count=0u64; let mut one=Sha1::new(); let mut two=Sha512::new();
            let mut notified=std::time::Instant::now();
            while let Some(chunk)=stream.next().await {
                let chunk=chunk.map_err(|e|e.to_string())?; count+=chunk.len() as u64;
                if count > MAX_CONTENT { return Err("Un archivo del pack supera el límite de 2 GB".into()); }
                one.update(&chunk); two.update(&chunk); output.write_all(&chunk).await.map_err(|e|e.to_string())?;
                if notified.elapsed()>=Duration::from_millis(120){operation.bytes(count,total);notified=std::time::Instant::now();}
            }
            output.sync_all().await.map_err(|e|e.to_string())?; drop(output);
            let hash=format!("{:x}",one.finalize());
            if size.is_some_and(|s|s>0&&s!=count) || sha1.filter(|s|!s.is_empty()).is_some_and(|s|!s.eq_ignore_ascii_case(&hash))
                || sha512.filter(|s|!s.is_empty()).is_some_and(|s|!s.eq_ignore_ascii_case(&format!("{:x}",two.finalize()))) {
                return Err("El archivo no coincide con el tamaño o los hashes publicados; vuelve a intentar la instalación".into());
            }
            fs::rename(&temporary,&cache).map_err(|e|e.to_string())?;
            fs::copy(&cache,target).map_err(|e|e.to_string())?;
            operation.bytes(count,total); Ok::<_,String>(DownloadedFile {size:count,sha1:hash})
        }.await;
        match result {
            Ok(info) => return Ok(info),
            Err(error) => {
                last = error;
                let _ = fs::remove_file(&temporary);
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_millis(600 * (attempt + 1))).await;
                }
            }
        }
    }
    Err(last)
}

async fn cf_download_url(client: &Client, project: &str, file: &Value) -> Result<String, String> {
    if let Some(url) = file["downloadUrl"].as_str().filter(|u| !u.is_empty()) {
        return Ok(url.into());
    }
    let id = identifier(file, "id");
    let url = get_json(
        client,
        &format!("{CF}/mods/{project}/files/{id}/download-url"),
        &[],
        true,
    )
    .await?;
    url["data"].as_str().filter(|s|!s.is_empty()).map(str::to_owned).ok_or_else(||format!("El autor deshabilitó la descarga externa de este archivo. Ábrelo desde CurseForge: https://www.curseforge.com/minecraft/mc-mods/{project}"))
}
fn commit_mod_files(dir: &Path, staging: &Path, files: &[ContentFile]) -> Result<(), String> {
    commit_content_files(dir, staging, files, "mods", "tps-content.json")
}
fn commit_content_files(
    dir: &Path,
    staging: &Path,
    files: &[ContentFile],
    folder: &str,
    index_name: &str,
) -> Result<(), String> {
    let old = if folder == "mods" {
        content(dir)
    } else {
        fs::read(dir.join(index_name))
            .ok()
            .and_then(|b| serde_json::from_slice::<Vec<ContentFile>>(&b).ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|f| {
                safe_file(&f.filename).is_ok() && f.filename.to_ascii_lowercase().ends_with(".zip")
            })
            .collect()
    };
    let replaced = files
        .iter()
        .map(|f| (&f.provider, &f.project_id))
        .collect::<HashSet<_>>();
    let mut next = old
        .iter()
        .filter(|f| !replaced.contains(&(&f.provider, &f.project_id)))
        .cloned()
        .collect::<Vec<_>>();
    next.extend_from_slice(files);
    let next_index = staging.join("content-index.next");
    save_json(&next_index, &next)?;
    let mut affected = files
        .iter()
        .map(|f| f.filename.clone())
        .collect::<HashSet<_>>();
    for file in old
        .iter()
        .filter(|f| replaced.contains(&(&f.provider, &f.project_id)))
    {
        affected.insert(file.filename.clone());
        affected.insert(format!("{}.disabled", file.filename));
    }
    for file in files {
        affected.insert(format!("{}.disabled", file.filename));
    }
    let backups = staging.join(".backup");
    fs::create_dir(&backups).map_err(|e| e.to_string())?;
    let mut moved = Vec::<(PathBuf, PathBuf)>::new();
    let mut placed = Vec::<PathBuf>::new();
    let result = (|| {
        for name in affected {
            let target = dir.join(folder).join(&name);
            if target.exists() {
                let kind = fs::symlink_metadata(&target)
                    .map_err(|e| e.to_string())?
                    .file_type();
                if !kind.is_file() || kind.is_symlink() {
                    return Err("Un archivo de destino no es un archivo regular".to_owned());
                }
                let backup = backups.join(&name);
                fs::rename(&target, &backup).map_err(|e| e.to_string())?;
                moved.push((target, backup));
            }
        }
        let index = dir.join(index_name);
        if index.exists() {
            let backup = backups.join("content-index.previous");
            fs::rename(&index, &backup).map_err(|e| e.to_string())?;
            moved.push((index.clone(), backup));
        }
        for file in files {
            let target = dir.join(folder).join(&file.filename);
            fs::rename(staging.join(&file.filename), &target).map_err(|e| e.to_string())?;
            placed.push(target);
        }
        fs::rename(&next_index, &index).map_err(|e| e.to_string())?;
        placed.push(index);
        Ok(())
    })();
    if let Err(error) = result {
        let mut failures = Vec::new();
        for target in placed.into_iter().rev() {
            if let Err(e) = fs::remove_file(target) {
                failures.push(e.to_string());
            }
        }
        for (target, backup) in moved.into_iter().rev() {
            if let Err(e) = fs::rename(backup, target) {
                failures.push(e.to_string());
            }
        }
        if !failures.is_empty() {
            return Err(format!(
                "{error}. No se pudo restaurar todo; copias originales conservadas en {}: {}",
                backups.display(),
                failures.join("; ")
            ));
        }
        return Err(format!("{error}. Se restauraron los archivos anteriores."));
    }
    Ok(())
}
#[tauri::command]
pub async fn studio_install_version(
    app: AppHandle,
    provider: String,
    project_id: String,
    file_id: String,
    project_type: String,
    pack_id: Option<String>,
    username: String,
    version_id: String,
) -> Result<String, String> {
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Ya hay una instalación en curso. Espera a que termine.")?;
    let operation = Operation::new(
        Some(app),
        if project_type == "modpack" {
            "Instalar modpack"
        } else if project_type == "shader" {
            "Instalar shaderpack"
        } else if project_type == "resourcepack" {
            "Instalar paquete de recursos"
        } else {
            "Instalar mod y dependencias"
        },
    );
    let result = install_version_inner(
        &operation,
        provider,
        project_id,
        file_id,
        project_type,
        pack_id,
        username,
        version_id,
        None,
    )
    .await;
    let success = result.as_ref().cloned().unwrap_or_default();
    operation.complete(result, &success)
}
async fn install_version_inner(
    operation: &Operation,
    provider: String,
    project_id: String,
    file_id: String,
    project_type: String,
    pack_id: Option<String>,
    username: String,
    version_id: String,
    destination: Option<(PathBuf, String, String)>,
) -> Result<String, String> {
    if !["modrinth", "curseforge"].contains(&provider.as_str())
        || !["mod", "modpack", "resourcepack", "shader"].contains(&project_type.as_str())
    {
        return Err("Proveedor o tipo de contenido no válido".into());
    }
    safe_id(&project_id)?;
    safe_id(&file_id)?;
    operation.stage(
        "Consultando archivo seleccionado",
        &format!("{provider} / {project_id} / {file_id}"),
        1.0,
    );
    let client = client()?;
    let first = raw_version(&client, &provider, &project_id, &file_id).await?;
    if project_type == "modpack" {
        let (url, size, hash) = if provider == "modrinth" {
            let file = first["files"]
                .as_array()
                .and_then(|files| {
                    files
                        .iter()
                        .find(|f| f["primary"].as_bool() == Some(true))
                        .or_else(|| files.first())
                })
                .ok_or("El pack no incluye ningún archivo")?;
            (
                text(file, "url"),
                file["size"].as_u64(),
                file.pointer("/hashes/sha1")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            )
        } else {
            (
                cf_download_url(&client, &project_id, &first).await?,
                first["fileLength"].as_u64(),
                first["hashes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|h| h["algo"].as_u64() == Some(1))
                    .and_then(|h| h["value"].as_str())
                    .map(str::to_owned),
            )
        };
        operation.stage(
            "Descargando archivo del modpack",
            &text(
                &first,
                if provider == "curseforge" {
                    "fileName"
                } else {
                    "name"
                },
            ),
            3.0,
        );
        let bytes =
            file_bytes_progress(&client, &url, size, hash.as_deref(), Some(operation)).await?;
        let mut pack = import_bytes(operation, bytes, "Modpack importado".into()).await?;
        pack.source_provider = provider.clone();
        pack.source_project_id = project_id.clone();
        let info = if provider == "modrinth" {
            get_json(&client, &format!("{MR}/project/{project_id}"), &[], false)
                .await
                .ok()
                .map(|value| text(&value, "icon_url"))
        } else {
            get_json(&client, &format!("{CF}/mods/{project_id}"), &[], true)
                .await
                .ok()
                .and_then(|value| {
                    value
                        .pointer("/data/logo/url")
                        .or_else(|| value.pointer("/data/logo/thumbnailUrl"))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
        };
        if let Some(url) = info {
            save_pack_cover(&client, &mut pack, &url).await;
        }
        save_json(&pack_game_dir(&pack.id)?.join("tps-pack.json"), &pack)?;
        return Ok(format!(
            "Modpack {} instalado como instancia aislada",
            pack.name
        ));
    }
    let (dir, game, loader) = if let Some(destination) = destination {
        destination
    } else if let Some(pack) = pack_id.as_deref().filter(|s| !s.is_empty()) {
        ensure_pack_ready(pack)?;
        let metadata = read_pack(pack)?;
        if !crate::content_targets::installed_profile(&crate::root_dir()?, &metadata.version_id) {
            return Err("La versión del modpack no está instalada. Termina o repara la instalación antes de añadir contenido.".into());
        }
        (pack_game_dir(pack)?, metadata.game_version, metadata.loader)
    } else {
        let root = crate::root_dir()?;
        if !crate::content_targets::installed_profile(&root, &version_id) {
            return Err("Elige una versión ya instalada para añadir contenido.".into());
        }
        let (game, loader) = crate::resolve_mod_game_version(&root, &version_id)?;
        (
            crate::version_game_dir(&root, &username, &version_id)?,
            game,
            loader,
        )
    };
    let visual_content = matches!(project_type.as_str(), "resourcepack" | "shader");
    let shader_loader = crate::hud_controls::shader_loader(&dir, &loader);
    let folder = match project_type.as_str() {
        "resourcepack" => "resourcepacks",
        "shader" => "shaderpacks",
        _ => "mods",
    };
    if !visual_content && !["fabric", "forge", "neoforge", "quilt"].contains(&loader.as_str()) {
        return Err(
            "Elige una instalación Forge, Fabric, NeoForge o Quilt para instalar mods".into(),
        );
    }
    fs::create_dir_all(dir.join(folder)).map_err(|e| e.to_string())?;
    let staging = dir
        .join(folder)
        .join(format!(".tps-install-{}", Uuid::new_v4().simple()));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let result = async {
        let mut pending = VecDeque::from([(project_id.clone(), first)]);
        let mut seen = HashMap::<String,String>::new();
        let mut filenames=HashSet::new();
        let mut files = Vec::new();
        while let Some((project, version)) = pending.pop_front() {
            if visual_content {
                let project_info = if provider == "modrinth" {
                    get_json(&client, &format!("{MR}/project/{project}"), &[], false).await?
                } else {
                    get_json(&client, &format!("{CF}/mods/{project}"), &[], true).await?["data"].clone()
                };
                let actual = if provider == "modrinth" { mr_project(&project_info) } else { cf_project(&project_info, "mod") };
                if actual.project_type != project_type {
                    return Err(format!("{} es de tipo {}. Instala esa dependencia desde su catálogo correspondiente.", actual.title, actual.project_type));
                }
                if provider == "modrinth" && version["project_id"].as_str() != Some(project.as_str()) {
                    return Err("El archivo no pertenece al proyecto seleccionado".into());
                }
            }
            let info = if provider == "modrinth" {
                mr_version(&version)
            } else {
                cf_version(&version)
            };
            if let Some(previous)=seen.get(&project){if previous!=&info.id{return Err(format!("Conflicto de dependencias: el proyecto {project} requiere los archivos exactos {previous} y {}. No se modificó la instalación.",info.id));}continue;}
            seen.insert(project.clone(),info.id.clone());
            if seen.len()>256{return Err("La cadena de dependencias supera el límite de seguridad".into());}
            if !crate::content_targets::file_compatible(&project_type, &info.game_versions, &info.loaders, &game, &loader, shader_loader.as_deref()) {
                return Err(format!(
                    "{} no es compatible con Minecraft {game} / {loader}",
                    info.name
                ));
            }
            let (url, filename, size, sha1) = if provider == "modrinth" {
                let file = version["files"]
                    .as_array()
                    .and_then(|files| {
                        files
                            .iter()
                            .find(|f| f["primary"].as_bool() == Some(true))
                            .or_else(|| files.first())
                    })
                    .ok_or("La versión no incluye un archivo")?;
                (
                    text(file, "url"),
                    text(file, "filename"),
                    file["size"].as_u64().unwrap_or(0),
                    file.pointer("/hashes/sha1")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                )
            } else {
                (
                    cf_download_url(&client, &project, &version).await?,
                    text(&version, "fileName"),
                    version["fileLength"].as_u64().unwrap_or(0),
                    version["hashes"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|h| h["algo"].as_u64() == Some(1))
                        .and_then(|h| h["value"].as_str())
                        .unwrap_or_default()
                        .into(),
                )
            };
            safe_file(&filename)?;
            if !filenames.insert(filename.clone()){return Err(format!("Dos proyectos declaran el mismo archivo {filename}; se canceló la instalación"));}
            if visual_content {
                if !filename.to_ascii_lowercase().ends_with(".zip") {
                    return Err("Selecciona un paquete ZIP de Minecraft Java".into());
                }
            } else if !filename.ends_with(".jar") {
                return Err("Este archivo no es un mod JAR de Minecraft Java".into());
            }
            operation.file(&filename, files.len(), files.len() + pending.len() + 1);
            let downloaded = download_content_file(&client,&url,&staging.join(&filename),Some(size),Some(&sha1),None,operation).await?;
            let archive = zip::ZipArchive::new(fs::File::open(staging.join(&filename)).map_err(|e|e.to_string())?)
                .map_err(|_| "El contenido descargado no es un archivo ZIP/JAR válido")?;
            if visual_content {
                let valid = if project_type == "resourcepack" { archive.file_names().any(|name| name == "pack.mcmeta") }
                    else { archive.file_names().any(|name| name.starts_with("shaders/")) };
                if !valid { return Err("El ZIP no contiene la estructura de un paquete de recursos o shaders de Minecraft Java".into()); }
            }
            drop(archive);
            files.push(ContentFile {
                provider: provider.clone(),
                project_id: project.clone(),
                version_id: info.id,
                filename,
                sha1: downloaded.sha1,
                url,
                size: downloaded.size,
            });
            for dependency in info
                .dependencies
                .into_iter()
                .filter(|d| d.kind == "required")
            {
                if let Some(previous)=seen.get(&dependency.id){if !dependency.version_id.is_empty()&&previous!=&dependency.version_id{return Err(format!("Conflicto de dependencias para {}: archivos {} y {}",dependency.id,previous,dependency.version_id));}continue;}
                let next = if !dependency.version_id.is_empty() {
                    raw_version(&client, &provider, &dependency.id, &dependency.version_id).await?
                } else if provider == "modrinth" {
                    let versions = get_json(
                        &client,
                        &format!("{MR}/project/{}/version", dependency.id),
                        &[
                            ("game_versions".into(), json!([game]).to_string()),
                            ("loaders".into(), if visual_content { "[]".into() } else { json!([loader]).to_string() }),
                        ],
                        false,
                    )
                    .await?;
                    versions
                        .as_array()
                        .and_then(|v| v.first())
                        .cloned()
                        .ok_or_else(|| {
                            format!(
                                "No hay una versión compatible para la dependencia {}",
                                dependency.id
                            )
                        })?
                } else {
                    cf_files(&client, &dependency.id, &game, if visual_content { "all" } else { &loader })
                        .await?
                        .into_iter()
                        .next()
                        .ok_or_else(|| {
                            format!(
                                "No hay una versión compatible para la dependencia {}",
                                dependency.id
                            )
                        })?
                };
                let dep_project = if dependency.id.is_empty() {
                    identifier(&next, "project_id")
                } else {
                    dependency.id
                };
                pending.push_back((dep_project, next));
            }
        }
        if visual_content {
            let index = format!("tps-{folder}.json");
            commit_content_files(&dir, &staging, &files, folder, &index)?;
            return Ok(format!("Se instalaron {} archivos en {}. {}", files.len(), dir.join(folder).display(),
                if project_type == "shader" { "Actívalos en el menú de shaders del juego con Iris u OptiFine compatible." }
                else { "Actívalos en Opciones → Paquetes de recursos dentro del juego." }));
        }
        commit_mod_files(&dir,&staging,&files)?;
        Ok(format!(
            "Se instalaron {} archivos con sus dependencias obligatorias: {}",
            files.len(),
            files
                .iter()
                .map(|f| f.filename.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
    .await;

    let has_backups =
        fs::read_dir(staging.join(".backup")).is_ok_and(|mut entries| entries.next().is_some());
    if result.is_ok() || !has_backups {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn decode_archive(encoded: &str, max: usize) -> Result<Vec<u8>, String> {
    if encoded.len() > max * 4 / 3 + 1024 {
        return Err("El archivo supera el tamaño permitido".into());
    }
    let data = encoded
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or(encoded);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())?;
    if bytes.len() > max {
        return Err("El archivo supera el tamaño permitido".into());
    }
    Ok(bytes)
}
fn tree_files(
    dir: &Path,
    prefix: &Path,
    out: &mut Vec<(PathBuf, String)>,
    visited: &mut usize,
) -> Result<(), String> {
    if prefix.components().count() > 12 {
        return Err(
            "La instancia supera los 12 niveles de carpetas permitidos para exportar".into(),
        );
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())?.take(20_001) {
        let entry = entry.map_err(|e| e.to_string())?;
        *visited += 1;
        if *visited > 20_000 {
            return Err("La instancia supera los 20.000 archivos permitidos para exportar. Divide el pack antes de compartirlo".into());
        }
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            continue;
        }
        let relative = prefix.join(entry.file_name());
        if kind.is_dir() {
            tree_files(&entry.path(), &relative, out, visited)?;
        } else if kind.is_file() {
            out.push((entry.path(), relative.to_string_lossy().replace('\\', "/")));
        }
    }
    Ok(())
}

fn dependencies_for_export(game: &str, loader: &str, version: &str) -> Result<Value, String> {
    let mut dependencies = json!({"minecraft":game});
    let key = match loader {
        "fabric" => "fabric-loader",
        "quilt" => "quilt-loader",
        "forge" => "forge",
        "neoforge" => "neoforge",
        "vanilla" | "optifine" => return Ok(dependencies),
        _ => return Err("El cargador no dispone de un formato de exportación verificado".into()),
    };
    if version.trim().is_empty() {
        return Err(format!(
            "No se pudo identificar la versión exacta de {loader} para exportarla sin cambiar el motor"
        ));
    }
    dependencies[key] = json!(if loader == "forge" {
        version.strip_prefix(&format!("{game}-")).unwrap_or(version)
    } else {
        version
    });
    Ok(dependencies)
}

pub(crate) fn export_loader_dependencies(
    root: &Path,
    version_id: &str,
    game: &str,
    loader: &str,
) -> Result<Value, String> {
    if ["vanilla", "optifine"].contains(&loader) {
        return dependencies_for_export(game, loader, "");
    }
    let metadata = root
        .join("versions")
        .join(version_id)
        .join(format!("{version_id}.json"));
    let profile = fs::metadata(&metadata)
        .ok()
        .filter(|m| m.len() <= 4 * 1024 * 1024)
        .and_then(|_| fs::read(metadata).ok())
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    let prefix = match loader {
        "fabric" => "net.fabricmc:fabric-loader:",
        "quilt" => "org.quiltmc:quilt-loader:",
        "forge" => "net.minecraftforge:forge:",
        "neoforge" => "net.neoforged:neoforge:",
        _ => "",
    };
    let declared = profile
        .as_ref()
        .and_then(|v| v["libraries"].as_array())
        .and_then(|libraries| {
            libraries
                .iter()
                .filter_map(|lib| lib["name"].as_str())
                .find_map(|name| name.strip_prefix(prefix))
        })
        .map(str::to_owned);
    let fallback = match loader {
        "fabric" => version_id
            .strip_prefix("fabric-loader-")
            .and_then(|v| v.strip_suffix(&format!("-{game}"))),
        "quilt" => version_id
            .strip_prefix("quilt-loader-")
            .and_then(|v| v.strip_suffix(&format!("-{game}"))),
        "forge" => version_id
            .strip_prefix(&format!("{game}-forge-"))
            .or_else(|| version_id.strip_prefix(&format!("{game}-forge"))),
        "neoforge" => version_id.strip_prefix("neoforge-"),
        _ => None,
    };
    dependencies_for_export(
        game,
        loader,
        declared.as_deref().or(fallback).unwrap_or_default(),
    )
}

#[tauri::command]
pub fn studio_export_pack(
    pack_id: String,
    format: String,
) -> Result<crate::export_policy::ExportResult, String> {
    if !["zip", "mrpack"].contains(&format.as_str()) {
        return Err("Formato de exportación inválido".into());
    }
    let dir = pack_game_dir(&pack_id)?;
    let pack = read_pack(&pack_id)?;
    let exports = crate::root_dir()?.join("exports");
    fs::create_dir_all(&exports).map_err(|e| e.to_string())?;
    let path = exports.join(format!(
        "{}-{}-{}.{}",
        pack.id,
        crate::unix_time_seconds(),
        Uuid::new_v4().simple(),
        format
    ));
    let mut tree = Vec::new();
    let mut visited = 0usize;
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.take(20_001) {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let lower = name.to_ascii_lowercase();

        if lower.starts_with('.')
            || [
                "tps-pack.json",
                "tps-content.json",
                "icon.png",
                "saves",
                "screenshots",
                "logs",
                "crash-reports",
                "backups",
                "session.lock",
                "usercache.json",
                "usernamecache.json",
                "account.dat",
                "curseforge-key.dat",
                "integrations.env",
                "servers.dat",
                "launcher_profiles.json",
            ]
            .contains(&lower.as_str())
        {
            continue;
        }
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            tree_files(&entry.path(), Path::new(&name), &mut tree, &mut visited)?;
        } else if kind.is_file() {
            tree.push((entry.path(), name));
        }
    }
    let mut plan = crate::export_policy::plan(tree, &crate::export_policy::load_records(&dir));
    if pack.loader == "optifine" {
        plan.omitted.push(crate::export_policy::ExportOmission { name:"OptiFine".into(),version:pack.version_id.clone(),filename:format!("versions/{}/{}.json",pack.version_id,pack.version_id),reason:"El motor OptiFine no se redistribuye. El destinatario debe importarlo desde el sitio del autor; el pack conserva la base Vanilla".into(),project_url:Some("https://optifine.net/downloads".into()) });
    }
    let dependencies =
        dependencies_for_export(&pack.game_version, &pack.loader, &pack.loader_version)?;
    let mut manifests = std::collections::BTreeMap::new();
    manifests.insert("tps-pack.json".into(),json!({"id":pack.id,"name":pack.name,"game_version":pack.game_version,"loader":pack.loader,"loader_version":pack.loader_version,"version_id":pack.version_id,"sharing_policy":1}));
    crate::export_policy::write_archive(&path, &format, plan, manifests, dependencies, &pack.name)
}

fn read_zip_json(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, name: &str) -> Option<Value> {
    let file = archive.by_name(name).ok()?;
    if file.size() > 2 * 1024 * 1024 {
        return None;
    }
    serde_json::from_reader(file).ok()
}
fn validate_content_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("El pack contiene una ruta fuera de su instancia".into());
    }
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        let device = name
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let reserved = [
            "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
            "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
        ]
        .contains(&device.as_str());
        if name.contains([':', '\\'])
            || name.chars().any(|c| c.is_control())
            || name.ends_with([' ', '.'])
            || reserved
        {
            return Err("El pack contiene una ruta incompatible con Windows".into());
        }
    }
    let first = path
        .components()
        .next()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_ascii_lowercase();
    if [
        "tps-pack.json",
        "tps-content.json",
        "account.dat",
        "curseforge-key.dat",
        "integrations.env",
        ".env",
    ]
    .contains(&first.as_str())
        || first.starts_with(".tps-")
    {
        return Err("El pack intenta reemplazar metadatos reservados del launcher".into());
    }
    Ok(())
}
fn extract_override_layer(
    archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    dir: &Path,
    prefix: &str,
    expanded: &mut u64,
) -> Result<(), String> {
    let mut names = HashSet::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        if file.is_dir() {
            continue;
        }
        if file.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err("El pack contiene enlaces simbólicos no admitidos".into());
        }
        let path = file
            .enclosed_name()
            .ok_or("El pack contiene rutas fuera de la instancia")?;
        let relative = if prefix.is_empty() {
            if [
                "tps-pack.json",
                "tps-content.json",
                "manifest-tps.json",
                "manifest.json",
                "modrinth.index.json",
            ]
            .contains(&path.to_string_lossy().as_ref())
            {
                continue;
            }
            path.to_path_buf()
        } else if let Ok(path) = path.strip_prefix(prefix) {
            path.to_path_buf()
        } else {
            continue;
        };
        validate_content_path(&relative)?;
        if !names.insert(relative.to_string_lossy().to_ascii_lowercase()) {
            return Err("El pack contiene archivos repetidos en una misma capa".into());
        }
        *expanded = expanded.saturating_add(file.size());
        if *expanded > MAX_PACK_EXPANDED || file.size() > MAX_DOWNLOAD {
            return Err("El contenido del pack supera el límite de 16 GB".into());
        }
        let target = dir.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut output = fs::File::create(target).map_err(|e| e.to_string())?;
        std::io::copy(&mut file, &mut output).map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[cfg(test)]
fn verify_mr_hashes(bytes: &[u8], file: &Value) -> Result<(), String> {
    let sha1 = file
        .pointer("/hashes/sha1")
        .and_then(Value::as_str)
        .ok_or("Falta el hash SHA-1 en el mrpack")?;
    let sha512 = file
        .pointer("/hashes/sha512")
        .and_then(Value::as_str)
        .ok_or("Falta el hash SHA-512 en el mrpack")?;
    if format!("{:x}", Sha1::digest(bytes)) != sha1.to_ascii_lowercase()
        || format!("{:x}", Sha512::digest(bytes)) != sha512.to_ascii_lowercase()
    {
        return Err("El archivo del mrpack no coincide con sus hashes SHA-1 / SHA-512".into());
    }
    Ok(())
}
async fn import_bytes(
    operation: &Operation,
    bytes: Vec<u8>,
    fallback_name: String,
) -> Result<StudioPack, String> {
    let archive_sha1 = format!("{:x}", Sha1::digest(&bytes));
    let cache_dir = crate::root_dir()?.join("cache/modpacks");
    fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    fs::write(cache_dir.join(format!("{archive_sha1}.zip")), &bytes).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("Archivo de modpack inválido: {e}"))?;
    let mr = read_zip_json(&mut archive, "modrinth.index.json");
    let cf = read_zip_json(&mut archive, "manifest.json");
    let tps = read_zip_json(&mut archive, "tps-pack.json");
    let legacy_tps = read_zip_json(&mut archive, "manifest-tps.json");
    let (name, game, loader, loader_version, overrides) = if let Some(index) = &mr {
        if index["formatVersion"].as_u64() != Some(1) || index["game"].as_str() != Some("minecraft")
        {
            return Err("Formato mrpack no compatible".into());
        }
        let dep = &index["dependencies"];
        let (loader, key) = [
            ("fabric", "fabric-loader"),
            ("quilt", "quilt-loader"),
            ("neoforge", "neoforge"),
            ("forge", "forge"),
        ]
        .into_iter()
        .find(|(_, key)| dep[*key].is_string())
        .unwrap_or(("vanilla", "minecraft"));
        (
            index["name"].as_str().unwrap_or(&fallback_name).to_string(),
            text(dep, "minecraft"),
            loader.to_string(),
            text(dep, key),
            "overrides".to_string(),
        )
    } else if let Some(manifest) = &cf {
        if manifest["manifestType"].as_str() != Some("minecraftModpack") {
            return Err("El ZIP no es un modpack de CurseForge".into());
        }
        let engine = manifest
            .pointer("/minecraft/modLoaders")
            .and_then(Value::as_array)
            .and_then(|loaders| {
                loaders
                    .iter()
                    .find(|v| v["primary"].as_bool() == Some(true))
                    .or_else(|| loaders.first())
            })
            .and_then(|v| v["id"].as_str())
            .unwrap_or("vanilla");
        let (loader, version) = engine.split_once('-').unwrap_or((engine, ""));
        (
            manifest["name"]
                .as_str()
                .unwrap_or(&fallback_name)
                .to_string(),
            manifest
                .pointer("/minecraft/version")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .into(),
            loader.into(),
            version.into(),
            manifest["overrides"].as_str().unwrap_or("overrides").into(),
        )
    } else if let Some(manifest) = &tps {
        (
            manifest["name"]
                .as_str()
                .unwrap_or(&fallback_name)
                .to_string(),
            text(manifest, "game_version"),
            text(manifest, "loader"),
            text(manifest, "loader_version"),
            String::new(),
        )
    } else if let Some(manifest) = &legacy_tps {
        if manifest["format"].as_str() != Some("minecrafttps-modpack")
            || manifest["formatVersion"].as_u64() != Some(1)
        {
            return Err("Formato de pack MinecraftTPS anterior no compatible".into());
        }
        (
            fallback_name.clone(),
            text(manifest, "minecraftVersion"),
            text(manifest, "loader"),
            String::new(),
            String::new(),
        )
    } else {
        return Err("El ZIP no contiene un manifiesto MinecraftTPS, Modrinth o CurseForge".into());
    };
    safe_id(&game)?;
    let loader_version = if loader == "forge" && !loader_version.starts_with(&format!("{game}-")) {
        format!("{game}-{loader_version}")
    } else {
        loader_version
    };
    let mut icon = tps
        .as_ref()
        .and_then(|manifest| manifest["icon_data"].as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_owned);
    if icon.is_none() {
        icon = jar_entry(&mut archive, "icon.png", 2 * 1024 * 1024)
            .and_then(|bytes| icon_png(&bytes, 128))
            .map(|bytes| image_data(&bytes));
    }
    let pending = studio_list_packs()?
        .into_iter()
        .find(|p| p.archive_sha1 == archive_sha1 && p.install_state != "ready");
    let mut pack = if let Some(pack) = pending {
        operation.stage("Reanudando instalación", &pack.name, 15.0);
        pack
    } else {
        create_pack_inner(operation, name, game, loader, loader_version, icon).await?
    };
    pack.archive_sha1 = archive_sha1.clone();
    pack.install_state = "installing".into();
    pack.install_error.clear();
    let dir = pack_game_dir(&pack.id)?;
    save_json(&dir.join("tps-pack.json"), &pack)?;
    let client = client()?;
    pack.missing_files.clear();
    operation.stage(
        "Consultando archivos del modpack",
        "Verificando versiones, permisos e integridad de los archivos",
        18.0,
    );
    let result: Result<(), String> = async {
        let mut expanded = 0u64;
        let mut downloaded_names = HashSet::new();
        let total = mr
            .as_ref()
            .and_then(|v| v["files"].as_array())
            .map(|files| {
                files
                    .iter()
                    .filter(|f| {
                        f.pointer("/env/client").and_then(Value::as_str) != Some("unsupported")
                    })
                    .count()
            })
            .or_else(|| {
                cf.as_ref()
                    .and_then(|v| v["files"].as_array())
                    .map(|files| {
                        files
                            .iter()
                            .filter(|f| f["required"].as_bool() != Some(false))
                            .count()
                    })
            })
            .unwrap_or(0);
        let mut installed = 0usize;
        if let Some(index) = &mr {
            for file in index["files"].as_array().into_iter().flatten() {
                if file.pointer("/env/client").and_then(Value::as_str) == Some("unsupported") {
                    continue;
                }
                let relative = Path::new(
                    file["path"]
                        .as_str()
                        .ok_or("El mrpack contiene un archivo sin ruta")?,
                );
                validate_content_path(relative)?;
                if !downloaded_names.insert(relative.to_string_lossy().to_ascii_lowercase()) {
                    return Err("El mrpack repite una ruta de descarga".into());
                }
                let urls = strings(&file["downloads"]);
                let url = urls
                    .iter()
                    .find(|u| {
                        url::Url::parse(u).is_ok_and(|u| {
                            u.scheme() == "https"
                                && u.host_str().is_some_and(|host| {
                                    host == "cdn.modrinth.com"
                                        || host == "edge.forgecdn.net"
                                        || host == "mediafilez.forgecdn.net"
                                        || host == "github.com"
                                        || host == "raw.githubusercontent.com"
                                        || host == "gitlab.com"
                                })
                        })
                    })
                    .ok_or("El mrpack contiene un servidor de descarga no permitido")?;
                operation.file(&relative.to_string_lossy(), installed, total);
                let sha1=file.pointer("/hashes/sha1").and_then(Value::as_str).ok_or("Falta el hash SHA-1 en el mrpack")?;
                let sha512=file.pointer("/hashes/sha512").and_then(Value::as_str).ok_or("Falta el hash SHA-512 en el mrpack")?;
                let downloaded=download_content_file(&client,url,&dir.join(relative),file["fileSize"].as_u64(),Some(sha1),Some(sha512),operation).await?;
                expanded += downloaded.size;
                if expanded > MAX_PACK_EXPANDED { return Err("El contenido del pack supera el límite de 16 GB".into()); }
                installed += 1;
            }
        }
        if let Some(manifest) = &cf {
            let metadata = cf_pack_metadata(manifest).await?;
            for file in manifest["files"].as_array().into_iter().flatten() {
                if file["required"].as_bool() == Some(false) {
                    continue;
                }
                let project = identifier(file, "projectID");
                let id = identifier(file, "fileID");
                operation.file(
                    &format!("CurseForge · proyecto {project} / archivo {id}"),
                    installed,
                    total,
                );
                let mut value = metadata.get(&id).cloned().ok_or("No se encontró la información del archivo del manifiesto")?;
                let filename = text(&value, "fileName");
                safe_file(&filename)?;
                if !filename.to_ascii_lowercase().ends_with(".jar") {
                    let project_info =
                        get_json(&client, &format!("{CF}/mods/{project}"), &[], true).await?;
                    value["classId"] = project_info["data"]["classId"].clone();
                }
                let hash = value["hashes"].as_array().into_iter().flatten().find(|h| h["algo"].as_u64() == Some(1)).and_then(|h| h["value"].as_str()).unwrap_or_default();
                let target_relative = curseforge_content_path(&value, &filename)?;
                let target = dir.join(&target_relative);
                if !downloaded_names.insert(target_relative.to_string_lossy().to_ascii_lowercase()) { return Err(format!("El pack repite el archivo {filename}")); }
                let existing = verified_content_file(&target,value["fileLength"].as_u64(),Some(hash),None);
                let cache = crate::root_dir()?.join("cache/content").join(hash.to_ascii_lowercase());
                let cached = if hash.len() == 40 { verified_content_file(&cache,value["fileLength"].as_u64(),Some(hash),None) } else { None };
                operation.file(&filename, installed, total);
                let downloaded = if let Some(info) = existing { info } else if let Some(info) = cached { if let Some(parent)=target.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;} fs::copy(&cache,&target).map_err(|e|e.to_string())?; info } else if let Some(url) = value["downloadUrl"].as_str().filter(|url| !url.is_empty()) {
                    download_content_file(&client,url,&target,value["fileLength"].as_u64(),Some(hash),None,operation).await
                        .map_err(|e| format!("No se pudo descargar {filename}: {e}"))?
                } else {
                    let info = get_json(&client, &format!("{CF}/mods/{project}"), &[], true).await.ok();
                    let website = info.as_ref().and_then(|v| v.pointer("/data/links/websiteUrl")).and_then(Value::as_str)
                        .filter(|u| url::Url::parse(u).is_ok_and(|u| u.scheme() == "https" && u.host_str() == Some("www.curseforge.com")))
                        .map(|u| format!("{}/files/{id}", u.trim_end_matches('/')))
                        .unwrap_or_else(|| format!("https://www.curseforge.com/projects/{project}/files/{id}"));
                    pack.missing_files.push(MissingPackFile { filename: filename.clone(), project_id: project, file_id: id,
                        sha1: hash.into(), size: value["fileLength"].as_u64().unwrap_or(0), website_url: website });
                    operation.log(&format!("{filename}: el autor requiere la descarga desde su página oficial de CurseForge."));
                    installed += 1;
                    continue;
                };
                expanded += downloaded.size;
                if expanded > MAX_PACK_EXPANDED {
                    return Err("El contenido del pack supera el límite de 16 GB".into());
                }
                installed += 1;
            }
        }
        operation.file("Verificación de archivos completada", installed.saturating_sub(pack.missing_files.len()), total);
        operation.stage(
            "Aplicando configuración del modpack",
            "Configuración, recursos y scripts del autor",
            93.0,
        );
        extract_override_layer(&mut archive, &dir, &overrides, &mut expanded)?;
        if mr.is_some() {
            extract_override_layer(&mut archive, &dir, "client-overrides", &mut expanded)?;
        }
        if !pack.missing_files.is_empty() {
            return Err(format!("Faltan {} archivos cuya descarga externa está restringida por sus autores. Descárgalos desde los enlaces oficiales de Mis modpacks e impórtalos para terminar la instalación.", pack.missing_files.len()));
        }
        Ok(())
    }
    .await;
    if let Err(error) = result {
        pack.install_state = "incomplete".into();
        pack.install_error = error.clone();
        save_json(&dir.join("tps-pack.json"), &pack)?;
        return Err(format!(
            "{error}\nLa instancia quedó pendiente. Usa Reintentar instalación; se conservarán las descargas verificadas."
        ));
    }
    if let Some(url) = tps
        .as_ref()
        .and_then(|manifest| manifest["icon_url"].as_str())
    {
        save_pack_cover(&client, &mut pack, url).await;
    }
    if pack.icon_data.is_empty() && mr.is_some() {


        let cover = async {
            let version = get_json(
                &client,
                &format!("{MR}/version_file/{archive_sha1}"),
                &[("algorithm".into(), "sha1".into())],
                false,
            )
            .await
            .ok()?;
            let project = version["project_id"].as_str()?;
            safe_id(project).ok()?;
            let project = get_json(&client, &format!("{MR}/project/{project}"), &[], false)
                .await
                .ok()?;
            Some(text(&project, "icon_url"))
        };
        if let Ok(Some(url)) = tokio::time::timeout(Duration::from_secs(10), cover).await {
            save_pack_cover(&client, &mut pack, &url).await;
        }
    }
    pack.install_state = ready_state();
    pack.install_error.clear();
    save_json(&dir.join("tps-pack.json"), &pack)?;
    Ok(pack)
}
#[tauri::command]
pub async fn studio_import_pack(
    app: AppHandle,
    archive_base64: String,
    file_name: String,
) -> Result<StudioPack, String> {
    let bytes = decode_archive(&archive_base64, MAX_DOWNLOAD as usize)?;
    let fallback = Path::new(&file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Pack importado")
        .to_string();
    let _guard = INSTALL_LOCK
        .try_lock()
        .map_err(|_| "Ya hay una instalación en curso. Espera a que termine.")?;
    let operation = Operation::new(Some(app), "Importar modpack");
    let result = import_bytes(&operation, bytes, fallback).await;
    let name = result.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    operation.complete(result, &format!("{name} se instaló correctamente."))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn streamed_cache_and_manual_files_reject_partial_or_wrong_versions() {
        let scratch = Scratch::new();
        let bytes = b"complete jar fixture";
        let hash = format!("{:x}", Sha1::digest(bytes));
        let sha512 = format!("{:x}", Sha512::digest(bytes));
        let file = scratch.0.join("mod.jar");
        fs::write(&file, bytes).unwrap();
        assert!(
            verified_content_file(&file, Some(bytes.len() as u64), Some(&hash), Some(&sha512))
                .is_some()
        );
        assert!(verified_content_file(&file, Some(1), Some(&hash), Some(&sha512)).is_none());
        assert!(verified_content_file(&file, None, Some("bad"), Some(&sha512)).is_none());
        assert!(verified_content_file(&file, None, Some(&hash), Some("bad")).is_none());
        let pack: StudioPack = serde_json::from_value(json!({"id":"test","name":"Test","game_version":"1.12.2","loader":"forge","loader_version":"x","version_id":"x","icon_data":"","created_at":0,
            "missing_files":[{"filename":"mod.jar","project_id":"1","file_id":"2","size":bytes.len(),"sha1":hash,"website_url":"https://www.curseforge.com/"}]})).unwrap();
        assert_eq!(pack.install_state, "ready");
        assert!(supplied_pack_file_at(&scratch.0, &pack, "other.jar", bytes).is_err());
        assert!(supplied_pack_file_at(&scratch.0, &pack, "mod.jar", b"incomplete").is_err());
        supplied_pack_file_at(&scratch.0, &pack, "mod.jar", bytes).unwrap();
        assert_eq!(fs::read(scratch.0.join(&hash)).unwrap(), bytes);
        let metadata = scratch.0.join("tps-pack.json");
        save_json(&metadata, &json!({"install_state":"installing"})).unwrap();
        save_json(&metadata, &json!({"install_state":"incomplete"})).unwrap();
        let result: Value = serde_json::from_slice(&fs::read(metadata).unwrap()).unwrap();
        assert_eq!(result["install_state"], "incomplete");
        assert!(
            !fs::read_dir(&scratch.0)
                .unwrap()
                .filter_map(Result::ok)
                .any(|e| e.path().extension().is_some_and(|ext| ext == "pending"))
        );
    }
    #[tokio::test]
    #[ignore = "Descarga por streaming un archivo oficial de Modrinth y verifica tamaño y SHA-512"]
    async fn live_content_stream_verifies_hashes_and_reuses_cache() {
        let scratch = Scratch::new();
        let client = client().unwrap();
        let versions = get_json(
            &client,
            &format!("{MR}/project/ferrite-core/version"),
            &[
                ("game_versions".into(), json!(["1.21.1"]).to_string()),
                ("loaders".into(), json!(["neoforge"]).to_string()),
            ],
            false,
        )
        .await
        .unwrap();
        let file = &versions[0]["files"][0];
        let size = file["size"].as_u64();
        let source = format!(
            "{}?launchertps-check={}",
            file["url"].as_str().unwrap(),
            Uuid::new_v4()
        );
        let op = Operation::new(None, "Prueba de streaming");
        let target = scratch.0.join("download.jar");
        let first = download_content_file(
            &client,
            &source,
            &target,
            size,
            None,
            file.pointer("/hashes/sha512").and_then(Value::as_str),
            &op,
        )
        .await
        .unwrap();
        assert_eq!(first.sha1, file["hashes"]["sha1"].as_str().unwrap());
        fs::write(&target, b"partial").unwrap();
        let second = download_content_file(
            &client,
            &source,
            &target,
            size,
            None,
            file.pointer("/hashes/sha512").and_then(Value::as_str),
            &op,
        )
        .await
        .unwrap();
        assert_eq!(first.sha1, second.sha1);
        assert_eq!(first.size, second.size);
        println!(
            "Streaming y caché: {} bytes, SHA-1 y SHA-512 correctos",
            second.size
        );
    }
    #[tokio::test]
    #[ignore = "Descarga RLCraft real y comprueba sus 187 archivos y las restricciones de autores"]
    async fn rlcraft_real_pack_installation() {
        let client = client().unwrap();
        let file = raw_version(&client, "curseforge", "285109", "4612979")
            .await
            .unwrap();
        let url = cf_download_url(&client, "285109", &file).await.unwrap();
        let hash = file["hashes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["algo"] == 1)
            .unwrap()["value"]
            .as_str();
        let op = Operation::new(None, "Prueba real de RLCraft");
        let bytes =
            file_bytes_progress(&client, &url, file["fileLength"].as_u64(), hash, Some(&op))
                .await
                .unwrap();
        let archive_hash = format!("{:x}", Sha1::digest(&bytes));
        let result = import_bytes(&op, bytes, "RLCraft".into()).await;
        let pack = studio_list_packs()
            .unwrap()
            .into_iter()
            .find(|p| p.archive_sha1 == archive_hash)
            .unwrap();
        assert_eq!(pack.game_version, "1.12.2");
        assert!(pack.loader_version.ends_with("14.23.5.2860"));
        let dir = pack_game_dir(&pack.id).unwrap();
        assert!(dir.join("config").is_dir());
        if result.is_ok() {
            assert_eq!(pack.install_state, "ready");
            ensure_pack_ready(&pack.id).unwrap();
        } else {
            assert_eq!(pack.install_state, "incomplete");
            assert_eq!(pack.missing_files.len(), 2, "{}", pack.install_error);
            assert!(
                pack.missing_files
                    .iter()
                    .any(|f| f.filename == "SRParasites-1.12.2v1.9.11.jar")
            );
            assert!(
                pack.missing_files
                    .iter()
                    .any(|f| f.filename == "RTG-1.12.2-6.1.0.0-snapshot.1.jar")
            );
            assert!(ensure_pack_ready(&pack.id).is_err());
            assert!(result.err().unwrap().contains("restringida"));
        }
        let archive = fs::read(
            crate::root_dir()
                .unwrap()
                .join("cache/modpacks")
                .join(format!("{archive_hash}.zip")),
        )
        .unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(archive)).unwrap();
        let manifest = read_zip_json(&mut zip, "manifest.json").unwrap();
        let metadata = cf_pack_metadata(&manifest).await.unwrap();
        let mut verified = 0;
        for file in manifest["files"].as_array().unwrap() {
            let value = &metadata[&identifier(file, "fileID")];
            let filename = text(value, "fileName");
            if pack.missing_files.iter().any(|f| f.filename == filename) {
                continue;
            }
            let target = if filename.ends_with(".jar") {
                dir.join("mods").join(&filename)
            } else {
                dir.join("resourcepacks").join(&filename)
            };
            let bytes = fs::read(&target).unwrap_or_else(|e| panic!("{}: {e}", target.display()));
            let hash = value["hashes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|h| h["algo"] == 1)
                .unwrap()["value"]
                .as_str();
            verify_download(&bytes, value["fileLength"].as_u64(), hash).unwrap();
            verified += 1;
        }
        assert_eq!(verified + pack.missing_files.len(), 187);
        println!(
            "RLCraft {}: {verified}/187 archivos verificados, {} descargas manuales del autor; Forge exacto {}",
            pack.id,
            pack.missing_files.len(),
            pack.version_id
        );
    }
    #[tokio::test]
    #[ignore = "Instala un modpack Fabric real de Modrinth y conserva la instancia de prueba"]
    async fn modrinth_real_pack_installation() {
        let client = client().unwrap();
        let versions = get_json(
            &client,
            &format!("{MR}/project/fabulously-optimized/version"),
            &[("game_versions".into(), json!(["1.21.1"]).to_string())],
            false,
        )
        .await
        .unwrap();
        let version = &versions.as_array().unwrap()[0];
        let file = version["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["primary"].as_bool() == Some(true))
            .unwrap();
        let op = Operation::new(None, "Prueba Modrinth Fabric");
        let bytes = file_bytes_progress(
            &client,
            file["url"].as_str().unwrap(),
            file["size"].as_u64(),
            file.pointer("/hashes/sha1").and_then(Value::as_str),
            Some(&op),
        )
        .await
        .unwrap();
        let pack = import_bytes(&op, bytes, "Prueba Fabric".into())
            .await
            .unwrap();
        assert_eq!(pack.install_state, "ready");
        assert_eq!(pack.loader, "fabric");
        assert_eq!(pack.game_version, "1.21.1");
        ensure_pack_ready(&pack.id).unwrap();
        assert!(
            fs::read_dir(pack_game_dir(&pack.id).unwrap().join("mods"))
                .unwrap()
                .count()
                > 20
        );
        println!("Fabric validado: {} · {}", pack.id, pack.version_id);
    }
    #[tokio::test]
    #[ignore = "Instala NeoForge 21.1.248 y FerriteCore oficial en una instancia de prueba"]
    async fn neoforge_real_pack_installation() {
        let operation = Operation::new(None, "Prueba NeoForge");
        let pack = create_pack_inner(
            &operation,
            "Prueba NeoForge".into(),
            "1.21.1".into(),
            "neoforge".into(),
            "21.1.248".into(),
            None,
        )
        .await
        .unwrap();
        let client = client().unwrap();
        let versions = get_json(
            &client,
            &format!("{MR}/project/ferrite-core/version"),
            &[
                ("game_versions".into(), json!(["1.21.1"]).to_string()),
                ("loaders".into(), json!(["neoforge"]).to_string()),
            ],
            false,
        )
        .await
        .unwrap();
        let version = &versions.as_array().unwrap()[0];
        let result = install_version_inner(
            &operation,
            "modrinth".into(),
            "ferrite-core".into(),
            text(version, "id"),
            "mod".into(),
            Some(pack.id.clone()),
            "TPSNeoTest".into(),
            pack.version_id.clone(),
            None,
        )
        .await
        .unwrap();
        assert!(result.contains("Se instalaron"));
        assert_eq!(pack.install_state, "ready");
        println!("NeoForge preparado: {} · {}", pack.id, pack.version_id);
    }
    #[test]
    fn content_types_and_integrity_are_checked_before_installation() {
        assert_eq!(
            curseforge_content_path(&json!({}), "example.jar").unwrap(),
            Path::new("mods/example.jar")
        );
        assert_eq!(
            curseforge_content_path(&json!({"classId":12}), "resources.zip").unwrap(),
            Path::new("resourcepacks/resources.zip")
        );
        assert_eq!(
            curseforge_content_path(&json!({"classId":6552}), "shaders.zip").unwrap(),
            Path::new("shaderpacks/shaders.zip")
        );
        assert!(curseforge_content_path(&json!({}), "../bad.jar").is_err());
        assert!(curseforge_content_path(&json!({}), "unknown.zip").is_err());
        let hash = format!("{:x}", Sha1::digest(b"complete"));
        assert!(verify_download(b"complete", Some(8), Some(&hash)).is_ok());
        assert!(verify_download(b"partial", Some(8), Some(&hash)).is_err());
        assert!(verify_download(b"modified", Some(8), Some(&hash)).is_err());
    }
    #[tokio::test]
    #[ignore = "Requiere acceso a la API real de CurseForge"]
    async fn curseforge_live_search_loads_mods_and_modpacks() {
        for kind in ["mod", "modpack"] {
            for provider in ["curseforge", "both"] {
                let result = studio_search(
                    provider.into(),
                    "".into(),
                    kind.into(),
                    "all".into(),
                    "all".into(),
                    "downloads".into(),
                    1,
                    None,
                )
                .await
                .unwrap_or_else(|error| {
                    panic!("La búsqueda real de {provider} para {kind} falló: {error}")
                });
                assert!(
                    result.total_hits > 0,
                    "No se publicaron resultados de {kind} para {provider}"
                );
                assert!(
                    result
                        .projects
                        .iter()
                        .any(|project| project.provider == "curseforge"),
                    "No se recibieron proyectos CurseForge de {kind} en {provider}: {:?}",
                    result.warnings
                );
                println!(
                    "{provider} / {kind}: {} resultados recibidos, catálogo de {} proyectos",
                    result.projects.len(),
                    result.total_hits
                );
            }
        }
    }
    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "minecrafttps-studio-test-{}",
                Uuid::new_v4().simple()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture_jar(path: &Path, entries: &[(&str, &[u8])]) {
        let mut archive = zip::ZipWriter::new(fs::File::create(path).unwrap());
        for (name, bytes) in entries {
            archive
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap();
    }
    #[test]
    fn jar_artwork_handles_author_paths_multiple_entries_and_conventional_fallbacks() {
        let scratch = Scratch::new();
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(16, 16)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        for (metadata, icon_path) in [
            (
                r#"[{"name":"Primary"},{"name":"Library","logoFile":"/assets/example/ICON.PNG"}]"#,
                "assets/example/icon.png",
            ),
            (
                r#"[{"name":"Example","logoFile":"./assets/example/icon.png"}]"#,
                "assets/example/icon.png",
            ),
            (r#"[{"name":"Example"}]"#, "logo.png"),
        ] {
            let path = scratch.0.join(format!("{}.jar", uuid::Uuid::new_v4()));
            fixture_jar(
                &path,
                &[
                    ("mcmod.info", metadata.as_bytes()),
                    (icon_path, png.get_ref()),
                ],
            );
            assert!(!jar_metadata_uncached(&path).icon_data.is_empty());
        }
        assert!(mod_icon_path("/../outside.png").is_none());
        assert!(mod_icon_path("https://example.com/icon.png").is_none());
        assert!(mod_icon_path("assets\\..\\outside.png").is_none());
        assert_eq!(
            json_icon(&json!({"16":"small.png","128":"large.png"})),
            "large.png"
        );
        assert_eq!(
            cf_logo(
                &json!({"logo":{"thumbnailUrl":"","url":"https://media.forgecdn.net/icon.png"}})
            ),
            Some("https://media.forgecdn.net/icon.png")
        );
    }
    #[test]
    #[ignore = "Lectura de una instancia local real; requiere TP_ARTWORK_AUDIT_PACK"]
    fn local_artwork_audit() {
        let id = std::env::var("TP_ARTWORK_AUDIT_PACK")
            .expect("Indica la instancia con TP_ARTWORK_AUDIT_PACK");
        safe_id(&id).unwrap();
        let dir = crate::root_dir()
            .unwrap()
            .join("instances")
            .join(id)
            .join("mods");
        let mut total = 0;
        let mut internal = 0;
        let mut declared = 0;
        for entry in fs::read_dir(dir).unwrap().filter_map(Result::ok) {
            if !entry.file_type().is_ok_and(|t| t.is_file())
                || !entry.file_name().to_string_lossy().ends_with(".jar")
            {
                continue;
            }
            total += 1;
            let metadata = jar_metadata_uncached(&entry.path());
            if !metadata.loader.is_empty() {
                declared += 1;
            }
            if !metadata.icon_data.is_empty() {
                internal += 1;
            }
        }
        assert!(total > 0);
        println!(
            "JAR reales revisados: {total}; metadatos reconocidos: {declared}; iconos internos válidos: {internal}; fallback necesario: {}",
            total - internal
        );
    }
    #[test]
    fn artwork_lookup_requires_exact_titles_and_approved_cdn() {
        let rows = vec![
            mr_project(
                &json!({"title":"RLCraft Dregora","icon_url":"https://cdn.modrinth.com/dregora.png"}),
            ),
            mr_project(
                &json!({"title":"RLCraft","icon_url":"https://cdn.modrinth.com/rlcraft.png"}),
            ),
            mr_project(
                &json!({"title":"RLCraft Dregora","icon_url":"https://example.com/wrong.png"}),
            ),
        ];
        let matches = exact_art_candidates(rows, &art_name("RLCraft Dregora"));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].title, "RLCraft Dregora");
        assert!(official_cover_url("https://user@cdn.modrinth.com/icon.png").is_none());
    }
    #[test]
    fn jar_metadata_reads_names_authors_and_icons_for_supported_loaders() {
        let scratch = Scratch::new();
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(32, 32)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        for (entry, data, expected_loader) in [
            (
                "fabric.mod.json",
                r#"{"schemaVersion":1,"id":"example","name":"Example Mod","version":"1.2.3","authors":[{"name":"Ada"}],"description":"A useful mod","icon":"assets/example/icon.png"}"#,
                "fabric",
            ),
            (
                "quilt.mod.json",
                r#"{"quilt_loader":{"id":"example","version":"1.2.3","metadata":{"name":"Example Mod","contributors":{"Ada":"Author"},"description":"A useful mod","icon":"assets/example/icon.png"}}}"#,
                "quilt",
            ),
            (
                "mcmod.info",
                r#"[{"modid":"example","name":"Example Mod","version":"1.2.3","authorList":["Ada"],"description":"A useful mod","logoFile":"assets/example/icon.png"}]"#,
                "forge",
            ),
            (
                "META-INF/mods.toml",
                "[[mods]]\nmodId='example'\ndisplayName='Example Mod'\nversion='1.2.3'\nauthors='Ada'\ndescription='A useful mod'\nlogoFile='assets/example/icon.png'\n",
                "forge",
            ),
            (
                "META-INF/neoforge.mods.toml",
                "[[mods]]\nmodId='example'\ndisplayName='Example Mod'\nversion='1.2.3'\nauthors='Ada'\ndescription='A useful mod'\nlogoFile='assets/example/icon.png'\n",
                "neoforge",
            ),
        ] {
            let path = scratch
                .0
                .join(format!("{expected_loader}-{}.jar", entry.replace('/', "-")));
            fixture_jar(
                &path,
                &[
                    (entry, data.as_bytes()),
                    ("assets/example/icon.png", png.get_ref()),
                ],
            );
            let result = jar_metadata_uncached(&path);
            assert_eq!(result.name, "Example Mod", "{entry}");
            assert_eq!(result.version, "1.2.3", "{entry}");
            assert_eq!(result.authors, ["Ada"], "{entry}");
            assert_eq!(result.description, "A useful mod", "{entry}");
            assert_eq!(result.loader, expected_loader, "{entry}");
            assert!(
                result.icon_data.starts_with("data:image/png;base64,"),
                "{entry}"
            );
        }
    }
    #[test]
    fn jar_metadata_resolves_manifest_versions_and_rejects_unsafe_or_oversize_entries() {
        let scratch = Scratch::new();
        let path = scratch.0.join("example.jar.disabled");
        let manifest = b"Manifest-Version: 1.0\r\nImplementation-Version: 4.5.6\r\n";
        let oversized = vec![b'x'; 1024 * 1024 + 1];
        fixture_jar(&path, &[
            ("META-INF/mods.toml", b"[[mods]]\nmodId='example'\nversion='${file.jarVersion}'\nlogoFile='../outside.png'\n"),
            ("META-INF/MANIFEST.MF", manifest),
            ("../outside.png", b"not an image"),
            ("too-large.json", &oversized),
        ]);
        let result = jar_metadata_uncached(&path);
        assert_eq!(result.name, "example");
        assert_eq!(result.version, "4.5.6");
        assert!(result.icon_data.is_empty());
        let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        assert!(jar_entry(&mut archive, "../outside.png", 1024).is_none());
        assert!(jar_entry(&mut archive, "too-large.json", 1024 * 1024).is_none());
        assert!(official_cover_url("https://example.com/icon.png").is_none());
        assert!(official_cover_url("http://cdn.modrinth.com/icon.png").is_none());
        assert!(official_cover_url("https://cdn.modrinth.com/data/example/icon.png").is_some());
    }
    fn record(name: &str, version: &str) -> ContentFile {
        ContentFile {
            provider: "modrinth".into(),
            project_id: "project".into(),
            version_id: version.into(),
            filename: name.into(),
            sha1: String::new(),
            url: String::new(),
            size: 3,
        }
    }
    #[test]
    fn pack_paths_confine_files_and_preserve_scripts() {
        for path in [
            "../outside.jar",
            "C:/outside.jar",
            "/outside.jar",
            "config/../../outside.jar",
            "tps-pack.json",
            "TPS-CONTENT.JSON",
            "config/file:stream",
            "config/CON",
            "config/trailing. ",
        ] {
            assert!(validate_content_path(Path::new(path)).is_err(), "{path}");
        }
        for path in [
            "mods/test.jar",
            "kubejs/server_scripts/main.js",
            "defaultconfigs/example.toml",
            "scripts/crafting.zs",
            "patchouli_books/test/book.json",
        ] {
            assert!(validate_content_path(Path::new(path)).is_ok(), "{path}");
        }
    }
    #[test]
    fn client_overrides_win_independently_of_zip_entry_order() {
        let scratch = Scratch::new();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, bytes) in [
            ("client-overrides/config/settings.txt", b"client".as_slice()),
            ("overrides/config/settings.txt", b"common".as_slice()),
            (
                "overrides/kubejs/server_scripts/main.js",
                b"script".as_slice(),
            ),
        ] {
            writer.start_file(name, options).unwrap();
            writer.write_all(bytes).unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut expanded = 0;
        extract_override_layer(&mut archive, &scratch.0, "overrides", &mut expanded).unwrap();
        extract_override_layer(&mut archive, &scratch.0, "client-overrides", &mut expanded)
            .unwrap();
        assert_eq!(
            fs::read(scratch.0.join("config/settings.txt")).unwrap(),
            b"client"
        );
        assert_eq!(
            fs::read(scratch.0.join("kubejs/server_scripts/main.js")).unwrap(),
            b"script"
        );
    }
    #[test]
    fn failed_multi_mod_commit_restores_original_mod_and_index() {
        let scratch = Scratch::new();
        let mods = scratch.0.join("mods");
        fs::create_dir(&mods).unwrap();
        fs::write(mods.join("old.jar"), b"old").unwrap();
        let old = vec![record("old.jar", "old")];
        save_json(&scratch.0.join("tps-content.json"), &old).unwrap();
        let staging = mods.join(".tps-stage-test");
        fs::create_dir(&staging).unwrap();
        fs::write(staging.join("new.jar"), b"new").unwrap();
        let files = vec![
            record("new.jar", "new"),
            ContentFile {
                project_id: "missing".into(),
                ..record("missing.jar", "missing")
            },
        ];
        assert!(commit_mod_files(&scratch.0, &staging, &files).is_err());
        assert_eq!(fs::read(mods.join("old.jar")).unwrap(), b"old");
        assert!(!mods.join("new.jar").exists());
        assert_eq!(content(&scratch.0)[0].version_id, "old");
    }
    #[test]
    fn exact_forge_revision_never_accepts_prefix_match() {
        let scratch = Scratch::new();
        let id = "1.20.1-forge-47.2.10";
        let folder = scratch.0.join("versions").join(id);
        fs::create_dir_all(&folder).unwrap();
        save_json(&folder.join(format!("{id}.json")),&json!({"inheritsFrom":"1.20.1","libraries":[{"name":"net.minecraftforge:forge:1.20.1-47.2.10:client"}]})).unwrap();
        assert!(loader_profile_matches(
            &scratch.0,
            id,
            "1.20.1",
            "forge",
            "1.20.1-47.2.10"
        ));
        assert!(!loader_profile_matches(
            &scratch.0,
            id,
            "1.20.1",
            "forge",
            "1.20.1-47.2.1"
        ));
    }
    #[test]
    fn neoforge_catalog_matches_exact_calver_releases_and_snapshot_metadata() {
        for (game, version) in [
            ("26.3", "26.3.0.95"),
            ("26.1", "26.1.0.10-beta"),
            ("26.1.2", "26.1.2.5"),
            ("26.1-snapshot-1", "26.1.0.0-alpha.2+snapshot-1"),
            ("26.1-pre-1", "26.1.0.0-alpha.3+pre-1"),
            ("1.21.1", "21.1.172"),
            ("1.21", "21.0.167"),
            ("1.20.1", "1.20.1-47.1.106"),
        ] {
            assert!(neoforge_version_matches(game, version), "{game}: {version}");
        }
        for (game, version) in [
            ("26.3", "3.0.95"),
            ("26.3", "26.3.1.95"),
            ("26.1", "26.1.0.0-alpha.2+snapshot-1"),
            ("26.1-snapshot-1", "26.1.0.0-alpha.2+snapshot-2"),
            ("26.1.2", "26.1.0.5"),
            ("1.21.1", "21.11.172"),
            ("1.20.1", "1.20.10-47.1.106"),
            ("24w01a", "26.1.0.0-alpha.2+snapshot-1"),
        ] {
            assert!(
                !neoforge_version_matches(game, version),
                "{game}: {version}"
            );
        }
        assert!(safe_id("neoforge-26.1.0.0-alpha.2+snapshot-1").is_ok());
        assert!(safe_id("../../neoforge+snapshot-1").is_err());
        let scratch = Scratch::new();
        let id = "1.20.1-forge-47.1.106";
        let folder = scratch.0.join("versions").join(id);
        fs::create_dir_all(&folder).unwrap();
        save_json(&folder.join(format!("{id}.json")),&json!({"inheritsFrom":"1.20.1","libraries":[{"name":"net.neoforged.fancymodloader:loader:47.2.2"}]})).unwrap();
        assert!(loader_profile_matches(
            &scratch.0,
            id,
            "1.20.1",
            "neoforge",
            "1.20.1-47.1.106"
        ));
        assert!(!loader_profile_matches(
            &scratch.0,
            id,
            "1.20.1",
            "neoforge",
            "1.20.1-47.1.10"
        ));
    }
    #[test]
    fn mrpack_checks_both_required_hashes() {
        let file = json!({"hashes":{"sha1":format!("{:x}",Sha1::digest(b"verified")),"sha512":format!("{:x}",Sha512::digest(b"verified"))}});
        assert!(verify_mr_hashes(b"verified", &file).is_ok());
        assert!(verify_mr_hashes(b"changed", &file).is_err());
        let mut missing = file;
        missing["hashes"].as_object_mut().unwrap().remove("sha512");
        assert!(verify_mr_hashes(b"verified", &missing).is_err());
    }
}
