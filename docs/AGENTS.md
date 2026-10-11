# Project documentation

## Purpose

Describe Herdr Projects' current architecture and operations, source-backed capabilities, and planned changes.

## Ownership

- `architecture.md`: implemented boundaries and explicit non-goals.
- `design-decisions.md`: cross-cutting choices, with implementation status.
- `development.md`: reproducible contributor verification and adoption provenance.
- `hermes-agent.md`: operator-facing Hermes setup, HERMES_HOME/HOME/cwd boundaries,
  integration status, and comparison/coexistence with Bot Mode and Kanban lanes.
- Existing getting-started, operations, manual-test and Herdr-notes documents remain their established operator/historical evidence surfaces.
- `product/`: current capability contracts.
- `changes/`: planned work, progress, archived receipts, and review packages.

## Local Contracts

- Separate source-backed current behaviour, accepted future direction, and third-party claims.
- CAP/CHG files supplement existing product docs; do not erase useful operating instructions or relabel historical manual checks as current runs.
- Never embed secrets, user profile contents, or invented execution results.
- Hermes examples must distinguish working Hermes provisioning commands from
  future Herdr integration commands. Keep README/getting-started entry points
  linked to the operator guide; do not copy a second setup recipe into each.
- Document the scheduler/lifecycle owner when describing multi-agent systems.
  A shared profile home is not an integration bridge or a cross-product lock.

## Work Guidance

Read both product/change child contracts before authoring records or choosing artifact locations. Keep this index current.

## Verification

`sh scripts/check-records.sh`; inspect architecture/decision links and source claims.

## Child DOX Index

| Child | Owns | Read when editing |
| --- | --- | --- |
| `product/AGENTS.md` | CAPs and optional canonical visuals | Current behaviour/evidence |
| `changes/AGENTS.md` | CHGs and review packages | Plans, gates, status, reviews |

Parent: `../AGENTS.md`.
