# Arqen phase specifications

This directory contains implementation-ready specifications. Each phase can
be assigned to one agent, but dependencies must be respected.

## Agent reading order

1. Read this file and `STATUS.md`.
2. Read the selected phase `README.md`.
3. Read `scope.md` and `interfaces.md`.
4. Implement `tasks.md` in order.
5. Run `test-plan.md`, complete `acceptance.md`, and write `handoff.md`.
6. Update `STATUS.md` with evidence.

Choose the earliest phase with status `ready`. Do not start a blocked phase.
Agents may change only the declared files and repositories. Changes to
Watchloom, thingd, or thingd-cloud require an explicit cross-repository note.
Record blockers with evidence, attempted alternatives, and the exact decision
or external change required.

The extension phases close gaps found during production review:

- Phase 08: native thingd crate integration.
- Phase 09: runtime production hardening.
- Phase 10: public thingd HTTP contract.
- Phase 11: CLI, packaging, and release validation.

A phase is complete only when acceptance, tests, public interfaces, limitations,
and next-phase handoff are all recorded.
