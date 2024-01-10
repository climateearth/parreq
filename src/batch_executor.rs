use async_trait::async_trait;
use tokio::sync::watch::Receiver;
use tracing::info;

#[async_trait]
pub trait Executable: Sized {
    type Result;
    // fn setup(&mut self);
    async fn execute(self) -> Self::Result;
}

pub(crate) struct BatchExecutor<E: Executable> {
    id: usize,
    tasks: Vec<E>
}
impl<E: Executable> BatchExecutor<E> {
    pub(crate) fn new(id: usize, tasks: Vec<E>) -> Self {
        Self { id, tasks }
    }
    pub(crate) async fn start(
        self,
        mut start_signal_receiver: Receiver<()>,
    ) {
        info!("starting executor: {}", self.id);
        start_signal_receiver
            .changed()
            .await
            .expect("error receiving start signal");
        for task in self.tasks {
            task.execute().await;
        }
    }
}
