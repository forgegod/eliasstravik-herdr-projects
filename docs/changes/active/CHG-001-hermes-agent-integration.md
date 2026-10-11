# CHG-001 — Profile-safe Hermes Agent integration

**Status:** blocked
**External request:** Direct operator request: Adopt the ../ai-software-blueprint/ to this project. Convert the findings and recommendation into CHG according to the phased-plan-design skill. Discuss with the operator how the profile concept of Hermes Agent comply with your findings regarding the architecture
**Impacts:** CAP-001, CAP-002, CAP-003
**Baseline:** `4e4548c` — release `0.2.34`; blueprint adoption adds records/tooling only, not this feature

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Explicit Hermes profile binding and safe launch/resume | in-progress | `cargo test --locked profiles && cargo test --locked paths && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; profile race/resume acceptance cases below pass |
| 2 | Reversible profile-scoped skill and startup adapter installation | pending | `cargo test --locked && python3 -m unittest discover -s tests/hermes -v && sh scripts/check-records.sh` exits 0; isolated install round-trip passes |
| 3 | Managed-pane progress, in-turn reminders and session isolation | pending | `cargo test --locked progress && cargo test --locked --test cli && python3 -m unittest discover -s tests/hermes -v && sh scripts/check-records.sh` exits 0; live managed/unmanaged checks pass |
| 4 | Classic Hermes CLI draft-safe brief and nudge delivery | pending | `cargo test --locked prompt_box && cargo test --locked brief && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; captured-screen and live nudge checks pass |
| 5 | Integrate current-state contracts and verify release | pending | `sh scripts/check.sh && git diff --check` exits 0; operator-guide walkthrough, documentation acceptance, Hermes live matrix and visual-scope review pass |

## Request and approved scope

The preceding request was: “This project have support for Codex, Claude skills/hooks but missing them for Hermes Agent. Can you analyse that first”. This CHG converts that analysis into material work. The operator authorized phased implementation and selected phase 1, “Explicit Hermes profile binding and safe launch/resume”, for this execution cycle. Later phases remain pending.

On 2026-10-11 the operator chose:

- “Operator-provisioned profiles with explicit bindings and reuse checks”.
- “Include profile integration, hooks/skills, and CLI input-box support; keep TUI separate”.

Additional direct request: “verify also open PRs and the https://github.com/afloury/herdr-projects/commit/90659dd patch if it make sense to apply them to the CHGs”. Assessments belong in the review package; reviewing a patch does not authorize merging it. Revalidate pinned upstream revisions before execution and sequence overlapping parser/progress changes deliberately.

[Review package](../reviews/CHG-001/README.md)

Upstream selection is recorded in the [pinned PR assessment](../reviews/CHG-001/upstream-pr-assessment.md).
PR #98 at `e12d74baaa76d64a97ac2ff8d1c41fd2977ef654` supplies prerequisite
managed-pane behaviour. PR #96 at `3363995828e1f683de5c025cb2b1fec01cce86cb`
is an installation/ownership design reference only. OMP parser work is isolated
in [CHG-003](CHG-003-omp-input-box-compatibility.md); if its shared parser changes
land first, rebase phase 4's fixtures/helper changes on that verified state.

Additional direct documentation request: “Do we have also created a comprehensive documentation regarding the Hermes Agent integeration, and how this all fits together? Update existing applicaple CHGs to create/update the documentation including README.md or create a new one. The documentation should also mention, how to setup Hermes Agent the right way (how to deal with HERMES_HOME) to get the full potencial of the integration. Compare it with https://hermes-agent.nousresearch.com/docs/user-guide/bot-mode and https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban-worker-lanes”.

The [operator guide](../../hermes-agent.md) now documents source-checked preparation guidance,
current limitations and the native-feature comparison. That is a documentation
deliverable, not execution of an implementation phase. Keep this CHG as the
single progress authority; no separate documentation CHG is needed.

## Documentation deliverables and acceptance

