
use base64::Engine;
use serde::Serialize;
use sha1::{Digest, Sha1};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
    path::Path,
};
#[path = "poser_item_models.rs"]
mod item_models;
#[derive(Default, Serialize)]
pub struct PoserAssets {
    pub textures: BTreeMap<String, String>,
    pub source: String,
    pub models: BTreeMap<String, item_models::LocalItemModel>,
    pub unavailable: Vec<String>,
}
const LIMIT: u64 = 512 * 1024;
const ITEMS: &[&str] = &[
    "diamond_sword",
    "netherite_sword",
    "iron_sword",
    "wooden_sword",
    "diamond_pickaxe",
    "diamond_axe",
    "netherite_axe",
    "mace",
    "totem",
    "shield",
    "trident",
    "bow",
    "bow_pulling_2",
    "crossbow",
    "crossbow_arrow",
    "arrow",
    "fishing_rod",
    "enchanted_golden_apple",
    "torch",
    "lantern",
    "ender_pearl",
    "grass_block",
    "stone_block",
    "cobblestone_block",
    "oak_planks_block",
    "diamond_block",
    "tnt_block",
    "command_block",
    "crafting_table",
    "anvil",
    "potion",
    "redstone",
    "book",
    "chest",
    "compass",
    "clock",
    "name_tag",
    "feather",
    "hopper",
    "painting",
    "map",
    "diamond",
    "emerald",
    "nether_star",
    "redstone_torch",
    "music_disc_13",

    "diamond_chestplate",
    "spyglass",
    "slime_ball",
    "iron_ingot",
    "cooked_beef",
    "golden_carrot",
    "experience_bottle",
    "firework_rocket",
    "obsidian",
];
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 160
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn local_file(root: &Path, path: &Path, limit: u64) -> Option<fs::File> {
    let path = path.canonicalize().ok()?;
    if !path.starts_with(root.canonicalize().ok()?) {
        return None;
    }
    let file = fs::File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    if !meta.is_file() || meta.len() > limit {
        return None;
    }
    Some(file)
}
fn local_json(root: &Path, path: &Path) -> Option<serde_json::Value> {
    let mut bytes = Vec::new();
    local_file(root, path, 16 * 1024 * 1024)?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 16 * 1024 * 1024 {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}
fn names(id: &str) -> Vec<String> {
    let aliases: &[&str] = match id {
        "totem" => &["totem_of_undying", "totem"],
        "grass_block" => &["grass_block_side", "grass_side"],
        "stone_block" => &["stone"],
        "cobblestone_block" => &["cobblestone"],
        "oak_planks_block" => &["oak_planks", "planks_oak"],
        "tnt_block" => &["tnt_side"],
        "command_block" => &["command_block_front"],
        "enchanted_golden_apple" => &["golden_apple"],
        "bow" => &["bow", "bow_standby"],
        "crossbow" => &["crossbow_standby", "crossbow"],
        "potion" => &["potion", "potion_bottle_drinkable"],
        "anvil" => &["anvil_top", "anvil_top_damaged_0"],
        "clock" => &["clock", "clock_00"],
        "compass" => &["compass", "compass_00"],
        "slime_ball" => &["slime_ball", "slimeball"],
        "cooked_beef" => &["cooked_beef", "beef_cooked"],
        "golden_carrot" => &["golden_carrot", "carrot_golden"],
        "experience_bottle" => &["experience_bottle", "exp_bottle"],
        "firework_rocket" => &["firework_rocket", "fireworks"],
        _ => &[id],
    };
    let mut paths = Vec::new();
    for name in aliases {
        for folder in ["item", "items", "block", "blocks"] {
            paths.push(format!("assets/minecraft/textures/{folder}/{name}.png"));
        }
    }
    paths
}
fn image_data(bytes: &[u8]) -> Option<String> {
    if bytes.len() > LIMIT as usize || bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    if w == 0 || h < w || w > 1024 || h > 16384 || u64::from(w) * u64::from(h) > 4 * 1024 * 1024 {
        return None;
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?.crop_imm(0, 0, w, w).resize_exact(
        16,
        16,
        image::imageops::FilterType::Nearest,
    );
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(out.into_inner())
    ))
}
fn jar_texture(jar: &mut zip::ZipArchive<fs::File>, paths: &[String]) -> Option<String> {
    for path in paths {
        let Ok(entry) = jar.by_name(path) else {
            continue;
        };
        if entry.size() > LIMIT {
            continue;
        }
        let mut bytes = Vec::new();
        if entry.take(LIMIT + 1).read_to_end(&mut bytes).is_ok() {
            if let Some(data) = image_data(&bytes) {
                return Some(data);
            }
        }
    }
    None
}
fn resolve_local(root: &Path, id: &str) -> Result<PoserAssets, String> {
    if !valid_id(id) {
        return Err("Identificador de instalación no válido".into());
    }
    let mut versions = Vec::new();
    let mut indexes = Vec::new();
    let mut seen = BTreeSet::new();
    let mut next = Some(id.to_owned());
    for _ in 0..8 {
        let Some(id) = next.take() else { break };
        if !valid_id(&id) || !seen.insert(id.clone()) {
            break;
        }
        let folder = root.join("versions").join(&id);
        versions.push((id.clone(), folder.join(format!("{id}.jar"))));
        if let Some(json) = local_json(root, &folder.join(format!("{id}.json"))) {
            if let Some(index) = json
                .pointer("/assetIndex/id")
                .and_then(serde_json::Value::as_str)
                .filter(|v| valid_id(v))
            {
                indexes.push(index.to_owned());
            }
            next = json["inheritsFrom"].as_str().map(str::to_owned);
        }
    }
    let mut textures = BTreeMap::new();
    let mut models = BTreeMap::new();
    let mut sources = Vec::new();
    for (version, path) in versions {
        if let Some(file) = local_file(root, &path, 256 * 1024 * 1024) {
            if let Ok(mut jar) = zip::ZipArchive::new(file) {
                let before = textures.len();
                for (id, model) in item_models::read_models(&mut jar) {
                    models.entry(id).or_insert(model);
                }
                for id in ITEMS {
                    if !textures.contains_key(*id) {
                        if let Some(data) = jar_texture(&mut jar, &names(id)) {
                            textures.insert((*id).to_owned(), data);
                        }
                    }
                }
                if textures.len() > before {
                    sources.push(version);
                }
            }
        }
    }
    for index in indexes {
        let Some(json) = local_json(
            root,
            &root.join("assets/indexes").join(format!("{index}.json")),
        ) else {
            continue;
        };
        let before = textures.len();
        for id in ITEMS {
            if textures.contains_key(*id) {
                continue;
            }
            for path in names(id) {
                let Some(hash) =
                    json["objects"][path.trim_start_matches("assets/")]["hash"].as_str()
                else {
                    continue;
                };
                if hash.len() != 40 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                    continue;
                }
                let hash = hash.to_ascii_lowercase();
                let file = root.join("assets/objects").join(&hash[..2]).join(&hash);
                let Some(file) = local_file(root, &file, LIMIT) else {
                    continue;
                };
                let mut bytes = Vec::new();
                if file.take(LIMIT + 1).read_to_end(&mut bytes).is_ok()
                    && format!("{:x}", Sha1::digest(&bytes)) == hash
                {
                    if let Some(data) = image_data(&bytes) {
                        textures.insert((*id).to_owned(), data);
                        break;
                    }
                }
            }
        }
        if textures.len() > before {
            sources.push(format!("assets {index}"));
        }
    }
    Ok(PoserAssets {
        source: if textures.is_empty() {
            "Arte original de TP · instala una versión para usar sus recursos locales".into()
        } else {
            format!("Recursos locales · {}", sources.join(" / "))
        },
        textures,
        unavailable: ITEMS
            .iter()
            .take(28)
            .filter(|id| !models.contains_key(**id))
            .map(|id| (*id).to_owned())
            .collect(),
        models,
    })
}
#[tauri::command]
pub async fn get_poser_assets(version_id: Option<String>) -> Result<PoserAssets, String> {
    let Some(id) = version_id else {
        return Ok(PoserAssets::default());
    };
    let root = crate::launcher_settings::data_root()?;
    tokio::task::spawn_blocking(move || resolve_local(&root, &id))
        .await
        .map_err(|e| e.to_string())?
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn reject_paths_and_invalid_png_before_decoding() {
        for id in ["../x", "..", "C:/data", "a/b", ""] {
            assert!(!valid_id(id));
        }
        assert!(valid_id("1.21.9-OptiFine_X"));
        assert!(image_data(b"not png").is_none());
    }
    #[test]
    fn plugin_artwork_has_fixed_modern_and_legacy_local_paths() {
        for id in ["diamond_chestplate", "spyglass", "slime_ball", "iron_ingot",
            "cooked_beef", "golden_carrot", "experience_bottle", "firework_rocket", "obsidian"] {
            assert!(ITEMS.contains(&id), "Missing plugin texture {id}");
            assert!(names(id).iter().all(|path| path.starts_with("assets/minecraft/textures/")
                && path.ends_with(".png") && !path.contains("..")));
        }
        assert!(names("clock").iter().any(|path| path.ends_with("/clock_00.png")));
        assert!(names("compass").iter().any(|path| path.ends_with("/compass_00.png")));
        assert!(names("firework_rocket").iter().any(|path| path.ends_with("/fireworks.png")));
    }
    #[test]
    fn inherited_local_jar_reads_fixed_paths_and_never_unpacks() {
        let root = std::env::temp_dir().join(format!("tp-local-art-{}", uuid::Uuid::new_v4()));
        let version = root.join("versions/child");
        let base = root.join("versions/base");
        fs::create_dir_all(&version).unwrap();
        fs::create_dir_all(&base).unwrap();
        fs::write(version.join("child.json"), br#"{"inheritsFrom":"base"}"#).unwrap();
        let png = image::DynamicImage::new_rgba8(16, 16);
        let mut bytes = Cursor::new(Vec::new());
        png.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        let file = fs::File::create(base.join("base.jar")).unwrap();
        let mut jar = zip::ZipWriter::new(file);
        jar.start_file(
            "assets/minecraft/textures/item/diamond_sword.png",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        jar.write_all(bytes.get_ref()).unwrap();
        jar.start_file(
            "../../outside.png",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        jar.write_all(b"never unpack").unwrap();
        jar.finish().unwrap();
        let art = resolve_local(&root, "child").unwrap();
        assert_eq!(art.textures.len(), 1);
        assert!(art.textures["diamond_sword"].starts_with("data:image/png;base64,"));
        assert!(art.source.contains("base"));
        assert!(!root.join("outside.png").exists());
        fs::remove_file(version.join("child.json")).unwrap();
        fs::remove_file(base.join("base.jar")).unwrap();
        fs::remove_dir(version).unwrap();
        fs::remove_dir(base).unwrap();
        fs::remove_dir(root.join("versions")).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn index_objects_verify_hashes_and_report_the_actual_source() {
        let root = std::env::temp_dir().join(format!("tp-index-art-{}", uuid::Uuid::new_v4()));
        let version = root.join("versions/base");
        let indexes = root.join("assets/indexes");
        fs::create_dir_all(&version).unwrap();
        fs::create_dir_all(&indexes).unwrap();
        fs::write(
            version.join("base.json"),
            br#"{"assetIndex":{"id":"fixture"}}"#,
        )
        .unwrap();
        let png = image::DynamicImage::new_rgba8(16, 16);
        let mut bytes = Cursor::new(Vec::new());
        png.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        let hash = format!("{:x}", Sha1::digest(bytes.get_ref()));
        let folder = root.join("assets/objects").join(&hash[..2]);
        fs::create_dir_all(&folder).unwrap();
        let object = folder.join(&hash);
        fs::write(&object, bytes.get_ref()).unwrap();
        let index = serde_json::json!({"objects":{"minecraft/textures/item/diamond_sword.png":{"hash":hash}}});
        fs::write(
            indexes.join("fixture.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let art = resolve_local(&root, "base").unwrap();
        assert_eq!(art.textures.len(), 1);
        assert_eq!(art.source, "Recursos locales · assets fixture");
        fs::write(&object, b"changed on disk").unwrap();
        assert!(resolve_local(&root, "base").unwrap().textures.is_empty());
        fs::remove_file(object).unwrap();
        fs::remove_file(indexes.join("fixture.json")).unwrap();
        fs::remove_file(version.join("base.json")).unwrap();
        for dir in [
            &folder,
            &root.join("assets/objects"),
            &indexes,
            &root.join("assets"),
            &version,
            &root.join("versions"),
            &root,
        ] {
            fs::remove_dir(dir).unwrap();
        }
    }
    #[test]
    fn outside_paths_are_not_read_and_animated_sprites_use_the_first_frame() {
        let parent = std::env::temp_dir().join(format!("tp-art-boundary-{}", uuid::Uuid::new_v4()));
        let root = parent.join("root");
        fs::create_dir_all(&root).unwrap();
        let outside = parent.join("outside.png");
        fs::write(&outside, b"outside root").unwrap();
        assert!(local_file(&root, &outside, LIMIT).is_none());
        #[cfg(windows)]
        {
            let link = root.join("escape.png");
            if std::os::windows::fs::symlink_file(&outside, &link).is_ok() {
                assert!(local_file(&root, &link, LIMIT).is_none());
                fs::remove_file(link).unwrap();
            }
        }
        #[cfg(unix)]
        {
            let link = root.join("escape.png");
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            assert!(local_file(&root, &link, LIMIT).is_none());
            fs::remove_file(link).unwrap();
        }
        let mut image = image::RgbaImage::from_pixel(32, 64, image::Rgba([255, 0, 0, 255]));
        for y in 32..64 {
            for x in 0..32 {
                image.put_pixel(x, y, image::Rgba([0, 0, 255, 255]));
            }
        }
        let mut bytes = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let data = image_data(bytes.get_ref()).unwrap();
        let normalized = base64::engine::general_purpose::STANDARD
            .decode(data.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        let normalized = image::load_from_memory(&normalized).unwrap().to_rgba8();
        assert_eq!(normalized.dimensions(), (16, 16));
        assert!(normalized.pixels().all(|p| p.0 == [255, 0, 0, 255]));
        fs::remove_file(outside).unwrap();
        fs::remove_dir(root).unwrap();
        fs::remove_dir(parent).unwrap();
    }
    #[test]
    #[ignore = "Lectura local de una instalación existente; requiere TP_LOCAL_ART_VERSION"]
    fn installed_client_art_is_read_without_writing_game_or_repository_files() {
        let id = std::env::var("TP_LOCAL_ART_VERSION")
            .expect("Define TP_LOCAL_ART_VERSION con una instalación ya descargada");
        let root = crate::launcher_settings::data_root().unwrap();
        let result = resolve_local(&root, &id).unwrap();
        assert!(
            !result.textures.is_empty(),
            "No se encontraron sprites sueltos en la instalación seleccionada"
        );
        assert!(result.textures.contains_key("diamond_sword"));
        assert!(result.models.contains_key("diamond_sword"));
        let sword = &result.models["diamond_sword"];
        assert_eq!(sword.kind, "sprite");
        assert!(sword.textures.contains_key("layer0"));
        if id == "1.21.1" || id == "26.2" || id == "1.12.2" {
            assert!(result.models.contains_key("shield"));
            assert_eq!(result.models["shield"].kind, "shield");
            assert!(result.models["shield"].textures.contains_key("entity"));
            assert_eq!(sword.display.right.rotation, [0., -90., 55.]);
            assert_eq!(sword.display.left.rotation, [0., 90., -55.]);
            assert!(result.models["grass_block"]
                .elements
                .iter()
                .any(|element| element.faces.contains_key("up")));
        }
        assert!(result
            .textures
            .values()
            .all(|v| v.starts_with("data:image/png;base64,")));
        println!(
            "{}: {} texturas locales y {} modelos de objeto válidos; ausentes {:?}",
            id,
            result.textures.len(),
            result.models.len(),
            result.unavailable
        );
    }
}
