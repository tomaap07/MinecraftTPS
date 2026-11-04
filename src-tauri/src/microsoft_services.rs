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
    minecraft: "https://api.minecraftservices.com/launcher/login",
};

pub(crate) fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("TP-LAUNCHER/", env!("CARGO_PKG_VERSION")))
        .build().map_err(|_| "No se pudo preparar la conexión de la cuenta".into())
}

fn invalid_app_registration(value: &Value) -> bool {
    ["/errorMessage", "/message", "/error", "/reason", "/details/reason"].into_iter()
        .filter_map(|path| value.pointer(path).and_then(Value::as_str))
        .any(|message| { let message=message.trim();message.eq_ignore_ascii_case("INVALID_APP_REGISTRATION") || message.eq_ignore_ascii_case("Invalid app registration") || message.to_ascii_lowercase().starts_with("invalid app registration, see https://aka.ms/") })
}

fn rejected(stage: &str, status: u16, value: &Value) -> String {
    let detail = match (stage, status, value["XErr"].as_u64()) {
        ("Minecraft Services", 403, _) if value.pointer("/details/reason").and_then(Value::as_str) == Some("ACCOUNT_SUSPENDED") => "Minecraft Services informó una suspensión de la cuenta (ACCOUNT_SUSPENDED). Consulta al soporte de Minecraft; este rechazo no indica que falte aprobar la aplicación.",
        ("Minecraft Services", 403, _) if invalid_app_registration(value) => "Minecraft Services devolvió INVALID_APP_REGISTRATION. Compara el Client ID indicado abajo con el enviado en el formulario aprobado. Si coinciden, consulta al soporte de Minecraft adjuntando la aprobación y este diagnóstico. El launcher no puede modificar la lista de acceso del servicio.",
        ("Minecraft Services", 403, _) => "Minecraft Services devolvió un rechazo HTTP 403 sin un motivo reconocido. Este estado por sí solo no demuestra que falte aprobar la aplicación ni que la cuenta carezca de licencia. Consulta al soporte de Minecraft con este diagnóstico y la confirmación de aprobación.",
        ("Minecraft Services", 401, _) => "Minecraft Services no aceptó la autorización de Xbox. Vuelve a iniciar sesión con Microsoft para obtener una autorización nueva.",
        ("Xbox XSTS", _, Some(2148916233)) => "La cuenta necesita un perfil Xbox. Completa su configuración en xbox.com y vuelve a conectar.",
        ("Xbox XSTS", _, Some(2148916238)) => "Xbox solicita revisar la configuración de familia y los permisos de esta cuenta.",
        (_, 429, _) => "El servicio limitó temporalmente las solicitudes. Espera antes de intentarlo otra vez.",
        (_, 500..=599, _) => "El servicio no está disponible temporalmente. Inténtalo más tarde.",
        _ => "El servicio rechazó la autorización. Revisa los permisos de la aplicación y la configuración de la cuenta.",
    };
    let mut message = format!("{stage} (HTTP {status}): {detail}");
    if stage == "Minecraft Services" {
        if let Ok(client_id) = crate::microsoft_oauth::client_id() {
            message.push_str(&format!(" Client ID utilizado: {client_id}."));
        }
        message.push_str(" No se completó la sesión de Minecraft.");
        message.push_str(" Endpoint: /launcher/login.");
    }
    message
}

fn response_summary(content_type: &str, bytes: &[u8], value: Option<&Value>) -> String {
    let declared = match content_type.split(';').next().unwrap_or("").trim() {
        "application/json" => "application/json",
        "application/problem+json" => "application/problem+json",
        "text/html" => "text/html",
        "text/plain" => "text/plain",
        "" => "sin Content-Type",
        _ => "otro Content-Type",
    };
    let shape = match value {
        Some(Value::Object(_)) => "objeto JSON",
        Some(Value::Array(_)) => "array JSON",
        Some(_) => "valor JSON",
        None if bytes.is_empty() => "cuerpo vacío",
        None => "cuerpo no JSON",
    };
    let fields: Vec<_> = ["error", "errorMessage", "message", "details", "reason", "requestId", "traceId", "status", "path"]
        .into_iter().filter(|key| value.and_then(|v| v.as_object()).is_some_and(|o| o.contains_key(*key))).collect();
    let class = ["/error", "/errorType", "/details/reason", "/reason"]
        .into_iter().filter_map(|path| value?.pointer(path)?.as_str())
        .find(|code| matches!(*code, "ForbiddenOperationException" | "UnauthorizedOperationException" | "IllegalArgumentException" | "FORBIDDEN" | "Forbidden" | "Unauthorized" | "INVALID_APP_REGISTRATION" | "ACCOUNT_SUSPENDED"));
    let mut summary = format!(" Respuesta: {declared}, {shape}, {} bytes. Campos conocidos: {}.", bytes.len(), if fields.is_empty() { "ninguno".into() } else { fields.join(", ") });
    if let Some(code) = class { summary.push_str(&format!(" Código del servicio: {code}.")); }
    summary
}

