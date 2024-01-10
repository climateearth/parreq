
use tokio::time::Instant;
use tracing::field::{self, Visit};

#[derive(Debug, Clone)]
pub(crate) struct InProgressState {
    start: Instant,
}

#[derive(Debug, Clone)]
pub(crate) enum CompletionResult {
    Ok,
    Error,
}
#[derive(Debug, Clone)]
pub(crate) struct CompletedState {
    pub(crate) start: Instant,
    pub(crate) end: Instant,
    pub(crate) result: CompletionResult
}

#[derive(Debug, Clone)]
pub(crate) enum RequestState {
    InProgress(InProgressState),
    Completed(CompletedState),
}
#[derive(Debug, Clone)]
pub(crate) struct RequestMetric {
    request_id: u64,
    executor_id: u64,
    task_in_executor: u64,
    status: RequestState,
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
            status: RequestState::InProgress(InProgressState {
                start: Instant::now(),
            }),
        }
    }

    pub(super) fn mark_end(&mut self, result: CompletionResult) {
        if let RequestState::InProgress(in_progress_state) = &self.status {
            self.status = RequestState::Completed(CompletedState {
                start: in_progress_state.start,
                end: Instant::now(),
                result
            });
        }
    }

    pub fn status(&self) -> &RequestState {
        &self.status
    }
}
