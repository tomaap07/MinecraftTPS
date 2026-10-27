

use super::LIMIT;
use base64::Engine;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
};

#[derive(Clone, Serialize)]
pub struct Display {
    pub rotation: [f64; 3],
    pub translation: [f64; 3],
    pub scale: [f64; 3],
}
impl Default for Display {
    fn default() -> Self {
        Self {
            rotation: [0.; 3],
            translation: [0.; 3],
            scale: [1.; 3],
        }
    }
}
#[derive(Serialize)]
pub struct Displays {
    pub right: Display,
    pub left: Display,
}
#[derive(Serialize)]
pub struct Face {
    pub texture: String,
    pub uv: [f64; 4],
    pub rotation: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tintindex: Option<u8>,
}
#[derive(Serialize)]
pub struct Rotation {
    pub origin: [f64; 3],
    pub axis: String,
    pub angle: f64,
    pub rescale: bool,
}
#[derive(Serialize)]
pub struct Element {
    pub from: [f64; 3],
    pub to: [f64; 3],
    pub faces: BTreeMap<String, Face>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,
    pub shade: bool,
}
#[derive(Serialize)]
pub struct LocalItemModel {
    pub kind: String,
    pub textures: BTreeMap<String, String>,
    pub elements: Vec<Element>,
    pub display: Displays,
    pub source_model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint_color: Option<String>,
}
#[derive(Default)]
struct Merged {
    textures: BTreeMap<String, String>,
    display: BTreeMap<String, Display>,
    elements: Vec<Value>,
    sprite: bool,
}

const HELD: &[&str] = &[
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
];

fn resource(value: &str) -> Option<String> {
    let value = if let Some((namespace, path)) = value.split_once(':') {
        if namespace != "minecraft" {
            return None;
        }
        path
    } else {
        value
    };
    if value.is_empty()
        || value.len() > 160
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-/".contains(&b))
    {
        return None;
    }
    Some(value.to_owned())
}
fn key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn vector<const N: usize>(value: &Value, bound: f64) -> Option<[f64; N]> {
    let values = value.as_array()?;
    if values.len() != N {
        return None;
    }
    let mut out = [0.; N];
    for (i, value) in values.iter().enumerate() {
        let number = value.as_f64()?;
        if !number.is_finite() || number.abs() > bound {
            return None;
        }
        out[i] = number;
    }
    Some(out)
}
fn display(value: &Value) -> Option<Display> {
    let result = Display {
        rotation: if value.get("rotation").is_some() {
            vector(&value["rotation"], 360.)?
        } else {
            [0.; 3]
        },
        translation: if value.get("translation").is_some() {
            vector(&value["translation"], 80.)?
        } else {
            [0.; 3]
        },
        scale: if value.get("scale").is_some() {
            vector(&value["scale"], 4.)?
        } else {
            [1.; 3]
        },
    };
    if result.scale.iter().any(|v| *v < 0.) {
        return None;
    }
    Some(result)
}
fn png_image(bytes: &[u8]) -> Option<image::DynamicImage> {
    if bytes.len() > LIMIT as usize || bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    if w == 0 || h == 0 || w > 1024 || h > 16384 || u64::from(w) * u64::from(h) > 4 * 1024 * 1024 {
        return None;
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().ok()
}
fn png_data(image: &image::DynamicImage) -> Option<String> {
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, image::ImageFormat::Png).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())
    ))
}

