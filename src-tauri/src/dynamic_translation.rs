use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::{collections::BTreeMap,sync::{Mutex,OnceLock},time::Duration};

#[derive(Clone,Serialize,Deserialize)]
struct Entry {text:String,created:u64}
#[derive(Default,Serialize,Deserialize)]
struct Disk {day:u64,characters:u64,entries:BTreeMap<String,Entry>}
#[derive(Serialize)]
pub struct Translation {text:String,translated:bool,cached:bool,status:String}
fn now()->u64 {crate::unix_time_seconds()}
fn path()->Result<std::path::PathBuf,String>{Ok(crate::launcher_settings::config_root()?.join("translation-cache.json"))}
fn load()->Disk{
    path().ok().and_then(|path|std::fs::metadata(&path).ok().filter(|meta|meta.len()<3*1024*1024).and_then(|_|std::fs::read(path).ok())).and_then(|bytes|serde_json::from_slice(&bytes).ok()).unwrap_or_default()
}
fn cache()->&'static Mutex<Disk>{static CACHE:OnceLock<Mutex<Disk>>=OnceLock::new();CACHE.get_or_init(||Mutex::new(load()))}
fn save(disk:&Disk){
    let Ok(path)=path()else{return;};let temp=path.with_extension("pending");
    if let Ok(bytes)=serde_json::to_vec(disk){if bytes.len()<=3*1024*1024&&std::fs::write(&temp,bytes).is_ok(){let _=std::fs::rename(temp,path);}}
}
fn target(code:&str)->Option<&'static str>{
    match code.to_ascii_lowercase().split('-').next().unwrap_or_default(){"es"=>Some("es"),"pt"=>Some("pt-BR"),"fr"=>Some("fr"),"de"=>Some("de"),"ru"=>Some("ru"),"it"=>Some("it"),"zh"=>Some("zh-CN"),"ja"=>Some("ja"),"ko"=>Some("ko"),"ca"=>Some("ca"),"eo"=>Some("eo"),_=>None}
}
fn chunks(text:&str)->Vec<String>{
    let mut result=Vec::new();let mut rest=text;
    while !rest.is_empty(){
        let mut end=rest.len().min(450);while !rest.is_char_boundary(end){end-=1;}
        if end<rest.len(){if let Some((index,character))=rest[..end].char_indices().rev().find(|(index,character)|*index>200&&character.is_whitespace()){end=index+character.len_utf8();}}
        result.push(rest[..end].to_owned());rest=&rest[end..];
    }
    result
}
fn key(text:&str,target:&str)->String{format!("{:x}",Sha256::digest(format!("{target}\0{text}")))}
fn decoded(value:&serde_json::Value)->Option<String>{
    if value["quotaFinished"].as_bool()==Some(true)||value["responseStatus"].as_u64()!=Some(200){return None;}
    let text=value.pointer("/responseData/translatedText")?.as_str()?;
    if text.trim().is_empty()||text.contains("MYMEMORY WARNING")||text.len()>48_000{return None;}
    let mut text=text.to_owned();for(a,b)in[("&quot;","\""),("&#39;","'"),("&apos;","'"),("&lt;","<"),("&gt;",">"),("&amp;","&")]{text=text.replace(a,b);}Some(text)
}
#[tauri::command]
pub async fn translate_public_text(text:String,target_language:String)->Translation{
    let fallback=|status:&str|Translation{text:text.clone(),translated:false,cached:false,status:status.into()};
    let Some(language)=target(&target_language)else{return fallback("original");};
    if text.trim().is_empty()||text.chars().count()>3500||text.contains('\0'){return fallback("original");}
    static QUEUE:OnceLock<tokio::sync::Mutex<()>>=OnceLock::new();
    let _queue=QUEUE.get_or_init(||tokio::sync::Mutex::new(())).lock().await;
    let key=key(&text,language);
    {
        let Ok(mut disk)=cache().lock()else{return fallback("unavailable");};
        disk.entries.retain(|_,entry|now().saturating_sub(entry.created)<30*86400);
        if let Some(entry)=disk.entries.get(&key){return Translation{text:entry.text.clone(),translated:true,cached:true,status:"translated".into()};}
        if disk.day!=now()/86400{disk.day=now()/86400;disk.characters=0;}
        let count=text.chars().count() as u64;
        if disk.characters+count>5000{return fallback("limit");}
        disk.characters+=count;save(&disk);
    }
    let request=async{
        let client=reqwest::Client::builder().timeout(Duration::from_secs(4)).user_agent("TP-LAUNCHER").build().ok()?;
        let mut translated=String::new();
        for chunk in chunks(&text){
            let leading=chunk.len()-chunk.trim_start().len();let trailing=chunk.trim_end().len();
            if leading>=trailing{translated.push_str(&chunk);continue;}
            let response=client.get("https://api.mymemory.translated.net/get").query(&[("q",chunk[leading..trailing].to_owned()),("langpair",format!("en|{language}"))]).send().await.ok()?;
            if !response.status().is_success()||response.content_length().is_some_and(|length|length>256_000){return None;}
            let mut bytes=Vec::new();let mut response=response;
            while let Some(part)=response.chunk().await.ok()?{if bytes.len()+part.len()>256_000{return None;}bytes.extend_from_slice(&part);}
            let value:serde_json::Value=serde_json::from_slice(&bytes).ok()?;
            translated.push_str(&chunk[..leading]);translated.push_str(decoded(&value)?.trim());translated.push_str(&chunk[trailing..]);
        }
        Some(translated)
    };
    let Ok(Some(result))=tokio::time::timeout(Duration::from_secs(9),request).await else{return fallback("unavailable");};
    if let Ok(mut disk)=cache().lock(){
        while disk.entries.len()>=500{let Some(oldest)=disk.entries.iter().min_by_key(|(_,entry)|entry.created).map(|(key,_)|key.clone())else{break;};disk.entries.remove(&oldest);}
        disk.entries.insert(key,Entry{text:result.clone(),created:now()});save(&disk);
    }
    Translation{text:result,translated:true,cached:false,status:"translated".into()}
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn respects_byte_limit_and_keeps_unicode_intact(){let text="A Minecraft shader 世界 🌍 ".repeat(130);let parts=chunks(&text);assert!(parts.iter().all(|part|part.len()<=450));assert_eq!(parts.concat(),text);}
    #[test]fn splits_long_sentences_at_whitespace(){let text="Minecraft resource packs provide custom visuals. ".repeat(20);let parts=chunks(&text);assert!(parts[..parts.len()-1].iter().all(|part|part.ends_with(char::is_whitespace)));assert_eq!(parts.concat(),text);}
    #[test]fn separates_languages_and_rejects_quota_error_content(){assert_ne!(key("Shaders","es"),key("Shaders","pt-BR"));assert_eq!(target("pt-BR"),Some("pt-BR"));assert_eq!(target("en-US"),None);assert!(decoded(&serde_json::json!({"responseStatus":429,"responseData":{"translatedText":"limit"}})).is_none());assert_eq!(decoded(&serde_json::json!({"responseStatus":200,"responseData":{"translatedText":"Mods &amp; Shaders"}})).as_deref(),Some("Mods & Shaders"));}
    #[tokio::test]
    #[ignore="Queries MyMemory with a public example and writes only its translation cache"]
    async fn public_description_translation_and_disk_cache(){
        let text="This shader pack adds soft lighting.".to_owned();
        let first=translate_public_text(text.clone(),"es-419".into()).await;
        assert!(first.translated,"Public translation unavailable: {}",first.status);
        let second=translate_public_text(text,"es-419".into()).await;
        assert!(second.cached);assert_eq!(second.text,first.text);
    }
}
