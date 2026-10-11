# Operator and verification scripts

## Purpose

Maintain operator and verification scripts for Herdr Projects.

## Ownership

Existing installer/link/development scripts; record/check gates; `check-candidate-smoke.sh` and its negative fixtures in `test-candidate-smoke.sh`.

## Local Contracts

- Keep the record validator dependency-free and importable as `validateRecords(root)`. Its optional root argument enables fixture tests.
- Retain CAP/CHG lifecycle, local-link, review-ownership and visual-inventory checks. Rust evidence recognition is structural, not a Rust parser or proof of execution.
- Use failing fixtures before changing validator semantics; retain the inherited generic negative tests.
- Do not run installation/link scripts as a verification shortcut: they can change the user’s active binary.
- Candidate smoke is an explicit manual gate, not a Docker dependency of `check.sh`. Keep its reviewed image and local runner fixed, repository read-only, host UID/GID, fresh user-owned scratch and clean environment. Only dependency fetch may use networking; tests run offline with no home/credential/installation mounts. Clean containers and scratch on exit; reject mount/runner overrides before launch.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`sh scripts/check-records.sh`; `sh -n scripts/check.sh scripts/check-records.sh`; complete gate `sh scripts/check.sh`.

Candidate harness: `sh -n scripts/check-candidate-smoke.sh scripts/test-candidate-smoke.sh` and `sh scripts/test-candidate-smoke.sh`. Explicit isolated gate: `sh scripts/check-candidate-smoke.sh`; prerequisites and evidence live in CHG-002 phase 2.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
