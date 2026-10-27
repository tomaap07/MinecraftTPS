#[tauri::command]
pub fn get_system_locale() -> String {
    #[cfg(windows)] {
        let mut name=[0u16;85];
        let length=unsafe {windows_sys::Win32::Globalization::GetUserDefaultLocaleName(name.as_mut_ptr(),name.len() as i32)};
        if length>1 {return String::from_utf16_lossy(&name[..length as usize-1]);}
    }
    std::env::var("LC_ALL").or_else(|_|std::env::var("LANG")).ok().and_then(|value|value.split('.').next().map(|value|value.replace('_',"-"))).filter(|value|!value.is_empty()&&value!="C").unwrap_or_else(||"es-419".into())
}

#[tauri::command]
pub async fn available_java_versions() -> Vec<u8> {
    let fallback=vec![8,11,17,21,22,25];
    let Ok(client)=reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).user_agent("TP-LAUNCHER").build() else {return fallback;};
    let Ok(response)=client.get("https://api.adoptium.net/v3/info/available_releases").send().await else{return fallback;};
    if !response.status().is_success(){return fallback;}
    let Ok(value)=response.json::<serde_json::Value>().await else{return fallback;};
    let mut versions=value["available_releases"].as_array().into_iter().flatten().filter_map(|value|value.as_u64()).filter(|value|(8..=40).contains(value)).map(|value|value as u8).collect::<Vec<_>>();
    versions.sort_unstable();versions.dedup();
    if versions.is_empty(){fallback}else{versions}
}
