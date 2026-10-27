

use std::{collections::BTreeMap,fs,path::{Path,PathBuf}};
use serde::{Deserialize,Serialize};
#[derive(Clone,Serialize,Deserialize)]#[serde(deny_unknown_fields)]pub struct Record{file:String,sha1:String}
pub struct Store{path:PathBuf,records:BTreeMap<String,Record>}
fn file_hash(path:&Path)->Option<String>{let meta=fs::symlink_metadata(path).ok()?;if !meta.is_file()||meta.file_type().is_symlink()||meta.len()>128*1024*1024{return None;}use sha1::Digest;Some(format!("{:x}",sha1::Sha1::digest(fs::read(path).ok()?)))}
fn valid_file(file:&str)->bool{file.len()<=240&&file.ends_with(".jar")&&!file.contains(['/', '\\',':'])&&!file.chars().any(char::is_control)&&Path::new(file).components().count()==1}
impl Store{
 pub fn compatible_profile(&self,dir:&Path,profile:&crate::launcher_plugins::HudProfile,game:&str,loader:&str)->Result<(crate::launcher_plugins::HudProfile,Vec<String>),String>{
  let bad=crate::addon_compatibility::HIT_COLOR_263_SHA1;
  if !crate::addon_compatibility::blocked(game,loader,"hit_color",bad){return Ok((profile.clone(),Vec::new()));}
  self.profile_without_artifact(dir,profile,bad)
 }
 fn profile_without_artifact(&self,dir:&Path,profile:&crate::launcher_plugins::HudProfile,bad:&str)->Result<(crate::launcher_plugins::HudProfile,Vec<String>),String>{
  let mut effective=profile.clone();let mut warnings=Vec::new();
  let owned=self.records.get("hit_color").filter(|record|record.sha1.eq_ignore_ascii_case(bad));
  let mods=dir.join("mods");
  if fs::symlink_metadata(&mods).is_ok_and(|m|m.file_type().is_symlink()){return Err("La carpeta de mods es un enlace; no se cambiarán sus archivos".into());}
  for entry in fs::read_dir(&mods).ok().into_iter().flatten().take(1024).filter_map(Result::ok){
   let path=entry.path();if path.extension().is_none_or(|ext|ext!="jar"){continue;}
   if file_hash(&path).as_deref()==Some(bad)&&!owned.is_some_and(|record|path.file_name().and_then(|s|s.to_str())==Some(record.file.as_str())){
    return Err(format!("Hit Color para Minecraft 26.3 provoca un fallo de shaders (Sampler1). El archivo {} no está gestionado por TP: desactívalo desde Mis mods antes de jugar. No se ha modificado.",entry.file_name().to_string_lossy()));
   }
  }
  if let Some(record)=owned{
   let active=mods.join(&record.file);let disabled=mods.join(format!("{}.disabled",record.file));
   if file_hash(&active).as_deref()==Some(bad)||file_hash(&disabled).as_deref()==Some(bad){
    if let Some(module)=effective.modules.iter_mut().find(|m|m.id=="hit_color"){module.enabled=false;}
    warnings.push(crate::addon_compatibility::reason());
   }
  }
  Ok((effective,warnings))
 }
 pub fn load(dir:&Path)->Result<Self,String>{let path=dir.join("config/tp-launcher/managed-addons.json");let records=if let Ok(meta)=fs::symlink_metadata(&path){if meta.len()>65536||!meta.is_file()||meta.file_type().is_symlink(){return Err("Registro de complementos inválido".into());}serde_json::from_slice::<BTreeMap<String,Record>>(&fs::read(&path).map_err(|e|e.to_string())?).map_err(|_|"Registro de complementos dañado")?}else{BTreeMap::new()};if records.len()>64||records.iter().any(|(id,r)|id.len()>40||!valid_file(&r.file)||r.sha1.len()!=40||!r.sha1.bytes().all(|b|b.is_ascii_hexdigit())){return Err("Registro de complementos fuera de límites".into());}Ok(Self{path,records})}
 pub fn reconcile(&self,dir:&Path,profile:&crate::launcher_plugins::HudProfile)->Result<Vec<String>,String>{
  let mods=dir.join("mods");if fs::symlink_metadata(&mods).is_ok_and(|m|m.file_type().is_symlink()){return Err("La carpeta de mods es un enlace; no se cambiarán sus archivos".into());}
  let mut changes=Vec::new();for(id,record)in &self.records{let Some(module)=profile.modules.iter().find(|m|&m.id==id)else{continue};let normal=mods.join(&record.file);let disabled=mods.join(format!("{}.disabled",record.file));let(source,target)=if module.enabled{(&disabled,&normal)}else{(&normal,&disabled)};
   if source.exists()&&fs::symlink_metadata(target).is_err()&&file_hash(source).is_some_and(|hash|hash.eq_ignore_ascii_case(&record.sha1)){fs::rename(source,target).map_err(|e|format!("No se pudo cambiar el estado de {}: {e}",record.file))?;changes.push(format!("{}: {}",record.file,if module.enabled{"activado"}else{"desactivado para este inicio"}));}
  }Ok(changes)
 }
 pub fn owns(&self,id:&str)->bool{self.records.contains_key(id)}
 pub fn confirm_compatibility(&self,dir:&Path,game:&str,loader:&str)->Result<(),String>{
  if let Some(record)=self.records.get("hit_color"){
   if crate::addon_compatibility::blocked(game,loader,"hit_color",&record.sha1)&&file_hash(&dir.join("mods").join(&record.file)).is_some_and(|hash|hash.eq_ignore_ascii_case(&record.sha1)){
    return Err(format!("No se pudo pausar {} porque ya existe su archivo de respaldo. Desactiva esa copia desde Mis mods para evitar el fallo Sampler1. No se sobrescribió ningún archivo.",record.file));
   }
  }Ok(())
 }
 pub fn installed(&mut self,dir:&Path,id:&str,file:&str,sha1:&str)->Result<(),String>{if !valid_file(file)||file_hash(&dir.join("mods").join(file)).as_deref()!=Some(sha1){return Err("No se pudo verificar el complemento instalado".into());}self.records.insert(id.into(),Record{file:file.into(),sha1:sha1.into()});fs::create_dir_all(self.path.parent().ok_or("Falta la carpeta de configuración")?).map_err(|e|e.to_string())?;let tmp=self.path.with_extension(format!("{}.pending",uuid::Uuid::new_v4()));fs::write(&tmp,serde_json::to_vec_pretty(&self.records).map_err(|e|e.to_string())?).and_then(|_|fs::rename(&tmp,&self.path)).map_err(|e|{let _=fs::remove_file(&tmp);e.to_string()})}
}
#[cfg(test)]mod tests{use super::*;#[test]fn managed_addons_never_disable_user_files(){let dir=std::env::temp_dir().join(format!("tp-addon-{}",uuid::Uuid::new_v4()));fs::create_dir_all(dir.join("mods")).unwrap();fs::write(dir.join("mods/owned.jar"),b"owned").unwrap();fs::write(dir.join("mods/user.jar"),b"user").unwrap();let mut store=Store::load(&dir).unwrap();let hash=file_hash(&dir.join("mods/owned.jar")).unwrap();store.installed(&dir,"sodium","owned.jar",&hash).unwrap();let mut profile=crate::launcher_plugins::HudProfile::default();store.reconcile(&dir,&profile).unwrap();assert!(dir.join("mods/owned.jar.disabled").exists());assert!(dir.join("mods/user.jar").exists());profile.modules.iter_mut().find(|m|m.id=="sodium").unwrap().enabled=true;store.reconcile(&dir,&profile).unwrap();assert!(dir.join("mods/owned.jar").exists());fs::write(dir.join("mods/owned.jar"),b"user modified").unwrap();profile.modules.iter_mut().find(|m|m.id=="sodium").unwrap().enabled=false;store.reconcile(&dir,&profile).unwrap();assert!(dir.join("mods/owned.jar").exists());let actual=dir.canonicalize().unwrap();assert_eq!(actual.parent().unwrap(),std::env::temp_dir().canonicalize().unwrap());assert!(actual.file_name().unwrap().to_string_lossy().starts_with("tp-addon-"));fs::remove_dir_all(actual).unwrap();}}