async fn response_json(response: reqwest::Response, stage: &str) -> Result<Value, String> {
    let status = response.status();
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v|v.to_str().ok()).unwrap_or("").to_owned();
    let request_id = ["x-request-id", "x-ms-request-id", "request-id"]
        .into_iter().filter_map(|name| response.headers().get(name)?.to_str().ok())
        .find_map(|value| uuid::Uuid::parse_str(value).ok());
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| format!("{stage}: respuesta incompleta"))?;
        if bytes.len() + chunk.len() > 65536 { return Err(format!("{stage}: respuesta demasiado grande")); }
        bytes.extend_from_slice(&chunk);
    }
    let value = serde_json::from_slice::<Value>(&bytes);
    if !status.is_success() {
        let summary = response_summary(&content_type, &bytes, value.as_ref().ok());
        let value = value.unwrap_or(Value::Null);
        let request_id = request_id.or_else(|| ["requestId", "traceId"].into_iter().find_map(|key| value[key].as_str().and_then(|id| uuid::Uuid::parse_str(id).ok())));
        let mut message = rejected(stage, status.as_u16(), &value);
        message.push_str(&summary);
        if let Some(id) = request_id { message.push_str(&format!(" Referencia del servicio: {id}.")); }
        return Err(message);
    }
    value.map_err(|_| format!("{stage}: respuesta inválida"))
}

fn required(value: &Value, field: &str, stage: &str) -> Result<String, String> {
    value.pointer(field).and_then(Value::as_str).filter(|v| !v.is_empty()).map(str::to_owned)
        .ok_or_else(|| format!("{stage}: falta un dato de autorización requerido"))
}

