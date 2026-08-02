# Typed agent tools

Agent tools are ordinary typed Rust operations with generated metadata.

## Tool metadata fields

Each tool must have:

- **name**: stable snake_case identifier (e.g., `create_user`)
- **description**: human-readable explanation of what the tool does
- **input**: JSON Schema for the tool's input parameters
- **output**: JSON Schema for the tool's return value
- **scopes**: required authorization scopes (e.g., `["users:write"]`)
- **effect**: `read` or `write` classification
- **idempotent**: whether the tool is idempotent (safe to retry)
- **enqueues_job**: optional job name if the tool enqueues a durable job
- **timeout**: optional request timeout in seconds

## Example tool definition

```rust
#[tool(
    name = "create_user",
    description = "Create a new user account",
    scopes = ["users:write"],
    effect = "write",
    idempotent = false
)]
async fn create_user(
    input: CreateUserInput,
    state: AppState,
) -> Result<User, AppError> {
    // implementation
}
```

## Generated metadata

The tool metadata is exposed via the `/agent/manifest` endpoint:

```json
{
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

## Protocol surfaces

The first protocol surfaces are HTTP endpoints and generated JSON schemas. MCP exposure can be an optional adapter. Arqen remains model-provider neutral.
