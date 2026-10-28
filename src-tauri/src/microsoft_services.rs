use futures_util::StreamExt;
use serde_json::{Value, json};

pub(crate) struct MinecraftToken {
    pub access_token: String,
    pub xuid: String,
    pub expires_in: u64,
}

struct Endpoints<'a> { xbox: &'a str, xsts: &'a str, minecraft: &'a str }
const ENDPOINTS: Endpoints<'static> = Endpoints {
    xbox: "https://user.auth.xboxlive.com/user/authenticate",
    xsts: "https://xsts.auth.xboxlive.com/xsts/authorize",
    minecraft: "https://api.minecraftservices.com/authentication/login_with_xbox",
};

pub(crate) fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("TP-LAUNCHER/", env!("CARGO_PKG_VERSION")))
        .build().map_err(|_| "No se pudo preparar la conexión de la cuenta".into())
}

fn rejected(stage: &str, status: u16, value: &Value) -> String {
    let detail = match (stage, status, value["XErr"].as_u64()) {
        ("Minecraft Services", 403, _) => "Microsoft y Xbox respondieron, pero Minecraft Services rechazó el acceso de TP LAUNCHER. Revisa la aprobación de tu Client ID para Minecraft Services en https://aka.ms/mce-reviewappid. Si ya está aprobado, revisa con soporte el rechazo del servicio. La sesión de Minecraft no se completó.",
        ("Xbox XSTS", _, Some(2148916233)) => "La cuenta necesita un perfil Xbox. Completa su configuración en xbox.com y vuelve a conectar.",
        ("Xbox XSTS", _, Some(2148916238)) => "Xbox solicita revisar la configuración de familia y los permisos de esta cuenta.",
        (_, 429, _) => "El servicio limitó temporalmente las solicitudes. Espera antes de intentarlo otra vez.",
        (_, 500..=599, _) => "El servicio no está disponible temporalmente. Inténtalo más tarde.",
        _ => "El servicio rechazó la autorización. Revisa los permisos de la aplicación y la configuración de la cuenta.",
    };
    format!("{stage} (HTTP {status}): {detail}")
}

async fn response_json(response: reqwest::Response, stage: &str) -> Result<Value, String> {
    let status = response.status();
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| format!("{stage}: respuesta incompleta"))?;
        if bytes.len() + chunk.len() > 65536 { return Err(format!("{stage}: respuesta demasiado grande")); }
        bytes.extend_from_slice(&chunk);
    }
    let value = serde_json::from_slice(&bytes);
    if !status.is_success() { return Err(rejected(stage, status.as_u16(), &value.unwrap_or(Value::Null))); }
    value.map_err(|_| format!("{stage}: respuesta inválida"))
}

fn required(value: &Value, field: &str, stage: &str) -> Result<String, String> {
    value.pointer(field).and_then(Value::as_str).filter(|v| !v.is_empty()).map(str::to_owned)
        .ok_or_else(|| format!("{stage}: falta un dato de autorización requerido"))
}

pub(crate) async fn exchange(client: &reqwest::Client, token: &str, notify: impl Fn(&str)) -> Result<MinecraftToken, String> {
    exchange_with(client, token, &ENDPOINTS, notify).await
}

