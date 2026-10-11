# DOX framework

- DOX is the AGENTS.md hierarchy installed in this repository.
- Follow applicable DOX instructions for every edit.

## Core Contract

- AGENTS.md files are binding work contracts for their subtrees.
- Work products, source materials, instructions, records, assets, and durable docs must stay understandable from the nearest applicable AGENTS.md plus every parent AGENTS.md above it.

## Read Before Editing

1. Read the root AGENTS.md.
2. Identify every file or folder you expect to touch.
3. Walk from the repository root to each target path.
4. Read every AGENTS.md found along each route.
5. If a parent AGENTS.md lists a child AGENTS.md whose scope contains the path, read that child and continue from there.
6. Use the nearest AGENTS.md as the local contract and parent docs for repo-wide rules.
7. If docs conflict, the closer doc controls local work details, but no child doc may weaken DOX.

Do not rely on memory. Re-read the applicable DOX chain in the current session before editing.

## Update After Editing

Every meaningful change requires a DOX pass before the task is done.

Update the closest owning AGENTS.md when a change affects:

- purpose, scope, ownership, or responsibilities;
- durable structure, contracts, workflows, or operating rules;
- required inputs, outputs, permissions, constraints, side effects, or artifacts;
- user preferences about behavior, communication, process, organization, or quality; or
- AGENTS.md creation, deletion, move, rename, or index contents.

Update parent docs when parent-level structure, ownership, workflow, or child index changes. Update child docs when parent changes alter local rules. Remove stale or contradictory text immediately. Small edits that do not change behavior or contracts may leave docs unchanged, but the DOX pass still must happen.

## Hierarchy

- Root AGENTS.md is the DOX rail: project-wide instructions, global preferences, durable workflow rules, and the top-level Child DOX Index.
- Child AGENTS.md files own domain-specific instructions and their own Child DOX Index.
- Each parent explains what its direct children cover and what stays owned by the parent.
- The closer a doc is to the work, the more specific and practical it must be.

## Child Doc Shape

- Create a child AGENTS.md when a folder becomes a durable boundary with its own purpose, rules, responsibilities, workflow, materials, or quality standards.
- Work Guidance must reflect the current standards of the project or user instructions; if there are no specific standards or instructions yet, leave it empty.
- Verification must reflect an existing check; if no verification framework exists yet, leave it empty and update it when one exists.

Default section order:

- Purpose
- Ownership
- Local Contracts
- Work Guidance
- Verification
- Child DOX Index

## Style

- Keep docs concise, current, and operational.
- Document stable contracts, not diary entries.
- Put broad rules in parent docs and concrete details in child docs.
- Prefer direct bullets with explicit names.
- Do not duplicate rules across many files unless each scope needs a local version.
- Delete stale notes instead of explaining history.
- Trim obvious statements, repeated rules, misplaced detail, and warnings for risks that no longer exist.

## Closeout

1. Re-check changed paths against the DOX chain.
2. Update nearest owning docs and any affected parents or children.
3. Refresh every affected Child DOX Index.
4. Remove stale or contradictory text.
5. Run existing verification when relevant.
6. Report any docs intentionally left unchanged and why.

## Project guidance

Herdr Projects is a Rust CLI and Herdr plugin for coordinating projects through
agent conversations, parallel worker threads, worktrees, shared project files,
and a background ticker. Preserve the existing product and installation flow.

- Treat `Cargo.toml`, `Cargo.lock`, and `herdr-plugin.toml` as the runtime/build
  contracts. Rust 1.89 or newer is required. Node 22+ is development-only tooling
  for documentation records; do not add a Node runtime dependency to the plugin.
- Source and executable tests decide behaviour. Read definitions and callers
  before editing. Use strict RED–GREEN–REFACTOR for new or changed behaviour.
- Do not commit, push, publish, merge upstream patches, or change user agent
  configuration without an explicit operator request. Preview logical commit
  batches with the `git-commit-message` skill before any approved commit.
- Keep changes scoped. Preserve unrelated work, existing Rust formatting, and
  the release workflow. No repository-wide formatting or warning cleanup as a
  side effect of another task.
- Do not read or record credentials. Tests use synthetic data, temporary homes,
  injected runners, and disposable sessions. Never install test hooks into a
  person's active profile.
- Preserve safety allow-lists, trust/draft guards, bounded external commands,
  reversible configuration ownership, and identity checks before pane actions.
- A Herdr launch profile, a Hermes persistent profile, a Herdr project, and a
  Git worktree are different objects. Current behaviour is in the CAPs; planned
  Hermes integration belongs only in its active CHG.

## Record authority

- `docs/product/capabilities/CAP-*.md` describes current material behaviour.
- `docs/changes/active/CHG-*.md` is the single progress authority for each
  material change. A direct request or external ticket explains why it exists.
- Read `docs/product/README.md` and `docs/changes/README.md` before changing
  records. Use the installed `application-records` and `phased-plan-*` skills.
- Update CAPs alongside implemented behaviour and tests, not when a plan is
  approved. Archive a CHG only after its final integration gate passes.
- Cross-cutting accepted decisions belong in `docs/design-decisions.md`;
  `docs/architecture.md` distinguishes implemented design from approved work.
- Review packages live under `docs/changes/reviews/CHG-<number>/`. They are not
  product evidence. Primary human-facing surfaces require canonical generated
  CAP-linked HTML/PNG wireframes; headless contracts honestly declare `none`.
- Optional authoring skills remain externally installed. Do not copy the
  blueprint's demonstration CAPs/CHGs or a profile-local skill tree here.

## Verification

- `cargo test --locked` runs unit/scenario and CLI integration tests.
- `cargo build --release --locked` proves the release binary builds.
- `node --test tests/records/*.test.mjs` proves record-validator behaviour.
- `sh scripts/check-records.sh` runs validator tests and validates this tree.
- `sh scripts/check.sh` runs the complete local/CI gate.
- Tests require Git, a POSIX shell, and rsync in addition to Rust. See
  `docs/development.md` for prerequisites and the isolated-container alternative.
- `git diff --check` verifies patch whitespace. Do not claim rustfmt/Clippy are
  globally clean unless actually checked; the adoption gate does not impose
  new formatting on the existing source.

## Child DOX Index

| Child | Owns | Read when editing |
| --- | --- | --- |
| `src/AGENTS.md` | Rust implementation and colocated tests | Runtime behaviour, state, launch, hooks, panes |
| `skill/AGENTS.md` | Shipped coordinator/worker instructions and autoproject | Agent-facing protocol and bundled skill |
| `tests/AGENTS.md` | CLI tests, screen fixtures, record-validator fixtures | Cross-boundary evidence and fixtures |
| `scripts/AGENTS.md` | Installation, development helpers, verification | Scripts and validator |
| `docs/AGENTS.md` | Architecture, operations, CAP/CHG documentation | Durable documentation |
| `.github/AGENTS.md` | CI and release automation | GitHub workflows |

Root-owned files: `README.md`, `Cargo.toml`, `Cargo.lock`, `build.rs`,
`herdr-plugin.toml`, `.gitignore`, and `LICENSE`. Existing promotional assets
under `assets/` remain root-owned; they are not canonical CAP wireframes.
