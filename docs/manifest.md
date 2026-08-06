# Agent Manifest Contract

The agent manifest is a JSON document that describes the application's capabilities for agent discovery.

## Endpoint

```text
GET /agent/manifest
```

## Response format

```json
{
  "name": "string",
  "version": "string",
  "description": "string",
  "storage_mode": "memory | http",
  "tools": [
    {
      "name": "string",
      "description": "string",
      "input": { "json_schema" },
      "output": { "json_schema" },
      "scopes": ["string"],
      "effect": "read | write",
      "idempotent": "boolean",
      "enqueues_job": "string | null",
      "timeout": "integer | null"
    }
  ],
  "jobs": [
    {
      "name": "string",
      "description": "string",
      "payload": { "json_schema" },
      "queue": "string",
      "max_retries": "integer",
      "timeout": "integer"
    }
  ],
  "endpoints": [
    {
      "path": "string",
      "method": "string",
      "description": "string",
      "authenticated": "boolean"
    }
  ]
}
```

## Fields

### Application fields

- **name**: Application name (snake_case)
- **version**: Semantic version
- **description**: Human-readable description
- **storage_mode**: Current storage mode (memory or http)

### Tool fields

- **name**: Stable snake_case identifier
- **description**: What the tool does
- **input**: JSON Schema for input parameters
- **output**: JSON Schema for return value
- **scopes**: Required authorization scopes
- **effect**: Read or write classification
- **idempotent**: Whether tool is safe to retry
- **enqueues_job**: Optional job name if tool enqueues a job
- **timeout**: Optional request timeout in seconds

### Job fields

- **name**: Job type identifier
- **description**: What the job does
- **payload**: JSON Schema for job payload
- **queue**: Queue name
- **max_retries**: Maximum retry attempts
- **timeout**: Job processing timeout in seconds

### Endpoint fields

- **path**: URL path
- **method**: HTTP method
- **description**: What the endpoint does
- **authenticated**: Whether authentication is required

## Example

```json
{
  "name": "user_service",
  "version": "0.1.0",
  "description": "User management service",
  "storage_mode": "memory",
  "tools": [
    {
      "name": "create_user",
      "description": "Create a new user",
      "input": {
        "type": "object",
        "properties": {
          "email": { "type": "string" },
          "name": { "type": "string" }
        },
        "required": ["email", "name"]
      },
      "output": {
        "type": "object",
        "properties": {
          "id": { "type": "string" }
        }
      },
      "scopes": ["users:write"],
      "effect": "write",
      "idempotent": false,
      "enqueues_job": null,
      "timeout": null
    }
  ],
  "jobs": [],
  "endpoints": [
    {
      "path": "/health",
      "method": "GET",
      "description": "Liveness check",
      "authenticated": false
    }
  ]
}
```
