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

- Use HTTPS in production
- Validate TLS certificates
- Set appropriate CORS policies
- Use secure cookie attributes

## Authentication and authorization

- Tools have explicit authorization scopes
- Health and readiness endpoints are unauthenticated by default
- Agent discovery endpoints may be authenticated in production
- Use shortest possible token lifetimes

`JwtAuth::with_issuer()` and `JwtAuth::with_audience()` validate the configured
issuer and audience. Set both when the identity provider issues tokens for
multiple services. See the [authentication guide](./authentication.md) for
the supported Arqen APIs.

The October 2026 source audit found that `Secret<T>` redacts `Debug` and
`Display`, while its `Serialize` implementation emits the wrapped value. Treat
serialized configuration containing `Secret<T>` as credential-bearing. Do not
serialize it into logs, API responses, or diagnostic output. The audit also
found that CLI bearer-token arguments can appear in process listings; prefer
environment variables or a secret provider on shared hosts. See the
[security audit](./security-audit-2026-10.md) for evidence, remaining risks, and
the recommended hardening order.

## Input validation

- Validate all input against JSON Schema
- Reject malformed requests early
- Use type-safe parsing
- Prevent injection attacks through proper escaping
