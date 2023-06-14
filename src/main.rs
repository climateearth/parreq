use std::path::PathBuf;

use confique::Config;
use futures::stream::FuturesUnordered;
mod batch_executor;
mod batcher;
mod config;
mod login;
mod request;
use futures::future::join_all;

use clap::Parser;

use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, fmt::format::FmtSpan, prelude::*};

use crate::batch_executor::BatchExecutor;

/// Simple program to run several requests in parallel
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file with authentication and request entries [default: config.yaml]
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let mut layers = Vec::new();
    let log = fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(true)
        .with_level(true)
        .with_filter(LevelFilter::INFO);
    layers.push(log);

    tracing_subscriber::registry()
        .with(layers)
        .init();

    let args = Args::parse();

    let config_file = args.config.unwrap_or(PathBuf::from("config.yaml"));
    let conf =
        config::Configuration::from_file(config_file).expect("error reading configuration file");

    let login_response = login::login(&conf.login).await;
    let access_token = &login_response.access_token;

    let mut requests_final = conf.requests.clone();

    for _ in 1..conf.iterations {
        requests_final.append(&mut conf.requests.clone());
    }

    let total_requests = requests_final.len();

    let batches = batcher::split(&requests_final.into_iter(), conf.concurrect_requests);
    let futures = FuturesUnordered::new();
    let tasks_per_executor = total_requests / batches.len();

    let (tx, rx) = tokio::sync::watch::channel(());
    batches
        .iter()
        .enumerate()
        .map(|(batch_counter, batch)| {
            let bt = access_token.clone();
            let auth = bt.as_str();
            let tasks: Vec<request::Request> = batch
                .clone()
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
            let batch_executor = BatchExecutor::new(batch_counter, tasks);
            batch_executor
        })
        .for_each(|e| {
            let rx = rx.clone();
            let join_handle = tokio::spawn(e.start(rx));
            futures.push(join_handle);
        });

    tx.send(()).expect("error sending start signal");
    join_all(futures).await;

    info!("Done!");
}
