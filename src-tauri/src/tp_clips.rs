use serde::Serialize;
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::{Arc, Mutex, OnceLock, atomic::{AtomicBool,Ordering}}, time::{Duration,Instant}};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[derive(Clone,Debug,Default,Serialize)]
pub struct Recording { pub active:bool, pub started_at:u64, pub duration_seconds:u16, pub pid:u32, pub message:String, pub error:bool }
#[derive(Serialize)]
pub struct Clip {pub id:String,pub size_bytes:u64,pub modified_ms:u64}
#[derive(Serialize)]
pub struct Workspace {pub supported:bool,pub ffmpeg_available:bool,pub ffprobe_available:bool,pub game_available:bool,pub recording:Recording,pub clips:Vec<Clip>,pub note:String}
struct State {recording:Recording,stop:Option<Arc<AtomicBool>>}
fn state()->&'static Mutex<State>{static VALUE:OnceLock<Mutex<State>>=OnceLock::new();VALUE.get_or_init(||Mutex::new(State{recording:Recording::default(),stop:None}))}
fn binary(name:&str)->Option<PathBuf>{
    let exe=if cfg!(windows){format!("{name}.exe")}else{name.into()};
    std::env::split_paths(&std::env::var_os("PATH")?).filter(|p|p.is_absolute()).map(|dir|dir.join(&exe)).find(|p|p.is_file())
}
fn clips_dir()->Result<PathBuf,String>{let dir=crate::launcher_settings::data_root()?.join("clips");if fs::symlink_metadata(&dir).is_ok_and(|m|m.file_type().is_symlink()){return Err("La carpeta clips no puede ser un enlace.".into());}fs::create_dir_all(&dir).map_err(|e|e.to_string())?;Ok(dir)}
fn id_valid(id:&str)->bool{id.strip_suffix(".mp4").is_some_and(|s|uuid::Uuid::parse_str(s).is_ok())&&id.len()==40}
fn clip_path(id:&str)->Result<PathBuf,String>{if !id_valid(id){return Err("Identificador de clip inválido.".into());}let path=clips_dir()?.join(id);let meta=fs::symlink_metadata(&path).map_err(|_|"El clip ya no existe.")?;if !meta.is_file()||meta.file_type().is_symlink(){return Err("El clip no es un archivo regular.".into());}Ok(path)}
fn valid_options(seconds:u16,fps:u16,quality:&str)->bool{[15,30,60,120].contains(&seconds)&&[30,60].contains(&fps)&&["720p","1080p"].contains(&quality)}
fn output_args(path:&Path,seconds:u16,fps:u16,quality:&str)->Vec<String>{
    let height=if quality=="1080p"{1080}else{720};
    let width=if quality=="1080p"{1920}else{1280};
    vec!["-t".into(),seconds.to_string(),"-an".into(),"-vf".into(),format!("scale=w='min(iw,{width})':h='min(ih,{height})':force_original_aspect_ratio=decrease,pad=ceil(iw/2)*2:ceil(ih/2)*2"),"-r".into(),fps.to_string(),"-c:v".into(),"libx264".into(),"-preset".into(),"ultrafast".into(),"-crf".into(),"25".into(),"-threads".into(),"2".into(),"-pix_fmt".into(),"yuv420p".into(),"-movflags".into(),"+faststart".into(),path.to_string_lossy().into_owned()]
}
fn verify_video(probe:&Path,path:&Path)->Result<(),String>{
    let output=crate::process_probe::run(probe,&["-v","error","-show_entries","format=duration:stream=codec_type,codec_name","-of","json",path.to_str().ok_or("Ruta de clip inválida.")?],Duration::from_secs(5)).map_err(|_|"No se pudo verificar el MP4 con ffprobe.")?;
    if !output.status.success(){return Err("ffprobe no pudo abrir el MP4 final.".into());}
    let value:serde_json::Value=serde_json::from_slice(&output.stdout).map_err(|_|"Informe ffprobe inválido.")?;
    let streams=value["streams"].as_array().ok_or("El MP4 no contiene pistas válidas.")?;
    if !streams.iter().any(|s|s["codec_type"]=="video"&&s["codec_name"]=="h264")||streams.iter().any(|s|s["codec_type"]=="audio")||value.pointer("/format/duration").and_then(serde_json::Value::as_str).and_then(|s|s.parse::<f64>().ok()).is_none_or(|v|!v.is_finite()||v<=0.){return Err("El archivo no contiene vídeo H.264 válido sin audio.".into());}
    Ok(())
}
fn list()->Result<Vec<Clip>,String>{let mut rows=Vec::new();for entry in fs::read_dir(clips_dir()?).map_err(|e|e.to_string())?.take(1024).filter_map(Result::ok){let id=entry.file_name().to_string_lossy().into_owned();if !id_valid(&id){continue;}let Ok(meta)=fs::symlink_metadata(entry.path())else{continue};if !meta.is_file()||meta.file_type().is_symlink(){continue;}rows.push(Clip{id,size_bytes:meta.len(),modified_ms:meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d|d.as_millis()as u64).unwrap_or(0)});}rows.sort_by_key(|r|std::cmp::Reverse(r.modified_ms));Ok(rows)}
fn workspace()->Result<Workspace,String>{Ok(Workspace{supported:cfg!(windows),ffmpeg_available:binary("ffmpeg").is_some(),ffprobe_available:binary("ffprobe").is_some(),game_available:!crate::game_session::tracked_pids().is_empty(),recording:state().lock().map_err(|_|"Grabación ocupada.")?.recording.clone(),clips:list()?,note:"Grabación manual de los próximos segundos, solo de la ventana Minecraft seguida por TP. Apagada por defecto; sin audio, micrófono ni subida. Usa FFmpeg externo y H.264 por CPU: su impacto no está medido. No es un búfer instantáneo de los segundos anteriores ni captura WGC/hardware. Pantalla completa exclusiva, ventana minimizada u ocluida pueden producir imagen negra; usa modo ventana o sin bordes.".into()})}
#[tauri::command]
pub async fn tp_clips_workspace()->Result<Workspace,String>{tauri::async_runtime::spawn_blocking(workspace).await.map_err(|e|e.to_string())?}
fn finish(message:String,error:bool){if let Ok(mut state)=state().lock(){state.recording.active=false;state.recording.message=message;state.recording.error=error;state.stop=None;}}
#[cfg(windows)]
struct CaptureJob(windows_sys::Win32::Foundation::HANDLE);
#[cfg(windows)]
impl CaptureJob {
    fn assign(child: &std::process::Child) -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() { return Err("No se pudo proteger el ciclo de vida de la captura".into()); }
        let job = Self(handle);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if unsafe { SetInformationJobObject(handle, JobObjectExtendedLimitInformation, (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(), std::mem::size_of_val(&limits) as u32) } == 0
            || unsafe { AssignProcessToJobObject(handle, child.as_raw_handle().cast()) } == 0 {
            return Err("Windows no pudo vincular la grabación al launcher; no se iniciará la captura".into());
        }
        Ok(job)
    }
}
#[cfg(windows)]
impl Drop for CaptureJob { fn drop(&mut self) { unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0); } } }
#[cfg(windows)]
fn record(ffmpeg:PathBuf,probe:PathBuf,window:crate::hud_windows::GameWindow,seconds:u16,fps:u16,quality:String,stop:Arc<AtomicBool>)->Result<String,String>{
    use std::os::windows::process::CommandExt;
    let folder=clips_dir()?;let id=format!("{}.mp4",uuid::Uuid::new_v4());let final_path=folder.join(&id);let pending=folder.join(format!(".{id}.part.mp4"));
    let mut command=Command::new(ffmpeg);command.args(["-hide_banner","-loglevel","error","-n","-f","gdigrab","-framerate",&fps.to_string(),"-draw_mouse","0","-i",&format!("hwnd={}",window.hwnd)]).args(output_args(&pending,seconds,fps,&quality)).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped()).creation_flags(0x08000000);
    let mut child=command.spawn().map_err(|_|"No se pudo iniciar FFmpeg. Comprueba que admita gdigrab y libx264.")?;
    let _job=match CaptureJob::assign(&child) { Ok(job)=>job, Err(error)=>{let _=child.kill();let _=child.wait();return Err(error);} };
    let stderr=child.stderr.take().ok_or("No se pudo abrir la salida de FFmpeg.")?;
    let reader=std::thread::spawn(move||{let mut pipe=stderr;let mut block=[0;4096];while pipe.read(&mut block).is_ok_and(|n|n>0){} });
    let start=Instant::now();let mut requested_stop=None;
    let outcome=loop{
        match child.try_wait(){Ok(Some(exit))=>break if exit.success(){Ok(())}else{Err("FFmpeg no pudo completar la captura. Comprueba su soporte gdigrab/libx264 y el modo de ventana de Minecraft.".into())},Err(_)=>break Err("No se pudo comprobar el proceso de grabación.".into()),Ok(None)=>{}}
        let wrong_target=!crate::game_session::tracked_pids().contains(&window.pid)||!crate::hud_windows::valid(window);
        let too_large=fs::metadata(&pending).is_ok_and(|m|m.len()>1024*1024*1024);
        if requested_stop.is_none()&&(stop.load(Ordering::Relaxed)||wrong_target||too_large||start.elapsed()>Duration::from_secs(seconds as u64+15)){
            if let Some(input)=&mut child.stdin{let _=input.write_all(b"q\n");let _=input.flush();}requested_stop=Some(Instant::now());
        }
        if requested_stop.is_some_and(|t|t.elapsed()>Duration::from_secs(5)){let _=child.kill();let _=child.wait();break Err("La captura no pudo finalizar a tiempo; no se publicó un clip incompleto.".into());}
        std::thread::sleep(Duration::from_millis(100));
    };
    let _=child.kill();let _=child.wait();let _=reader.join();
    let result=outcome.and_then(|_|verify_video(&probe,&pending)).and_then(|_|fs::rename(&pending,&final_path).map_err(|_|"No se pudo guardar el clip verificado.".into()));
    if result.is_err(){let _=fs::remove_file(&pending);}result.map(|_|format!("Clip guardado y verificado: {id}"))
}
#[tauri::command]
pub async fn tp_clips_start(seconds:u16,fps:u16,quality:String,consent:bool)->Result<Recording,String>{
    if !consent{return Err("Confirma que quieres grabar la ventana de Minecraft antes de comenzar.".into());}
    if !valid_options(seconds,fps,&quality){return Err("Duración, calidad o FPS de grabación inválidos.".into());}
    #[cfg(not(windows))]{return Err("La captura de ventana está disponible solo en Windows.".into());}
    #[cfg(windows)]{
        let ffmpeg=binary("ffmpeg").ok_or("FFmpeg no está instalado en PATH; no se descargará automáticamente.")?;
        let probe=binary("ffprobe").ok_or("ffprobe no está instalado en PATH; hace falta para verificar el MP4.")?;
        let window=crate::hud_windows::find(&crate::game_session::tracked_pids()).ok_or("No hay una ventana Minecraft visible iniciada por TP para grabar.")?;
        let stop=Arc::new(AtomicBool::new(false));let result;
        {let mut state=state().lock().map_err(|_|"Grabación ocupada.")?;if state.recording.active{return Err("Ya hay una grabación en curso.".into());}
            state.recording=Recording{active:true,started_at:crate::launcher_overlay::now(),duration_seconds:seconds,pid:window.pid,message:"Grabando la ventana Minecraft, sin audio. Puedes detener para guardar antes.".into(),error:false};state.stop=Some(stop.clone());result=state.recording.clone();}
        std::thread::spawn(move||{match record(ffmpeg,probe,window,seconds,fps,quality,stop){Ok(message)=>finish(message,false),Err(message)=>finish(message,true)}});Ok(result)
    }
}
#[tauri::command]
pub fn tp_clips_stop()->Result<Recording,String>{let mut state=state().lock().map_err(|_|"Grabación ocupada.")?;if let Some(stop)=&state.stop{stop.store(true,Ordering::Relaxed);state.recording.message="Finalizando y verificando el clip…".into();}Ok(state.recording.clone())}
#[tauri::command]
pub fn tp_clips_open(app:AppHandle,id:String)->Result<(),String>{let path=clip_path(&id)?;app.opener().open_path(path.to_string_lossy().as_ref(),None::<&str>).map_err(|e|e.to_string())}
#[tauri::command]
pub fn tp_clips_folder(app:AppHandle)->Result<(),String>{app.opener().open_path(clips_dir()?.to_string_lossy().as_ref(),None::<&str>).map_err(|e|e.to_string())}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn options_and_paths_are_bounded(){assert!(valid_options(15,30,"720p"));for (s,f,q)in[(1,30,"720p"),(120,240,"720p"),(30,30,"native")]{assert!(!valid_options(s,f,q));}assert!(id_valid("6a4f8732-6bf7-49c3-a749-16607f97281d.mp4"));for id in["../../file.mp4","file.mp4","6a4f8732-6bf7-49c3-a749-16607f97281d.mp4.exe"]{assert!(!id_valid(id));}}
    #[test]fn encoding_has_no_audio_and_no_implicit_desktop_capture(){let args=output_args(Path::new("fixture.mp4"),15,30,"720p");assert!(args.contains(&"-an".into()));assert!(args.contains(&"libx264".into()));assert!(!args.iter().any(|arg|arg.contains("desktop")||arg.contains("dshow")||arg.contains("wasapi")));}
    #[tokio::test]async fn recording_requires_explicit_consent_before_any_process(){assert!(tp_clips_start(15,30,"720p".into(),false).await.unwrap_err().contains("Confirma"));}
    #[cfg(windows)]
    #[test]
    #[ignore="Subprocess fixture invoked only by capture_job_stops_its_child_on_drop"]
    fn capture_job_child_fixture(){if std::env::var("TP_CAPTURE_JOB_CHILD").as_deref()==Ok("1"){std::thread::sleep(Duration::from_secs(30));}}
    #[cfg(windows)]
    #[test]
    fn capture_job_stops_its_child_on_drop(){
        use std::os::windows::process::CommandExt;
        let mut child=Command::new(std::env::current_exe().unwrap()).args(["--exact","tp_clips::tests::capture_job_child_fixture","--ignored"]).env("TP_CAPTURE_JOB_CHILD","1").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(0x08000000).spawn().unwrap();
        let job=match CaptureJob::assign(&child){Ok(job)=>job,Err(error)=>{let _=child.kill();let _=child.wait();panic!("{error}");}};
        std::thread::sleep(Duration::from_millis(100));assert!(child.try_wait().unwrap().is_none());drop(job);
        let start=Instant::now();loop{if child.try_wait().unwrap().is_some(){break;}if start.elapsed()>Duration::from_secs(3){let _=child.kill();let _=child.wait();panic!("Job Object did not stop its child");}std::thread::sleep(Duration::from_millis(25));}
    }
    #[test]
    #[ignore="Explicit synthetic lavfi encode; does not capture a screen, camera, microphone or game"]
    fn ffmpeg_synthetic_mp4_is_verified_without_audio(){
        let ffmpeg=binary("ffmpeg").expect("FFmpeg required for explicit synthetic test");let probe=binary("ffprobe").expect("ffprobe required");let folder=std::env::temp_dir().join(format!("tp-clips-test-{}",uuid::Uuid::new_v4()));fs::create_dir(&folder).unwrap();let file=folder.join("fixture.mp4");
        let mut args=vec!["-hide_banner".to_string(),"-loglevel".into(),"error".into(),"-n".into(),"-f".into(),"lavfi".into(),"-i".into(),"testsrc2=size=320x180:rate=30".into()];let mut encode=output_args(&file,15,30,"720p");encode[1]="1".into();args.extend(encode);let refs=args.iter().map(String::as_str).collect::<Vec<_>>();let result=crate::process_probe::run(&ffmpeg,&refs,Duration::from_secs(15)).unwrap();assert!(result.status.success());verify_video(&probe,&file).unwrap();assert!(fs::metadata(&file).unwrap().len()>1024);
        let report=crate::process_probe::run(&probe,&["-v","error","-show_entries","stream=width,height","-of","json",file.to_str().unwrap()],Duration::from_secs(5)).unwrap();assert!(report.status.success());let value:serde_json::Value=serde_json::from_slice(&report.stdout).unwrap();assert_eq!(value["streams"][0]["width"],320);assert_eq!(value["streams"][0]["height"],180);
        let actual=folder.canonicalize().unwrap();assert_eq!(actual.parent().unwrap(),std::env::temp_dir().canonicalize().unwrap());assert!(actual.file_name().unwrap().to_string_lossy().starts_with("tp-clips-test-"));fs::remove_dir_all(actual).unwrap();
    }
}
