# Agent discovery

Generated applications should be understandable by both people and coding agents.

Expose:

```text
GET /agent
GET /agent/manifest
GET /docs
GET /health
GET /ready
```

## Endpoint responses

### GET /agent

Returns a minimal agent description:

```json
{
  "name": "my-app",
  "version": "0.1.0",
  "description": "A sample Arqen application",
  "storage_mode": "memory"
}
```

### GET /agent/manifest

Returns the full agent manifest with tool definitions, schemas, and metadata:

```json
{
  "name": "my-app",
  "version": "0.1.0",
  "description": "A sample Arqen application",
  "storage_mode": "memory",
  "tools": [
    {
      "name": "create_user",
      "description": "Create a new user account",
      "input": {
        "type": "object",
        "properties": {
          "email": { "type": "string", "format": "email" },
          "name": { "type": "string" }
        },
        "required": ["email", "name"]
      },
      "output": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "email": { "type": "string" },
          "name": { "type": "string" }
        }
      },
      "scopes": ["users:write"],
      "effect": "write",
      "idempotent": false
    }
  ]
}
```

### GET /docs

Returns an HTML page with API documentation, or redirects to OpenAPI/Swagger UI.

### GET /health

Returns liveness status:

```json
{
  "status": "ok"
}
```

### GET /ready

Returns readiness status, checking dependencies:

```json
{
  "status": "ok",
  "checks": {
    "thingd": "ok"
  }
}
```

The manifest describes the application, tools, input/output schemas, required scopes, read/write effects, idempotency behavior, and operations that enqueue jobs.

Every generated repository should include `AGENTS.md` with the project layout, start commands, test commands, storage mode, credential rules, and instructions for adding routes, tools, repositories, and jobs.

## Example AGENTS.md

```markdown
# Project: my-app

## Overview

An Arqen application with user management tools.

## Start commands

- Development: `arqen dev`
- Production: `arqen start`
- Tests: `arqen test`

## Storage mode

Memory (development) / HTTP (production)

## Credential rules

- Never commit credentials
- Use ARQEN_* environment variables
- See docs/security.md

## Adding routes

1. Create handler in src/handlers/
2. Register in src/routes.rs
3. Add OpenAPI annotations

## Adding tools

1. Define tool function with #[tool] attribute
2. Implement in src/tools/
3. Tool metadata is auto-generated

## Adding repositories

1. Define the trait in `arqen::core`
2. Implement the adapter in `arqen::thingd`
3. Register in application state

## Adding jobs

1. Define job payload
2. Implement worker function
3. Register in job configuration
```
