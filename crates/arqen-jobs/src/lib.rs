use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

pub mod worker;

pub use worker::Worker;

#[derive(Debug, Clone)]
pub struct JobConfig {
    pub queue: String,
    pub worker_id: String,
    pub poll_interval: Duration,
    pub lease_seconds: u32,
    pub max_retries: u32,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            queue: "default".to_string(),
            worker_id: uuid::Uuid::new_v4().to_string(),
            poll_interval: Duration::from_secs(1),
            lease_seconds: 30,
            max_retries: 3,
        }
    }
}

pub struct JobWorker {
    config: JobConfig,
    thingd: Arc<dyn arqen_thingd::ThingdBackend>,
    handler: Box<dyn JobHandler>,
    shutdown_rx: watch::Receiver<bool>,
}

#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    async fn handle(&self, payload: serde_json::Value) -> Result<(), arqen_core::AppError>;
}

impl JobWorker {
    pub fn new(
        config: JobConfig,
        thingd: Arc<dyn arqen_thingd::ThingdBackend>,
        handler: Box<dyn JobHandler>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            config,
            thingd,
            handler,
            shutdown_rx,
        }
    }

    pub async fn run(&mut self) {
        info!(
            "Starting worker {} for queue {}",
            self.config.worker_id, self.config.queue
        );

        loop {
            // Check for shutdown signal
            if *self.shutdown_rx.borrow() {
                info!("Worker {} received shutdown signal", self.config.worker_id);
                break;
            }

            match self
                .thingd
                .claim_job(
                    &self.config.queue,
                    &self.config.worker_id,
                    self.config.lease_seconds,
                )
                .await
            {
                Ok(Some(job)) => {
                    info!(
                        "Processing job {} on worker {}",
                        job.id, self.config.worker_id
                    );

                    // Check idempotency (if job has been processed before)
                    if job.attempts > 1 {
                        warn!("Job {} has been attempted {} times", job.id, job.attempts);
                    }

                    match self.handler.handle(job.payload).await {
                        Ok(()) => {
                            if let Err(e) =
                                self.thingd.complete_job(&self.config.queue, &job.id).await
                            {
                                error!("Failed to complete job {}: {}", job.id, e);
                            } else {
                                info!("Job {} completed successfully", job.id);
                            }
                        }
                        Err(e) => {
                            error!("Job {} failed: {}", job.id, e);
                            if let Err(e) = self.thingd.nack_job(&self.config.queue, &job.id).await
                            {
                                error!("Failed to nack job {}: {}", job.id, e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No jobs available, wait before polling again
                    tokio::select! {
                        _ = sleep(self.config.poll_interval) => {},
                        _ = self.shutdown_rx.changed() => {
                            if *self.shutdown_rx.borrow() {
                                info!("Worker {} received shutdown signal during sleep", self.config.worker_id);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to claim job: {}", e);
                    sleep(self.config.poll_interval).await;
                }
            }
        }

        info!("Worker {} shutting down", self.config.worker_id);
    }
}
