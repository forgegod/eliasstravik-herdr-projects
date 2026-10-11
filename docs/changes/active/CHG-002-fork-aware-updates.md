# CHG-002 — Fork-aware updates without losing local provenance

**Status:** planned
**External request:** Direct operator request: because we forked the original project, check if the concept of https://github.com/afloury/herdr-projects/commit/08da209462e736f29b6e9d2325e0cd2b79613f0e could also be applied to this fork and create an appropriate CHR for it -- if applicable
**Impacts:** CAP-004
**Baseline:** `4e4548c` — release `0.2.34`; origin `forgegod/eliasstravik-herdr-projects`

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Explicit fork source and branch-channel discovery | pending | `cargo test --locked update && sh scripts/check-records.sh` exits 0; same-version branch/provenance cases pass |
| 2 | Stage and verify candidates without changing the live checkout | pending | `cargo test --locked update && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; disposable-remotes staging/failure cases pass |
| 3 | Serialized promotion and explicit ticker recovery | pending | `cargo test --locked update && cargo test --locked ticker && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; transactional failure matrix passes |
| 4 | Fork documentation, current CAP and release integration | pending | `sh scripts/check.sh && git diff --check` exits 0; disposable install/update smoke and provenance review pass |

## Applicability and decision boundary

The requested “CHR” is represented using this repository's canonical CHG record
type, not a second record namespace. The fork-update concept applies; the exact
script is not approved for import. [Review package](../reviews/CHG-002/README.md).

The updater currently discovers origin release tags and compares versions.
New commits on the fork's main can remain invisible when the version is
unchanged. Managed installation source and Git origin can also disagree.
The reference patch's statement that update always follows upstream is not
accurate for this baseline. Preserve this distinction in documentation/tests.

Recommended design: an explicit, operator-selected branch channel in the existing
Rust updater, not a second shell updater. Keep release updates compatible.
CLI spelling, opt-in persistence and managed-install support must be settled
with the operator in phase 1 before implementation; do not silently change the
meaning of ordinary `update` for existing users.

This CHG is independent of CHG-001. Neither authorizes applying third-party
patches, changing remotes, detaching a checkout, publishing releases or updating
the user's active installation during planning.

Documentation coordination: the operator's comprehensive Hermes setup request
is tracked in [CHG-001](CHG-001-hermes-agent-integration.md). Its
[operator guide](../../hermes-agent.md) must remain accurate when this updater
lands. This adds an upgrade-documentation handoff, not Hermes profile management
or a dependency on implementing the entire Hermes feature first.

## Phase 1 — Explicit fork source and branch-channel discovery

**Goal:** Read-only update checks identify the intended source/ref and distinguish
same-version fork commits from release-version updates.

1. Read `src/update.rs`, installer/build metadata, Herdr installation-source
   behaviour and the pinned reference assessment. Confirm the chosen CLI/API
   for explicit branch-channel opt-in with the operator.
2. Add RED tests for origin fork/upstream URLs, managed-source mismatches,
   no release tags, same-version new commit, equal commit, ahead/diverged local
   history, offline errors, invalid refs, renamed remotes and detached worktrees.
3. Implement bounded source/ref resolution and a check-only report naming
   repository, ref, installed commit and candidate commit. Never infer fork
   ownership from a convenient remote name or silently fall back to upstream.
4. Preserve current release-channel semantics. Require explicit selection where
   origin, linked checkout and Herdr source metadata disagree. Record only
   implemented behaviour in CAP-004; design approval alone is not a CAP change.

**Verification gate:** `cargo test --locked update && sh scripts/check-records.sh` exits 0; same-version branch/provenance cases above pass with named tests.

## Phase 2 — Stage and verify candidates without changing the live checkout

**Goal:** Fetch/build/test failures leave the operator's branch, worktree and
installed binary untouched.

1. Add RED tests using temporary local Git repositories/remotes and synthetic
   installer processes; no network credentials or active installation.
2. Stage the selected immutable commit in a separate directory/worktree. Keep
   user branches/HEAD unchanged; check tracked/untracked collisions and
   concurrent edits before promotion, without stashing/resetting user work.
