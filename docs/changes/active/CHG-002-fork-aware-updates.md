# CHG-002 — Fork-aware updates without losing local provenance

**Status:** blocked
**External request:** Direct operator request: because we forked the original project, check if the concept of https://github.com/afloury/herdr-projects/commit/08da209462e736f29b6e9d2325e0cd2b79613f0e could also be applied to this fork and create an appropriate CHR for it -- if applicable
**Impacts:** CAP-004
**Baseline:** `4e4548c` — release `0.2.34`; origin `forgegod/eliasstravik-herdr-projects`

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Explicit fork source and branch-channel discovery | done (`cargo test --offline --locked update && sh scripts/check-records.sh`: 24 filter matches and 73 record tests passed in the isolated Rust 1.89/Node 22 container; `git diff --check` passed) | `cargo test --locked update && sh scripts/check-records.sh` exits 0; same-version branch/provenance cases pass |
| 2 | Stage and verify candidates without changing the live checkout | done (`cargo test --offline --locked update && cargo test --offline --locked --test cli && sh scripts/check-records.sh`: 41 filter matches, 4 CLI tests and 73 record tests passed; durable scratch smoke gate passed; `git diff --check` passed) | `cargo test --locked update && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; disposable-remotes staging/failure cases pass |
| 3 | Serialized promotion and explicit ticker recovery | pending | Operator selects the promotion/confirmation contract below before production edits; then `cargo test --locked update && cargo test --locked ticker && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0 and transactional failure matrix passes |
| 4 | Fork documentation, current CAP and release integration | pending | `sh scripts/check.sh && git diff --check` exits 0; disposable install/update smoke and provenance review pass |

## Applicability and decision boundary

The requested “CHR” is represented using this repository's canonical CHG record
type, not a second record namespace. The fork-update concept applies; the exact
script is not approved for import. [Review package](../reviews/CHG-002/README.md).

At the baseline, the updater discovers origin release tags and compares versions.
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

**Execution:** Phase 1 started from clean `main` at `d71286b` and is complete;
its acceptance evidence is below. Phase 2 is also complete. Phase 3 is blocked
before implementation on the contract decision below; phase 4 remains pending.

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

Implementation used focused RED→GREEN cycles for the selected contract.
Equal/ahead/diverged classification must not guess from remote SHAs alone when
ancestry objects are missing; object acquisition is isolated from the live
checkout. The implementation evidence below supersedes the initial incomplete
checkpoint. CAP-004 was updated with the verified check-only behaviour.

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
  was updated with the implemented check-only boundary at that gate. Phase 2's
  subsequent completion and the phase-3 blocker are recorded separately below.

## Phase 2 — Stage and verify candidates without changing the live checkout

**Goal:** Fetch/build/test failures leave the operator's branch, worktree and
installed binary untouched.

**Operator-selected boundary:** expose staging only as
`herdr-projects update --stage --channel branch --repository <owner>/<repo>[/<subdir>] --ref <ref>`.
It stages and verifies the selected immutable candidate in disposable scratch,
reports the verification result, then cleans it up. It must not promote, replace
the installed binary, change the live checkout, persist selection or act as an
automatic fallback from either release update or `--check`.

**Implementation evidence:** stage-only candidate verification rejects
incomplete/mixed options, dirty or unsafe live state, source mismatch, failed
build, candidate version/provenance mismatch, non-runnable candidates, timeouts
and post-verification artifact changes. It preserves the live checkout and
removes scratch before returning. The durable manual smoke gate below verifies a
real local candidate build/probe in the reviewed isolated container. No promotion
behaviour is implemented in this phase.

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
   the exact command in this phase. Keep it an explicit manual phase gate when
   inclusion in the complete gate would require Docker.

**Verification gate:** `cargo test --locked update && cargo test --locked --test cli && sh scripts/check-records.sh` exits 0; disposable-remotes staging/failure cases above pass.

### Durable scratch smoke gate

From the repository root, run exactly:

