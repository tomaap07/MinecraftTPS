use std::{io::{self,Read},path::Path,process::{Command,Output,Stdio},time::{Duration,Instant}};

pub(crate) fn run(path:&Path,args:&[&str],timeout:Duration)->io::Result<Output>{
    let mut command=Command::new(path);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]{use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
    let mut child=command.spawn()?;
    fn drain(mut pipe:impl Read+Send+'static)->std::thread::JoinHandle<Vec<u8>>{
        std::thread::spawn(move||{let mut bytes=Vec::new();let mut chunk=[0;4096];loop{match pipe.read(&mut chunk){Ok(0)|Err(_)=>break,Ok(length)=>{let keep=length.min((128*1024usize).saturating_sub(bytes.len()));bytes.extend_from_slice(&chunk[..keep]);}}}bytes})
    }
    let out=drain(child.stdout.take().ok_or_else(||io::Error::other("stdout unavailable"))?);
    let err=drain(child.stderr.take().ok_or_else(||io::Error::other("stderr unavailable"))?);
    let deadline=Instant::now()+timeout;
    let status=loop{
        match child.try_wait(){Ok(Some(status))=>break Ok(status),Err(error)=>{let _=child.kill();let _=child.wait();break Err(error)},Ok(None)=>{}}
        if Instant::now()>=deadline{let _=child.kill();let _=child.wait();break Err(io::Error::new(io::ErrorKind::TimedOut,"Java probe timed out"));}
        std::thread::sleep(Duration::from_millis(15));
    };
    let stdout=out.join().map_err(|_|io::Error::other("stdout reader failed"))?;
    let stderr=err.join().map_err(|_|io::Error::other("stderr reader failed"))?;
    Ok(Output{status:status?,stdout,stderr})
}

#[cfg(all(test,windows))]
mod tests{
    use super::*;
    #[link(name="kernel32")]
    unsafe extern "system"{fn GetConsoleWindow()->*mut std::ffi::c_void;}
    #[test]
    #[ignore="Child fixture invoked by the console and timeout regression tests"]
    fn child_fixture(){
        println!("console={}",!unsafe{GetConsoleWindow()}.is_null());
        if std::env::args().any(|arg|arg=="--show-output"){std::thread::sleep(Duration::from_secs(5));}
    }
    #[test]
    fn java_probes_have_no_console_window(){
        let exe=std::env::current_exe().unwrap();
        let output=run(&exe,&["--exact","process_probe::tests::child_fixture","--ignored","--nocapture"],Duration::from_secs(3)).unwrap();
        assert!(output.status.success());assert!(String::from_utf8_lossy(&output.stdout).contains("console=false"));
    }
    #[test]
    fn stalled_probe_is_terminated(){
        let exe=std::env::current_exe().unwrap();let start=Instant::now();
        let error=run(&exe,&["--exact","process_probe::tests::child_fixture","--ignored","--nocapture","--show-output"],Duration::from_millis(250)).unwrap_err();
        assert_eq!(error.kind(),io::ErrorKind::TimedOut);assert!(start.elapsed()<Duration::from_secs(3));
    }
}