struct Reader<'a> {
    jar: &'a mut zip::ZipArchive<fs::File>,
    json: BTreeMap<String, Option<Value>>,
    png: BTreeMap<String, Option<String>>,
}
impl<'a> Reader<'a> {
    fn bytes(&mut self, path: &str, limit: u64) -> Option<Vec<u8>> {
        let entry = self.jar.by_name(path).ok()?;
        if entry.size() > limit {
            return None;
        }
        let mut bytes = Vec::new();
        entry.take(limit + 1).read_to_end(&mut bytes).ok()?;
        if bytes.len() > limit as usize {
            return None;
        }
        Some(bytes)
    }
    fn json_document(&mut self, folder: &str, id: &str) -> Option<Value> {
        let id = resource(id)?;
        let cache_key = format!("{folder}/{id}");
        if let Some(value) = self.json.get(&cache_key) {
            return value.clone();
        }
        if self.json.len() >= 256 {
            return None;
        }
        let value = self
            .bytes(&format!("assets/minecraft/{folder}/{id}.json"), 65536)
            .and_then(|bytes| serde_json::from_slice(&bytes).ok());
        self.json.insert(cache_key, value.clone());
        value
    }
    fn json(&mut self, id: &str) -> Option<Value> {
        self.json_document("models", id)
    }
    fn item_model(&mut self, id: &str) -> Option<(String, Merged)> {
        let value = self.json_document("items", id)?;
        let name = default_item_model(&value["model"], 0)?;
        let merged = self.merged(&name, &mut BTreeSet::new())?;
        Some((name, merged))
    }
    fn texture(&mut self, id: &str, entity: bool) -> Option<String> {
        let id = resource(id)?;
        let cache_key = format!("{entity}:{id}");
        if let Some(value) = self.png.get(&cache_key) {
            return value.clone();
        }
        if self.png.len() >= 256 {
            return None;
        }
        let value = self
            .bytes(&format!("assets/minecraft/textures/{id}.png"), LIMIT)
            .and_then(|bytes| png_image(&bytes))
            .and_then(|image| {
                if entity {
                    png_data(&image)
                } else if image.height() >= image.width() {
                    png_data(&image.crop_imm(0, 0, image.width(), image.width()))
                } else {
                    None
                }
            });
        self.png.insert(cache_key, value.clone());
        value
    }
    fn merged(&mut self, id: &str, seen: &mut BTreeSet<String>) -> Option<Merged> {
        let id = resource(id)?;
        if seen.len() >= 12 || !seen.insert(id.clone()) {
            return None;
        }
        let value = self.json(&id)?;
        let mut merged = if let Some(parent) = value["parent"].as_str() {
            match parent.trim_start_matches("minecraft:") {
                "builtin/generated" => Merged {
                    sprite: true,
                    ..Default::default()
                },
                "builtin/entity" => Merged::default(),
                _ => self.merged(parent, seen)?,
            }
        } else {
            Merged::default()
        };
        if let Some(textures) = value["textures"].as_object() {
            if textures.len() > 64 {
                return None;
            }
            for (k, v) in textures {
                if !key(k) {
                    return None;
                }
                let v = v.as_str()?;
                if v.len() > 160
                    || (!v.starts_with('#') && resource(v).is_none())
                    || (v.starts_with('#') && !key(&v[1..]))
                {
                    return None;
                }
                merged.textures.insert(k.clone(), v.to_owned());
            }
        }
        if let Some(displays) = value["display"].as_object() {
            for name in [
                "thirdperson_righthand",
                "thirdperson_lefthand",
                "thirdperson",
            ] {
                if let Some(value) = displays.get(name) {
                    merged.display.insert(name.into(), display(value)?);
                }
            }
        }
        if let Some(elements) = value["elements"].as_array() {
            if elements.len() > 128 {
                return None;
            }
            merged.elements = elements.clone();
            merged.sprite = false;
        }
        Some(merged)
    }
    fn tint(&mut self) -> Option<String> {
        let bytes = self.bytes("assets/minecraft/textures/colormap/grass.png", LIMIT)?;
        let image = png_image(&bytes)?.to_rgb8();
        if image.width() != 256 || image.height() != 256 {
            return None;
        }
        let color = image.get_pixel(127, 127);
        Some(format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]))
    }
}