async fn exchange_with(client: &reqwest::Client, token: &str, endpoints: &Endpoints<'_>, notify: impl Fn(&str)) -> Result<MinecraftToken, String> {
    notify("Verificando la autorización de Xbox Live…");
    let mut xbl = None;
    for ticket in [format!("d={token}"), token.to_owned(), format!("t={token}")] {
        let response = client.post(endpoints.xbox)
            .header("Accept", "application/json").header("x-xbl-contract-version", "1")
            .json(&json!({"Properties":{"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":ticket},"RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"}))
            .send().await.map_err(|_| "Xbox Live: no se pudo conectar con el servicio")?;
        if matches!(response.status().as_u16(), 400 | 401) { continue; }
        xbl = Some(response_json(response, "Xbox Live").await?); break;
    }
    let xbl = xbl.ok_or("Xbox Live rechazó la autorización con los tres formatos de ticket. Revisa los permisos Xbox de la aplicación.")?;
    let xbl_token = required(&xbl, "/Token", "Xbox Live")?;
    notify("Verificando los permisos Xbox XSTS…");
    let response = client.post(endpoints.xsts)
        .header("Accept", "application/json").header("x-xbl-contract-version", "1")
        .json(&json!({"Properties":{"SandboxId":"RETAIL","UserTokens":[xbl_token]},"RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"}))
        .send().await.map_err(|_| "Xbox XSTS: no se pudo conectar con el servicio")?;
    let xsts = response_json(response, "Xbox XSTS").await?;
    let xsts_token = required(&xsts, "/Token", "Xbox XSTS")?;
    let user_hash = required(&xsts, "/DisplayClaims/xui/0/uhs", "Xbox XSTS")?;
    let xuid = xsts.pointer("/DisplayClaims/xui/0/xid").or_else(|| xbl.pointer("/DisplayClaims/xui/0/xid")).and_then(Value::as_str).unwrap_or_default().to_owned();
    notify("Conectando con Minecraft Services…");
    let response = client.post(endpoints.minecraft).header("Accept", "application/json")
        .json(&json!({"identityToken":format!("XBL3.0 x={user_hash};{xsts_token}")}))
        .send().await.map_err(|_| "Minecraft Services: no se pudo conectar con el servicio")?;
    let minecraft = response_json(response, "Minecraft Services").await?;
    let access_token = required(&minecraft, "/access_token", "Minecraft Services")?;
    let expires_in = minecraft["expires_in"].as_u64().filter(|v| *v > 0).ok_or("Minecraft Services: falta la duración de la sesión")?;
    Ok(MinecraftToken { access_token, xuid, expires_in })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn server(responses: Vec<(u16, Value)>) -> (String, tokio::task::JoinHandle<Vec<(String, Value)>>) {
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url=format!("http://{}",listener.local_addr().unwrap());
        let handle=tokio::spawn(async move {
            let mut requests=Vec::new();
            for (status, body) in responses {
                let (mut stream,_)=tokio::time::timeout(std::time::Duration::from_secs(3),listener.accept()).await.unwrap().unwrap();
                let mut bytes=Vec::new();let mut block=[0;2048];
                let request=loop {
                    let n=stream.read(&mut block).await.unwrap();assert!(n>0);bytes.extend_from_slice(&block[..n]);
                    if let Some(end)=bytes.windows(4).position(|v|v==b"\r\n\r\n") {
                        let headers=String::from_utf8(bytes[..end].to_vec()).unwrap();
                        let len:usize=headers.lines().find_map(|line|line.to_ascii_lowercase().strip_prefix("content-length:").map(|v|v.trim().parse().unwrap())).unwrap();
                        if bytes.len()>=end+4+len {break (headers,serde_json::from_slice(&bytes[end+4..end+4+len]).unwrap());}
                    }
                };
                requests.push(request);let body=body.to_string();
                stream.write_all(format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
            }
            requests
        });(url,handle)
    }
    fn xbl() -> Value {json!({"Token":"xbl-fixture","DisplayClaims":{"xui":[{"uhs":"xbl-hash","xid":"42"}]}})}
    fn xsts() -> Value {json!({"Token":"xsts-fixture","DisplayClaims":{"xui":[{"uhs":"xsts-hash"}]}})}
    #[tokio::test]
    async fn chain_uses_xsts_claims_and_minecraft_expiry_with_required_headers() {
        let (url,server)=server(vec![(200,xbl()),(200,xsts()),(200,json!({"access_token":"minecraft-fixture","expires_in":7200}))]).await;
        let result=exchange_with(&client().unwrap(),"microsoft-fixture",&Endpoints{xbox:&url,xsts:&url,minecraft:&url}, |_|{}).await.unwrap();
        assert_eq!(result.expires_in,7200);assert_eq!(result.xuid,"42");assert_eq!(result.access_token,"minecraft-fixture");
        let calls=server.await.unwrap();assert_eq!(calls.len(),3);
        assert_eq!(calls[0].1["Properties"]["RpsTicket"],"d=microsoft-fixture");
        assert_eq!(calls[0].1["RelyingParty"],"http://auth.xboxlive.com");
        assert_eq!(calls[1].1["RelyingParty"],"rp://api.minecraftservices.com/");
        assert_eq!(calls[2].1["identityToken"],"XBL3.0 x=xsts-hash;xsts-fixture");
        for (i,(headers,_)) in calls.iter().enumerate(){let headers=headers.to_ascii_lowercase();assert!(headers.contains("content-type: application/json"));assert!(headers.contains("accept: application/json"));assert!(headers.contains("user-agent: tp-launcher/"));if i<2 {assert!(headers.contains("x-xbl-contract-version: 1"));}}
    }
    #[tokio::test]
    async fn minecraft_403_is_an_actionable_failure_and_never_leaks_body() {
        let (url,server)=server(vec![(200,xbl()),(200,xsts()),(403,json!({"errorMessage":"private-fixture-never-display"}))]).await;
        let stages=Arc::new(Mutex::new(Vec::new()));let seen=stages.clone();
        let result=exchange_with(&client().unwrap(),"microsoft-fixture",&Endpoints{xbox:&url,xsts:&url,minecraft:&url}, move |v|seen.lock().unwrap().push(v.to_owned())).await;
        let error=result.err().unwrap();assert!(error.contains("Minecraft Services (HTTP 403)"));assert!(error.contains("https://aka.ms/mce-reviewappid"));assert!(!error.contains("private-fixture"));assert_eq!(stages.lock().unwrap().len(),3);server.await.unwrap();
    }
    #[tokio::test]
    async fn rejected_tickets_retry_but_xsts_failure_stops_before_minecraft() {
        let (url,server)=server(vec![(401,json!({})),(400,json!({})),(200,xbl()),(401,json!({"XErr":2148916233u64}))]).await;
        let error=exchange_with(&client().unwrap(),"fixture",&Endpoints{xbox:&url,xsts:&url,minecraft:&url}, |_|{}).await.err().unwrap();assert!(error.contains("perfil Xbox"));
        let calls=server.await.unwrap();assert_eq!(calls.len(),4);assert_eq!(calls[1].1["Properties"]["RpsTicket"],"fixture");assert_eq!(calls[2].1["Properties"]["RpsTicket"],"t=fixture");
    }
    #[tokio::test]
    async fn incomplete_success_is_not_a_minecraft_session() {
        let (url,server)=server(vec![(200,xbl()),(200,xsts()),(200,json!({"access_token":"fixture"}))]).await;
        assert!(exchange_with(&client().unwrap(),"fixture",&Endpoints{xbox:&url,xsts:&url,minecraft:&url}, |_|{}).await.is_err());server.await.unwrap();
    }
}
