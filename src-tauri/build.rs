fn main() {


    println!("cargo:rerun-if-env-changed=VITE_MS_CLIENT_ID");
    let mut oauth_id=std::env::var("VITE_MS_CLIENT_ID").ok();
    for name in [".env.local",".env"]{
        let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("project root").join(name);
        println!("cargo:rerun-if-changed={}",path.display());
        if oauth_id.as_ref().is_some_and(|value|!value.trim().is_empty()){continue;}
        if let Ok(text)=std::fs::read_to_string(path){oauth_id=text.lines().filter_map(|line|line.trim().strip_prefix("VITE_MS_CLIENT_ID=")).map(|value|value.trim().trim_matches(['\"','\'']).to_owned()).find(|value|!value.is_empty());}
    }
    if let Some(value)=oauth_id.filter(|value|!value.trim().is_empty()){
        if value.len()!=36||!value.bytes().all(|b|b.is_ascii_hexdigit()||b==b'-'){panic!("VITE_MS_CLIENT_ID debe contener un UUID propio válido");}
        println!("cargo:rustc-env=TP_MS_CLIENT_ID={}",value.trim());
    }
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {



        tauri_build::try_build(
            tauri_build::Attributes::new()
                .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest()),
        )
        .expect("No se pudieron preparar los recursos de Windows");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    } else {
        tauri_build::build();
    }
}
