# Rust implementation

## Purpose

Maintain rust implementation for Herdr Projects.

## Ownership

All Rust modules and their colocated unit/scenario tests. `main.rs` enters `cli.rs`; external commands run through `runner.rs`.

## Local Contracts

- Pass environment through `paths::Env` and external calls through `Runner` so tests do not touch live user state.
- Keep project/runtime data separate from user-owned safety/configuration. Preserve atomic writes and pane/socket/terminal identity checks.
- Treat an unknown input box or trust screen conservatively. Do not broaden a parser without negative fixtures.
- Keep hook errors fail-open and side effects scoped; launch/authorization failures must not silently select a different agent identity.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`cargo test --locked` and `cargo build --release --locked`; focused module tests during TDD.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
