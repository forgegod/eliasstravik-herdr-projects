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
- Branch-history checks acquire local Git objects only in a disposable bare repository. Never fetch into or change the live installation; report incomplete ancestry as unavailable. Checkout history is not installed-binary provenance: an unverified installed commit cannot support an installed-ahead/equal/diverged claim.
- Candidate staging in `update.rs` is exposed only through `update --stage --channel branch --repository … --ref …`; it is not a replacement for release updates or phase-1 checks. Fetch only the selected full commit into disposable scratch; native builds use isolated Cargo home/target directories, `--offline --locked` dependencies and bounded Runner commands. Never invoke an installer or automatic container fallback.
- The candidate live-state guard is read-only and must be repeated under phase-3 installation ownership before any future promotion. It rejects tracked dirt, ignored candidate-path collisions, unsafe index flags and observed HEAD/branch/remote/binary changes. Candidate staging rechecks the verified artifact digest after that guard; this detects tampering but is not a filesystem lock. A passing guard is not a promotion transaction or a sandbox guarantee.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`cargo test --locked` and `cargo build --release --locked`; focused module tests during TDD.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
