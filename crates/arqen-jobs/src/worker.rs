use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::sync::watch;
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
        thingd: Arc<dyn arqen_thingd::ThingdBackend>,
        handler: Box<dyn JobHandler>,
    ) {
        let shutdown_rx = self.shutdown_tx.subscribe();
        let mut worker = JobWorker::new(config, thingd, handler, shutdown_rx);
        let handle = tokio::spawn(async move {
            worker.run().await;
        });
        self.handles.push(handle);
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