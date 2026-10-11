# Design decisions

## D-001 — Preserve the Rust product; adopt repository-owned records

**Status:** adopted

The operator requested adoption of the sibling AI Software Blueprint. Keep Cargo
as the runtime/build authority and preserve existing product docs and release
packaging. Adopt the DOX core, CAP/CHG lifecycle, generic validator and fixture
suite. Use Node 22+ directly for development-only validation, without importing
pnpm, a package manifest, blueprint proof CAPs/CHGs, skill-hosting infrastructure,
or a duplicate engineering-skill tree. Extend evidence recognition for Rust
colocated tests through RED–GREEN tests. See [development](development.md).

## D-002 — Operator-owned Hermes identities, explicit launch bindings

**Status:** accepted direction; implementation pending in [CHG-001](changes/active/CHG-001-hermes-agent-integration.md)

The operator selected “Operator-provisioned profiles with explicit bindings and
reuse checks” and “Include profile integration, hooks/skills, and CLI input-box
support; keep TUI separate”.

A Herdr profile remains a launch preset (harness/model/arguments/allow-list), not
an alias for a Hermes profile. Bind Hermes launches to an explicit persistent
home, independently of the worktree. Do not rely on inherited HERMES_HOME or
Hermes' sticky default. Operator provisioning/credentials remain outside Herdr.
Prevent concurrent managed use of one canonical home across projects, roles and
Herdr sessions; specify the limitation for unrelated external processes rather
than claiming a machine-wide exclusivity guarantee.

Preserve the selected home with native session identity through restart/resume.
Distinguish account HOME/Herdr configuration, Hermes state home and tool cwd.
Shared memory providers remain an explicit operator choice, not accidental
cross-profile state. Profiles are not filesystem sandboxes and do not imply
separate provider logins. No profile cloning, deletion or automatic pool
allocation is part of this change.

Use a thin profile-scoped Hermes adapter for its actual hook semantics; keep
progress logic in Rust. Classic-CLI screen detection is independent from hooks.
Hook registration must not bypass user consent, and matching a profile alone
must never authorize writes to an unrelated pane.

The [operator guide](hermes-agent.md) explains how to provision these identities
and compares native Bot Mode/Kanban. The accepted local-CLI scope does not add a
second dispatcher or a bridge to either system. Their profiles may use the same
primitive, but external runs are not covered by a Herdr reservation. Preserve
task/claim lineage when diagnosing unsupported nested execution; do not strip
it to manufacture authority. A future bridge requires a separate approved design.

This decision constrains future implementation; it does not describe current
capabilities. Detailed tradeoffs and acceptance boundaries are in the
[review package](changes/reviews/CHG-001/README.md).
