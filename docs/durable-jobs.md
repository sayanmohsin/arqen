# Durable jobs

Arqen jobs use thingd queues rather than an in-process task list.

## Job states

Jobs progress through the following states:

```text
queued → leased → completed
                   ↓
                 retrying → leased (retry)
                   ↓
                  dead
```

- **queued**: Job is waiting to be processed.
- **leased**: Job is claimed by a worker and being processed.
- **completed**: Job finished successfully.
- **retrying**: Job failed and is scheduled for retry with backoff.
- **dead**: Job has exhausted all retries and requires manual intervention.

## Required behavior

- structured payloads;
- deterministic idempotency keys;
- delayed availability;
- leases and lease expiry;
- retries with backoff;
- acknowledgement;
- dead-letter handling;
- graceful worker shutdown;
- structured job logs.

## Worker rules

Workers must be server-side. Mobile and browser clients may request work but must never claim or acknowledge jobs directly.

## Job metadata

Each job has:

- **id**: unique identifier
- **queue**: queue name
- **payload**: structured JSON data
- **idempotency_key**: deterministic key for deduplication
- **state**: current state (queued, leased, completed, retrying, dead)
- **attempts**: number of processing attempts
- **max_retries**: maximum allowed retries
- **lease_expires_at**: timestamp when lease expires
- **created_at**: creation timestamp
- **updated_at**: last update timestamp

## Durable scheduler

Arqen's `Scheduler` mirrors the Thingd SDK scheduler operations while adapting
handlers to Rust workers. Use `schedule`, `schedule_interval`, or
`schedule_once` to persist a schedule, then inspect it with `get`, `list`, and
`stats`; lifecycle operations are `pause`, `resume`, `run`, `remove`, `start`,
and `stop`.

The scheduler heartbeat only creates a durable queue job. It does not call the
provider or application handler:

```text
schedule heartbeat -> Thingd queue job -> Arqen worker -> job handler
```

Each occurrence uses `schedule:<schedule-id>:<scheduled-run-at>` as its
deterministic queue identity. This protects against duplicate ticks and lets a
restarted scheduler recover overdue schedules. Native Thingd supports the
deterministic and delayed queue options. The current public HTTP Thingd queue
contract does not, so Arqen returns an explicit `not_impl` error for those
operations instead of silently degrading to an in-memory timer.

Memory storage is for tests only and is not restart durable. A paused schedule
does not cancel a job that has already been enqueued. Repeated enqueue or
worker failures are recorded on the schedule and disable it after its
configured consecutive failure limit.

Example weekly schedule:

```rust,ignore
let scheduler = arqen::Scheduler::new(state.storage.clone());
scheduler.schedule("ott-release-refresh", arqen::ScheduleOptions {
    expression: Some("0 0 * * 0".into()),
    queue: "imports".into(),
    job_type: "ott_release_refresh".into(),
    payload: serde_json::json!({
        "countries": ["CA", "US"],
        "release_window": "weekly",
        "requested_horizon": 7,
    }),
    ..arqen::ScheduleOptions::new("ott_release_refresh")
}).await?;
scheduler.start().await?;
```

The registered `ott_release_refresh` worker receives the schedule ID,
scheduled run time, run timestamp, idempotency key, and application payload.
