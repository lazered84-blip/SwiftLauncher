//! Authentification Microsoft (OAuth 2.0 + PKCE), chaîne Xbox → Minecraft, vérification de licence.
//!
//! Garanties : aucun mot de passe Microsoft n'est manipulé. Le refresh token vit dans le trousseau
//! système (Keychain / Credential Manager / Secret Service). Les access tokens restent en mémoire.
//!
//! ⚠ La chaîne d'API Minecraft Services exige un identifiant d'application validé (voir docs/GUIDE_INSTALLATION.md §6).
//! Les endpoints et formats sont ceux documentés publiquement, mais doivent être validés sur un compte réel.

use crate::error::{LauncherError, Result};
use crate::modules::database::Database;
use base64::Engine;
use rand::RngCore;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Port de la redirection loopback. Doit être enregistré dans l'application Microsoft Entra.
pub const REDIRECT_PORT: u16 = 53682;
const REDIRECT_URI: &str = "http://localhost:53682";
const AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const SCOPE: &str = "XboxLive.signin offline_access";
const KEYRING_SERVICE: &str = "minecraft-launcher";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

const SUCCESS_HTML: &str = "<!doctype html><html lang=\"fr\"><meta charset=\"utf-8\"><body style=\"font-family:sans-serif;padding:40px\"><h2>Connexion réussie</h2><p>Vous pouvez fermer cette fenêtre et revenir au launcher.</p></body></html>";
const FAILURE_HTML: &str = "<!doctype html><html lang=\"fr\"><meta charset=\"utf-8\"><body style=\"font-family:sans-serif;padding:40px\"><h2>Connexion échouée</h2><p>Vous pouvez fermer cette fenêtre et réessayer dans le launcher.</p></body></html>";

/// Identifiant client, injecté à la compilation : `MS_CLIENT_ID=... npm run tauri build`.
pub fn client_id() -> Result<&'static str> {
    option_env!("MS_CLIENT_ID")
        .filter(|id| !id.is_empty())
        .ok_or_else(|| LauncherError::Auth("MS_CLIENT_ID non défini à la compilation (voir .env.example)".into()))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub uuid: String,
    pub name: String,
}

// ---------------------------------------------------------------- PKCE

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn new_pkce() -> Pkce {
    let mut rng = rand::thread_rng();
    let mut verifier_bytes = [0u8; 32];
    rng.fill_bytes(&mut verifier_bytes);
    let verifier = b64(&verifier_bytes);
    let challenge = b64(&Sha256::digest(verifier.as_bytes()));

    let mut state_bytes = [0u8; 16];
    rng.fill_bytes(&mut state_bytes);
    Pkce {
        verifier,
        challenge,
        state: b64(&state_bytes),
    }
}

pub fn authorize_url(client_id: &str, pkce: &Pkce) -> Result<String> {
    let url = url::Url::parse_with_params(
        AUTH_URL,
        &[
            ("client_id", client_id),
            ("response_type", "code"),
            ("redirect_uri", REDIRECT_URI),
            ("scope", SCOPE),
            ("code_challenge", pkce.challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("state", pkce.state.as_str()),
            ("prompt", "select_account"),
        ],
    )
    .map_err(|e| LauncherError::Invalid(e.to_string()))?;
    Ok(url.to_string())
}

// ---------------------------------------------------------------- Redirection loopback

/// Extrait le code d'une requête `GET /?code=...&state=...`. `None` si la requête n'est pas le callback.
fn parse_callback(request_line: &str, expected_state: &str) -> Option<Result<String>> {
    let target = request_line.split_whitespace().nth(1)?;
    if !target.starts_with("/?") {
        return None;
    }
    let url = url::Url::parse(&format!("http://localhost{target}")).ok()?;

    let (mut code, mut state, mut error) = (None, None, None);
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            _ => {}
        }
    }

    if let Some(e) = error {
        return Some(Err(LauncherError::Auth(format!("connexion refusée ({e})"))));
    }
    if state.as_deref() != Some(expected_state) {
        return Some(Err(LauncherError::Auth("état OAuth invalide, connexion annulée".into())));
    }
    Some(code.ok_or_else(|| LauncherError::Auth("code d'autorisation manquant".into())))
}

