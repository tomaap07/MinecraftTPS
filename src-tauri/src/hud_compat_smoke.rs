
use crate::*;

#[test]
#[ignore = "Launches two isolated Minecraft 26.3 profiles; requires the installed diagnostic source"]
fn hit_color_263_real_client_ab() {
    let root = root_dir().unwrap();
    let source = root.join("instances/Fariiiiuh/profiles/fabric-loader-0.20.1-26.3");
    assert!(source.join("mods/fabric-26.3-0.1.jar").is_file(), "Diagnostic source is unavailable");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    let app = tauri::Builder::default().any_thread().build(context).unwrap();
    for (name, bad) in [("TPHudBad263", true), ("TPHudGood263", false)] {
        let dir = version_game_dir(&root, name, "fabric-loader-0.20.1-26.3").unwrap();
        assert!(!dir.join("saves").exists(), "Test directory contains worlds");
        fs::create_dir_all(dir.join("mods")).unwrap();
        for entry in fs::read_dir(source.join("mods")).unwrap().take(512).flatten() {
            let path = entry.path();
            let meta = fs::symlink_metadata(&path).unwrap();
            if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 128 * 1024 * 1024 || path.extension().is_none_or(|ext| ext != "jar") { continue; }
            let bytes = fs::read(&path).unwrap();
            if !bad && format!("{:x}", Sha1::digest(&bytes)) == addon_compatibility::HIT_COLOR_263_SHA1 { continue; }
            fs::write(dir.join("mods").join(entry.file_name()), bytes).unwrap();
        }
        fs::write(dir.join("options.txt"), "fullscreen:false\npauseOnLostFocus:false\nresourcePacks:[]\n").unwrap();
        let log = dir.join("logs/latest.log");

        if log.exists() {fs::rename(&log, dir.join(format!("logs/previous-{}.log",Uuid::new_v4()))).unwrap();}
        runtime.block_on(launch_game_session(app.handle().clone(),None,name.into(),"fabric-loader-0.20.1-26.3".into(),4,None,Some(false),None)).unwrap();
        struct StopTest(&'static str);
        impl Drop for StopTest {fn drop(&mut self){for pid in launcher_tools::test_session_pids(self.0){let _=Command::new("taskkill").args(["/PID",&pid.to_string(),"/T","/F"]).output();}}}
        let stop = StopTest(name);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(100);
        let mut sound_at=None;
        let mut text=String::new();
        while std::time::Instant::now()<deadline {
            text=fs::read_to_string(&log).unwrap_or_default();
            if bad && text.contains("Unable to find shader defined uniform (Sampler1)"){break;}
            if text.contains("Sound engine started") && sound_at.is_none(){sound_at=Some(std::time::Instant::now());}
            if sound_at.is_some_and(|at|at.elapsed().as_secs()>=18){break;}
            if launcher_tools::test_session_pids(name).is_empty(){break;}
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        let failed=text.contains("Unable to find shader defined uniform (Sampler1)");
        let alive=!launcher_tools::test_session_pids(name).is_empty();
        println!("{name}: Sampler1={failed}; sound={}; alive={alive}; log={}",sound_at.is_some(),log.display());
        drop(stop);
        std::thread::sleep(std::time::Duration::from_secs(2));
        if bad {assert!(failed,"The original failure was not reproduced");}
        else {assert!(!failed && sound_at.is_some() && alive,"The corrected profile failed: {}",text.lines().rev().take(18).collect::<Vec<_>>().join("\n"));}
    }
}
