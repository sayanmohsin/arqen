# Agent guide

Arqen treats agents as clients of explicit application capabilities, not as a
replacement for application architecture.

## What “agent-ready” means

Document capabilities so they are discoverable, typed, permission-aware,
auditable, and automation-friendly. A capability should have a stable name,
structured input and output, clear authorization rules, and an observable
result.

## Start with the manifest

Run an application and inspect:

```bash
curl http://127.0.0.1:8888/agent
curl http://127.0.0.1:8888/agent/manifest
```

The manifest is a public description of endpoints, tools, jobs, and runtime
metadata. Keep it truthful: omit unfinished capabilities rather than claiming
they are available.

## Repository conventions

Read the project `AGENTS.md`, then the relevant guide and contract before
editing. Prefer existing domain interfaces, avoid direct provider calls in
business logic, and record security or permission assumptions next to the
capability they protect.

See [agent discovery](agent-discovery.md), [typed tools](typed-tools.md), and
[security](security.md) for the current contracts.