| Phase | Required documentation update | Evidence required at its gate |
| --- | --- | --- |
| 1 | `docs/hermes-agent.md` and `docs/architecture.md`: explicit root/profile/home/cwd selection, actual binding schema, concurrency, resume and external-writer limitations | Isolated standard/custom roots, inherited root/named HERMES_HOME, sticky selection, symlink aliases, virtual HOME, renamed profiles and resumed-cwd cases; resolved paths recorded without secrets |
| 2 | Operator guide, `README.md`, `docs/getting-started.md`, `docs/operations.md`: exact preview/install/consent/doctor/remove commands, skill/tool requirements and supported versions | Walk the commands on disposable profiles, including foreign/user-edited files and a negative wrong-profile case; no invented flags or global auto-accept |
| 3 | Operator guide and operations: hook/report flow, top-level versus child ownership, progress semantics and troubleshooting | Demonstrate shell reports and context injection reaching the same managed pane; failures, unmanaged panes and task-lineage restrictions remain visible and safe |
| 4 | Operator guide and operations: classic-CLI selection, terminal layouts, draft/menu guards and unsupported Ink TUI | Captured fixture provenance plus live empty/draft/nudge walkthrough; no claim that changing profile settings alone enables nudges |
| 5 | README entry point, operator guide, architecture, decisions and CAPs agree on current status and versions | Full clean-profile operator walkthrough and the acceptance list below; links/snippet syntax checked and no pending behavior advertised as available |

Final documentation acceptance:

- Explain the complete flow from operator provisioning through Herdr preset,
  selected Hermes home, worktree/pane, hooks, reports, inbox, nudge, review and resume.
- Provide a safe standard-root setup and an intentional custom-root variant;
  distinguish HERMES_HOME from account HOME, HERMES_REAL_HOME, tool cwd and Herdr
  config/socket identity. Never advise global worker-home export or credential copying.
- Compare Bot Mode, Kanban worker lanes and Herdr by role, dispatcher, workspace,
  state, completion, review, recurring work and capacity. Explain why a
  terminal-disabled Kanban orchestrator is not a drop-in Herdr coordinator.
- State that there is no Bot Chat/Kanban bridge or cross-product lease. Describe
  dedicated fleets, external-writer quiescence, hidden Bots whose routines still
  run, one Kanban dispatcher where applicable, and preservation of task/run/claim
  descendant scope. Do not turn this comparison into an unapproved bridge project.
- Add a reproducible smoke checklist and troubleshooting for wrong profile,
  wrong cwd, missing skills/consent, hook-versus-shell identity, draft protection,
  occupied profiles, resume after rename and update/unconfigure after user edits.
- Verify all documented commands against the supported runtime and execute the
  applicable workflow on disposable operator-approved profiles. Syntax/help
  checks alone do not establish integration. Record versions, results and bounded
  limitations in this phase; do not expose credentials or private profile contents.

## Baseline findings

- Hermes is already a generic agent kind with resume arguments (`src/agents.rs`). Herdr presets can select it but do not bind/lease Hermes homes (`src/profiles.rs`).
- Hook client registration/CLI validation excludes Hermes; skill linking and repair explicitly target Claude/Codex (`src/setup.rs`, `src/cli.rs`, `src/doctor.rs`).
- Existing hook response shapes cannot be reused directly: Hermes session-start/post-tool observers ignore returns; per-turn context uses `pre_llm_call`; mid-turn reminders need a supported transform path.
- Current global progress hooks can reach unmanaged Herdr panes. Matching a Hermes profile or HERDR_ENV is not sufficient ownership evidence (`src/progress.rs`).
- Herdr configuration follows HOME (`src/paths.rs`). Hermes may provide tool subprocesses a profile-specific HOME. The hook process and the agent's report command must resolve the same Herdr configuration, root and pane.
- Hermes is unknown to `src/prompt_box.rs`; the quiet-input gate in `src/steps.rs` consequently prevents automatic Hermes coordinator nudges.

## Phase 1 — Explicit Hermes profile binding and safe launch/resume

**Goal:** A managed local Hermes coordinator or worker launches and resumes in the intended operator-provisioned home, never a concurrently occupied managed home or an accidental inherited default.