async fn write_response(socket: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Bad Request",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket.write_all(response.as_bytes()).await?;
    socket.shutdown().await
}

async fn accept_callback(listener: TcpListener, expected_state: &str) -> Result<String> {
    loop {
        let (mut socket, _) = listener.accept().await?;
        let mut buf = vec![0u8; 8192];
        let n = socket.read(&mut buf).await?;
        let head = String::from_utf8_lossy(&buf[..n]).into_owned();
        let line = head.lines().next().unwrap_or("").to_string();

        match parse_callback(&line, expected_state) {
            // Requête parasite (favicon, préconnexion) : on répond 404 et on continue.
            None => write_response(&mut socket, 404, "Introuvable").await?,
            Some(result) => {
                let (status, body) = if result.is_ok() {
                    (200, SUCCESS_HTML)
                } else {
                    (400, FAILURE_HTML)
                };
                write_response(&mut socket, status, body).await?;
                return result;
            }
        }
    }
}

// ---------------------------------------------------------------- Jetons Microsoft

struct MsTokens {
    access: String,
    refresh: String,
}

#[derive(Deserialize)]
struct MsTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

async fn request_tokens(http: &reqwest::Client, form: &[(&str, &str)], fallback_refresh: &str) -> Result<MsTokens> {
    let resp = http.post(TOKEN_URL).form(form).send().await?;
    if !resp.status().is_success() {
        // Pas de corps de réponse dans le message : il peut contenir des indices sensibles.
        return Err(LauncherError::Auth(format!(
            "jeton Microsoft refusé (HTTP {})",
            resp.status().as_u16()
        )));
    }
    let body: MsTokenResponse = resp.json().await?;
    Ok(MsTokens {
        access: body.access_token,
        refresh: body.refresh_token.unwrap_or_else(|| fallback_refresh.to_string()),
    })
}

async fn exchange_code(http: &reqwest::Client, client_id: &str, code: &str, verifier: &str) -> Result<MsTokens> {
    let form = [
        ("client_id", client_id),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", REDIRECT_URI),
        ("code_verifier", verifier),
        ("scope", SCOPE),
    ];
    request_tokens(http, &form, "").await
}

async fn refresh_tokens(http: &reqwest::Client, client_id: &str, refresh: &str) -> Result<MsTokens> {
    let form = [
        ("client_id", client_id),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh),
        ("scope", SCOPE),
    ];
    request_tokens(http, &form, refresh).await
}

// ---------------------------------------------------------------- Chaîne Xbox → Minecraft

#[derive(Deserialize)]
struct XboxResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Deserialize)]
struct Xui {
    uhs: String,
}

#[derive(Deserialize)]
struct McLoginResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct McEntitlements {
    items: Vec<McEntitlement>,
}

#[derive(Deserialize)]
struct McEntitlement {
    name: String,
}

#[derive(Deserialize)]
struct McProfile {
    id: String,
    name: String,
}

async fn parse_xbox(resp: reqwest::Response, step: &str) -> Result<XboxResponse> {
    if !resp.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "{step} a refusé la connexion (HTTP {})",
            resp.status().as_u16()
        )));
    }
    Ok(resp.json().await?)
}

async fn xbox_live(http: &reqwest::Client, ms_access: &str) -> Result<String> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={ms_access}")
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    let resp = http.post(XBL_AUTH_URL).json(&body).send().await?;
    Ok(parse_xbox(resp, "Xbox Live").await?.token)
}

/// Renvoie (jeton XSTS, identifiant utilisateur Xbox).
async fn xsts(http: &reqwest::Client, xbl_token: &str) -> Result<(String, String)> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });
    let resp = http.post(XSTS_AUTH_URL).json(&body).send().await?;
    let data = parse_xbox(resp, "XSTS").await?;
    let uhs = data
        .display_claims
        .xui
        .into_iter()
        .next()
        .ok_or_else(|| LauncherError::Auth("réponse XSTS incomplète".into()))?
        .uhs;
    Ok((data.token, uhs))
}

