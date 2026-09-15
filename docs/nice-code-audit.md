# Nice Code audit

Nice Code is a source-backed review layer for engineering risks that compiler,
formatter, and linter checks do not fully judge. Arqen runs it through
`scripts/check-nice-code.sh`, using a checkout of
[`sayanmohsin/nice-code`](https://github.com/sayanmohsin/nice-code).

## Current baseline

The full Nice Code `v0.3.2` audit on 2026-09-14 scanned 73 supported files and
reported `PASS`: zero findings and no blocked checks. The audit is advisory for
review findings, while critical findings and failed native checks remain
actionable in CI.

## Improvement plan

1. Review each development output site and classify it as CLI presentation,
   operator-facing logging, or test-only output.
2. Keep CLI presentation human-readable, but use structured tracing for
   operational events that need filtering or correlation.
3. Add tests around any output contract that is consumed by scripts or users.
4. Re-run Nice Code after changes and keep the explicit docs exception narrow.

## Dependency upgrade plan

`cargo update --workspace` was run against the Arqen root workspace on
2026-09-14. The lockfile was refreshed for the latest compatible releases,
including `async-compression`, `compression-codecs`, the Clap family, Quinn,
and Rustls. Three newer transitive versions remain constrained by exact
requirements in their upstream packages.

Treat those as a separate compatibility upgrade: update one dependency family
at a time, run the full-feature build and tests, review public API changes,
then update the release documentation. Keep Thingd constrained to
`>=0.87.0, <0.88.0` until the Arqen adapter contract is explicitly revalidated (validated 2026-09-14 for 0.87.0 + open-envault 0.4.0).
