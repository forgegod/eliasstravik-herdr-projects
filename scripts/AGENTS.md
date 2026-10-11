# Operator and verification scripts

## Purpose

Maintain operator and verification scripts for Herdr Projects.

## Ownership

Existing installer/link/development scripts; `check-product-records.mjs`, `check-records.sh`, and `check.sh`.

## Local Contracts

- Keep the record validator dependency-free and importable as `validateRecords(root)`. Its optional root argument enables fixture tests.
- Retain CAP/CHG lifecycle, local-link, review-ownership and visual-inventory checks. Rust evidence recognition is structural, not a Rust parser or proof of execution.
- Use failing fixtures before changing validator semantics; retain the inherited generic negative tests.
- Do not run installation/link scripts as a verification shortcut: they can change the user’s active binary.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`sh scripts/check-records.sh`; `sh -n scripts/check.sh scripts/check-records.sh`; complete gate `sh scripts/check.sh`.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
