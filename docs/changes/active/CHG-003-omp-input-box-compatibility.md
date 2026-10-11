# CHG-003 — Reconciled OMP input-box compatibility

**Status:** in-progress
**External request:** Direct operator request: verify also open PRs and the https://github.com/afloury/herdr-projects/commit/90659dd patch if it make sense to apply them to the CHGs
**Impacts:** CAP-003
**Baseline:** `4e4548c` — release `0.2.34`; OMP remains an unsupported input-box kind

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Default OMP editor with draft-safe hint handling | done (synthetic default-layout parser and delivery-isolation matrix passed) | `cargo test --locked prompt_box && cargo test --locked brief && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; parser matrix and automatic-delivery isolation pass |
| 2 | Guarded alternate editor and nudge integration | in-progress | `cargo test --locked prompt_box && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; alternate-layout and transcript-negative matrix passes |
| 3 | Current capability, live compatibility and integration | pending | `sh scripts/check.sh && git diff --check` exits 0; isolated OMP draft/nudge smoke and visual-scope review pass |

## Applicability and source selection

The explicit patch request is applicable to OMP, not Hermes. Baseline OMP is a
recognized launch kind but its input box is Unknown, blocking automatic
coordinator nudges. Keep this change separate from Hermes profile/hook work.

Pinned reference chain:

