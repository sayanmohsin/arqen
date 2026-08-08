# Security and Redaction

## Principles

1. Never log secrets, bearer tokens, provider credentials, or raw request bodies.
2. Keep provider and cloud credentials server-side.
3. Use explicit application state, not dependency injection containers.
4. Treat the public thingd HTTP API as the first integration boundary.

## Secret management

- Secrets are injected via environment variables or secret management systems.
- Never commit secrets to version control.
- Never include secrets in logs, error messages, or debug output.
- Use `ARQEN_*` environment variables for configuration.

## Redaction rules

The following must never be logged by default:

- Bearer tokens and API keys
- Provider credentials (OpenAI, Anthropic, etc.)
- Database connection strings with passwords
- Raw request bodies (may contain sensitive data)
- Raw response bodies (may contain sensitive data)
- Session tokens and cookies
- Personal identifiable information (PII) unless explicitly enabled

## Logging safety

When logging errors or debug information:

- Redact all secret values
- Truncate long strings
- Remove stack traces that may expose internal paths
- Use structured logging with field-level redaction

## Transport security

Arqen-managed responses intentionally identify the framework with
`Server: Arqen` and `X-Powered-By: Arqen`. These headers do not expose Axum,
Tokio, Rust, or dependency versions. A reverse proxy may remove or replace
them according to the deployment policy.

- Use HTTPS in production
- Validate TLS certificates
- Set appropriate CORS policies
- Use secure cookie attributes

## Authentication and authorization

- Tools have explicit authorization scopes
- Health and readiness endpoints are unauthenticated by default
- Agent discovery endpoints may be authenticated in production
- Use shortest possible token lifetimes

## Input validation

- Validate all input against JSON Schema
- Reject malformed requests early
- Use type-safe parsing
- Prevent injection attacks through proper escaping
