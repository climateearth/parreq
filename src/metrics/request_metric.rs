use std::time::Duration;

use tokio::time::Instant;
use tracing::field::{self, Visit};

#[derive(Debug, PartialEq)]
pub(super) enum RequestResult {
    Ok,
    Error(String),
}
#[derive(Debug, PartialEq)]
pub(super) enum RequestStatus {
    InProgress,
    Completed(RequestResult),
}

#[derive(Debug)]
pub(super) struct RequestMetric {
    request_id: u64,
    executor_id: u64,
    task_in_executor: u64,
    start: Instant,
    end: Option<Instant>,
    pub(super) status: RequestStatus,
}
impl Visit for RequestMetric {
    fn record_u64(&mut self, field: &field::Field, value: u64) {
        match field.name() {
            "metric_executor_id" => self.executor_id = value,
            "metric_task_in_executor" => self.task_in_executor = value,
            "metric_request_number" => self.request_id = value,
            &_ => {}
        }
    }
    fn record_debug(&mut self, _field: &field::Field, _value: &dyn std::fmt::Debug) {}
}
impl RequestMetric {
    pub(super) fn new() -> Self {
        RequestMetric {
            task_in_executor: 0,
            executor_id: 0,
            request_id: 0,
            start: Instant::now(),
            end: None,
            status: RequestStatus::InProgress,
        }
    }

    pub(super) fn mark_end(&mut self) {
        self.end = Some(Instant::now());
    }

    pub(super) fn request_duration(&self) -> Duration {
        self.end.unwrap() - self.start
    }
}