async fn minecraft_token(http: &reqwest::Client, uhs: &str, xsts_token: &str) -> Result<String> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={uhs};{xsts_token}")
    });
    let resp = http.post(MC_LOGIN_URL).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "connexion à Minecraft Services refusée (HTTP {})",
            resp.status().as_u16()
        )));
    }
    let data: McLoginResponse = resp.json().await?;
    Ok(data.access_token)
}

async fn ms_to_minecraft(http: &reqwest::Client, ms_access: &str) -> Result<String> {
    let xbl = xbox_live(http, ms_access).await?;
    let (xsts_token, uhs) = xsts(http, &xbl).await?;
    minecraft_token(http, &uhs, &xsts_token).await
}

async fn owns_java(http: &reqwest::Client, mc_token: &str) -> Result<bool> {
    let resp = http.get(MC_ENTITLEMENTS_URL).bearer_auth(mc_token).send().await?;
    if !resp.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "vérification de la licence impossible (HTTP {})",
            resp.status().as_u16()
        )));
    }
    let entitlements: McEntitlements = resp.json().await?;
    Ok(entitlements
        .items
        .iter()
        .any(|item| matches!(item.name.as_str(), "game_minecraft" | "product_minecraft")))
}

async fn fetch_profile(http: &reqwest::Client, mc_token: &str) -> Result<Option<McProfile>> {
    let resp = http.get(MC_PROFILE_URL).bearer_auth(mc_token).send().await?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !resp.status().is_success() {
        return Err(LauncherError::Auth(format!(
            "profil Minecraft indisponible (HTTP {})",
            resp.status().as_u16()
        )));
    }
    Ok(Some(resp.json().await?))
}

// ---------------------------------------------------------------- Trousseau et base

fn keyring_entry(uuid: &str) -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(KEYRING_SERVICE, uuid)?)
}

fn store_refresh(uuid: &str, token: &str) -> Result<()> {
    keyring_entry(uuid)?.set_password(token)?;
    Ok(())
}

fn load_refresh(uuid: &str) -> Result<String> {
    Ok(keyring_entry(uuid)?.get_password()?)
}

fn delete_refresh(uuid: &str) -> Result<()> {
    match keyring_entry(uuid)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

fn active_account(db: &Database) -> Result<Option<Account>> {
    db.with(|c| {
        Ok(c.query_row(
            "SELECT uuid, name FROM accounts WHERE is_active = 1 LIMIT 1",
            [],
            |r| Ok(Account { uuid: r.get(0)?, name: r.get(1)? }),
        )
        .optional()?)
    })
}

fn set_active_account(db: &Database, account: &Account) -> Result<()> {
    db.with(|c| {
        c.execute("UPDATE accounts SET is_active = 0", [])?;
        c.execute(
            "INSERT INTO accounts (uuid, name, is_active) VALUES (?1, ?2, 1)
             ON CONFLICT(uuid) DO UPDATE SET name = excluded.name, is_active = 1",
            rusqlite::params![account.uuid, account.name],
        )?;
        Ok(())
    })
}

// ---------------------------------------------------------------- API publique

/// Connexion complète. `open_browser` ouvre l'URL d'autorisation (fourni par la couche Tauri).
pub async fn login(
    http: &reqwest::Client,
    db: &Database,
    open_browser: impl FnOnce(&str) -> Result<()>,
) -> Result<Account> {
    let client_id = client_id()?;
    let pkce = new_pkce();

    // Le port doit être ouvert AVANT d'ouvrir le navigateur, sinon le retour serait perdu.
    let listener = TcpListener::bind(("127.0.0.1", REDIRECT_PORT))
        .await
        .map_err(|e| LauncherError::Auth(format!("le port {REDIRECT_PORT} est indisponible ({e})")))?;

    let auth_url = authorize_url(client_id, &pkce)?;
    open_browser(auth_url.as_str())?;

    let code = tokio::time::timeout(LOGIN_TIMEOUT, accept_callback(listener, &pkce.state))
        .await
        .map_err(|_| LauncherError::Auth("délai de connexion dépassé".into()))??;

    let ms = exchange_code(http, client_id, &code, &pkce.verifier).await?;
    if ms.refresh.is_empty() {
        return Err(LauncherError::Auth("aucun jeton de renouvellement reçu".into()));
    }

    let mc_token = ms_to_minecraft(http, &ms.access).await?;
    if !owns_java(http, &mc_token).await? {
        return Err(LauncherError::NoJavaLicense);
    }
    let profile = fetch_profile(http, &mc_token)
        .await?
        .ok_or(LauncherError::NoJavaLicense)?;

    let account = Account {
        uuid: profile.id,
        name: profile.name,
    };
    store_refresh(&account.uuid, &ms.refresh)?;
    set_active_account(db, &account)?;
    tracing::info!("compte connecté (identifiant masqué)");
    Ok(account)
}

/// Au démarrage : renouvelle silencieusement la session du compte actif.
/// Renvoie `None` si aucun compte, si le trousseau est vide ou si le renouvellement échoue.
pub async fn restore_session(http: &reqwest::Client, db: &Database) -> Result<Option<Account>> {
    let Some(account) = active_account(db)? else {
        return Ok(None);
    };
    let client_id = match client_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("restauration de session ignorée : {e}");
            return Ok(None);
        }
    };
    let refresh = match load_refresh(&account.uuid) {
        Ok(token) => token,
        Err(e) => {
            tracing::warn!("jeton absent du trousseau : {e}");
            return Ok(None);
        }
    };
    match refresh_tokens(http, client_id, &refresh).await {
        Ok(tokens) => {
            // Microsoft peut faire tourner le refresh token : on stocke le nouveau.
            store_refresh(&account.uuid, &tokens.refresh)?;
            Ok(Some(account))
        }
        Err(e) => {
            tracing::warn!("renouvellement de session impossible : {e}");
            Ok(None)
        }
    }
}

