
use tokio::sync::watch::Receiver;
use tracing::info;
use crate::request;

pub(crate) struct BatchExecutor {
    id: usize,
    requests: Vec<request::Request>,
}
impl BatchExecutor {
    pub(crate) fn new(id: usize, requests: Vec<request::Request>) -> Self {
        Self { id, requests }
    }
    pub(crate) async fn start(self, mut start_signal_receiver: Receiver<()>) {
        info!("starting executor: {}", self.id);
        start_signal_receiver
            .changed()
            .await
            .expect("error receiving start signal");
        #[allow(unused_must_use)]
        for task in self.requests {
            task.execute().await;
        }
    }
}