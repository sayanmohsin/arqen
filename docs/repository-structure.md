# Repository structure

The workspace contains one Cargo package:

```text
arqen/
  crates/
    arqen/                 # Single published package: library + CLI binary
      Cargo.toml
      src/
        bin/arqen.rs       # CLI entry point
        core/              # Core types and errors
        http/              # Arqen HTTP server and routes
        agent/             # Tools and manifest generation
        auth/              # Authentication adapters and policies
        thingd/            # thingd adapters
        jobs/              # Durable job workers
        logging/           # Tracing and redaction
        config.rs          # Configuration loading
        health.rs          # Health checks and readiness
        module.rs          # Module composition
        observability.rs   # Metrics and percentiles
        openapi.rs         # OpenAPI spec generation
        state.rs           # AppState builder
        testutil.rs        # Test helpers
  docs/                    # VitePress documentation site
    .vitepress/config.ts   # Site configuration and sidebar
    index.md               # Landing page
    getting-started.md     # Quickstart guide
    commands.md            # CLI reference
    configuration.md       # Config docs
    modules.md             # Module composition
    typed-tools.md         # Tool definitions
    durable-jobs.md        # Job handlers
    authentication.md      # Auth docs
    validation.md          # Request validation
    testing.md             # Test guide
    deployment.md          # Deployment guide
    health.md              # Health and readiness
    examples.md            # Code examples
    standards.md           # Coding standards
    troubleshooting.md     # Common issues
    migration.md           # Upgrade notes
    agent-guide.md         # Agent integration
    feature-status.md      # Capability status
    architecture.md        # Architecture overview
    in-memory-mode.md      # Storage modes
    thingd-integration.md  # thingd contract
    observability.md       # Metrics
    openapi.md             # OpenAPI docs
    security.md            # Security guide
    logging.md             # Logging guide
    repository-structure.md # This file
    roadmap.md             # Project roadmap
    contributing.md        # Contribution guide
    public/                # Static assets (logo, etc.)
  examples/
    memory-backend/        # In-memory storage example
    minimal-api/           # Minimal HTTP API example
  specs/                   # Phase specifications
  .github/workflows/       # CI workflows
  CHANGELOG.md
  AGENTS.md                # Repository-local contributor instructions
  README.md
```

## Key files

- **`Cargo.toml`**: workspace root with one package, feature flags, and
  dependencies.
- **Root `AGENTS.md`**: repository-local contributor instructions. Generated
  applications receive a separate portable `AGENTS.md` with starter guidance.
- **`CHANGELOG.md`**: user-facing changes for each release.
- **`docs/.vitepress/config.ts`**: documentation site configuration and
  navigation sidebar.

## Module organization inside `crates/arqen/src/`

Core types stay independent of Axum and model providers. The thingd module
owns storage and queue adapters. The CLI is enabled with the `cli` feature
and is not a second published package.

Keep the modules composable inside the public `arqen` crate. Generated
scaffolding is replaceable without changing application domain code.
Templates should be replaceable without changing application domain code.

## Documentation site

The docs directory uses VitePress:

```bash
cd docs
pnpm install
pnpm dev        # local dev server
pnpm build      # build static site
```

The site is deployed to GitHub Pages at
https://sayanmohsin.github.io/arqen/.

## Private files

`AGENTS.md` contains private instructions for coding agents. It is not part
of the public API or documentation. Do not reference it in public docs or
commit messages.