3. Build with a separate target directory, locked dependencies and bounded
   processes. Validate candidate source/commit/version and executable compatibility;
   test a checksum/provenance mismatch and a candidate that cannot run.
4. Native Cargo is the first supported build path. If an optional container
   backend is added, require explicit backend/host-target compatibility,
   reviewed image version/digest, least necessary mounts and UID handling.
   Do not ship an automatic Alpine/Linux binary fallback on macOS.
5. Add a scratch integration harness for candidate staging and cleanup. Document
   the exact command in this phase and include it in the complete gate once added.

**Verification gate:** `cargo test --locked update && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; disposable-remotes staging/failure cases above pass.

## Phase 3 — Serialized promotion and explicit ticker recovery

**Goal:** Promotion has an observable transaction boundary with bounded recovery,
correct project/config identity and no false success after failed repair/restart.

1. Write RED tests for concurrent updates, stale locks, ticker-stop refusal,
   promotion failure, interrupted promotion, doctor failure, restart failure,
   custom project root/config and moved installation paths.
2. Serialize the whole fetch/stage/promote operation for the installation,
   not only the build. Acquire/release ownership safely and revalidate the
   candidate/source before swapping.
3. Stop the correct ticker with explicit root/config using the old executable.
   Do not ignore stop failure. Atomically replace the binary only after candidate
   validation, preserving a recoverable old executable and installation receipt.
4. Run repair and ticker restart with explicit context. Define which failures
   permit binary rollback and which require a degraded-state diagnostic after
   data changes. Never claim doctor/data migrations were undone by a binary swap.
5. Keep exit codes and diagnostics truthful. Test that every failure retains
   or restores a usable binary where safe and reports any ticker needing action.
6. Synchronize CAP-004, architecture and an accepted design-decision record for
   update ownership/recovery only as the behaviour becomes implemented.

**Verification gate:** `cargo test --locked update && cargo test --locked ticker && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; transactional failure matrix above passes.

## Phase 4 — Fork documentation, current CAP and release integration

**Goal:** Users can install/update this fork without accidentally following an
upstream-only recipe, and the current record has behavioural evidence.

1. Update README/getting-started/operations and CLI help with fork attribution,
   source/channel inspection, explicit installation and safe update/recovery.
   Retain original MIT/license attribution; do not assert upstream is paused
   without evidence. Replace or label upstream-specific links/commands coherently.
   Update the Hermes operator guide's supported-version and upgrade/repair
   instructions too. Distinguish updating this fork's binary/adapter from
   `hermes update` and Hermes profile provisioning; neither authorizes moving
   HERMES_HOME, resetting profiles, copying credentials or starting a second
   Bot/Kanban/Herdr writer. If CHG-001 is still pending, retain that warning
   instead of inventing installer commands for it.
2. Keep release packaging and version/checksum expectations aligned. A development
   branch channel does not silently replace the published-release policy.
3. Run a disposable install/update smoke including a same-version new commit,
   custom root, failed build, failed repair and restart. Do not mutate the active
   plugin or publish a release without separate authorization.
   When the Hermes adapter exists, include its documented selected-profile
   repair/read-back flow and assert unrelated profiles and native-session
   bindings remain unchanged. Otherwise explicitly record the unavailable
   integration check without claiming Hermes upgrade compatibility.
4. Run the full gate and update CAP-004 to the actual evidence boundary. This is
   a headless updater/CLI contract with `Primary surface: none`; if a primary
   interactive updater dialog is added, add its canonical generator/rendering
   and CAP-linked HTML/PNG gate before closure.
5. Freeze the review package at its stable path. Only this final phase may mark
   done/archive after all gates pass; fix links on archive.

**Verification gate:** `sh scripts/check.sh && git diff --check` exits 0; disposable install/update smoke and provenance review above pass.

## Out of scope

- Copying the reference updater unchanged, automatic detached checkout, or
  silently merging upstream into this fork.
- Automatic upstream PR imports, version bumps, pushes, releases or a live update.
- Mandatory Docker/runtime dependencies or unverified cross-platform binaries.
- Hermes profile management; that belongs to CHG-001.