fn default_item_model(value: &Value, depth: u8) -> Option<String> {
    if depth >= 12 {
        return None;
    }
    match resource(value["type"].as_str()?)?.as_str() {
        "model" => resource(value["model"].as_str()?),
        "special" => resource(value["base"].as_str()?),
        "condition" => default_item_model(&value["on_false"], depth + 1),
        "select" | "range_dispatch" => default_item_model(&value["fallback"], depth + 1),
        _ => None,
    }
}
fn texture_ref(textures: &BTreeMap<String, String>, value: &str) -> Option<String> {
    let mut current = value.to_owned();
    let mut seen = BTreeSet::new();
    for _ in 0..12 {
        if !current.starts_with('#') {
            return resource(&current);
        }
        let alias = &current[1..];
        if !key(alias) || !seen.insert(alias.to_owned()) {
            return None;
        }
        current = textures.get(alias)?.clone();
    }
    None
}
fn default_uv(direction: &str, from: [f64; 3], to: [f64; 3]) -> [f64; 4] {
    match direction {
        "down" => [from[0], 16. - to[2], to[0], 16. - from[2]],
        "up" => [from[0], from[2], to[0], to[2]],
        "north" => [16. - to[0], 16. - to[1], 16. - from[0], 16. - from[1]],
        "south" => [from[0], 16. - to[1], to[0], 16. - from[1]],
        "west" => [from[2], 16. - to[1], to[2], 16. - from[1]],
        _ => [16. - to[2], 16. - to[1], 16. - from[2], 16. - from[1]],
    }
}
fn model_names(id: &str) -> Vec<String> {
    let names: &[&str] = match id {
        "totem" => &["totem_of_undying", "totem"],
        "trident" => &["trident_in_hand"],
        "grass_block" => &["grass_block", "grass"],
        "stone_block" => &["stone"],
        "cobblestone_block" => &["cobblestone"],
        "oak_planks_block" => &["oak_planks", "planks"],
        "diamond_block" => &["diamond_block"],
        "tnt_block" => &["tnt"],
        "command_block" => &["command_block"],
        "enchanted_golden_apple" => &["enchanted_golden_apple", "golden_apple"],
        _ => &[id],
    };
    names.iter().map(|v| format!("item/{v}")).collect()
}
fn build_model(reader: &mut Reader<'_>, id: &str) -> Option<LocalItemModel> {
    let mut chosen = None;
    let names = model_names(id);
    for name in &names {
        if let Some(model) = reader.merged(&name, &mut BTreeSet::new()) {
            chosen = Some((name.clone(), model));
            break;
        }
    }
    if chosen.is_none() {
        for name in &names {
            if let Some(model) = reader.item_model(name.strip_prefix("item/")?) {
                chosen = Some(model);
                break;
            }
        }
    }
    let (source_model, merged) = chosen?;
    let right = merged
        .display
        .get("thirdperson_righthand")
        .or_else(|| merged.display.get("thirdperson"))
        .cloned()
        .unwrap_or_default();
    let left = merged
        .display
        .get("thirdperson_lefthand")
        .cloned()
        .unwrap_or_else(|| right.clone());
    let mut model = LocalItemModel {
        kind: if id == "shield" {
            "shield"
        } else if id == "trident" {
            "trident"
        } else if merged.sprite {
            "sprite"
        } else {
            "model"
        }
        .into(),
        textures: BTreeMap::new(),
        elements: Vec::new(),
        display: Displays { right, left },
        source_model,
        tint_color: None,
    };
    if model.kind == "shield" || model.kind == "trident" {
        let names = if id == "shield" {
            vec![
                "entity/shield/shield_base_nopattern",
                "entity/shield_base_nopattern",
                "entity/shield/shield_base",
                "entity/shield_base",
            ]
        } else {
            vec!["entity/trident/trident", "entity/trident"]
        };
        for name in names {
            if let Some(data) = reader.texture(name, true) {
                model.textures.insert("entity".into(), data);
                return Some(model);
            }
        }
        return None;
    }
    if model.kind == "sprite" {
        for layer in 0..5 {
            let name = format!("layer{layer}");
            if let Some(raw) = merged.textures.get(&name) {
                let resolved = texture_ref(&merged.textures, raw)?;
                model
                    .textures
                    .insert(name, reader.texture(&resolved, false)?);
            }
        }
        if !model.textures.contains_key("layer0") {
            return None;
        }
        return Some(model);
    }
    for value in merged.elements {
        let from = vector(&value["from"], 64.)?;
        let to = vector(&value["to"], 64.)?;
        if from.iter().zip(to).any(|(a, b)| *a > b) {
            return None;
        }
        let mut faces = BTreeMap::new();
        let raw_faces = value["faces"].as_object()?;
        if raw_faces.len() > 6 {
            return None;
        }
        for (direction, face) in raw_faces {
            if !["north", "south", "east", "west", "up", "down"].contains(&direction.as_str()) {
                return None;
            }
            let raw = face["texture"].as_str()?;
            let resolved = texture_ref(&merged.textures, raw)?;
            let texture = resolved.replace('/', "_");
            if !model.textures.contains_key(&texture) {
                model
                    .textures
                    .insert(texture.clone(), reader.texture(&resolved, false)?);
            }
            let uv = if face.get("uv").is_some() {
                vector(&face["uv"], 64.)?
            } else {
                default_uv(direction, from, to)
            };
            let rotation = face["rotation"].as_u64().unwrap_or(0);
            if ![0, 90, 180, 270].contains(&rotation) {
                return None;
            }
            let tintindex = face["tintindex"]
                .as_i64()
                .filter(|v| *v >= 0 && *v <= 15)
                .map(|v| v as u8);
            if tintindex.is_some() && id == "grass_block" {
                model.tint_color = reader.tint();
            }
            faces.insert(
                direction.clone(),
                Face {
                    texture,
                    uv,
                    rotation: rotation as u16,
                    tintindex,
                },
            );
        }
        let rotation = if let Some(value) = value.get("rotation") {
            let axis = value["axis"].as_str()?;
            if !["x", "y", "z"].contains(&axis) {
                return None;
            }
            let angle = value["angle"].as_f64()?;
            if !angle.is_finite() || angle.abs() > 45. {
                return None;
            }
            Some(Rotation {
                origin: vector(&value["origin"], 64.)?,
                axis: axis.into(),
                angle,
                rescale: value["rescale"].as_bool().unwrap_or(false),
            })
        } else {
            None
        };
        model.elements.push(Element {
            from,
            to,
            faces,
            rotation,
            shade: value["shade"].as_bool().unwrap_or(true),
        });
    }
    if model.elements.is_empty() {
        None
    } else {
        Some(model)
    }
}
pub fn read_models(jar: &mut zip::ZipArchive<fs::File>) -> BTreeMap<String, LocalItemModel> {
    let mut reader = Reader {
        jar,
        json: BTreeMap::new(),
        png: BTreeMap::new(),
    };
    HELD.iter()
        .filter_map(|id| build_model(&mut reader, id).map(|model| ((*id).into(), model)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn refuses_remote_namespaces_paths_cycles_and_bad_numbers() {
        for path in [
            "../x",
            "minecraft:../x",
            "mod:thing",
            "https://example.invalid/x",
            "item//sword",
            "/item/a",
            "item/a.png",
        ] {
            assert!(resource(path).is_none());
        }
        assert_eq!(
            resource("minecraft:item/diamond_sword").as_deref(),
            Some("item/diamond_sword")
        );
        assert!(display(&serde_json::json!({"scale":[-1,1,1]})).is_none());
        assert!(vector::<3>(&serde_json::json!([1, 2, 999]), 64.).is_none());
        let textures = BTreeMap::from([("a".into(), "#b".into()), ("b".into(), "#a".into())]);
        assert!(texture_ref(&textures, "#a").is_none());
        assert!(default_item_model(
            &serde_json::json!({"type":"minecraft:model","model":"othermod:block/stone"}),
            0
        )
        .is_none());
        assert!(default_item_model(
            &serde_json::json!({"type":"minecraft:model","model":"minecraft:../stone"}),
            0
        )
        .is_none());
        assert!(default_item_model(
            &serde_json::json!({"type":"minecraft:model","model":"block/stone"}),
            12
        )
        .is_none());
    }

    #[test]
    fn local_model_inheritance_keeps_each_hand_and_real_face_textures() {
        let path =
            std::env::temp_dir().join(format!("tp-item-models-{}.jar", uuid::Uuid::new_v4()));
        let mut jar = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            16,
            16,
            image::Rgba([45, 120, 200, 255]),
        ))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
        for (name,bytes) in [
            ("assets/minecraft/models/item/base.json",br#"{"parent":"builtin/generated","display":{"thirdperson_righthand":{"rotation":[0,-90,55],"translation":[0,4,0.5],"scale":[0.85,0.85,0.85]},"thirdperson_lefthand":{"rotation":[0,90,-55],"translation":[0,4,0.5],"scale":[0.85,0.85,0.85]}}}"#.as_slice()),
            ("assets/minecraft/models/item/diamond_sword.json",br#"{"parent":"minecraft:item/base","textures":{"layer0":"minecraft:item/diamond_sword"}}"#.as_slice()),
            ("assets/minecraft/models/item/stone.json",br#"{"parent":"block/fixture"}"#.as_slice()),
            ("assets/minecraft/items/grass_block.json",br#"{"model":{"type":"minecraft:select","fallback":{"type":"minecraft:condition","on_false":{"type":"minecraft:model","model":"minecraft:block/fixture"},"on_true":{"type":"minecraft:model","model":"remote:forbidden"}}}}"#.as_slice()),
            ("assets/minecraft/models/block/fixture.json",br##"{"textures":{"side":"block/stone","all":"#side"},"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"up":{"texture":"#all","rotation":90},"north":{"texture":"#side","uv":[0,0,16,16]}}}]}"##.as_slice()),
            ("assets/minecraft/textures/item/diamond_sword.png",png.get_ref().as_slice()),
            ("assets/minecraft/textures/block/stone.png",png.get_ref().as_slice()),
        ]{jar.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();jar.write_all(bytes).unwrap();}
        jar.finish().unwrap();
        let mut archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
        let models = read_models(&mut archive);
        let sword = &models["diamond_sword"];
        assert_eq!(sword.kind, "sprite");
        assert_eq!(sword.display.right.rotation, [0., -90., 55.]);
        assert_eq!(sword.display.left.rotation, [0., 90., -55.]);
        assert_eq!(sword.display.right.scale, [0.85; 3]);
        assert!(sword.textures["layer0"].starts_with("data:image/png;base64,"));
        let block = &models["stone_block"];
        assert_eq!(block.kind, "model");
        assert_eq!(block.elements[0].faces["up"].rotation, 90);
        assert_eq!(block.elements[0].faces["north"].uv, [0., 0., 16., 16.]);
        assert_eq!(block.textures.len(), 1);
        let modern_block = &models["grass_block"];
        assert_eq!(modern_block.source_model, "block/fixture");
        assert_eq!(
            modern_block.elements[0].faces["north"].uv,
            [0., 0., 16., 16.]
        );
        drop(archive);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn cyclic_or_foreign_parent_models_do_not_produce_invented_equipment() {
        let path =
            std::env::temp_dir().join(format!("tp-unsafe-models-{}.jar", uuid::Uuid::new_v4()));
        let mut jar = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        for (name, bytes) in [
            ("diamond_sword", r#"{"parent":"item/cycle"}"#),
            ("cycle", r#"{"parent":"item/diamond_sword"}"#),
            ("bow", r#"{"parent":"https://example.invalid/model"}"#),
            ("stone", r#"{"parent":"othermod:secret"}"#),
        ] {
            jar.start_file(
                format!("assets/minecraft/models/item/{name}.json"),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            jar.write_all(bytes.as_bytes()).unwrap();
        }
        jar.finish().unwrap();
        let mut archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
        assert!(read_models(&mut archive).is_empty());
        drop(archive);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn modern_nested_entity_texture_paths_preserve_the_complete_sheet() {
        let path =
            std::env::temp_dir().join(format!("tp-nested-entities-{}.jar", uuid::Uuid::new_v4()));
        let mut jar = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(64, 64)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        for (name, bytes) in [
            (
                "assets/minecraft/models/item/shield.json",
                br#"{"display":{"thirdperson_righthand":{"translation":[10,6,-4]}}}"#.as_slice(),
            ),
            (
                "assets/minecraft/models/item/trident_in_hand.json",
                br#"{"display":{"thirdperson_righthand":{"translation":[11,17,-2]}}}"#.as_slice(),
            ),
            (
                "assets/minecraft/textures/entity/shield/shield_base_nopattern.png",
                png.get_ref().as_slice(),
            ),
            (
                "assets/minecraft/textures/entity/trident/trident.png",
                png.get_ref().as_slice(),
            ),
        ] {
            jar.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            jar.write_all(bytes).unwrap();
        }
        jar.finish().unwrap();
        let mut archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
        let models = read_models(&mut archive);
        for id in ["shield", "trident"] {
            let data = models[id].textures["entity"]
                .strip_prefix("data:image/png;base64,")
                .unwrap();
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .unwrap();
            let image = image::load_from_memory(&bytes).unwrap();
            assert_eq!((image.width(), image.height()), (64, 64));
            assert_eq!(models[id].kind, id);
        }
        drop(archive);
        fs::remove_file(path).unwrap();
    }
}