1. Re-read D-002, the review package, live Hermes profile/CLI contracts, and coordinator/thread/remote launch paths. Choose and document the typed preset binding and explicit configuration-root propagation; do not claim a proposed field already exists.
2. Add RED tests for coordinator/worker bindings, invalid/missing profiles, conflicting raw `-p`/`--profile` arguments, inherited HERMES_HOME pointing at either a root or named home, standard/custom roots, sticky defaults, named versus default profiles, spaces/symlink aliases and virtual tool HOME. Validate resolved identity rather than assume the environment wins over CLI/runtime selection. Preserve other agents' argument behaviour and per-role allow-lists.
3. Implement explicit binding resolution. Record the canonical target home together with native session identity. Verify local tool backend/cwd requirements without reading credentials or silently changing user settings; operator-owned profile configuration must allow the assigned project/worktree to remain the tool cwd.
4. Add a bounded, atomic managed-use reservation keyed by canonical Hermes home across projects, Herdr sockets and configured project roots for the same account. Test two competing launchers, coordinator versus worker, failed launch, ticker restart, pane exit, stale ownership and unknown/unreachable ownership. Never release a lease just because an observation timed out. Ordinary external Hermes processes remain outside this guarantee; expose that limitation and warn/refuse when conflicting use is observable.
5. Persist binding through launch/restart/resume, including user edits/renames/removal after initial launch. A missing/mismatched home must produce an actionable refusal, not resume the same ID in another profile or silently start fresh. Audit generic fresh-session fallbacks and Hermes' recorded-cwd restore for this distinction; verify the supported --in/--no-restore-cwd path rather than leaving a resumed worker in the wrong worktree.
6. Separate account/Herdr configuration home, Hermes state home and worktree cwd. Ensure hooks and shell reports use the same explicit Herdr context even under `terminal.home_mode: profile`. Do not globally rewrite HOME or another profile's settings.
7. Update CAP-002 and architecture only for the behaviour actually implemented. Remote Hermes bindings that cannot be verified must be explicitly unsupported/refused, not treated as local paths.
8. Fulfil the phase-1 documentation row. Document lack of shared leasing with Bot Chat, cron/gateway and Kanban writers. Add negative launch tests for unsupported inherited Kanban task/run/claim/descendant contexts; diagnose/refuse rather than stripping lineage or inventing a lifecycle bridge.

