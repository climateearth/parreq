use std::path::PathBuf;

use confique::Config;
use futures::stream::FuturesUnordered;
mod batcher;
mod config;
mod login;
mod request;
use futures::future::join_all;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        // .with_env_filter("request=debug")
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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

    let batches = batcher::split(&requests_final.into_iter(), conf.concurrect_requests);
    let futures = FuturesUnordered::new();

    let mut batch_counter: usize = 0;
    for batch in batches {
        let bt = access_token.clone();
        batch_counter += 1;
        let batch_executor = tokio::spawn(async move {
            let auth = bt.as_str();
            let executor_task = &AtomicUsize::new(1);
            let tasks: Vec<_> = batch
                .map(|req| async {
                    let current = executor_task.load(Relaxed);
                    executor_task.store(current + 1, Relaxed);

                    let request = request::Request::new(req, auth, batch_counter, current);
                    request.execute().await
                })
                .collect();

            #[allow(unused_must_use)]
            for task in tasks {
                task.await;
            }
        });
        futures.push(batch_executor);
    }
    join_all(futures).await;

    info!("Done!");
}
