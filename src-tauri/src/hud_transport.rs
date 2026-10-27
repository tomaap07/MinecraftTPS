
use std::{collections::BTreeMap,net::UdpSocket,sync::{Mutex,OnceLock},time::{Duration,Instant}};
struct Endpoint{port:u16,token:String}
fn samples()->&'static Mutex<BTreeMap<u32,(Instant,Vec<u8>)>>{static DATA:OnceLock<Mutex<BTreeMap<u32,(Instant,Vec<u8>)>>>=OnceLock::new();DATA.get_or_init(||Mutex::new(BTreeMap::new()))}
pub fn endpoint()->Result<(u16,String),String>{
    static ENDPOINT:OnceLock<Result<Endpoint,String>>=OnceLock::new();
    let endpoint=ENDPOINT.get_or_init(||{
        let socket=UdpSocket::bind("127.0.0.1:0").map_err(|e|e.to_string())?;let port=socket.local_addr().map_err(|e|e.to_string())?.port();
        socket.set_read_timeout(Some(Duration::from_secs(1))).map_err(|e|e.to_string())?;
        let token=format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple());let expected=token.clone();
        std::thread::spawn(move||{let mut bytes=[0u8;32768];loop{let Ok((size,source))=socket.recv_from(&mut bytes)else{continue};if !source.ip().is_loopback(){continue;}
            let Ok(value)=serde_json::from_slice::<serde_json::Value>(&bytes[..size])else{continue};if value["token"].as_str()!=Some(&expected){continue;}
            let payload=&value["payload"];let Some(pid)=payload["pid"].as_u64().and_then(|n|u32::try_from(n).ok())else{continue};
            let tracked=crate::game_session::tracked_pids();if !tracked.contains(&pid){continue;}let Ok(data)=serde_json::to_vec(payload)else{continue};
            if let Ok(mut samples)=samples().lock(){samples.retain(|pid,(time,_)|tracked.contains(pid)&&time.elapsed()<Duration::from_secs(4));if samples.len()<32||samples.contains_key(&pid){samples.insert(pid,(Instant::now(),data));}}
        }});Ok(Endpoint{port,token})
    });
    endpoint.as_ref().map(|e|(e.port,e.token.clone())).map_err(Clone::clone)
}
pub fn latest(pid:u32)->Option<Vec<u8>>{let samples=samples().lock().ok()?;let(time,bytes)=samples.get(&pid)?;(time.elapsed()<Duration::from_secs(3)).then(||bytes.clone())}
