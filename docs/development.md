# Development and verification

## Toolchain

- Rust/Cargo 1.89 or newer, matching `Cargo.toml`'s MSRV.
- Git, a C toolchain, a POSIX shell, and rsync. One local-copy test invokes real
  rsync, so a minimal Rust container without it is not a complete test environment.
- Node 22+ for the dependency-free record validator and `node:test` fixtures.
  Node is not required to run/install the Herdr Projects binary. No npm/pnpm
  dependencies, package manifest or lockfile are introduced by record adoption.
- Herdr 0.9.1+ and an agent CLI for live integration testing; normal Rust tests
  use injected/fake runners and temporary homes rather than a live agent.

## Commands

```sh
sh scripts/check-records.sh      # record fixtures, then current CAP/CHG tree
cargo test --locked             # Rust unit/scenario and built-CLI tests
cargo build --release --locked  # build without installing/linking the binary
sh scripts/check.sh             # complete CI/local gate
```

Use focused Cargo filters while implementing, but record the matched tests:
Cargo succeeds even when a filter matches nothing. `git diff --check` checks
patch whitespace. Preserve existing Rust style; this adoption does not declare
repository-wide rustfmt/strict Clippy clean or reformat unrelated modules.

The existing `scripts/dev-server`, `scripts/dev-hp`, and `scripts/dev-herdr`
helpers support disposable Herdr sessions; read them and
[manual checks](manual-test.md) before invoking them. Never run the installer or
`doctor --fix` against an active user installation merely to test a change.

## When Rust is not installed locally

Use a disposable `rust:1.89-bookworm` container with the repository mounted
read-only and writable Cargo home/target under a user-owned scratch directory.
Install rsync inside that container, run Cargo as the host UID/GID, and give it
a writable HOME (retaining `/root` after dropping privileges makes Git exclusion
checks fail). TMPDIR must also be writable. Mount a compatible Node installation
read-only if running the combined gate in that Rust container. This is a
verification environment, not a product runtime or a mandatory Docker install.

The adoption baseline `4e4548c` passed `cargo test --locked` (325 unit/scenario
and 4 CLI integration tests) and `cargo build --release --locked` in that
container. The first probe lacked rsync; the next inherited an inaccessible
root HOME. Both were environment failures; source was not changed to hide them.
Remote Actions execution is a separate fact from local command success.

After record adoption, the complete `sh scripts/check.sh` gate passed in a
disposable combined Rust 1.89.0 / Node 22.23.3 / rsync environment: 73 record
fixture tests, 325 Rust unit/scenario tests, 4 CLI integration tests, record
validation and the release build. The same record suite also passed on local
Node 26.7.0. New CI uses Node 22 and Rust 1.89; it has not run remotely yet.

## Record adoption provenance

The operator requested adopting the sibling `ai-software-blueprint` checkout at
`f6f5a558f48fe10daddb740f866d37b0ca06e098`. Adapted surfaces:

- Root template DOX core and project-owned child contracts.
- Product/change lifecycle contracts, current-behaviour index and phased CHGs.
- Dependency-free `scripts/check-product-records.mjs` and generic fixtures under
  `tests/records/`; Rust test references were added with RED–GREEN evidence.

Not imported: blueprint product CAPs, archived bootstrap receipts, its Node
package manager configuration, skill-hosting tests or skill sources. Existing
Cargo manifests, release workflow, product instructions, assets and license are
preserved. Engineering skills remain externally installed. Initial CAP coverage
is explicitly limited to source-backed contracts relevant to current requests;
more product domains are inventoried when materially changed.

The validator proves structure, local references, record lifecycle and visual
inventory. It does not prove a cited test ran, parse Rust, judge surface
classification, render screenshots or prove a third-party runtime works.
