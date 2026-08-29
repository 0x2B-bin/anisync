use color_eyre::eyre::{Result, WrapErr, eyre};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl, basic::BasicClient,
};
use tiny_http::{Header, Response, Server};
use url::Url;

use crate::config::Config;

pub fn run(config: &mut Config) -> Result<()> {
    let client = BasicClient::new(ClientId::new(config.myanimelist.client_id.clone()))
        .set_client_secret(ClientSecret::new(config.myanimelist.client_secret.clone()))
        .set_auth_uri(AuthUrl::new(
            "https://myanimelist.net/v1/oauth2/authorize".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://myanimelist.net/v1/oauth2/token".to_string(),
        )?)
        .set_redirect_uri(RedirectUrl::new("http://127.0.0.1:6767".to_string())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_plain();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("write".to_string()))
        .add_scope(Scope::new("users".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Browse to: {auth_url}");

    let code = listen_for_code(csrf_token.secret())?;

    let http_client = oauth2::ureq::Agent::new();

    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(pkce_verifier)
        .request(&http_client)
        .wrap_err("Failed to exchange code for token")?;

    config.myanimelist.access_token = Some(token.access_token().secret().clone());
    config.myanimelist.refresh_token = token.refresh_token().map(|r| r.secret().clone());
    config.serialize()?;
    println!("Tokens saved");
    Ok(())
}

fn listen_for_code(expected_csrf: &str) -> Result<String> {
    let http_server = Server::http("127.0.0.1:6767")
        .map_err(|e| eyre!("Failed to  start local HTTP server: {e}"))?;

    println!("Started local web server");

    for request in http_server.incoming_requests() {
        let url = Url::parse(&format!("http://127.0.0.1:6767{}", request.url()))
            .wrap_err("Failed to parse url")?;

        let mut state = None;
        let mut code = None;
        for (key, value) in url.query_pairs() {
            if key == "state" {
                state = Some(value);
            } else if key == "code" {
                code = Some(value);
            }
        }

        if code.is_none() || state.is_none() {
            let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            continue;
        }

        let state_final = state.unwrap();
        let code_final = code.unwrap();

        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();

        if state_final != expected_csrf {
            let html = "<h1>Authenication Failed</h1><p>Mismatched CSRF Token</p>";
            let mut response = Response::from_string(html);
            response.add_header(header);
            let _ = request.respond(response);
            return Err(eyre!("CSRF Token mismatch"));
        }

        let html = "<html><body><h1>Authentication Successful!</h1><p>You may naviagte back to the terminal!</p></body></html>";
        let mut response = Response::from_string(html);
        response.add_header(header);
        let _ = request.respond(response);

        return Ok(code_final.to_string());
    }

    Err(eyre!("HTTP server closed before code exchange"))
}