1. [Upstream PR #104](https://github.com/eliasstravik/herdr-projects/pull/104), head `615f6bef981c10cc6115640819c6fd8bbb56601a`: default OMP editor and italic hints.
2. [397467d657aa86774eff732cafa7747f319d4715](https://github.com/afloury/herdr-projects/commit/397467d657aa86774eff732cafa7747f319d4715): multi-glyph hint-key handling; parent is #104's head.
3. [90659ddf1ae74c2210dbe204fd97fc5d75494f18](https://github.com/afloury/herdr-projects/commit/90659ddf1ae74c2210dbe204fd97fc5d75494f18): alternate bar editor fallback; parent is the hint fix.

[PR #105](https://github.com/eliasstravik/herdr-projects/pull/105), head
`2024b4ad98ab2406296ff6af7d7baad4b4a6ee08`, supplies the alternate-layout idea;
do not stack it as another parser arm. PR #96 also modifies OMP parsing and
creates overlapping fixture names; its extension/permission changes are not
part of this CHG.

The complete per-PR assessment lives in the shared
[review package owned by CHG-001](../reviews/CHG-001/README.md), specifically
[the pinned review](../reviews/CHG-001/upstream-pr-assessment.md).
There is no second mutable progress authority in that package.

## Phase 1 — Default OMP editor with draft-safe hint handling

**Goal:** Supported default OMP editor screens classify empty, drafted and
unknown states without discarding user text as a decorative hint.

1. Re-fetch the pinned raw patches and capture or verify sanitized OMP screen
   fixtures with runtime version, terminal size, symbol preset and capture
   method. Preserve ANSI bytes; rendered tool output is not a patch transport.
2. Add RED cases in `src/prompt_box.rs` for initial/after-turn empty, one/multiline
   draft, busy screen, slash menu, default/nerd hint glyphs, missing editor,
   narrow/wrapped input and truncated/malformed ANSI styling.
3. Reimplement/adapt the default-editor and multi-glyph hint behaviour together.
   Prove typed text before/next to an italic suffix remains Typed; a one-space
   token chain must not let hint stripping consume an actual draft.
4. Preserve all existing harness classifications. Reconcile Cell/ANSI changes
   with any already-landed #108/Hermes parser work rather than replacing another
   adapter's state fields. Use distinct fixture names for different layouts.
5. Update CAP-003 only for verified default-layout support, with exact tests and
   known unsupported versions/layouts. Do not claim the alternate layout yet.
6. Keep parser recognition separate from automatic delivery. OMP must not
   advance the nudge quiet gate, send/retry/confirm ticker briefs, trigger an
   automatic PR follow-up, or dismiss a trust screen based on parser Empty.
   Phase 2 owns explicit enablement after its delivery-safety matrix passes.

**Verification gate:** `cargo test --locked prompt_box && cargo test --locked brief && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; default-layout fixture matrix and automatic-delivery isolation pass. Record matched test names/counts.

### Phase 1 evidence and limits

- Phase 1 is complete on synthetic default-layout evidence. The confirmed
  automatic-delivery and trust-reporting regressions were corrected before its
  final gate. Phase 2 is now active; phase 3 remains pending.
- Executed only in `/home/raphael/src/herdr-projects-chg003-phase1`, branch
  `chg003-phase1`. Phase 1 entered in-progress before production edits.
- Re-fetched all three pinned commit `.patch` byte streams over public HTTPS;
  inspected them without applying/importing commits. The one-space backward
  token walk was rejected: only a complete padded hint row with an allow-listed
  key, exact label and italic label styling is discarded. Draft-plus-hint rows
  are retained whole. Existing Cell fields and non-OMP policy are preserved.
- Host Cargo was absent (exit 127, not RED evidence). Used the existing
  `herdr-projects-blueprint-verify:latest` container with Cargo 1.89.0, repository
  mounted read-only, host UID/GID, and isolated HOME/TMPDIR/Cargo home/target under
  `/home/raphael/.hermes/cache/scratch/chg003-phase1`. No live agent or config used.
- Each row below ran as
  `cargo test --locked prompt_box::tests::<name> -- --exact`: exactly one test
  failed (exit 101) before its production change, then passed (exit 0).

| Test name | Observed RED mismatch |
| --- | --- |
| `omp_default_empty_editor_is_readable` | Unknown instead of Empty |
| `omp_default_hints_do_not_consume_adjacent_drafts` | Typed instead of Empty for the default italic hint |
| `omp_default_preserves_styled_drafts_and_continuations` | Empty string instead of `hello draft` for a dim draft |
| `omp_default_malformed_ansi_is_unknown` | Empty instead of Unknown for a trailing ESC |

- The phase command passed: `prompt_box` matched 8 tests (the four above and
  `every_supported_kind_tells_an_empty_box_from_a_draft`,
  `a_screen_without_the_box_or_an_unknown_kind_is_unknown`,
  `menus_and_multi_line_drafts_count_as_typed`, `a_real_character_under_a_cursor_is_typed`).
- `brief` matched 16 passing tests: `brief_order_and_memory_cap`,
  `an_unconfirmed_follow_up_is_reported_and_not_sent_again`,
  `a_blocked_answer_types_nothing_and_does_not_count`,
  `the_brief_script_works_against_a_real_repository_with_a_hostile_path`,
  `a_fresh_claim_is_left_alone_and_a_dead_one_is_checked_on_screen`,
  `a_remote_brief_goes_out_between_machine_polls`,
  `a_tab_thread_gets_a_brief_with_the_project_header_and_prompts_are_recorded`,
  `thread_brief_delivers_a_pending_brief_once_and_only_to_a_ready_agent`,
  `a_kind_whose_box_cannot_be_read_gets_one_try_then_goes_to_a_person`,
  `a_stalled_brief_left_in_the_box_gets_enter_never_a_second_copy`,
  `adopting_a_ready_agent_writes_a_brief_and_prompts_it`,
  `a_brief_waits_while_a_trust_screen_shows_even_when_herdr_reads_idle`,
  `a_tab_thread_with_a_repo_gets_its_brief_seconds_after_its_agent_is_ready`,
  `a_draft_in_the_box_or_an_unreadable_screen_after_a_try_waits`,
  `a_start_up_screen_that_reads_idle_gets_nothing_until_the_box_shows_and_settles`,
  `a_dropped_brief_is_typed_again_up_to_the_cap_then_one_item_and_waiting_on_you`.
- Records: 73 tests passed; tree validation passed (4 CAPs, 3 CHGs).
  Additional `cargo test --locked --quiet` passed 362 unit/scenario and 4 CLI
  tests. `git diff --check` passed. No release build or live smoke is claimed.
- Approved evidence substitution: labeled synthetic inline fixtures, not new
  runtime captures. Phase 1's runtime-version/terminal-size/capture-provenance
  requirement was not performed; live certification remains phase 3. Source
  references describe 18.4.10/default and 18.6.1/nerd, not locally verified runs.
- The preceding parser-cycle evidence is retained from the prior implementation
  record; the independently executed correction evidence follows below.
- Limits: only the thinking-effort hint is recognized; other labels, narrow
  unpadded hints and unsupported ANSI conservatively block empty detection.
  Exact transcript/styled-input imitations are not certified safe, so automatic
  OMP delivery is disabled. Alternate-layout, stale-anchor and safe OMP nudge
  enablement remain phase 2; no phase 2/3 work was implemented.
- DOX pass: `src/AGENTS.md` now owns the parser-versus-delivery policy and
  automatic follow-up entry point. Root/docs/product/changes DOX contracts and
  Child DOX Indexes remain unchanged: ownership and structure did not change.
  CAP/CHG records hold the behavior/evidence update.

### Phase 1 safety correction evidence

- Direct operator correction: the independent review's “no findings” conclusion
  was rejected because `steps::box_quiet` consumed parser Empty directly. This
  was a scope regression, not an accepted limitation.
- Direct request: “Correct a confirmed Phase-1 scope regression in
  `/home/raphael/src/herdr-projects-chg003-phase1` only.”
- First edit added only
  `scenarios::omp_default_empty_cannot_advance_the_nudge_quiet_gate`. Exact test
  command: `cargo test --locked scenarios::omp_default_empty_cannot_advance_the_nudge_quiet_gate -- --exact`.
  RED exited 101: the synthetic default editor parsed Empty, the discovered
  coordinator was OMP, the real screen-read path ran, and the quiet timestamp
  incorrectly became nonempty. Failure: `parser Empty must not start OMP's
  automatic quiet interval`. GREEN exited 0 (1 passed) after the policy gate;
  the same test also proves an old persisted quiet interval cannot send.
- Additional RED coverage (`cargo test --locked omp_`, exit 101): automatic
  brief delivery cleared pending state; a PR routine sent 1 prompt instead of
  0; trust detection returned None instead of the trust phrase. The brief
  test's initial probe passed because its stored/listed kinds mismatched; that
  harness defect was corrected before the observed RED, not counted as RED.
- `delivery_enabled` leaves `check`, `box_text` and `knows` available but denies
  automatic OMP delivery. Audited all their callers: `steps::nudge` resets its
  quiet state; both ticker brief entry points reach the gate before first
  send/retry/Enter/transcript confirmation; `fire_pr_routines` uses
  `threads::prompt_automatic`, which gates the resolved agent kind before send.
  `trust_screen::detect` cannot treat OMP Empty as proof a trust phrase is only
  transcript. Explicit `thread prompt`/`next`, `thread brief` and adoption
  remain operator actions; the PR regression verifies explicit follow-up still
  works. No launch, approval/yolo or profile policy was changed.
- Fresh scratch:
  `/home/raphael/.hermes/cache/scratch/chg003-phase1-safety-vrp50spq`.
  Container: `herdr-projects-blueprint-verify:latest`, Cargo 1.89.0, Node
  22.23.3, Git and rsync; only this worktree mounted at `/repo:ro`; host UID/GID;
  isolated HOME, TMPDIR, CARGO_HOME and CARGO_TARGET_DIR under `/scratch`.
  No live agent, profile or credentials used. Inline screens are synthetic,
  not runtime captures or live certification.
- GREEN (all exit 0): `cargo test --locked omp_` — 8 tests;
  `cargo test --locked prompt_box` — 8; `cargo test --locked brief` — 17;
  `cargo test --locked scenarios` — 85; `cargo test --locked trust_screen` — 5.
  Filtered CLI integration binaries matched 0 tests, not an integration pass.
  Exact outputs: `red-nudge.log`, `green-nudge.log`,
  `red-remaining-paths.log`, `green-rust.log` in the scratch directory.
- `sh scripts/check-records.sh` exited 0: 73 fixture tests passed and the
  current record tree validated. Additional `cargo test --locked` exited 0:
  366 unit/scenario tests and 4 CLI integration tests passed. Output:
  `records-and-full-tests.log` in the same scratch directory.
- `git diff --check` passed on the host worktree, not in the container: its
  worktree-only mount does not resolve the external Git metadata. CHG-001/002
  were not edited. No commit, push, release build or live smoke was performed.
- Final trust-reporting correction: the ticker brief gate initially returned
  `Waiting("automatic delivery is not enabled for omp")` before checking an OMP
  trust phrase. The focused RED exited 101; GREEN moved trust detection before
  the delivery gate and proved no prompt/key/attempt mutation. A fresh offline
  container gate passed 8 `prompt_box`, 18 `brief`, 85 `scenarios`, 6
  `trust_screen` and 73 record tests; host `git diff --check` passed.

## Phase 2 — Guarded alternate editor and nudge integration

**Goal:** The alternate OMP bar editor supports safe automatic nudges while
transcript quotations and stale anchors remain Unknown or Typed.

1. Add RED cases for alternate-layout empty/draft/multiline screens, menu/busy
   states, a quoted bar block with zero, one and multiple trailing transcript
   lines, blank padding, a stale default anchor above a newer alternate editor,
   and windows that omit the true input box.
2. Adapt the 90659dd fallback concept. Its at-most-one-nonempty-line guard does
   not validate a status bar by itself; require sufficient observed layout
   evidence before returning Empty. Do not import standalone #105's competing arm.
3. Test `knows`, `box_text`, brief-delivery fallback and the quiet-input nudge
   path together. Simulate drafts between ticks, transient capture failure,
   trust/approval dialogs, failed prompt delivery and eventual readiness.
4. Keep parser support independent of progress extensions and launch permission
   defaults. OMP's policy is not changed by recognizing its input box.
5. Update CAP-003 and supported-layout docs for the now-proven alternate layout.

**Verification gate:** `cargo test --locked prompt_box && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; alternate-layout and transcript-negative matrix above passes.

## Phase 3 — Current capability, live compatibility and integration

**Goal:** Source, tests and operator documentation agree on OMP support without
weakening draft protection for other harnesses.

1. Run the full gate and an isolated real OMP/Herdr smoke for each supported
   layout: verify a quiet empty box receives one nudge, typed input survives,
   and a menu/unknown screen receives nothing. Record exact runtime versions
   and capture provenance; author-reported upstream runs are not this evidence.
2. Reconcile CAP-003, operations, fixture provenance and nearest DOX contracts.
   Check composition with CHG-001's Hermes parser if already implemented; neither
   CHG may silently override the other's supported-kind tests or ANSI fields.
3. Visual classification stays `none`: this changes an automatic parser/delivery
   contract, not OMP's owned UI or a new Herdr Projects screen. If a diagnostic
   screen/dialog is added, stop and extend the owning CAP with canonical
   `docs/product/wireframes/generate.mjs`, an explicit renderer, HTML/PNG links
   and visual inspection rather than treating terminal captures as wireframes.
4. Preserve the shared review package under its owner at the stable path. Only
   this phase may mark done/archive after every gate passes. Repair links to
   both CHGs when either moves; committing or importing upstream commits still
   requires operator authorization.

**Verification gate:** `sh scripts/check.sh && git diff --check` exits 0; isolated OMP draft/nudge smoke and visual-scope review above pass.

## Out of scope

- Hermes profiles/hooks or Ink TUI support; see CHG-001.
- Pi/OMP progress extensions, Bun dependencies, or OMP yolo/approval policy.
- Other open PRs, blanket fork merges, automatic live updates or source changes
  during this planning task.
- Claiming individual patch application proves combined-stack correctness or
  that a trailing-line count proves a transcript can never mimic input.
