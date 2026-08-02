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
