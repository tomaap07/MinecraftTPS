
use base64::Engine;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub const SCOPE: &str = "XboxLive.signin offline_access";


pub const DEFAULT_CLIENT_ID: &str = "6a4f8732-6bf7-49c3-a749-16607f97281d";
pub const REDIRECT_URI: &str = "http://localhost:8080";
const LOOPBACK_PORT: u16 = 8080;
const AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
static LOGIN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
pub fn client_id() -> Result<String, String> {
    configured_client_id(
        std::env::var("VITE_MS_CLIENT_ID").ok(),
        option_env!("TP_MS_CLIENT_ID"),
    )
}
fn configured_client_id(runtime: Option<String>, compiled: Option<&str>) -> Result<String, String> {
    let value = runtime
        .filter(|v| !v.trim().is_empty())
        .or_else(|| compiled.filter(|v| !v.trim().is_empty()).map(str::to_owned))
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.into());
    let id = uuid::Uuid::parse_str(value.trim()).map_err(|_| {
        "VITE_MS_CLIENT_ID debe ser el identificador UUID de tu propia aplicación Microsoft"
    })?;
    if id.is_nil() {
        return Err("Configura un Client ID propio válido".into());
    }
    Ok(id.to_string())
}
fn callback(target: &str, state: &str) -> Result<String, String> {
    if !target.starts_with('/') || target.starts_with("//") || target.contains('#') {
        return Err("Redirección OAuth inválida".into());
    }
    let url = url::Url::parse(&format!("http://localhost{target}"))
        .map_err(|_| "Redirección OAuth inválida")?;
    for key in ["state", "code", "error"] {
        if url.query_pairs().filter(|(k, _)| k == key).count() > 1 {
            return Err("Redirección OAuth ambigua".into());
        }
    }
    if url.path() != "/"
        || url
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.into_owned())
            .as_deref()
            != Some(state)
    {
        return Err("No coincide el estado de la solicitud Microsoft".into());
    }
    if url.query_pairs().any(|(k, _)| k == "error") {
        return Err(
            "Microsoft no autorizó el inicio de sesión. Puedes intentarlo otra vez.".into(),
        );
    }
    url.query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.into_owned())
        .filter(|v| !v.is_empty() && v.len() < 8192)
        .ok_or("Microsoft no devolvió un código de autorización".into())
}
fn request_callback(request: &str, state: &str) -> Result<String, String> {
    let mut lines = request.split("\r\n");
    let line = lines.next().ok_or("Redirección incompleta")?;
    let mut words = line.split(' ');
    if words.next() != Some("GET") {
        return Err("Método OAuth no permitido".into());
    }
    let target = words.next().ok_or("Redirección incompleta")?;
    if !matches!(words.next(), Some("HTTP/1.1" | "HTTP/1.0")) || words.next().is_some() {
        return Err("Redirección incompleta".into());
    }
    let hosts: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim())
        .collect();
    if hosts.len() != 1 || !hosts[0].eq_ignore_ascii_case("localhost:8080") {
        return Err("Host OAuth no permitido".into());
    }
    callback(target, state)
}
fn authorize_url(client: &str, state: &str, verifier: &str) -> Result<url::Url, String> {
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(Sha256::digest(verifier.as_bytes()));
    let mut authorize = url::Url::parse(AUTHORIZE_URL).map_err(|e| e.to_string())?;
    authorize.query_pairs_mut().extend_pairs([
        ("client_id", client),
        ("response_type", "code"),
        ("response_mode", "query"),
        ("redirect_uri", REDIRECT_URI),
        ("scope", SCOPE),
        ("state", state),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
        ("prompt", "select_account"),
    ]);
    Ok(authorize)
}
async fn token(form: &[(&str, &str)]) -> Result<serde_json::Value, String> {
    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?
        .post(TOKEN_URL)
        .header(reqwest::header::ACCEPT, "application/json")
        .form(form)
        .send()
        .await
        .map_err(|_| "No se pudo contactar con Microsoft. Comprueba tu conexión")?;
    let success = response.status().is_success();
    if response.content_length().is_some_and(|n| n > 65536) {
        return Err("La respuesta Microsoft supera el tamaño esperado".into());
    }
    let mut bytes = Vec::new();
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_| "Microsoft devolvió una respuesta incompleta")?;
        if bytes.len() + chunk.len() > 65536 {
            return Err("La respuesta Microsoft supera el tamaño esperado".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "Microsoft devolvió una respuesta inválida")?;
    if !success || value.get("error").is_some() {
        return Err(match value["error"].as_str(){Some("invalid_grant")=>"La sesión necesita volver a iniciarse con la aplicación Microsoft propia de TP LAUNCHER",Some("unauthorized_client"|"invalid_client")=>"Microsoft no acepta esta aplicación para cuentas personales. Revisa su Client ID y los tipos de cuenta en Microsoft Entra",Some("invalid_scope")=>"Microsoft rechazó los permisos Xbox solicitados. Revisa la autorización Xbox/Minecraft de esta aplicación",_=>"Microsoft rechazó la solicitud. Revisa el registro de tu aplicación, los permisos Xbox y la redirección http://localhost:8080"}.into());
    }
    if !value["access_token"]
        .as_str()
        .is_some_and(|v| !v.is_empty())
    {
        return Err("Microsoft no devolvió un token de acceso válido".into());
    }
    Ok(value)
}
pub async fn login(app: &AppHandle) -> Result<serde_json::Value, String> {
    let _login = LOGIN
        .try_lock()
        .map_err(|_| "Ya hay un inicio de sesión Microsoft en curso en el navegador")?;
    let client = client_id()?;
    let listener=tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST,LOOPBACK_PORT)).await.map_err(|error|if error.kind()==std::io::ErrorKind::AddrInUse{"El puerto local 8080 está ocupado. Cierra la aplicación que lo usa y vuelve a iniciar sesión"}else{"No se pudo preparar la redirección local http://localhost:8080"})?;
    let state = uuid::Uuid::new_v4().simple().to_string();
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let authorize = authorize_url(&client, &state, &verifier)?;
    app.opener()
        .open_url(authorize.as_str(), None::<&str>)
        .map_err(|_| "No se pudo abrir el navegador oficial de Microsoft")?;
    let code=tokio::time::timeout(std::time::Duration::from_secs(300),async{
  loop{let(mut stream,peer)=listener.accept().await.map_err(|e|e.to_string())?;if !peer.ip().is_loopback(){continue;}
   let mut request=Vec::new();let mut block=[0u8;1024];let received=tokio::time::timeout(std::time::Duration::from_secs(5),async{while request.len()<16384{let n=stream.read(&mut block).await.map_err(|e|e.to_string())?;if n==0{break;}request.extend_from_slice(&block[..n]);if request.windows(4).any(|v|v==b"\r\n\r\n"){return Ok::<_,String>(());}}Err("Redirección incompleta".into())}).await;
   if !matches!(received,Ok(Ok(()))){continue;}
   let text=String::from_utf8_lossy(&request);let result=request_callback(&text,&state);
   let body=if result.is_ok(){"Respuesta de Microsoft recibida. Vuelve a TP LAUNCHER para comprobar la cuenta y su licencia. Puedes cerrar esta pestana."}else{"Solicitud no valida. Vuelve a TP LAUNCHER para iniciar una nueva solicitud."};
   let reply=format!("HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",if result.is_ok(){"200 OK"}else{"400 Bad Request"},body.len());let _=stream.write_all(reply.as_bytes()).await;let _=stream.shutdown().await;


   match result{Ok(code)=>break Ok::<_,String>(code),Err(reason)if reason=="Microsoft no autorizó el inicio de sesión. Puedes intentarlo otra vez."=>break Err(reason),Err(_)=>{}}
  }
 }).await.map_err(|_|"Se agotó el tiempo para iniciar sesión con Microsoft")??;
    drop(listener);
    token(&[
        ("client_id", &client),
        ("grant_type", "authorization_code"),
        ("scope", SCOPE),
        ("code", &code),
        ("redirect_uri", REDIRECT_URI),
        ("code_verifier", &verifier),
    ])
    .await
}
pub async fn refresh(refresh_token: &str) -> Result<serde_json::Value, String> {
    let client = client_id()?;
    token(&[
        ("client_id", &client),
        ("grant_type", "refresh_token"),
        ("scope", SCOPE),
        ("refresh_token", refresh_token),
    ])
    .await
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_is_bound_to_state_and_path() {
        assert_eq!(callback("/?state=abc&code=a%2Bb", "abc").unwrap(), "a+b");
        for path in [
            "/?state=other&code=secret",
            "/other?state=abc&code=secret",
            "/?state=abc&error=denied",
            "/",
            "//evil.invalid/?state=abc&code=secret",
            "/?state=abc&state=other&code=secret",
            "/?state=abc&code=first&code=second",
            "/?state=abc&code=secret#ignored",
        ] {
            assert!(callback(path, "abc").is_err(), "{path}");
        }
    }
    #[test]
    fn callback_checks_http_method_and_loopback_host() {
        assert_eq!(
            request_callback(
                "GET /?state=abc&code=secret HTTP/1.1\r\nHost: localhost:8080\r\n\r\n",
                "abc"
            )
            .unwrap(),
            "secret"
        );
        for request in ["POST /?state=abc&code=secret HTTP/1.1\r\nHost: localhost:8080\r\n\r\n","GET /?state=abc&code=secret HTTP/1.1\r\nHost: evil.invalid\r\n\r\n","GET /?state=abc&code=secret HTTP/1.1\r\nHost: localhost:8080\r\nHost: evil.invalid\r\n\r\n"]{assert!(request_callback(request,"abc").is_err());}
    }
    #[test]
    fn application_id_defaults_to_owners_registration_and_allows_own_overrides() {
        assert_eq!(configured_client_id(None, None).unwrap(), DEFAULT_CLIENT_ID);
        assert_eq!(
            configured_client_id(Some(" ".into()), Some(" ")).unwrap(),
            DEFAULT_CLIENT_ID
        );
        let other = "a53d92c3-5cb1-47d0-99fb-84117723051c";
        assert_eq!(
            configured_client_id(Some(other.into()), Some(DEFAULT_CLIENT_ID)).unwrap(),
            other
        );
        assert!(configured_client_id(Some("not-a-client-id".into()), None).is_err());
        assert!(configured_client_id(Some(uuid::Uuid::nil().to_string()), None).is_err());
    }
    #[test]
    fn authorization_uses_registered_redirect_scope_and_pkce() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let url = authorize_url(DEFAULT_CLIENT_ID, "request-state", verifier).unwrap();
        let params: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(url.host_str(), Some("login.microsoftonline.com"));
        assert_eq!(params["client_id"], DEFAULT_CLIENT_ID);
        assert_eq!(params["redirect_uri"], "http://localhost:8080");
        assert_eq!(params["scope"], "XboxLive.signin offline_access");
        assert_eq!(params["response_type"], "code");
        assert_eq!(params["response_mode"], "query");
        assert_eq!(params["code_challenge_method"], "S256");
        assert_eq!(
            params["code_challenge"],
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        assert!(!params.contains_key("client_secret"));
        assert!(!params.contains_key("code_verifier"));
    }
}
