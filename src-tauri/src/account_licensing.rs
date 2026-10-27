

use serde::{Deserialize,Serialize};
use std::{fs,io::Read,sync::Mutex};
static RECEIPTS:Mutex<()>=Mutex::new(());
const OFFLINE_SECONDS:u64=30*24*60*60;
#[derive(Clone,Serialize,Deserialize)]struct Receipt{profile_id:String,verified_at:u64}
fn valid(receipt:&Receipt,now:u64)->bool{receipt.profile_id.len()==32&&receipt.profile_id.bytes().all(|v|v.is_ascii_hexdigit())&&receipt.verified_at<=now&&now-receipt.verified_at<=OFFLINE_SECONDS}
fn path()->Result<std::path::PathBuf,String>{Ok(crate::launcher_settings::config_root()?.join("minecraft-license-cache.dat"))}
fn load()->Result<Vec<Receipt>,String>{let path=path()?;if !path.exists(){return Ok(Vec::new());}let meta=fs::symlink_metadata(&path).map_err(|e|e.to_string())?;if meta.file_type().is_symlink()||!meta.is_file()||meta.len()>65536{return Err("La caché de licencias no es válida".into());}let mut bytes=Vec::new();fs::File::open(path).map_err(|e|e.to_string())?.take(65537).read_to_end(&mut bytes).map_err(|e|e.to_string())?;if bytes.len()>65536{return Err("La caché de licencias supera el límite".into());}let clear=crate::token_storage::unprotect(&bytes)?;let rows:Vec<Receipt>=serde_json::from_slice(&clear).map_err(|_|"La caché de licencias no es válida")?;if rows.len()>128{return Err("Demasiadas licencias guardadas".into());}Ok(rows)}
pub fn remember_verified(profile:&crate::MinecraftProfile)->Result<(),String>{
 let _lock=RECEIPTS.lock().map_err(|_|"Caché de licencias ocupada")?;let now=crate::unix_time_seconds();let mut rows=load().unwrap_or_default();rows.retain(|r|valid(r,now)&&r.profile_id!=profile.id);rows.push(Receipt{profile_id:profile.id.to_ascii_lowercase(),verified_at:now});if rows.len()>128{rows.remove(0);}
 let bytes=crate::token_storage::protect(&serde_json::to_vec(&rows).map_err(|e|e.to_string())?)?;let path=path()?;let pending=path.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));fs::write(&pending,bytes).and_then(|_|fs::rename(&pending,path)).map_err(|e|{let _=fs::remove_file(pending);e.to_string()})
}
pub fn verified_at(profile_id:&str)->Option<u64>{let _lock=RECEIPTS.lock().ok()?;let now=crate::unix_time_seconds();load().ok()?.into_iter().find(|r|r.profile_id==profile_id&&valid(r,now)).map(|r|r.verified_at)}
pub async fn verify(client:&reqwest::Client,access_token:&str)->Result<(),String>{
 let response=client.get("https://api.minecraftservices.com/entitlements/mcstore").bearer_auth(access_token).timeout(std::time::Duration::from_secs(20)).send().await.map_err(|_|"No se pudo verificar la licencia de Minecraft. Conéctate para validar tu cuenta")?;
 if !response.status().is_success(){return Err("Minecraft no pudo confirmar la licencia de esta cuenta".into());}
 let bytes=response.bytes().await.map_err(|_|"La verificación de licencia quedó incompleta")?;if bytes.len()>65536{return Err("Respuesta de licencia demasiado grande".into());}
 let value:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|"Respuesta de licencia inválida")?;
 if !has_java_entitlement(&value){return Err("Esta cuenta no tiene una licencia Java de Minecraft confirmada. No se completó el inicio oficial".into());}Ok(())
}
fn has_java_entitlement(value:&serde_json::Value)->bool{value["items"].as_array().is_some_and(|items|items.iter().any(|item|item["name"].as_str().is_some_and(|name|["game_minecraft","product_minecraft"].contains(&name))))}
#[cfg(test)]mod tests{use super::*;#[test]fn offline_grace_is_bounded_and_requires_a_valid_id(){let now=10000000;let mut r=Receipt{profile_id:"a".repeat(32),verified_at:now-30};assert!(valid(&r,now));r.verified_at=now+1;assert!(!valid(&r,now));r.verified_at=now-OFFLINE_SECONDS-1;assert!(!valid(&r,now));r.verified_at=now;r.profile_id="not-a-license".into();assert!(!valid(&r,now));}#[test]fn unrelated_products_do_not_unlock_java(){use serde_json::json;assert!(has_java_entitlement(&json!({"items":[{"name":"game_minecraft"}]})));for value in [json!({"items":[]}),json!({"items":[{"name":"minecraft_dungeons"}]}),json!({"items":[{"name":""}]}),json!({"owns_game":true})]{assert!(!has_java_entitlement(&value));}}}
