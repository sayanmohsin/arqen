//! Durable, Thingd-backed scheduling for Arqen applications.
//!
//! The scheduler owns timing and schedule state. It never runs application
//! work directly; each due occurrence is written to a Thingd queue for an
//! Arqen worker to claim.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, watch};
use tokio::task::JoinHandle;
use tokio::time::{Duration as TokioDuration, sleep};
use tracing::{error, info, warn};

use crate::core::{AppError, ErrorKind};
use crate::thingd::{PushJobOptions, QueryOptions, ThingdBackend};

const COLLECTION: &str = "_arqen_schedules";
const DEFAULT_QUEUE: &str = "default";
const DEFAULT_MAX_FAILURES: u32 = 5;

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("scheduler storage error: {0}")]
    Storage(#[from] AppError),
    #[error("schedule '{0}' was not found")]
    NotFound(String),
    #[error("invalid schedule: {0}")]
    Invalid(String),
    #[error("scheduler operation is not supported by this Thingd mode: {0}")]
    Unsupported(String),
    #[error("scheduler is not running")]
    NotRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleStatus {
    Running,
    Completed,
    Failed,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub expression: String,
    pub timezone: Option<String>,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub enabled: bool,
    pub next_run_at: String,
    pub last_run_at: Option<String>,
    pub last_status: Option<ScheduleStatus>,
    pub last_error: Option<String>,
    pub last_duration_ms: Option<u64>,
    pub run_count: u64,
    pub fail_count: u64,
    pub consecutive_fails: u32,
    pub max_consecutive_fails: u32,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct ScheduleOptions {
    pub expression: Option<String>,
    pub interval: Option<TokioDuration>,
    pub timezone: Option<String>,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub enabled: bool,
    pub max_consecutive_fails: u32,
    pub metadata: Option<serde_json::Value>,
}

impl ScheduleOptions {
    pub fn new(job_type: impl Into<String>) -> Self {
        Self {
            job_type: job_type.into(),
            queue: DEFAULT_QUEUE.to_string(),
            payload: serde_json::json!({}),
            enabled: true,
            max_consecutive_fails: DEFAULT_MAX_FAILURES,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduleIntervalOptions {
    pub interval: TokioDuration,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub enabled: bool,
    pub max_consecutive_fails: u32,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ScheduleOnceOptions {
    pub run_at: DateTime<Utc>,
    pub queue: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEvent {
    pub schedule_id: String,
    pub status: ScheduleStatus,
    pub timestamp: String,
    pub run_count: u64,
    pub fail_count: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub running: usize,
    pub next_run: Option<(String, String)>,
}

pub struct Scheduler {
    backend: Arc<dyn ThingdBackend>,
    running: Arc<Mutex<HashSet<String>>>,
    heartbeat: Mutex<Option<JoinHandle<()>>>,
    shutdown: watch::Sender<bool>,
    started: Mutex<bool>,
    heartbeat_interval: TokioDuration,
}

impl Scheduler {
    pub fn new(backend: Arc<dyn ThingdBackend>) -> Arc<Self> {
        let (shutdown, _) = watch::channel(false);
        Arc::new(Self {
            backend,
            running: Arc::new(Mutex::new(HashSet::new())),
            heartbeat: Mutex::new(None),
            shutdown,
            started: Mutex::new(false),
            heartbeat_interval: TokioDuration::from_secs(1),
        })
    }

    pub async fn schedule(
        &self,
        id: impl Into<String>,
        options: ScheduleOptions,
    ) -> Result<Schedule, SchedulerError> {
        let id = id.into();
        if options.queue.is_empty() || options.job_type.is_empty() {
            return Err(SchedulerError::Invalid(
                "queue and job_type are required".into(),
            ));
        }
        let expression = match (options.expression, options.interval) {
            (Some(expression), None) => expression,
            (None, Some(interval)) => format!("{}ms", interval.as_millis()),
            _ => {
                return Err(SchedulerError::Invalid(
                    "provide exactly one expression or interval".into(),
                ));
            }
        };
        let next = next_run(&expression, options.timezone.as_deref(), Utc::now())?;
        let now = Utc::now().to_rfc3339();
        let created_at = self
            .get(&id)
            .await?
            .map(|s| s.created_at)
            .unwrap_or_else(|| now.clone());
        let schedule = Schedule {
            id: id.clone(),
            expression,
            timezone: options.timezone,
            queue: options.queue,
            job_type: options.job_type,
            payload: options.payload,
            enabled: options.enabled,
            next_run_at: next.to_rfc3339(),
            last_run_at: None,
            last_status: None,
            last_error: None,
            last_duration_ms: None,
            run_count: 0,
            fail_count: 0,
            consecutive_fails: 0,
            max_consecutive_fails: options.max_consecutive_fails.max(1),
            created_at,
            updated_at: now,
            metadata: options.metadata,
        };
        self.save(&schedule).await?;
        Ok(schedule)
    }

    pub async fn schedule_interval(
        &self,
        id: impl Into<String>,
        options: ScheduleIntervalOptions,
    ) -> Result<Schedule, SchedulerError> {
        self.schedule(
            id,
            ScheduleOptions {
                interval: Some(options.interval),
                queue: options.queue,
                job_type: options.job_type,
                payload: options.payload,
                enabled: options.enabled,
                max_consecutive_fails: options.max_consecutive_fails,
                metadata: options.metadata,
                ..Default::default()
            },
        )
        .await
    }

    pub async fn schedule_once(
        &self,
        id: impl Into<String>,
        options: ScheduleOnceOptions,
    ) -> Result<Schedule, SchedulerError> {
        let id = id.into();
        let now = Utc::now().to_rfc3339();
        let created_at = self
            .get(&id)
            .await?
            .map(|s| s.created_at)
            .unwrap_or_else(|| now.clone());
        let schedule = Schedule {
            id,
            expression: format!("once:{}", options.run_at.to_rfc3339()),
            timezone: None,
            queue: options.queue,
            job_type: options.job_type,
            payload: options.payload,
            enabled: true,
            next_run_at: options.run_at.to_rfc3339(),
            last_run_at: None,
            last_status: None,
            last_error: None,
            last_duration_ms: None,
            run_count: 0,
            fail_count: 0,
            consecutive_fails: 0,
            max_consecutive_fails: 1,
            created_at,
            updated_at: now,
            metadata: options.metadata,
        };
        self.save(&schedule).await?;
        Ok(schedule)
    }

    pub async fn get(&self, id: &str) -> Result<Option<Schedule>, SchedulerError> {
        self.backend
            .get_object(COLLECTION, id)
            .await?
            .map(|object| {
                serde_json::from_value(object.data)
                    .map_err(|error| SchedulerError::Invalid(error.to_string()))
            })
            .transpose()
    }

    pub async fn list(&self) -> Result<Vec<Schedule>, SchedulerError> {
        let objects = self
            .backend
            .query_objects(COLLECTION, QueryOptions::default())
            .await?;
        objects
            .into_iter()
            .map(|object| {
                serde_json::from_value(object.data)
                    .map_err(|e| SchedulerError::Invalid(e.to_string()))
            })
            .collect()
    }

    pub async fn pause(&self, id: &str) -> Result<Schedule, SchedulerError> {
        let mut schedule = self.require(id).await?;
        schedule.enabled = false;
        schedule.updated_at = Utc::now().to_rfc3339();
        self.save(&schedule).await?;
        Ok(schedule)
    }

    pub async fn resume(&self, id: &str) -> Result<Schedule, SchedulerError> {
        let mut schedule = self.require(id).await?;
        schedule.enabled = true;
        schedule.next_run_at = next_run(
            &schedule.expression,
            schedule.timezone.as_deref(),
            Utc::now(),
        )?
        .to_rfc3339();
        schedule.updated_at = Utc::now().to_rfc3339();
        self.save(&schedule).await?;
        Ok(schedule)
    }

    pub async fn remove(&self, id: &str) -> Result<bool, SchedulerError> {
        let existed = self.get(id).await?.is_some();
        self.backend.delete_object(COLLECTION, id).await?;
        Ok(existed)
    }

    pub async fn run(&self, id: &str) -> Result<(), SchedulerError> {
        let schedule = self.require(id).await?;
        self.enqueue(schedule, Utc::now()).await
    }

    pub async fn stats(&self) -> Result<SchedulerStats, SchedulerError> {
        let schedules = self.list().await?;
        let enabled = schedules.iter().filter(|s| s.enabled).count();
        let next_run = schedules
            .iter()
            .filter(|s| s.enabled)
            .min_by_key(|s| s.next_run_at.clone())
            .map(|s| (s.id.clone(), s.next_run_at.clone()));
        Ok(SchedulerStats {
            total: schedules.len(),
            enabled,
            disabled: schedules.len() - enabled,
            running: self.running.lock().await.len(),
            next_run,
        })
    }

    pub async fn is_started(&self) -> bool {
        *self.started.lock().await
    }

    pub async fn start(self: &Arc<Self>) -> Result<(), SchedulerError> {
        let mut started = self.started.lock().await;
        if *started {
            return Ok(());
        }
        let _ = self.shutdown.send(false);
        *started = true;
        let this = Arc::clone(self);
        let mut shutdown = this.shutdown.subscribe();
        let handle = tokio::spawn(async move {
            info!("Starting Arqen durable scheduler");
            loop {
                tokio::select! {
                    _ = sleep(this.heartbeat_interval) => {
                        if let Err(error) = this.heartbeat_once().await { warn!(%error, "scheduler heartbeat failed"); }
                    }
                    result = shutdown.changed() => {
                        if result.is_err() || *shutdown.borrow() { break; }
                    }
                }
            }
            info!("Arqen durable scheduler stopped");
        });
        *self.heartbeat.lock().await = Some(handle);
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), SchedulerError> {
        let _ = self.shutdown.send(true);
        if let Some(handle) = self.heartbeat.lock().await.take() {
            let _ = handle.await;
        }
        *self.started.lock().await = false;
        Ok(())
    }

    async fn heartbeat_once(&self) -> Result<(), SchedulerError> {
        let now = Utc::now();
        for schedule in self.list().await? {
            if schedule.enabled
                && DateTime::parse_from_rfc3339(&schedule.next_run_at)
                    .map(|at| at.with_timezone(&Utc) <= now)
                    .unwrap_or(false)
                && let Err(error) = self.enqueue(schedule, now).await
            {
                error!(%error, "failed to enqueue scheduled run");
            }
        }
        Ok(())
    }

    async fn enqueue(
        &self,
        mut schedule: Schedule,
        run_at: DateTime<Utc>,
    ) -> Result<(), SchedulerError> {
        {
            let mut running = self.running.lock().await;
            if !running.insert(schedule.id.clone()) {
                return Ok(());
            }
        }
        let result = self.enqueue_inner(&mut schedule, run_at).await;
        self.running.lock().await.remove(&schedule.id);
        result
    }

    async fn enqueue_inner(
        &self,
        schedule: &mut Schedule,
        run_at: DateTime<Utc>,
    ) -> Result<(), SchedulerError> {
        let run_key = format!("schedule:{}:{}", schedule.id, schedule.next_run_at);
        let payload = serde_json::json!({
            "job_type": schedule.job_type,
            "payload": schedule.payload,
            "schedule_id": schedule.id,
            "scheduled_run_at": schedule.next_run_at,
            "run_timestamp": run_at.to_rfc3339(),
            "idempotency_key": run_key,
        });
        let result = self
            .backend
            .push_job_with_options(
                &schedule.queue,
                payload,
                3,
                PushJobOptions {
                    idempotency_key: Some(run_key),
                    delay_ms: None,
                },
            )
            .await;
        match result {
            Ok(_)
            | Err(AppError {
                kind: ErrorKind::Conflict,
                ..
            }) => {
                schedule.run_count += 1;
                schedule.last_status = Some(ScheduleStatus::Running);
                schedule.last_run_at = Some(run_at.to_rfc3339());
                schedule.last_error = None;
                schedule.next_run_at = if schedule.expression.starts_with("once:") {
                    schedule.enabled = false;
                    schedule.next_run_at.clone()
                } else {
                    next_run(&schedule.expression, schedule.timezone.as_deref(), run_at)?
                        .to_rfc3339()
                };
            }
            Err(error) => {
                schedule.fail_count += 1;
                schedule.consecutive_fails += 1;
                schedule.last_status = Some(ScheduleStatus::Failed);
                schedule.last_error = Some(error.to_string());
                schedule.last_run_at = Some(run_at.to_rfc3339());
                if schedule.consecutive_fails >= schedule.max_consecutive_fails {
                    schedule.enabled = false;
                    schedule.last_status = Some(ScheduleStatus::Disabled);
                }
                schedule.updated_at = Utc::now().to_rfc3339();
                self.save(schedule).await?;
                return Err(error.into());
            }
        }
        schedule.updated_at = Utc::now().to_rfc3339();
        self.save(schedule).await
    }

    async fn require(&self, id: &str) -> Result<Schedule, SchedulerError> {
        self.get(id)
            .await?
            .ok_or_else(|| SchedulerError::NotFound(id.to_string()))
    }
    async fn save(&self, schedule: &Schedule) -> Result<(), SchedulerError> {
        self.backend
            .put_object(
                COLLECTION,
                &schedule.id,
                serde_json::to_value(schedule)
                    .map_err(|e| SchedulerError::Invalid(e.to_string()))?,
            )
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

fn next_run(
    expression: &str,
    timezone: Option<&str>,
    from: DateTime<Utc>,
) -> Result<DateTime<Utc>, SchedulerError> {
    if let Some(value) = expression.strip_prefix("once:") {
        return DateTime::parse_from_rfc3339(value)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|e| SchedulerError::Invalid(e.to_string()));
    }
    if let Some(value) = expression.strip_suffix("ms") {
        let millis: i64 = value
            .parse()
            .map_err(|_| SchedulerError::Invalid("interval must be numeric milliseconds".into()))?;
        return Ok(from + Duration::milliseconds(millis));
    }
    if let Some(value) = expression.strip_suffix('s') {
        let seconds: i64 = value
            .parse()
            .map_err(|_| SchedulerError::Invalid("interval must be numeric seconds".into()))?;
        return Ok(from + Duration::seconds(seconds));
    }
    if let Some(value) = expression.strip_suffix('m') {
        let minutes: i64 = value
            .parse()
            .map_err(|_| SchedulerError::Invalid("interval must be numeric minutes".into()))?;
        return Ok(from + Duration::minutes(minutes));
    }
    if let Some(value) = expression.strip_suffix('h') {
        let hours: i64 = value
            .parse()
            .map_err(|_| SchedulerError::Invalid("interval must be numeric hours".into()))?;
        return Ok(from + Duration::hours(hours));
    }
    if let Some(value) = expression.strip_suffix('d') {
        let days: i64 = value
            .parse()
            .map_err(|_| SchedulerError::Invalid("interval must be numeric days".into()))?;
        return Ok(from + Duration::days(days));
    }
    if let Some(timezone) = timezone
        && timezone != "UTC"
        && timezone != "Etc/UTC"
    {
        return Err(SchedulerError::Unsupported(
            "only UTC timezone is supported by the Rust adapter".into(),
        ));
    }
    cron_next(expression, from)
}

fn cron_next(expression: &str, from: DateTime<Utc>) -> Result<DateTime<Utc>, SchedulerError> {
    let parts: Vec<_> = expression.split_whitespace().collect();
    if !(parts.len() == 5 || parts.len() == 6) {
        return Err(SchedulerError::Invalid(
            "cron requires 5 or 6 fields".into(),
        ));
    }
    let (minute, hour, day, month, weekday) = if parts.len() == 5 {
        (&parts[0], &parts[1], &parts[2], &parts[3], &parts[4])
    } else {
        (&parts[1], &parts[2], &parts[3], &parts[4], &parts[5])
    };
    let mut candidate = from + Duration::minutes(1);
    candidate = candidate
        .with_second(0)
        .and_then(|value| value.with_nanosecond(0))
        .ok_or_else(|| SchedulerError::Invalid("invalid cron time".into()))?;
    for _ in 0..(366 * 24 * 60) {
        if field_matches(minute, candidate.minute(), 0, 59)
            && field_matches(hour, candidate.hour(), 0, 23)
            && field_matches(day, candidate.day(), 1, 31)
            && field_matches(month, candidate.month(), 1, 12)
            && weekday_matches(weekday, candidate.weekday())
        {
            return Ok(candidate);
        }
        candidate += Duration::minutes(1);
    }
    Err(SchedulerError::Invalid(
        "cron expression has no run within one year".into(),
    ))
}

fn field_matches(field: &str, value: u32, min: u32, max: u32) -> bool {
    field == "*"
        || field.split(',').any(|part| {
            if let Some(step) = part.strip_prefix("*/") {
                step.parse::<u32>()
                    .ok()
                    .is_some_and(|step| step > 0 && (value - min).is_multiple_of(step))
            } else if let Some((start, end)) = part.split_once('-') {
                start.parse::<u32>().ok().is_some_and(|start| {
                    end.parse::<u32>()
                        .ok()
                        .is_some_and(|end| value >= start && value <= end)
                })
            } else {
                part.parse::<u32>()
                    .ok()
                    .is_some_and(|parsed| parsed >= min && parsed <= max && parsed == value)
            }
        })
}

fn weekday_matches(field: &str, weekday: Weekday) -> bool {
    if field == "*" {
        return true;
    }
    let value = match weekday {
        Weekday::Sun => 0,
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
    };
    field_matches(field, value, 0, 7) || (value == 0 && field_matches(field, 7, 0, 7))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thingd::{MemoryThingdBackend, ThingdBackend};
    use chrono::TimeZone;

    fn scheduler() -> Arc<Scheduler> {
        Scheduler::new(Arc::new(MemoryThingdBackend::new()))
    }

    #[tokio::test]
    async fn persists_and_lists_interval_schedule() {
        let scheduler = scheduler();
        let schedule = scheduler
            .schedule_interval(
                "heartbeat",
                ScheduleIntervalOptions {
                    interval: TokioDuration::from_secs(60),
                    queue: "jobs".into(),
                    job_type: "heartbeat".into(),
                    payload: serde_json::json!({"ok": true}),
                    enabled: true,
                    max_consecutive_fails: 3,
                    metadata: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(schedule.id, "heartbeat");
        assert_eq!(scheduler.list().await.unwrap().len(), 1);
        assert_eq!(scheduler.stats().await.unwrap().enabled, 1);
    }

    #[tokio::test]
    async fn manual_run_uses_deterministic_queue_identity() {
        let backend = Arc::new(MemoryThingdBackend::new());
        let scheduler = Scheduler::new(backend.clone());
        scheduler
            .schedule_interval(
                "manual",
                ScheduleIntervalOptions {
                    interval: TokioDuration::from_secs(60),
                    queue: "jobs".into(),
                    job_type: "work".into(),
                    payload: serde_json::json!({"value": 1}),
                    enabled: true,
                    max_consecutive_fails: 3,
                    metadata: None,
                },
            )
            .await
            .unwrap();
        scheduler.run("manual").await.unwrap();
        let job = backend
            .claim_job("jobs", "test", 30)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.payload["job_type"], "work");
        assert_eq!(job.payload["schedule_id"], "manual");
        assert!(
            job.payload["idempotency_key"]
                .as_str()
                .unwrap()
                .starts_with("schedule:manual:")
        );
    }

    #[tokio::test]
    async fn pause_resume_and_remove_are_persistent() {
        let scheduler = scheduler();
        scheduler
            .schedule_interval(
                "lifecycle",
                ScheduleIntervalOptions {
                    interval: TokioDuration::from_secs(60),
                    queue: "jobs".into(),
                    job_type: "work".into(),
                    payload: serde_json::json!({}),
                    enabled: true,
                    max_consecutive_fails: 2,
                    metadata: None,
                },
            )
            .await
            .unwrap();
        assert!(!scheduler.pause("lifecycle").await.unwrap().enabled);
        assert!(scheduler.resume("lifecycle").await.unwrap().enabled);
        assert!(scheduler.remove("lifecycle").await.unwrap());
        assert!(scheduler.get("lifecycle").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn once_schedule_is_disabled_after_enqueue() {
        let scheduler = scheduler();
        let run_at = Utc::now() - Duration::seconds(1);
        scheduler
            .schedule_once(
                "once",
                ScheduleOnceOptions {
                    run_at,
                    queue: "jobs".into(),
                    job_type: "work".into(),
                    payload: serde_json::json!({}),
                    metadata: None,
                },
            )
            .await
            .unwrap();
        scheduler.heartbeat_once().await.unwrap();
        assert!(!scheduler.get("once").await.unwrap().unwrap().enabled);
    }

    #[test]
    fn calculates_cron_and_rejects_unsupported_timezone() {
        let from = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let next = next_run("0 0 * * 0", None, from).unwrap();
        assert_eq!(next.hour(), 0);
        assert!(next > from);
        assert!(matches!(
            next_run("0 0 * * 0", Some("America/Toronto"), from),
            Err(SchedulerError::Unsupported(_))
        ));
    }
}
