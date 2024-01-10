use confique::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Config)]
pub struct Configuration {
    #[config(nested)]
    pub login: LoginParameters,
    #[config(default = 1)]
    pub concurrect_requests: usize,
    #[config(default = 1)]
    pub iterations: usize,
    #[config(default=[])]
    pub requests: Vec<RequestParameters>,
}

#[derive(Debug, Config)]
pub struct LoginParameters {
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub grant_type: String,
}

#[derive(Debug, Clone, Config, Serialize, Deserialize)]
pub struct RequestParameters {
    pub url: String,
    pub action: String,
    pub data: Option<Value>,
    pub status_code: Option<u16>,
}
