# Interfaces

- Base URL: `THINGD_URL`, default `http://127.0.0.1:8757`.
- Auth: `Authorization: Bearer <server token>`; optional `X-Tenant-Id`.
- Objects: `/v1/objects`, `/v1/objects/{collection}/{id}`, `/v1/objects/batch`.
- Events: `/v1/events/{stream}` and `/v1/events`.
- Queues: `/v1/queues/{queue}/push|claim|ack|nack|jobs|dead`.
- Search and links: documented `/v1/search` and `/v1/links` contracts.