```sh
sh scripts/test-candidate-smoke.sh
sh scripts/check-candidate-smoke.sh
```

Prerequisites: Linux, non-root host UID/GID, `/usr/bin/docker` with access to
`/var/run/docker.sock`, and `TMPDIR` naming an existing user-owned scratch
directory outside (and not containing) the repository. The already-reviewed
local image must exist as
`sha256:32083d2ba467a8af4f60e97857f7e50f7dc74c7633a670b4963bb425f8d18237`
(Rust/Cargo 1.89.0, Node 22.23.3, rsync 3.2.7). The harness never pulls/builds
an image or installs tooling. It rejects arguments and inherited Docker
host/context/config/TLS overrides; it never reads the host Docker configuration.

The only host mounts are the read-only repository and a fresh private scratch
directory. Both container stages run as the host UID/GID with a read-only root
filesystem and clean environment; HOME, CARGO_HOME, target and TMPDIR are under
scratch. No host home, credentials or active installation is mounted. A separate
network-enabled `cargo fetch --locked` populates the fresh dependency cache;
compilation, fixtures, both exact-name native candidate smoke tests and record
checks run with `--network none` and Cargo offline. Missing smoke tests fail
rather than silently accepting zero matches. Exit/signal traps remove the
container and scratch (uncatchable SIGKILL/host failure is outside that guarantee).
No install, link, doctor, update or promotion command is invoked.

This is an **explicit manual phase gate**: `scripts/check.sh` is intentionally
unchanged so Docker and network-dependent cold-cache acquisition do not become
mandatory normal-gate requirements. This harness is not the full phase-2 gate
and does not change any phase status or CAP.

TDD evidence for the harness boundary:

- RED: `sh scripts/test-candidate-smoke.sh` exited 1 before implementation:
  `not ok - unsafe mount override (expected safety refusal, exit 2)`;
  the following shell diagnostic identified the missing harness file.
- GREEN: the same fixtures exited 0, reporting `ok` for unsafe mount override,
  unsafe runner override, remote Docker runner, inherited Docker context, host
  Docker credentials and repository scratch mount. They also passed inside the
  approved isolated runner.
- `sh scripts/check-candidate-smoke.sh` exited 0. Existing tests
  `update::tests::candidate_native_smoke_builds_and_cleans_without_live_mutation`
  and `update::tests::candidate_native_smoke_invalid_executable_cleans_without_success`
  each reported `1 passed; 0 failed; 0 ignored; 0 measured; 357 filtered out`.
  Record checks reported 73 passed, 0 failed and
  `Product record check passed: 4 capabilities, 3 change records.` The harness
  ended with `candidate-smoke: PASS`.

## Phase 3 — Serialized promotion and explicit ticker recovery

**Goal:** Promotion has an observable transaction boundary with bounded recovery,
correct project/config identity and no false success after failed repair/restart.

### Contract decision — blocked before production edits

**Execution:** Blocked. The phase table uses `pending` for phase 3 because no
implementation is authorized; the CHG status is `blocked`, consistent with
`docs/changes/README.md`'s phase-status vocabulary. Phases 1–2 retain their
completed evidence, phase 4 remains pending, and no phase-3 gate has passed.

**Specification scope:** Read the existing update, ticker, repair, installation,
binary/link and review surfaces before production edits. This is a read-only
decision pass: it cannot mutate the active installation, ticker, configuration,
remotes, binaries, network or credentials, and it does not authorize source,
tests, CAPs, staging or commits. If no current contract defines an explicit
promotion confirmation, record the resulting decision frontier rather than
inventing one.

**Finding:** No explicit user-facing branch-promotion command or transaction
confirmation/commit interface is defined by the current repository contracts.
The phase's recovery requirements and review recommendations do not select one.
Inspection baseline: `4f0465cf9ff41f2a991a6d7d3c283109fca825c7` (clean `main`
before these record edits). Evidence is repository source, not a live-install
experiment; no updater, ticker, doctor, installer or candidate binary was run.

