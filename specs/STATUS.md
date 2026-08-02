# Phase status

| Phase | Status | Depends on | Owner | Last update |
|---|---|---|---|---|
| 01 | review | — | opencode | 2026-08-01 |
| 02 | review | 01 | opencode | 2026-08-01 |
| 03 | review | 01, 02 | opencode | 2026-08-01 |
| 04 | review | 03 | opencode | 2026-08-01 |
| 05 | review | 02, 03, 04 | opencode | 2026-08-01 |
| 06 | planned | 02, 03, 04, 05 | unassigned | — |
| 07 | blocked | 03, 04, 05, public cloud contract | unassigned | — |
| 08 | ready | 01, 03 | unassigned | 2026-08-01 |
| 09 | ready | 02, 03, 04 | unassigned | 2026-08-01 |
| 10 | ready | 03, public thingd REST contract | unassigned | 2026-08-01 |
| 11 | planned | 02, 05, 08, 09 | unassigned | — |
| 12 | ready | 01, 02, 05, 08, 09, 11 | unassigned | 2026-08-01 |

Allowed statuses: `planned`, `ready`, `in_progress`, `blocked`, `review`, `completed`.
Normally only one phase is `in_progress`.
