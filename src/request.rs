use std::fmt;
use std::fmt::Debug;

use crate::config::RequestParameters;
use reqwest::RequestBuilder;
use serde_json::Value;

use reqwest::header::AUTHORIZATION;
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
            request_builder = request_builder
                .json(&req_data);
            data = Some(req_data);
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

    #[tracing::instrument(err)]
    pub async fn execute(self) -> Result<(), RequestError> {
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
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}
