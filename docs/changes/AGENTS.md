# Change records

## Purpose

Own restart-safe implementation progress for material requests.

## Ownership

`README.md` defines lifecycle; `active/CHG-*.md` holds plans/progress; `archive/CHG-*.md` holds receipts; optional `reviews/CHG-<number>/` holds review-only proposals and external-patch assessments.

## Local Contracts

- A CHG is the only progress authority for its request. Keep active/archive flat and record-only.
- Planned records have all phases pending. Execute one phase at a time and attach real evidence before marking done.
- Record direct requests verbatim and pin external patches to reviewed SHAs. A clean application check is not a behavioural review or a passing test suite.
- Proposal assets stay review-only and retain stable paths when the owner is archived. Never link them from CAPs.
- Do not mark accepted design choices as implemented. Only the final integration phase may archive a completed CHG.

## Work Guidance

Read `README.md`, the product contract, and affected CAPs. Use the installed phase skills; do not mirror status in private plans. Review-package recommendations do not authorize merging third-party code.

## Verification

`sh scripts/check-records.sh` plus each CHG's phase gate and the final `sh scripts/check.sh` gate.

## Child DOX Index

No children. Parent: `../AGENTS.md`.