Source-backed boundaries:

- **CLI and authorization:** `src/cli.rs:286–304,913–928` defines only release
  update, branch `--check` and branch `--stage`; branch dispatch requires exactly
  one of the latter two. There is no `--promote`, candidate commit argument,
  confirmation prompt or transaction receipt interface. CAP-004
  (`docs/product/capabilities/CAP-004-installation-and-update.md:11–15`) promises
  no promotion from staging and explicitly disclaims transactional branch updates.
  `docs/getting-started.md:104–124` and `docs/operations.md:73` describe release
  updates, not authorization to repurpose bare `update` for branch promotion.
- **Candidate lifetime:** `src/update.rs:362–380` verifies then deletes scratch
  before returning; `527–547` makes the candidate private and calls its final
  guard read-only, not a lock or authorization. A later promotion cannot consume
  an artifact from today's `--stage`. `383–460,550–597,616–691` reconcile source,
  ref and managed provenance, verify the immutable candidate and compare live
  state, but do not persist a promotion plan, acquire installation ownership or
  define a commit step. The installed commit remains explicitly unverified
  (`333–338`); checkout HEAD must not become a binary receipt by assumption.
- **Existing release path:** `src/update.rs:218–258,463–524` stops the ticker,
  reinstalls through Herdr with a release tag or pulls linked `origin main`,
  runs the installed binary's `doctor --fix`, then attempts ticker start.
  It has no branch transaction lock, recoverable receipt or binary rollback
  protocol. `run_binary` supplies `--root` but does not explicitly bind config
  identity. Existing internal Herdr `--yes` is not a branch-promotion consent
  contract. The failure message at `523` can say "ticker was restarted" while
  appending a start-failed note; phase 3 must not inherit that as success evidence.
- **Ticker lifecycle:** `src/ticker.rs:32–75,100–169,229–268` owns a lock per
  projects root, not per installation. Stop requests use a stop file and wait
  up to 60 seconds for lock release, never signal the displayed PID. Start can
  return without a ticker when there are no projects, or return after spawning
  without waiting for readiness. Spawn forwards `--root`, removes pane/session
  variables and otherwise inherits environment; ticker info has no config
  identity (`48–56`). A successful start call alone does not prove recovery.
- **Root/config and repair:** `src/paths.rs:48–50,67–109` distinguishes the
  resolved executable, project root and HOME-derived project config directory;
  `src/setup.rs:144–150` separately resolves Herdr config via
  `HERDR_CONFIG_PATH`/`XDG_CONFIG_HOME`/HOME. There is no global `--config` flag
  (`src/cli.rs:14–23,639–650`). `src/doctor.rs:49–110,212–231,350–439` can repair
  priming files, links, configured hooks/skills and sidebar configuration;
  its healthy result excludes warnings, and CLI failure follows required-check
  failure (`src/cli.rs:861–865`). Its quiet release probe can contact origin
  (`src/doctor.rs:88–90`, `src/update.rs:155–171`). Repair is neither read-only
  nor a reversible binary transaction; restoring a binary cannot undo its writes.
- **Installation unit and paths:** `src/update.rs:145–175` assumes
  `<plugin-root>/target/release/herdr-projects`; `src/setup.rs:262–265` locates
  bundled skills relative to that root. `herdr-plugin.toml:8–13` invokes the
  installer and starts the binary there. `scripts/install.sh:20–30,53–79,97–122`
  can build/download, rename the executable and call the link script; its
  unrecognized-origin release fallback is not approved for branch promotion.
  `scripts/link-command.sh:18–45` targets Herdr's final managed path rather than
  its temporary checkout. `src/command_link.rs:15–85` uses XDG_BIN_HOME or
  HOME/.local/bin and preserves foreign files/live foreign links; startup also
  calls link repair (`src/cli.rs:888–896`). A binary-only swap does not establish
  matching source/skills or new managed provenance. Installation relocation,
  linked versus managed ownership and the promoted artifact set need an
  explicit contract, not an inferred call to the existing installer.
