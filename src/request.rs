use std::sync::OnceLock;
use std::{fmt, fmt::Debug};

use crate::batch_executor::Executable;
use crate::config::RequestParameters;
use async_trait::async_trait;
use reqwest::header::AUTHORIZATION;
use reqwest::{RequestBuilder, StatusCode};
use serde_json::Value;

static DEFAULT_USER_CLIENT: OnceLock<String> = OnceLock::new();

pub struct Request {
    _request_builder: RequestBuilder,
    executor: usize,
    task_in_executor: usize,
    request_number: usize,
    data: Option<Value>,
    _status_code: Option<u16>,
}

#[derive(Debug)]
pub struct RequestError {
    pub msg: String,
}

impl From<reqwest::Error> for RequestError {
    fn from(value: reqwest::Error) -> Self {
        RequestError {
            msg: value.to_string(),
        }
    }
}
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "RequestError ({})", self.msg)
    }
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

#[async_trait]
impl Executable for Request {
    type Result = Result<StatusCode, RequestError>;

    #[tracing::instrument(err, ret,
        skip(self),
        fields(
            metric_executor_id=self.executor,
            metric_task_in_executor=self.task_in_executor,
            metric_request_number=self.request_number
        )
    )]
    async fn execute(self) -> Self::Result {
        // info!("starting request");
        let resp = self._request_builder.send().await;
        // info!("ending request");
        match resp {
            Ok(resp) => {
                if let Some(expected_status) = self._status_code {
                    if resp.status().as_u16() != expected_status {
                        let msg = format!(
                            "status code error: expected {}, actual {}",
                            expected_status,
                            resp.status()
                        );
                        Err(RequestError { msg })
                    } else {
                        Ok(resp.status())
                    }
                } else {
                    Ok(resp.status())
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}
impl Request {
    pub fn new(
        req: RequestParameters,
        auth: &str,
        executor: usize,
        tasks_per_executor: usize,
        task_in_executor: usize,
        client: &reqwest::Client
    ) -> Self {
        DEFAULT_USER_CLIENT.get_or_init(|| {
            let default_user_agent =
                format!("{}_v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            default_user_agent
        });
        let _auth = "Bearer ".to_owned() + &auth;
        // let client = reqwest::Client::new();
        let request_number = (executor * tasks_per_executor) + task_in_executor;

        let request_builder = match req.action.as_str() {
            "POST" => client.post(&req.url),
            "PUT" => client.put(&req.url),
            "GET" => client.get(&req.url),
            _ => panic!("action not supported"),
        };

        let mut data: Option<Value> = None;
        if let Some(orig_data) = req.data {
            data = Some(orig_data);
        }
        Self {
            _request_builder: request_builder.header(AUTHORIZATION, _auth.clone()),
            executor,
            task_in_executor,
            request_number,
            data,
            _status_code: req.status_code,
        }
    }

}
