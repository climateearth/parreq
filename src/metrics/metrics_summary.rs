

use super::request_metric::{CompletionResult, RequestMetric, RequestState};

#[derive(Debug)]
pub(crate) struct MetricsSummary {
    pub in_progress: usize,
    pub errors: usize,
    pub ok: usize,
    pub total_expected: usize,
    pub ok_durations_ms: Vec<f32>,
    pub error_durations_ms: Vec<f32>,
}

impl MetricsSummary {
    pub fn new(total_expected: usize) -> Self {
        Self {
            in_progress: 0,
            errors: 0,
            ok: 0,
            total_expected,
            ok_durations_ms: Vec::with_capacity(total_expected),
            error_durations_ms: Vec::with_capacity(total_expected),
        }
    }

    pub fn record(&mut self, metrics: RequestMetric) {
        match metrics.status() {
            RequestState::InProgress(_) => {
                self.in_progress += 1;
            }
            RequestState::Completed(completion_result) => {
                self.in_progress -= 1;
                let duration = completion_result.end - completion_result.start;
                match completion_result.result {
                    CompletionResult::Error => {
                        self.errors += 1;
                        self.error_durations_ms.push(duration.as_secs_f32() * 1000_f32);
                        self.ok_durations_ms.push(0f32);
                    }
                    CompletionResult::Ok => {
                        self.ok += 1;
                        self.ok_durations_ms.push(duration.as_secs_f32() * 1000_f32);
                        self.error_durations_ms.push(0f32);
                    }
                }
            }
        }
    }

    pub fn is_completed(&self) -> bool {
        self.total_expected == self.ok + self.errors
    }
}
