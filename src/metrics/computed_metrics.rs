use std::{
    sync::{
        atomic::{AtomicU16, Ordering},
    },
};

use crate::metrics::request_metric::{RequestResult, RequestStatus};

use super::{metrics_summary::MetricsSummary, request_metric::RequestMetric};

#[derive(Debug)]
pub(crate) struct ComputedMetrics {
    in_progress: AtomicU16,
    errors: AtomicU16,
    ok: AtomicU16,
    total_expected: u16,
    // ok_durations: Arc<Mutex<Vec<Duration>>>,
    // error_durations: Arc<Mutex<Vec<Duration>>>,
}

impl ComputedMetrics {
    pub(super) fn new(total_expected: u16) -> Self {
        Self {
            in_progress: AtomicU16::new(0),
            errors: AtomicU16::new(0),
            ok: AtomicU16::new(0),
            total_expected,
            // ok_durations: Arc::new(Mutex::new(Vec::with_capacity(total_expected as usize))),
            // error_durations: Arc::new(Mutex::new(Vec::with_capacity(total_expected as usize))),
        }
    }
    pub(super) fn record(&self, metric: &RequestMetric) {
        match &metric.status {
            RequestStatus::InProgress => {
                self.in_progress.fetch_add(1, Ordering::AcqRel);
            }
            RequestStatus::Completed(cs) => {
                let _duration = metric.request_duration();
                self.in_progress.fetch_sub(1, Ordering::AcqRel);
                match cs {
                    RequestResult::Error(_) => {
                        // let mut error_durations = self
                        //     .error_durations
                        //     .lock()
                        //     .expect("failed to get the ok_times lock");
                        // error_durations.push(duration);
                        self.errors.fetch_add(1, Ordering::AcqRel);
                    },
                    RequestResult::Ok => {
                        // let mut ok_times = self
                        //     .ok_durations
                        //     .lock()
                        //     .expect("failed to get the ok_times lock");
                        // ok_times.push(duration);
                        self.ok.fetch_add(1, Ordering::Acquire);
                    }
                };
            }
        }
    }

    pub(crate) fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            in_progress: self.in_progress.load(Ordering::Relaxed).into(),
            ok: self.ok.load(Ordering::Relaxed).into(),
            errors: self.errors.load(Ordering::Relaxed).into(),
            total_expected: self.total_expected.into(),
            // ok_durations: self.ok_durations.lock().expect("error getting...").clone(),
            // error_durations: self.error_durations.lock().expect("error getting...").clone(),
        }
    }
}
