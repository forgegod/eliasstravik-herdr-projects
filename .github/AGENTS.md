# Repository automation

## Purpose

Maintain repository automation for Herdr Projects.

## Ownership

`workflows/ci.yml` for read-only quality checks; existing `workflows/release.yml` for published-release artifacts.

## Local Contracts

- CI runs the same verification commands as local development, with Rust, Node, Git and rsync available.
- Keep workflow permissions minimal. Do not expose secrets to pull-request code or use pull_request_target for these tests.
- Preserve release tags, target matrix, checksum verification, and published-release triggers.

## Work Guidance

Follow root DOX; preserve the owning boundary and source-backed evidence.

## Verification

`sh scripts/check.sh` locally; remote Actions status is not established until the workflow actually runs.

## Child DOX Index

No child DOX documents. Parent: `../AGENTS.md`.
