# Shipped agent protocols

## Purpose

Maintain shipped agent protocols for Herdr Projects.

## Ownership

`COORDINATOR.md`, `THREAD.md`, and `autoproject/` are product assets, distinct from externally installed engineering workflow skills.

## Local Contracts

- Keep coordinator instructions about coordination, worker instructions about assigned work, and autoproject rules aligned with its protocol reference.
- Instruction text cannot override Rust safety gates or authorize unrelated work. Do not equate a model saying “done” with verified completion.
- The bundled autoproject skill is installed through the ownership-aware setup code; document only supported destinations.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`cargo test --locked` covers brief/priming generation; review changed instructions against `src/cli.rs`, `src/thread.rs`, and `src/progress.rs`.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
