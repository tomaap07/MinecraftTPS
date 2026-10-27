
use serde::Serialize;
use std::{collections::BTreeMap,sync::{Mutex,OnceLock}};
use tauri::{AppHandle,Emitter};
use tokio::sync::watch;

#[derive(Clone,Default,Serialize)]
pub struct GameSessionSnapshot {pub running:bool,pub active_games:usize,pub username:String,pub version:String,pub server:String}
#[derive(Clone)]
struct Process {username:String,version:String,server:String,game_dir:std::path::PathBuf}
fn processes()->&'static Mutex<BTreeMap<u32,Process>>{static STATE:OnceLock<Mutex<BTreeMap<u32,Process>>>=OnceLock::new();STATE.get_or_init(||Mutex::new(BTreeMap::new()))}
fn notifier()->&'static watch::Sender<bool>{static WATCH:OnceLock<watch::Sender<bool>>=OnceLock::new();WATCH.get_or_init(||watch::channel(false).0)}
pub fn subscribe_running()->watch::Receiver<bool>{notifier().subscribe()}
pub fn is_running()->bool{*notifier().borrow()}
pub fn directory(pid:u32)->Option<std::path::PathBuf>{processes().lock().ok()?.get(&pid).map(|p|p.game_dir.clone())}
pub fn identity(pid:u32)->Option<(String,String)>{processes().lock().ok()?.get(&pid).map(|p|(p.username.clone(),p.version.clone()))}
pub fn tracked_pids()->Vec<u32>{processes().lock().map(|state|state.keys().copied().collect()).unwrap_or_default()}
#[tauri::command]
pub fn get_game_session()->GameSessionSnapshot{
    let Ok(state)=processes().lock()else{return GameSessionSnapshot::default()};
    let current=state.values().next_back();
    GameSessionSnapshot{running:!state.is_empty(),active_games:state.len(),username:current.map(|p|p.username.clone()).unwrap_or_default(),
        version:current.map(|p|p.version.clone()).unwrap_or_default(),server:current.map(|p|p.server.clone()).unwrap_or_default()}
}
fn publish(app:&AppHandle){
    let snapshot=get_game_session();notifier().send_replace(snapshot.running);
    let _=app.emit("tp-game-session",&snapshot);
    let _=super::launcher_overlay::publish_launcher_overlay(app.clone(),super::launcher_overlay::OverlayState{
        running:snapshot.running,username:snapshot.username,version:snapshot.version,server:snapshot.server,active_games:snapshot.active_games.min(128) as u32});
}
pub struct GameSessionGuard{app:AppHandle,pid:u32}
impl Drop for GameSessionGuard{fn drop(&mut self){if let Ok(mut state)=processes().lock(){state.remove(&self.pid);}publish(&self.app);}}
pub fn track(app:AppHandle,pid:u32,username:&str,version:&str,server:Option<&str>,game_dir:&std::path::Path)->GameSessionGuard{
    if let Ok(mut state)=processes().lock(){state.insert(pid,Process{username:username.into(),version:version.into(),server:server.unwrap_or_default().into(),game_dir:game_dir.to_path_buf()});}
    publish(&app);GameSessionGuard{app,pid}
}
