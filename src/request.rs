use std::fmt::Debug;

use crate::config::RequestParameters;
use reqwest::RequestBuilder;
use serde_json::Value;

use reqwest::header::AUTHORIZATION;
use tracing::info;
pub struct Request {
    _request_builder: RequestBuilder,
    executor: usize,
    task_in_executor: usize,
    request_number: usize,
    data: Option<Value>,
    _auth: String,
}

impl Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("executor", &self.executor)
            .field("task_in_executor", &self.task_in_executor)
            .field("request_number", &self.request_number)
            .field("data", &self.data)
            .finish()
    }
}
impl Request {
    pub fn new(
        req: RequestParameters,
        auth: &str,
        executor: usize,
        tasks_per_executor: usize,
        task_in_executor: usize,
    ) -> Self {
        let _auth = "Bearer ".to_owned() + &auth;
        let client = reqwest::Client::new();
        let request_number = (executor * tasks_per_executor) + task_in_executor;

        let mut request_builder = match req.action.as_str() {
            "POST" => client.post(&req.url),
            "PUT" => client.put(&req.url),
            "GET" => client.get(&req.url),
            _ => panic!("action not supported"),
        };
        let mut data: Option<Value> = None;
        if let Some(orig_data) = &req.data {
            let mut req_data_str;
            req_data_str = orig_data.to_string();
            req_data_str = req_data_str.replace("{i}", &request_number.to_string());
            let req_data: Value = serde_json::from_str(&req_data_str).unwrap();
            request_builder = request_builder.json(&req_data);
            data = Some(req_data);
        }
        Self {
            _request_builder: request_builder,
            executor,
            task_in_executor,
            request_number,
            data,
            _auth,

        }
    }

    #[tracing::instrument(err, ret)]
    pub async fn execute(self) -> Result<String, reqwest::Error> {
        info!("starting request");
        let resp = self
            ._request_builder
            .header(AUTHORIZATION, self._auth)
            .send()
            .await?
            .text()
            .await;
        // info!("ending request");
        resp
    }
}
