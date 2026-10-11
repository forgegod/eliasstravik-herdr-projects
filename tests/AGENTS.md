# Test evidence

## Purpose

Maintain test evidence for Herdr Projects.

## Ownership

`cli.rs` owns built-binary integration tests; `fixtures/prompt_box/` owns terminal captures; `records/` owns dependency-free record-validator tests.

## Local Contracts

- Scrub process environments and use temporary homes for tests. Never use real credentials or active agent profiles.
- Label synthetic fixtures as synthetic; record agent version and capture method for real screen fixtures. Test empty, drafted, menu, busy, and unknown states.
- Record-validator fixture trees are synthetic, not product CAP/CHG records. Their mock image files test inventory only, never rendering.
- A Rust evidence reference names an existing source/test file containing a test attribute; execution, not structural detection, establishes proof.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`cargo test --locked` and `node --test tests/records/*.test.mjs`.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
