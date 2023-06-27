mod batch_executor;
mod batcher;
mod config;
mod login;
mod metrics;
mod request;
mod ui;

use std::path::PathBuf;
use std::sync::mpsc::Sender;

use config::RequestParameters;
use confique::Config;

use futures::future::join_all;
use futures::stream::FuturesUnordered;

use clap::Parser;

use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::metadata::LevelFilter;
use tracing::{info, instrument};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{fmt, prelude::*};

use crate::batch_executor::BatchExecutor;
use crate::metrics::metrics_summary::MetricsSummary;

/// Simple program to run several requests in parallel
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Display logs in standard output
    #[arg(short, long, default_value_t = false)]
    verbose_output: bool,
    /// Config file with authentication and request entries [default: config.yaml]
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config_file = args.config.unwrap_or(PathBuf::from("config.yaml"));
    let conf =
        config::Configuration::from_file(config_file).expect("error reading configuration file");

    let total_requests = conf.iterations * conf.requests.len();
    let requests_final = create_requests_from_configuration(&conf);

    let (metrics_sender, metrics_receiver) = std::sync::mpsc::channel::<MetricsSummary>();
    init_tracing(total_requests, metrics_sender, args.verbose_output);
    info!("initialization");
    let login_response = login::login(&conf.login).await;
    let access_token = &login_response.access_token;

    let (start_signal_sender, start_signal_receiver) = tokio::sync::watch::channel(());

    let executors = create_executors(
        start_signal_receiver,
        total_requests,
        requests_final,
        &access_token,
        conf.concurrect_requests,
    );
    info!("executors created");
    start_signal_sender
        .send(())
        .expect("error sending start signal");
    if !args.verbose_output {
        tokio::spawn(async move {
            ui::run_ui(metrics_receiver).expect("error running tui");
        });
    }
    join_all(executors).await;

    info!("Done!");
}

fn create_requests_from_configuration(
    conf: &config::Configuration,
) -> impl Iterator<Item = RequestParameters> + Clone {
    conf.requests
        .clone()
        .into_iter()
        .cycle()
        .take(conf.iterations)
}

#[instrument(skip(requests_final, access_token))]
fn create_executors(
    start_signal_receiver: tokio::sync::watch::Receiver<()>,
    total_requests: usize,
    requests_final: impl Iterator<Item = RequestParameters> + Clone + Send,
    access_token: &str,
    n_batches: usize,
) -> FuturesUnordered<JoinHandle<()>> {
    let executors = FuturesUnordered::new();
    let requests_final = requests_final.clone();
    let batches = batcher::split(requests_final.clone(), n_batches);
    let batches = Box::new(batches);
    let tasks_per_executor = total_requests / batches.len();

    let batch_executors: Vec<_> = batches
        .into_iter()
        .enumerate()
        .map(|(batch_counter, batch)| {
            let bt = access_token.clone();
            let auth = bt;

            //FIXME: most of the initialization time is spent creating this vector... it should be enough to pass in a task creator function or an iterator
            let start = Instant::now();
            let tasks = batch
                .enumerate()
                .map(|(task_in_executor, req)| {
                    let request = request::Request::new(
                        req,
                        auth,
                        batch_counter,
                        tasks_per_executor,
                        task_in_executor + 1,
                    );
                    request
                })
                .collect();
            info!("time creating tasks vector: {:?}", start.elapsed());
            let batch_executor = BatchExecutor::new(batch_counter, tasks);
            batch_executor
        })
        .collect();

    batch_executors.into_iter().for_each(|batch_executor| {
        let rx = start_signal_receiver.clone();
        let join_handle = tokio::spawn(async move {
            batch_executor.start(rx).await;
        });
        executors.push(join_handle);
    });
    executors
}

fn init_tracing(num_requests: usize, metrics_sender: Sender<MetricsSummary>, display_logs: bool) {
    let mut layers = Vec::new();
    let metrics_layer = metrics::layer(num_requests as u16, metrics_sender).boxed();
    layers.push(metrics_layer);
    if display_logs {
        let log = fmt::layer()
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_target(true)
            .with_level(true)
            .with_filter(LevelFilter::INFO)
            .boxed();
        layers.push(log);
    }

    tracing_subscriber::registry().with(layers).init();
}
