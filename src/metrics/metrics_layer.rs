use std::sync::mpsc::Sender;
use std::sync::Mutex;
use tracing::span;

use tracing::Metadata;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::request_metric::CompletionResult;
use super::request_metric::RequestMetric;

pub struct MetricsLayer {
    metrics_sender: Mutex<Sender<RequestMetric>>,
}

impl MetricsLayer {
    pub(crate) fn new(metrics_sender: Sender<RequestMetric>) -> Self {
        Self {
            metrics_sender: Mutex::new(metrics_sender),
        }
    }

    fn record(&self, metric: &RequestMetric) {
        if let Ok(metrics_sender) = self.metrics_sender.lock() {
            metrics_sender.send(metric.clone()).unwrap();
        }
    }
}

impl<S> Layer<S> for MetricsLayer
where
    S: Subscriber + for<'b> LookupSpan<'b>,
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
                    metric.mark_end(CompletionResult::Ok);
                } else if event.fields().any(|f| f.name().starts_with("error")) {
                    metric.mark_end(CompletionResult::Error);
                }
                self.record(metric);
            }
        }
    }
}
