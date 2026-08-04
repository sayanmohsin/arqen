pub mod worker;

pub use worker::Worker;

use std::sync::Arc;
use std::time::Instant;
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
    /// Maximum number of concurrent jobs this worker can process.
    pub max_concurrency: u32,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            queue: "default".to_string(),
            worker_id: uuid::Uuid::new_v4().to_string(),
            poll_interval: Duration::from_secs(1),
            lease_seconds: 30,
            max_retries: 3,
            max_concurrency: 1,
        }
    }
}

/// Metrics for job processing.
#[derive(Debug, Clone, Default)]
pub struct JobMetrics {
    /// Total jobs processed.
    pub processed: u64,
    /// Total jobs completed successfully.
    pub completed: u64,
    /// Total jobs failed.
    pub failed: u64,
    /// Total jobs dead-lettered.
    pub dead_lettered: u64,
    /// Total processing time in milliseconds.
    pub total_duration_ms: u64,
}

impl JobMetrics {
    /// Get average processing time per job.
    pub fn avg_duration_ms(&self) -> u64 {
        self.total_duration_ms
            .checked_div(self.processed)
            .unwrap_or(0)
    }
}

/// A worker that polls a queue and processes jobs.
pub struct JobWorker {
    config: JobConfig,
    thingd: Arc<dyn crate::thingd::ThingdBackend>,
    handler: Box<dyn JobHandler>,
    shutdown_rx: watch::Receiver<bool>,
    metrics: JobMetrics,
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
            metrics: JobMetrics::default(),
        }
    }

    /// Get current metrics.
    pub fn metrics(&self) -> &JobMetrics {
        &self.metrics
    }

    /// Run the worker loop until shutdown signal is received.
    pub async fn run(&mut self) {
        info!(
            worker_id = %self.config.worker_id,
            queue = %self.config.queue,
            "Starting job worker"
        );

        loop {
            if *self.shutdown_rx.borrow() {
                info!(
                    worker_id = %self.config.worker_id,
                    "Received shutdown signal"
                );
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
                    let start = Instant::now();
                    info!(
                        job_id = %job.id,
                        worker_id = %self.config.worker_id,
                        queue = %self.config.queue,
                        attempt = job.attempts,
                        "Processing job"
                    );

                    if job.attempts > 1 {
                        warn!(
                            job_id = %job.id,
                            attempts = job.attempts,
                            "Job has been retried"
                        );
                    }

                    let result = self.handler.handle(job.payload).await;
                    let duration_ms = start.elapsed().as_millis() as u64;
                    self.metrics.processed += 1;
                    self.metrics.total_duration_ms += duration_ms;

                    match result {
                        Ok(()) => {
                            if let Err(e) =
                                self.thingd.complete_job(&self.config.queue, &job.id).await
                            {
                                error!(
                                    job_id = %job.id,
                                    error = %e,
                                    "Failed to complete job"
                                );
                            } else {
                                self.metrics.completed += 1;
                                info!(
                                    job_id = %job.id,
                                    duration_ms = duration_ms,
                                    "Job completed successfully"
                                );
                            }
                        }
                        Err(e) => {
                            self.metrics.failed += 1;
                            error!(
                                job_id = %job.id,
                                error = %e,
                                duration_ms = duration_ms,
                                "Job failed"
                            );
                            if let Err(e) = self.thingd.nack_job(&self.config.queue, &job.id).await
                            {
                                error!(
                                    job_id = %job.id,
                                    error = %e,
                                    "Failed to nack job"
                                );
                            }
                        }
                    }
                }
                Ok(None) => {
                    tokio::select! {
                        _ = sleep(self.config.poll_interval) => {},
                        _ = self.shutdown_rx.changed() => {
                            if *self.shutdown_rx.borrow() {
                                info!(
                                    worker_id = %self.config.worker_id,
                                    "Received shutdown signal during poll interval"
                                );
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        worker_id = %self.config.worker_id,
                        error = %e,
                        "Failed to claim job"
                    );
                    sleep(self.config.poll_interval).await;
                }
            }
        }

        info!(
            worker_id = %self.config.worker_id,
            metrics = ?self.metrics,
            "Worker shutting down"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_config_default() {
        let config = JobConfig::default();
        assert_eq!(config.queue, "default");
        assert_eq!(config.poll_interval, Duration::from_secs(1));
        assert_eq!(config.lease_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.max_concurrency, 1);
    }

    #[test]
    fn test_job_metrics_default() {
        let metrics = JobMetrics::default();
        assert_eq!(metrics.processed, 0);
        assert_eq!(metrics.completed, 0);
        assert_eq!(metrics.failed, 0);
        assert_eq!(metrics.avg_duration_ms(), 0);
    }

    #[test]
    fn test_job_metrics_avg_duration() {
        let metrics = JobMetrics {
            processed: 10,
            total_duration_ms: 1000,
            ..Default::default()
        };
        assert_eq!(metrics.avg_duration_ms(), 100);
    }
}
