# Nice Code audit

Nice Code is a source-backed review layer for engineering risks that compiler,
formatter, and linter checks do not fully judge. Arqen runs it through
`scripts/check-nice-code.sh`, using a checkout of
[`sayanmohsin/nice-code`](https://github.com/sayanmohsin/nice-code).

## Baseline findings

The full audit on 2026-08-27 scanned 71 supported files and reported two
documented exceptions:

- `docs/.vitepress/config.ts` intentionally falls back when Git metadata is
  unavailable in an archive.
- `crates/arqen/src/dev.rs` is an intentional human-readable local CLI and
  child-process output path, not production telemetry.

The current Nice Code result is `PASS` with zero unsuppressed findings.

## Improvement plan

1. Review each development output site and classify it as CLI presentation,
   operator-facing logging, or test-only output.
2. Keep CLI presentation human-readable, but use structured tracing for
   operational events that need filtering or correlation.
3. Add tests around any output contract that is consumed by scripts or users.
4. Re-run Nice Code after changes and keep the explicit docs exception narrow.

## Dependency upgrade plan

`cargo update --workspace` was run against the Arqen root workspace on
2026-08-27. The lockfile already contains the newest versions allowed by the
current manifest ranges, so no dependency files changed. Cargo reports 16
packages with newer major releases, including Axum 0.8, Reqwest 0.13, Tower
0.5, Thiserror 2, and Toml 1.1.

Treat those as a separate compatibility upgrade: update one dependency family
at a time, run the full-feature build and tests, review public API changes,
then update the release documentation. Keep Thingd constrained to
`>=0.85.0, <0.86.0` until the Arqen adapter contract is explicitly revalidated.

The checker is advisory for review findings; critical findings and failed
native checks remain actionable in CI.
