use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::LoginParameters;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
}

pub async fn login(params: &LoginParameters) -> LoginResponse {
    let client = reqwest::Client::new();
    let login_params = [
        ("username", &params.username),
        ("password", &params.password),
        ("client_id", &params.client_id),
        ("client_secret", &params.client_secret),
        ("grant_type", &params.grant_type),
    ];
    let login: LoginResponse = client
        .post(&params.token_url)
        .form(&login_params)
        .send()
        .await
        .expect("login error")
        .json()
        .await
        .expect("error parsing access token");
    info!("logged in....");
    login
}
