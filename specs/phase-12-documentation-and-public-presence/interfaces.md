# Interfaces

## Repository identity

Repository target: `sayanmohsin/arqen`.

- Homepage: `https://sayanmohsin.github.io/arqen`
- Topics: `rust`, `backend`, `axum`, `thingd`, `agents`, `automation`, `durable-jobs`, `developer-tools`
- Description: the public description in `README.md` and this phase README.

## Documentation interface

`docs/package.json` provides `dev`, `build`, and `preview`. `docs/.vitepress/config.ts`
uses base `/arqen/`, local search, and the navigation groups Start, Concepts,
Guides, Operations, Agent Integration, Reference, and Project.

## Agent-readable conventions

Every capability claim must identify its status, public boundary, permission
assumptions, and verification command or evidence. “Agent-ready” means
discoverable, typed, permission-aware, auditable, and automation-friendly; it
does not mean AI-only.

## Deployment interface

`.github/workflows/docs.yml` builds from `docs/`, uploads
`docs/.vitepress/dist`, and deploys GitHub Pages only for `main`.
