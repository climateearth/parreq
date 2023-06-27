use std::sync::Mutex;
use std::sync::mpsc::Sender;

use tracing::span;

use tracing::Metadata;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

mod computed_metrics;
pub(crate) mod metrics_summary;
mod request_metric;
use request_metric::RequestMetric;

use crate::metrics::request_metric::RequestResult;
use crate::metrics::request_metric::RequestStatus;

use self::computed_metrics::ComputedMetrics;
use self::metrics_summary::MetricsSummary;

pub struct MetricsLayer {
    computed_metrics: ComputedMetrics,
    metrics_sender: Mutex<Sender<MetricsSummary>>
}

impl MetricsLayer {
    pub(crate) fn new(total_requests_expected: u16, metrics_sender: Sender<MetricsSummary>) -> Self {
        Self {
            computed_metrics: ComputedMetrics::new(total_requests_expected),
            metrics_sender: Mutex::new(metrics_sender)
        }
    }
    fn record(&self, metric: &RequestMetric) {
        self.computed_metrics.record(metric);
        let metrics_sender = self.metrics_sender.lock().expect("unable to get the metrics sender mutex");
        metrics_sender.send(self.computed_metrics.summary()).expect("unable to send metrics");
    }
}

impl<S> Layer<S> for MetricsLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn enabled(&self, metadata: &Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        metadata.target().ends_with("::request")
    }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("expected span");
        let mut extensions = span.extensions_mut();
        let mut metric = RequestMetric::new();
        attrs.record(&mut metric);
        self.record(&metric);
        extensions.insert(metric);
    }
    // ...
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(parent_span) = ctx.event_span(event) {
            let mut extensions = parent_span.extensions_mut();
            if let Some(metric) = extensions.get_mut::<RequestMetric>() {
                if event.fields().any(|f| f.name().starts_with("return")) {
                    metric.mark_end();
                    metric.status = RequestStatus::Completed(RequestResult::Ok)
                } else if event.fields().any(|f| f.name().starts_with("error")) {
                    metric.mark_end();
                    metric.status =
                        RequestStatus::Completed(RequestResult::Error("some error".to_string()))
                }
                self.record(metric);
            }
        }
    }
}

pub(crate) fn layer(num_requests: u16, metrics_sender: Sender<MetricsSummary>) -> MetricsLayer {
    MetricsLayer::new(num_requests, metrics_sender)
}
