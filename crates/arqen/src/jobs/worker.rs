use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{JobConfig, JobHandler, JobWorker};

pub struct Worker {
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: watch::Sender<bool>,
}

impl Worker {
    pub fn new() -> Self {
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            handles: Vec::new(),
            shutdown_tx,
        }
    }

    pub fn spawn_worker(
        &mut self,
        config: JobConfig,
        thingd: Arc<dyn crate::thingd::ThingdBackend>,
        handler: Box<dyn JobHandler>,
    ) {
        let concurrency = config.max_concurrency.max(1);
        let handler: Arc<dyn JobHandler> = Arc::from(handler);
        for index in 0..concurrency {
            let mut worker_config = config.clone();
            if concurrency > 1 {
                worker_config.worker_id = format!("{}-{index}", worker_config.worker_id);
            }
            let shutdown_rx = self.shutdown_tx.subscribe();
            let mut worker =
                JobWorker::new_shared(worker_config, thingd.clone(), handler.clone(), shutdown_rx);
            let handle = tokio::spawn(async move {
                worker.run().await;
            });
            self.handles.push(handle);
        }
    }

    pub fn shutdown_signal(&self) -> watch::Sender<bool> {
        self.shutdown_tx.clone()
    }

    pub async fn shutdown(self) {
        info!("Shutting down workers");
        // Send shutdown signal
        let _ = self.shutdown_tx.send(true);

        // Wait for all workers to finish
        for handle in self.handles {
            let _ = handle.await;
        }
        info!("All workers shut down");
    }
}

impl Default for Worker {
    fn default() -> Self {
        Self::new()
    }
}