**Verification gate:** `cargo test --locked profiles && cargo test --locked paths && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; profile race/resume acceptance cases above pass. Record focused test names and counts so an empty Cargo filter cannot masquerade as evidence.

### Phase 1 execution blocker — launch transport decision

Phase 1 began from clean `main` at `b59f18df7a42651980d975de9d4ccb650549dd71`.
No implementation or regression tests have been added; the phase is not done.
The existing-pane launch transport cannot carry the explicit environment this
phase requires without an additional launch strategy or an upstream change:

- Installed `herdr --version` reports `herdr 0.9.3`. Its `agent start --help`
  accepts kind, pane, timeout and agent arguments, but no environment override.
  The pinned [AgentStartParams schema](https://raw.githubusercontent.com/herdrdev/herdr/v0.9.3/src/api/schema/agents.rs)
  likewise has only `name`, `kind`, `pane_id`, `args` and `timeout_ms`.
- `src/herdr.rs::agent_start`, coordinator tab/reuse launches and
  `src/ticker.rs::launch_pass` use that transport. Setting environment on
  the client command is not a per-agent environment transfer into an existing
  server-owned shell. `open --here` instead uses `Runner::run_foreground` and
  can supply an explicit child environment; that does not solve worker launch.
- `herdr workspace create --help` and `herdr tab create --help` expose `--env`;
  `herdr worktree create --help` and `herdr worktree open --help` do not. New-tab
  environment support alone does not cover recorded-pane/worktree reuse.
- `pane run` is not a guarded replacement for `agent start`: its pinned
  [implementation](https://raw.githubusercontent.com/herdrdev/herdr/v0.9.3/src/cli/pane.rs)
  joins command text and sends `PaneSendInput` with Enter, not a process spawn
  carrying a terminal-identity precondition. No shell-input workaround was used.

**Operator decision required:** authorize a Hermes-specific fresh-process/pane
launch lifecycle (including its effect on worktree root panes and restart pane
reuse), or require the existing-pane lifecycle and make guarded per-launch
environment support in Herdr an upstream prerequisite. The former can be
investigated within this repository using the documented
[layout.apply argv/env contract](https://raw.githubusercontent.com/herdrdev/herdr/v0.9.3/docs/next/website/src/content/docs/socket-api.mdx),
but is not yet implemented or live-verified. Do not weaken identity checks or
silently fall back to the inherited home while deciding.

Environment discovery also found no `cargo`/`rustc` on PATH or executable Cargo
at `$HOME/.cargo/bin/cargo`; Docker and the `rust:1.89-bookworm` image are
available. The documented disposable-container verification route remains an
alternative, not a reason to waive TDD. RED/GREEN and the full phase gate have
not run because implementation is paused at this decision. All later phases
remain pending; no current-state CAP or shipped-behaviour claim changes.

Documentation-only closeout: `sh scripts/check-records.sh && git diff --check`
exited 0 (73 record tests passed; 4 capabilities and 3 change records validated).
Applicable AGENTS.md files remain unchanged: ownership, contracts and workflows
have not changed; this records a blocked decision, not a new launch contract.

## Phase 2 — Reversible profile-scoped skill and startup adapter installation

**Goal:** Operators can explicitly configure, inspect, repair and remove Hermes integration in one selected profile without touching another profile or losing user edits.

1. Add failing setup/doctor tests for explicit target selection, no automatic all-profile installation, repeated configure, dry-run, missing source, moved checkout, foreign symlink/file, user-edited adapter, partial install and rollback/removal.
2. Extend setup's format/installation boundary rather than passing Hermes YAML through the JSONC editor. Use supported profile-targeted Hermes management commands where configuration mutation is needed; do not overwrite the whole config or change global auto-accept. Journal only owned artifacts/settings and verify read-back before reporting success.
3. Link the shared autoproject skill into the chosen profile's discovery path. Verify actual skill discovery and plain `/autoproject` invocation; plugin namespacing must not silently rename this entry point. Do not duplicate the skill's protocol.
4. Before enabling startup injection, implement and test managed-pane authorization using the selected #98 behaviour; do not introduce an interim adapter that injects into unrelated panes. Add a small Python plugin adapter using documented registration. Keep progress-state decisions in Rust and calls bounded/argv-safe. Inject full startup guidance plus per-turn reminders through `pre_llm_call`; `on_session_start` must not be mistaken for a context injection hook. Phase 3 expands event/session coverage and long-turn delivery.
5. Add adapter tests under the proposed `tests/hermes/` boundary and a child DOX contract at the chosen adapter location. Use synthetic plugin/event contracts for deterministic tests and an isolated real Hermes profile for registration checks. Add Python adapter tests to the full gate once they exist.
6. Doctor must distinguish installed, enabled/consented, stale, foreign/edited and unavailable integrations. Declined consent is a diagnostic, not something to bypass. Unconfigure removes only this integration, never a profile or its memory/auth/session data.
7. Update CAP-001, CLI help, `docs/hermes-agent.md`, README, getting-started and operations for verified destinations and supported Hermes versions. Fulfil the phase-2 documentation row with executable examples. Document profile selection precedence rather than hardcoding this operator's profile name; keep current/pending status explicit until each behavior is proven.

**Verification gate:** `cargo test --locked && python3 -m unittest discover -s tests/hermes -v && sh scripts/check-records.sh` exits 0; isolated install round-trip passes. The full Rust suite includes the prerequisite managed-hook CLI/progress regressions. Tests/hermes is a deliverable of this phase, not an existing baseline suite.

## Phase 3 — Managed-pane progress, in-turn reminders and session isolation

**Goal:** Hermes receives useful progress instructions/reminders only for its managed top-level pane, and agent-issued reports reach the same record as the hook.

1. Extend phase 2's managed-pane scoping using the pinned upstream PR #98 evidence and the non-JSON extension ownership pattern in #96 from the review package. Reuse justified behaviour/tests, not unreviewed patches or unrelated harness/safety changes.
2. Write RED tests for unmanaged panes, managed coordinators/additional coordinators/workers, separate worktrees, socket/pane/terminal reuse, detection lag, failed/malformed ownership queries and remote records that must not authorize local panes. Preserve existing hooks' behaviour except for the explicitly recorded managed-scope change.
3. Normalize Hermes event fields to the Rust protocol. Establish top-level session identity explicitly; never let a delegated child's startup reset the parent record. Test task/session/parent markers, resume, reset, compaction and a hook installed after a session already exists.
4. Use a documented `transform_tool_result` path to add due reminders during long turns, preserving original output and respecting transform composition. `post_tool_call` return values and once-per-turn `pre_llm_call` are not substitutes. Test actual consumption by the supported Hermes runtime, not merely callback return values.
5. Preserve throttle, self-report suppression, wait-state clearing and fail-open timeouts. Do not mark work complete because a turn/tool ended. Test terminal calls through `execute_code` and delegation so callbacks cannot spuriously attribute helper work to the parent.
6. In isolated live Herdr/Hermes sessions, prove managed guidance, no unmanaged context/write, one due reminder during a long turn, an actual shell `report` reaching the correct pane, and correct new-turn/resume behaviour. Include profile-specific HOME and distinct concurrent profiles. Do not use the operator's live profile or expose credentials.
7. Update CAP-001, the operator guide and operations with proven ownership and limitations, fulfilling the phase-3 documentation row. Pin runtime versions and exact test evidence in this phase before marking it done. Explain that report percentage/turn exit is neither a Herdr review decision nor a Kanban terminating action.

**Verification gate:** `cargo test --locked progress && cargo test --locked --test cli && python3 -m unittest discover -s tests/hermes -v && sh scripts/check-records.sh` exits 0; live managed/unmanaged checks above pass.

## Phase 4 — Classic Hermes CLI draft-safe brief and nudge delivery

**Goal:** A classic-CLI Hermes coordinator can receive automatic nudges without overwriting a draft or answering a trust/approval prompt.

1. Reconcile upstream parser/brief candidates from the review package before touching shared helpers. Preserve unrelated agents and do not import OMP layouts as Hermes fixtures.
2. Capture sanitized real classic-CLI screens with Hermes version, interface selection, terminal size and capture method: initial/after-turn empty, single/multiline draft, dim suggestion, menu, trust/hook-consent prompt, busy state, resized/wrapped input and unsupported layout. Add provenance beside fixtures under a durable tests contract.
3. Write RED parser tests and implement only positively identified layouts. Keep Unknown for Ink TUI and unsupported skins/layouts; do not weaken generic rule detection to make a sample pass.
4. Add scenario tests for quiet interval, a draft appearing between ticks, one nudge per unseen event set, failed/blocked send retry, first brief, prompt left unsubmitted, and manual recovery. Readiness must be supported by current pane evidence, not inferred from a glyph alone.
5. Run live brief/nudge tests in an isolated Herdr session. Confirm a typed draft is unchanged and a quiet empty coordinator receives and processes a nudge. Record failures/unsupported layouts explicitly.
6. Update CAP-003, the operator guide and operations with classic-CLI-only support and fail-safe limitations, fulfilling the phase-4 documentation row. Document actual --cli/display.interface selection and known layouts; profile choice alone does not imply classic CLI. No new human-facing settings/popup flow is required by this scope.

**Verification gate:** `cargo test --locked prompt_box && cargo test --locked brief && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; captured-screen and live nudge checks above pass.

