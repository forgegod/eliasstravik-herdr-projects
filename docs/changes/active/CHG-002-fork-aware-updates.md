# CHG-002 — Fork-aware updates without losing local provenance

**Status:** in-progress
**External request:** Direct operator request: because we forked the original project, check if the concept of https://github.com/afloury/herdr-projects/commit/08da209462e736f29b6e9d2325e0cd2b79613f0e could also be applied to this fork and create an appropriate CHR for it -- if applicable
**Impacts:** CAP-004
**Baseline:** `4e4548c` — release `0.2.34`; origin `forgegod/eliasstravik-herdr-projects`

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Explicit fork source and branch-channel discovery | done (`cargo test --offline --locked update && sh scripts/check-records.sh`: 24 filter matches and 73 record tests passed in the isolated Rust 1.89/Node 22 container; `git diff --check` passed) | `cargo test --locked update && sh scripts/check-records.sh` exits 0; same-version branch/provenance cases pass |
| 2 | Stage and verify candidates without changing the live checkout | in-progress | `cargo test --locked update && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; disposable-remotes staging/failure cases pass |
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
4. Preserve current release-channel semantics. Branch checks require the selected
   canonical repository/ref to agree with installation provenance; refuse missing,
   ambiguous or conflicting provenance rather than using an explicit flag to
   override it. Record only implemented behaviour in CAP-004; design approval
   alone is not a CAP change.

**Verification gate:** `cargo test --locked update && sh scripts/check-records.sh` exits 0; same-version branch/provenance cases above pass with named tests.

### Phase 1 contract selection and implementation status

**Execution:** Phase 1 started from clean `main` at `d71286b`. It remains
in-progress; phases 2–4 remain pending. No phase-completion claim is made.

Additional direct operator instruction: “If the decision cannot be derived from
an explicit current repository contract, mark CHG-002 phase 1 blocked with
source-backed alternatives/evidence before production changes; do not claim
completion. Report the precise decision required.”

The current contracts did not select a CLI/API spelling, persistence policy or
managed-install boundary. The review package recommended a Rust branch channel,
not a particular approved interface. Source investigation found:

- `src/cli.rs:286–291,896` defines only `update [--check]` and dispatches to
  `update::run(ctx, check_only)`. `docs/getting-started.md:118–124` and
  `docs/operations.md:73` promise release behaviour. CLI long-option conventions
  support adding explicit flags; nested commands such as `Ticker` also exist,
  but do not authorize converting bare `update` into a required subcommand.
- `src/update.rs:50–91,127–145,234–265` parses local/GitHub installation kinds,
  lists release tags on `origin`, and compares versions. Managed source and
  origin are not reconciled. `newer_release` is the separate quiet doctor probe
  (`src/doctor.rs:88–90`); `own_root` also locates setup's bundled skill
  (`src/setup.rs:264–265`). Preserve these callers' existing contracts.
- `src/update.rs:193–208` reinstalls managed sources with Herdr's `--ref`, but
  pulls linked `origin main`. `docs/herdr-notes.md:156–166` records historical
  disposable-server evidence that managed clones are detached and carry
  `requested_ref`, `resolved_commit` and `managed_path`. The updater's parser
  currently discards those fields, including in its synthetic managed fixture
  (`src/update.rs:315–318`). Historical notes are not a new live Herdr check.
- `build.rs:7–20` embeds a short SHA/timestamp or `nogit`; `src/main.rs:46`
  exposes it in the version string. `src/update.rs:151–155` reads only the
  version. Neither checkout HEAD nor managed metadata alone proves which
  commit built an installed binary. Missing/conflicting provenance must not be
  reported as a verified installed commit.
- `scripts/install.sh:53–79,112–120` checks the release commit before downloading,
  validates the executable version, and retains an upstream URL fallback for
  an unrecognized origin. `herdr-plugin.toml:8–10` invokes that installer. These
  are existing release contracts, not authorization to reuse that fallback for
  branch checks or invoke installation in phase 1.
- `src/paths.rs:48–50,85–109` owns user config/root resolution;
  `src/profiles.rs:118–145` owns launch defaults, not update-source settings.
  No update-channel persistence contract exists. `tests/cli.rs` has no updater
  integration coverage; `src/update.rs:308–364` covers installation parsing,
  version/tag comparison and the quiet offline doctor probe, not the phase matrix.

**Operator-selected contract:**

1. **CLI/API:** add branch checking only as
   `herdr-projects update --check --channel branch --repository <owner>/<repo>[/<subdir>] --ref <ref>`.
   `--repository` is a canonical, credential-free GitHub identity—not a URL—and
   accepts neither userinfo nor arbitrary transports/paths. Branch-mode flags
   require `--check`; a partial branch invocation is refused. Bare `update` and
   `update --check` retain their release-channel behaviour.
2. **Persistence:** selections are per invocation only in phase 1. Do not write
   configuration, remember a source/ref, or implicitly reuse prior selection.
3. **Managed support:** support linked and Herdr-managed GitHub installations
   read-only. Canonical source identity (including subdirectory) and requested
   ref must agree with available provenance; refuse missing, ambiguous or
   conflicting evidence. Where the installed binary's commit cannot be proved,
   report it as `unverified`, never as a verified commit. Neither choice
   authorizes promotion, source switching or installation.

Implementation has started with focused RED→GREEN cycles for the selected
contract. The phase remains incomplete: equal/ahead/diverged classification must
not guess from remote SHAs alone when ancestry objects are missing, and any
necessary object acquisition must be isolated from the live checkout. CAP-004,
architecture and accepted design decisions remain unchanged until the complete
behaviour and phase gate are verified.

### Phase 1 baseline verification before implementation

- Before the contract selection, no RED/GREEN cycle was started because choosing
  behaviour tests for an unapproved CLI/persistence/managed-support contract
  would have pre-empted the required decision.
- `cargo test --locked update && sh scripts/check-records.sh` exited 0 in a
  disposable container using the existing `herdr-projects-blueprint-verify`
  image `sha256:32083d2ba467a8af4f60e97857f7e50f7dc74c7633a670b4963bb425f8d18237`.
  Its filesystem layers were verified to extend `rust:1.89-bookworm`; runtime
  checks reported Rust/Cargo 1.89.0, Node 22.23.3 and installed rsync. The repo
  was read-only at `/repo`; Cargo ran as UID/GID 1000 with a clean environment,
  writable scratch HOME/CARGO_HOME/target/TMPDIR and only `/repo` trusted by Git.
  No host home, credential directory or active installation was mounted.
- Cargo ran 8 passing unit tests (317 filtered out): the six updater tests
  `update::tests::a_linked_checkout_is_detected`,
  `update::tests::a_github_install_is_detected_with_its_repository`,
  `update::tests::a_missing_plugin_or_unknown_source_is_an_error`,
  `update::tests::release_tags_compare_as_numbers`,
  `update::tests::the_newest_release_is_picked_from_ls_remote`,
  `update::tests::doctor_check_is_silent_outside_a_checkout_and_offline`, plus
  filter matches `project::tests::coordinator_updates_keep_other_fields` and
  `thread::tests::updates_are_atomic_and_keep_other_fields`. The CLI target ran
  zero tests (4 filtered out); it is not updater integration evidence.
- The record suite passed all 73 tests and validated 4 CAPs / 3 CHGs.
  `git diff --check` exited 0. These passes establish baseline/documentation
  health, not phase-1 acceptance or a new branch-channel implementation.
- The pre-implementation DOX pass changed only this CHG and
  `docs/changes/README.md`. Root, `docs/AGENTS.md` and
  `docs/changes/AGENTS.md` remain applicable and unchanged: no ownership,
  durable workflow or child index changed. The final phase DOX pass and evidence
  update remain required before completion.

### Phase 1 implementation evidence

- The selected check-only CLI, source/ref/provenance refusal, candidate validation,
  truthful unverified installed-commit report and checkout-history classification
  are implemented in `src/cli.rs` and `src/update.rs`.
- Tests use injected runners plus disposable local Git repositories. The history
  safety regressions have mutation-backed RED→GREEN evidence; their temporary
  wrong classifications failed assertions, the exact implementation was restored
  byte-for-byte, and each test passed green.
- The phase gate passed in the isolated Rust 1.89/Node 22 container: 24 `update`
  filter matches and 73 record tests passed. `git diff --check` passed. CAP-004
  now describes the implemented check-only boundary; later phases remain pending.

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