- **Review versus approval:** the [source assessment](../reviews/CHG-002/fork-update-assessment.md)
  pins `08da209462e736f29b6e9d2325e0cd2b79613f0e`, recommends explicit selection,
  staging, atomic replacement and bounded recovery (`24–66`), and rejects the
  detached-checkout/ignored-stop/doctor-warning approach. It supplies no approved
  CLI spelling or confirmation protocol. Its [README](../reviews/CHG-002/README.md)
  is `review-only`; `docs/design-decisions.md` contains no accepted updater
  transaction decision. This phase's steps below are safety requirements, not
  approval of an unspecified interface.

### Alternatives requiring operator selection

**A — Design an explicit promotion confirmation/commit interface.** Add a new,
opt-in `herdr-projects update --promote --channel branch --repository <owner>/<repo>[/<subdir>] --ref <ref>`
mode only after approval. This is a proposed command family, not a runnable or
selected contract. Preserve bare release update and the existing check/stage
contracts. Explicitly choose how a person authorizes the transaction against
one immutable candidate and one installation:

- A one-invocation transaction that restages/revalidates, displays its exact
  source/ref/full commit, installation/binary, root/config identity and repair
  effects, then requires an affirmative confirmation bound to that plan; or
- A preview/commit interface with a durable plan/receipt and a separate explicit
  commit confirmation, with defined expiry, integrity, cleanup and refusal on
  changed candidate/installation state. This cannot silently turn existing
  `--stage` into persistent staging.

The operator must select the exact confirmation/commit spelling and whether
noninteractive confirmation is supported (and how it binds the full commit and
target). Neither `--promote` alone, an old check/stage success, an inherited
`--yes`, nor a moving branch name supplies that missing consent. These alternatives
follow the review's print-before-mutation requirement and the current disposable
candidate boundary; neither is authorized by it.

**B — Retain stage-only branch behavior pending a future promotion design.**
Keep the phase-1/2 CLI and cleanup guarantees unchanged, add no promotion command,
and defer this phase and dependent phase-4 promotion/integration acceptance.
This is already supported by CAP-004 and `stage_branch`; it permits verification,
not installation. Do not label stage success as update success, close CHG-002 as
fully implemented, or use ordinary release update as a branch-promotion fallback.

**Precise operator decision required:** Select A or B. If A, select the
one-invocation or preview/commit model, approve its exact user-facing confirmation
syntax and noninteractive policy, and define the transaction's installation
scope (linked, managed or both; binary-only versus matching plugin assets and
managed provenance). Approve how project/Herdr config identity is bound across
old/new executables, whether an initially absent ticker stays absent, what
constitutes successful restart, and the rollback/degraded-state boundary after
repair writes. Until these choices are explicit in this CHG, implementation and
behavior-test design remain blocked; no CLI flags or receipt format are assumed.
Selecting A authorizes contract specification, not an update of the active install.

### Implementation requirements after the decision

The following requirements remain pending. Refine their RED tests and exact
transaction states from the selected contract before making production edits.

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

### Specification-only verification and DOX boundary

Only `sh scripts/check-records.sh` is authorized for this specification pass;
the implementation gate above is not run or claimed. No source, tests, CAPs,
installation/configuration, remote, credential or binary changes are authorized.
`sh scripts/check-records.sh` exited 0: 73 record-validator tests passed, none
failed, and the tree validated as 4 capabilities / 3 change records. This is
documentation evidence only, not phase-3 implementation acceptance. No skill
installation or network retrieval was attempted.

The DOX pass leaves root, `docs/AGENTS.md`, `docs/changes/AGENTS.md` and
`docs/product/AGENTS.md` unchanged: ownership, workflow and child indexes are
unchanged. Only this CHG and its status in `docs/changes/README.md` change;
CAP-004, architecture, accepted decisions and the review package remain unchanged
because no new behavior or design selection is approved.

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