## Phase 5 — Integrate current-state contracts and verify release

**Goal:** The repository truth, automated gate and live evidence agree on the supported Hermes contract.

1. Run the full suite, release build and adapter tests added to `scripts/check.sh`. Verify the minimum supported Hermes version and record the exact tested Herdr/Hermes/Python versions.
2. Exercise distinct concurrent coordinator/worker profiles, same-profile refusal, restart/resume after preset changes, installation/removal after user edits, virtual HOME, and classic-CLI draft protection. No default-profile leakage, secret reads, model-privilege widening or weakened trust gates.
3. Bring CAP-001/002/003, architecture, D-002 implementation status, CLI help, README, `docs/hermes-agent.md`, getting-started and operations into agreement with shipped behaviour. Run the documented setup-to-resume/removal walkthrough on disposable profiles and complete every documentation acceptance item above. Retain the native Bot Mode/Kanban comparison and no-bridge boundary. Leave upstream recommendations not implemented clearly deferred.
4. Recheck primary-surface classification. These CAPs cover headless launch/configuration/progress/delivery, not a new primary screen; `none` is intentional. If implementation adds a profile-picker/dialog/popup workflow, stop to extend the owning CAP and adopt canonical `docs/product/wireframes/generate.mjs`, an explicit renderer command, HTML/PNG/CAP links and visual inspection before closure. Never substitute review artifacts for canonical visuals.
5. Retain review packages at stable paths as frozen review-only receipts. Repair links when archiving. Only this phase may set the CHG done and move it to archive after every phase gate passes; repository commit/push still needs operator authorization.

**Verification gate:** `sh scripts/check.sh && git diff --check` exits 0; operator-guide walkthrough, documentation acceptance, Hermes live matrix and visual-scope review pass.

## Out of scope

- Implementing this feature during blueprint adoption/planning.
- Automatic Hermes profile creation, cloning, pool scheduling, deletion or credential copying.
- Treating profiles as OS sandboxes or claiming exclusive ownership of unrelated external processes.
- Ink TUI, Desktop, gateway, remote terminal backends and remote Hermes profile provisioning.
- Blanket adoption of open upstream PRs, unrelated harnesses, timezone/worktree policy features or permission-default changes.
- Replacing the Rust reporter with a second progress database/daemon or a profile-private plan.
- Implementing a Bot Chat, group messaging, cron or Kanban worker-lane bridge;
  comparing their native behavior and documenting safe coexistence are in scope.
