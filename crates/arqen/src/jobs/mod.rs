pub mod worker;

pub use worker::Worker;

use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};
use tracing::{error, info, warn};

/// Configuration for a job worker.
#[derive(Debug, Clone)]
pub struct JobConfig {
    /// Queue name to consume from.
    pub queue: String,
    /// Unique worker identifier.
    pub worker_id: String,
    /// How long to sleep between polls when the queue is empty.
    pub poll_interval: Duration,
    /// Lease duration in seconds for claimed jobs.
    pub lease_seconds: u32,
    /// Maximum retries before dead-lettering.
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

/// A worker that polls a queue and processes jobs.
pub struct JobWorker {
    config: JobConfig,
    thingd: Arc<dyn crate::thingd::ThingdBackend>,
    handler: Box<dyn JobHandler>,
    shutdown_rx: watch::Receiver<bool>,
}

/// Trait for job handlers.
///
/// Implement this trait to define how jobs are processed.
#[async_trait::async_trait]
pub trait JobHandler: Send + Sync {
    /// Handle a job payload. Return `Ok(())` on success, or `Err` to nack the job.
    async fn handle(&self, payload: serde_json::Value) -> Result<(), crate::core::AppError>;
}

impl JobWorker {
    /// Create a new job worker.
    pub fn new(
        config: JobConfig,
        thingd: Arc<dyn crate::thingd::ThingdBackend>,
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

    /// Run the worker loop until shutdown signal is received.
    pub async fn run(&mut self) {
        info!(
            "Starting worker {} for queue {}",
            self.config.worker_id, self.config.queue
        );

        loop {
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