pub(crate) async fn exchange(client: &reqwest::Client, token: &str, notify: impl Fn(&str)) -> Result<MinecraftToken, String> {
    notify(&format!("Aplicación Microsoft: {}", crate::microsoft_oauth::client_id()?));
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
    notify("Xbox XSTS respondió. Conectando con Minecraft Services (/launcher/login)…");
    let response = client.post(endpoints.minecraft).header("Accept", "application/json")
        .json(&json!({"xtoken":format!("XBL3.0 x={user_hash};{xsts_token}"),"platform":"PC_LAUNCHER"}))
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
        assert_eq!(calls[2].1["xtoken"],"XBL3.0 x=xsts-hash;xsts-fixture");
        assert_eq!(calls[2].1["platform"],"PC_LAUNCHER");
        assert!(calls[2].1.get("identityToken").is_none());
        assert_eq!(ENDPOINTS.minecraft,"https://api.minecraftservices.com/launcher/login");
        for (i,(headers,_)) in calls.iter().enumerate(){let headers=headers.to_ascii_lowercase();assert!(headers.contains("content-type: application/json"));assert!(headers.contains("accept: application/json"));assert!(headers.contains("user-agent: tp-launcher/"));if i<2 {assert!(headers.contains("x-xbl-contract-version: 1"));}}
    }
    #[tokio::test]
    async fn minecraft_403_is_an_actionable_failure_and_never_leaks_body() {
        let (url,server)=server(vec![(200,xbl()),(200,xsts()),(403,json!({"errorMessage":"private-fixture-never-display"}))]).await;
        let stages=Arc::new(Mutex::new(Vec::new()));let seen=stages.clone();
        let result=exchange_with(&client().unwrap(),"microsoft-fixture",&Endpoints{xbox:&url,xsts:&url,minecraft:&url}, move |v|seen.lock().unwrap().push(v.to_owned())).await;
        let error=result.err().unwrap();assert!(error.contains("Minecraft Services (HTTP 403)"));assert!(error.contains("sin un motivo reconocido"));assert!(!error.contains("INVALID_APP_REGISTRATION"));assert!(error.contains("Client ID utilizado:"));assert!(!error.contains("private-fixture"));assert_eq!(stages.lock().unwrap().len(),3);server.await.unwrap();
    }
    #[test]
    fn minecraft_403_distinguishes_registration_and_account_rejections() {
        let registration = rejected("Minecraft Services", 403, &json!({"errorMessage":"Invalid app registration, see https://aka.ms/AppRegInfo for more information"}));
        assert!(registration.contains("INVALID_APP_REGISTRATION"));
        assert!(registration.contains("formulario aprobado"));
        assert!(registration.contains("Endpoint: /launcher/login."));
        assert!(!registration.contains("https://aka.ms/AppRegInfo"));
        let suspended = rejected("Minecraft Services", 403, &json!({"details":{"reason":"ACCOUNT_SUSPENDED"},"errorMessage":"private-account-fixture"}));
        assert!(suspended.contains("ACCOUNT_SUSPENDED"));
        assert!(!suspended.contains("private-account-fixture"));
        assert!(!suspended.contains("INVALID_APP_REGISTRATION"));
        let unknown = rejected("Minecraft Services", 403, &json!({"errorMessage":"private-token-fixture Invalid app registration"}));
        assert!(unknown.contains("sin un motivo reconocido"));
        assert!(!unknown.contains("private-token-fixture"));
        for value in [json!({"error":"INVALID_APP_REGISTRATION"}),json!({"message":"Invalid app registration"}),json!({"details":{"reason":"INVALID_APP_REGISTRATION"}})] {
            assert!(rejected("Minecraft Services",403,&value).contains("devolvió INVALID_APP_REGISTRATION"));
        }
    }
    #[tokio::test]
    async fn failure_exposes_only_a_valid_service_reference() {
        for (reference, expected) in [("a53d92c3-5cb1-47d0-99fb-84117723051c",true),("secret-token-never-display",false)] {
            let (url,server)=server(vec![(403,json!({"requestId":reference,"access_token":"private-fixture","errorMessage":"unrecognized-private-fixture"}))]).await;
            let response = client().unwrap().post(url).json(&json!({})).send().await.unwrap();
            let error = response_json(response,"Minecraft Services").await.unwrap_err();
            assert_eq!(error.contains(reference),expected);
            assert!(!error.contains("private-fixture"));
            server.await.unwrap();
        }
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
    #[tokio::test]
    async fn registration_rejection_stops_without_trying_other_endpoints() {
        let (url,server)=server(vec![(200,xbl()),(200,xsts()),(403,json!({"errorMessage":"Invalid app registration"}))]).await;
        let error=exchange_with(&client().unwrap(),"fixture",&Endpoints{xbox:&url,xsts:&url,minecraft:&url}, |_|{}).await.err().unwrap();
        assert!(error.contains("INVALID_APP_REGISTRATION"));
        assert!(error.contains("Endpoint: /launcher/login."));
        assert_eq!(server.await.unwrap().len(),3);
    }
    #[test]
    fn technical_summary_never_copies_private_values_or_arbitrary_field_names() {
        let value=json!({"error":"ForbiddenOperationException","errorMessage":"secret-fixture-message","access_token":"dummy-access-token","secret-fixture-key":"private","details":{"reason":"unknown-private-value"},"requestId":"secret-fixture-reference"});
        let text=response_summary("application/json; secret-fixture-header",b"fixture",Some(&value));
        assert!(text.contains("objeto JSON"));assert!(text.contains("ForbiddenOperationException"));assert!(text.contains("errorMessage"));assert!(!text.contains("secret-fixture"));assert!(!text.contains("access_token"));assert!(!text.contains("dummy-access-token"));assert!(!text.contains("private"));
        assert!(response_summary("text/html",b"<html>private</html>",None).contains("cuerpo no JSON"));
        assert!(response_summary("",b"",None).contains("cuerpo vacío"));
    }
}
