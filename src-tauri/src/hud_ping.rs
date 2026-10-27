

use std::{collections::BTreeMap,sync::{Mutex,OnceLock},time::{Duration,Instant}};
struct Probe{address:String,started:Instant,value:Option<u32>}
fn probes()->&'static Mutex<BTreeMap<u32,Probe>>{static CACHE:OnceLock<Mutex<BTreeMap<u32,Probe>>>=OnceLock::new();CACHE.get_or_init(||Mutex::new(BTreeMap::new()))}
pub fn decorate(data:&mut crate::launcher_overlay::Telemetry){
 if !data.has_world{return;}
 if data.singleplayer{data.ping=Some(0);data.ping_source="Local".into();if !data.capabilities.iter().any(|v|v=="ping"){data.capabilities.push("ping".into());}return;}
 if data.ping.is_some(){data.ping_source="Cliente".into();}
 let Ok(address)=crate::servers::parse_address(&data.server_address)else{return};
 let key=address.display();let tracked=crate::game_session::tracked_pids();if !tracked.contains(&data.pid){return;}
 let mut cache=match probes().lock(){Ok(v)=>v,Err(_)=>return};cache.retain(|pid,_|tracked.contains(pid));
 if let Some(probe)=cache.get(&data.pid).filter(|p|p.address==key){
  if let Some(value)=probe.value{data.ping=Some(value);data.ping_source="TCP".into();if !data.capabilities.iter().any(|v|v=="ping"){data.capabilities.push("ping".into());}}
  if probe.started.elapsed()<Duration::from_secs(5){return;}
 }
 if cache.len()>=32&&!cache.contains_key(&data.pid){return;}
 let pid=data.pid;let started=Instant::now();cache.insert(pid,Probe{address:key.clone(),started,value:None});drop(cache);
 tauri::async_runtime::spawn(async move{
  let value=tokio::time::timeout(Duration::from_secs(3),async{
   let(host,port)=crate::servers::endpoint(&address).await;

   let endpoints:Vec<_>=tokio::net::lookup_host((host.as_str(),port)).await.ok()?.take(8).collect();
   for endpoint in endpoints{let clock=Instant::now();if let Ok(Ok(stream))=tokio::time::timeout(Duration::from_millis(1100),tokio::net::TcpStream::connect(endpoint)).await{drop(stream);return Some(clock.elapsed().as_millis().min(600000)as u32);}}None
  }).await.ok().flatten();
  if let Ok(mut cache)=probes().lock(){if let Some(probe)=cache.get_mut(&pid).filter(|p|p.address==key&&p.started==started){probe.value=value;}}
 });
}
