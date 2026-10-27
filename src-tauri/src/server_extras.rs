
use serde::Serialize;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::{net::TcpStream, time::timeout};

#[derive(Serialize)]
pub struct ServerLatency { pub address:String,pub latency_ms:u64,pub quality:String,pub checked_at:u64 }
#[tauri::command]
pub async fn measure_server_latency(address:String)->Result<ServerLatency,String>{
    crate::server_directory::while_idle(async {
        let parsed=crate::servers::parse_address(&address)?;
        let latency_ms=timeout(Duration::from_secs(3),async{
            let(host,port)=crate::servers::endpoint(&parsed).await;
            let addresses=tokio::net::lookup_host((host.as_str(),port)).await
                .map_err(|_|"No se pudo resolver el servidor.")?.take(8).collect::<Vec<_>>();
            if addresses.is_empty(){return Err("No se encontraron direcciones para el servidor.");}

            let start=Instant::now();
            let stream=TcpStream::connect(addresses.as_slice()).await
                .map_err(|_|"No se pudo conectar con el servidor desde tu equipo.")?;
            let measured=start.elapsed().as_millis().min(u64::MAX as u128)as u64;drop(stream);Ok(measured)
        }).await.map_err(|_|"La consulta TCP superó el límite de 3 segundos.")??;
        Ok(ServerLatency{address:parsed.display(),latency_ms,
            quality:if latency_ms<60{"good"}else if latency_ms<140{"fair"}else{"poor"}.into(),
            checked_at:SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u64::MAX as u128)as u64})
    }).await
}
#[derive(Serialize)]
pub struct HostingRecommendation {pub enabled_mods:u32,pub recommended_gb:u32,pub estimated:bool,pub explanation:String}
#[tauri::command]
pub fn get_hosting_recommendation(username:String,version_id:String,instance_id:Option<String>)->Result<HostingRecommendation,String>{
    if crate::game_session::is_running(){return Err("El cálculo de memoria se pausa mientras juegas.".into());}
    let directory=if let Some(id)=instance_id.filter(|id|!id.is_empty()){crate::mod_studio::pack_game_dir(&id)?}
        else{crate::version_game_dir(&crate::root_dir()?,&username,&version_id)?};
    let mods=directory.join("mods");
    let mut count=0u32;
    if mods.is_dir(){for entry in std::fs::read_dir(mods).map_err(|_|"No se pudo leer la carpeta de mods.")?.take(20000).flatten(){
        if entry.file_type().map(|kind|kind.is_file()).unwrap_or(false)
            &&entry.path().extension().and_then(|value|value.to_str()).is_some_and(|value|value.eq_ignore_ascii_case("jar")){count+=1;}
    }}
    let recommended_gb=match count{0=>2,1..=30=>4,31..=100=>6,101..=180=>8,181..=280=>10,_=>12};
    Ok(HostingRecommendation{enabled_mods:count,recommended_gb,estimated:true,
        explanation:"Estimación inicial para un grupo pequeño de amigos. El consumo real depende de los mods, jugadores, mundo y distancia de simulación; no garantiza ausencia de lag.".into()})
}