#[cfg(test)]mod compatibility_tests {
 use super::*;
 #[test]fn incompatible_owned_artifact_is_disabled_without_changing_the_saved_selection(){
  let dir=std::env::temp_dir().join(format!("tp-compat-{}",uuid::Uuid::new_v4()));fs::create_dir_all(dir.join("mods")).unwrap();
  fs::write(dir.join("mods/owned.jar"),b"incompatible fixture").unwrap();fs::write(dir.join("mods/user.jar"),b"unrelated user mod").unwrap();
  let hash=file_hash(&dir.join("mods/owned.jar")).unwrap();let mut store=Store::load(&dir).unwrap();store.installed(&dir,"hit_color","owned.jar",&hash).unwrap();
  let mut selected=crate::launcher_plugins::HudProfile::default();selected.modules.iter_mut().find(|m|m.id=="hit_color").unwrap().enabled=true;
  let original=serde_json::to_string(&selected).unwrap();let(effective,warnings)=store.profile_without_artifact(&dir,&selected,&hash).unwrap();assert_eq!(warnings.len(),1);
  store.reconcile(&dir,&effective).unwrap();assert!(dir.join("mods/owned.jar.disabled").exists());assert!(!dir.join("mods/owned.jar").exists());assert!(dir.join("mods/user.jar").exists());assert_eq!(original,serde_json::to_string(&selected).unwrap());
  let(effective,_)=store.profile_without_artifact(&dir,&selected,&hash).unwrap();store.reconcile(&dir,&effective).unwrap();assert!(!dir.join("mods/owned.jar").exists());
  fs::write(dir.join("mods/renamed-user.jar"),b"incompatible fixture").unwrap();assert!(store.profile_without_artifact(&dir,&selected,&hash).is_err());assert_eq!(fs::read(dir.join("mods/renamed-user.jar")).unwrap(),b"incompatible fixture");
  let actual=dir.canonicalize().unwrap();assert_eq!(actual.parent().unwrap(),std::env::temp_dir().canonicalize().unwrap());assert!(actual.file_name().unwrap().to_string_lossy().starts_with("tp-compat-"));fs::remove_dir_all(actual).unwrap();
 }
}