/// Prépare une session de jeu : renouvelle le jeton Microsoft, obtient le jeton Minecraft.
/// Le jeton Minecraft n'est jamais persisté : il est recalculé à chaque lancement.
pub async fn launch_session(http: &reqwest::Client, db: &Database) -> Result<(Account, String)> {
    let account = active_account(db)?
        .ok_or_else(|| LauncherError::Auth("aucun compte connecté".into()))?;
    let client_id = client_id()?;
    let refresh = load_refresh(&account.uuid)?;
    let tokens = refresh_tokens(http, client_id, &refresh).await?;
    store_refresh(&account.uuid, &tokens.refresh)?;
    let mc_token = ms_to_minecraft(http, &tokens.access).await?;
    Ok((account, mc_token))
}

/// Déconnexion : supprime le jeton du trousseau puis le compte local.
pub fn sign_out(db: &Database) -> Result<()> {
    if let Some(account) = active_account(db)? {
        delete_refresh(&account.uuid)?;
    }
    db.with(|c| {
        c.execute("DELETE FROM accounts WHERE is_active = 1", [])?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_verifier() {
        let pkce = new_pkce();
        let expected = b64(&Sha256::digest(pkce.verifier.as_bytes()));
        assert_eq!(pkce.challenge, expected);
        assert!(pkce.verifier.len() >= 43, "verifier PKCE trop court");
    }

    #[test]
    fn authorize_url_contains_pkce_parameters() {
        let pkce = new_pkce();
        let url = authorize_url("test-client", &pkce).unwrap();
        assert!(url.starts_with(AUTH_URL));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("client_id=test-client"));
    }

    #[test]
    fn callback_accepts_matching_state() {
        let line = "GET /?code=abc123&state=xyz HTTP/1.1";
        let code = parse_callback(line, "xyz").unwrap().unwrap();
        assert_eq!(code, "abc123");
    }

    #[test]
    fn callback_rejects_wrong_state() {
        let line = "GET /?code=abc123&state=autre HTTP/1.1";
        assert!(parse_callback(line, "xyz").unwrap().is_err());
    }

    #[test]
    fn callback_ignores_unrelated_requests() {
        assert!(parse_callback("GET /favicon.ico HTTP/1.1", "xyz").is_none());
    }

    #[test]
    fn callback_reports_user_refusal() {
        let line = "GET /?error=access_denied&state=xyz HTTP/1.1";
        assert!(parse_callback(line, "xyz").unwrap().is_err());
    }
}
