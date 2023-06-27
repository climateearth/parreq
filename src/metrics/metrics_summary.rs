

#[derive(Debug, Clone)]
pub(crate) struct MetricsSummary {
    pub in_progress: usize,
    pub errors: usize,
    pub ok: usize,
    pub total_expected: usize,
    // pub ok_durations: Vec<Duration>,
    // pub error_durations: Vec<Duration>,
}