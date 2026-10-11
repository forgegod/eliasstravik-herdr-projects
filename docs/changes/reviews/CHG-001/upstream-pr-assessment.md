# Open upstream PR review

**Status:** review-only

## Scope and evidence

Reviewed on 2026-10-11 against baseline `4e4548c` (release 0.2.34).
The current fork `forgegod/eliasstravik-herdr-projects` had no open PRs;
upstream `eliasstravik/herdr-projects` had 15. Pagination was checked and the
inventory deduplicated by PR number. The table pins the reviewed head of every PR.
PR descriptions, changed code, discussion/review metadata and baseline call
sites were examined. All heads remained unchanged on the final fetch.

All 15 fresh raw diffs passed `git apply --check` individually against this
checkout. Some terminal fixtures produce whitespace warnings. This is NOT a
combined-stack check, a Rust test run for those patches, or approval to merge.
No status checks were reported by GitHub for these heads; author-reported test
and live results are third-party evidence, not runs performed during this review.
The unchanged baseline and this adoption's full gate were executed separately.

Earlier pretty-printed captures lost ANSI/line-ending/trailing-context bytes:
#109 appeared corrupt and #105 appeared conflicting. Parent verification passed
the freshly fetched byte streams directly to Git, confirming both apply. Never
apply a patch copied out of a rendered tool result; preserve the raw bytes.

## Dispositions

“Separate candidate” is a recommendation, not an active change or promise to
implement. Only the directly relevant Hermes boundary is scheduled in CHG-001.
The explicitly requested OMP patch family is scheduled separately in
[CHG-003](../../active/CHG-003-omp-input-box-compatibility.md).

| PR | Reviewed head | Disposition | Reason and gate before adoption |
| --- | --- | --- | --- |
| [#87 — fix(prompt_box): accept a rule that carries a label in its border](https://github.com/eliasstravik/herdr-projects/pull/87) | `189bd63db7d97db68b9503f4b6ac487eb010f4ae` | Separate candidate; prefer over #93 | Claude-labelled top border only; preserves strict bottom/Pi rules. Real fixture plus negative cases. Keep narrow-pane limitation explicit. |
| [#88 — feat(thread): let a thread start name its branch and worktree folder](https://github.com/eliasstravik/herdr-projects/pull/88) | `614e410f0d92f393b569a82b7df08ed88f667b63` | Defer; separate worktree-policy request | Branch/path override is unrelated to Hermes. It also carries a nudge-bookkeeping hunk: split scope before reuse. Baseline Thread already has struct-level #[serde(default)], so missing per-field defaults are not a demonstrated bug; add legacy-record regression tests anyway. |
| [#93 — A label in the input box's top border still reads as a rule (#85)](https://github.com/eliasstravik/herdr-projects/pull/93) | `d7e9b011ec3fc6a1e3b64d86b533774b520ba31d` | Do not select alongside #87 | Alternative fix for the same labelled-border issue; broadens generic is_rule. #87 is more narrowly scoped. |
| [#94 — The project's workspace is found by a pane's foreground cwd too (#92)](https://github.com/eliasstravik/herdr-projects/pull/94) | `ced79603b6e4f653ce8aaa1aefe79b5a7b49940f` | Separate candidate | Foreground-cwd workspace lookup corrects open --here placement; not a Hermes profile/hook dependency. Add broader multi-workspace regressions before adoption. |
| [#96 — Pi and OMP: progress extension, and OMP never asks](https://github.com/eliasstravik/herdr-projects/pull/96) | `3363995828e1f683de5c025cb2b1fec01cce86cb` | CHG-001 design reference only | Reuse the non-JSON adapter/ownership/doctor-verification approach, not Pi/OMP code, Bun tooling or permission defaults. This head also contains OMP parser/fixture changes despite the PR body listing nudges out of scope; inspect the diff, not only its description. |
| [#98 — Limit progress hooks to managed project panes](https://github.com/eliasstravik/herdr-projects/pull/98) | `e12d74baaa76d64a97ac2ff8d1c41fd2977ef654` | CHG-001 phase 3 prerequisite behaviour | Managed-pane scope directly applies to profile-installed Hermes hooks. Preserve read-only discovery, socket/terminal checks, child exclusion and fail-silent observations. Author live evidence explicitly does not establish successful sandboxed shell reports. |
| [#99 — The digest shows the local time, so agents stop passing UTC times on as local](https://github.com/eliasstravik/herdr-projects/pull/99) | `fc3ac79529a4028bbbb67722d7f2ae802191baff` | Defer; separate digest-localization candidate | Local-time hint is unrelated to Hermes installation. Preserve UTC storage and DST tests if selected later; do not mislabel it as a required profile fix. |
| [#101 — Recognize grok and agy input boxes](https://github.com/eliasstravik/herdr-projects/pull/101) | `74dc4f3914ac79d4f50d659a3d00f84bc51b46d5` | Defer; split compatibility from delivery semantics | Adds grok/agy layouts and switches all nudges to confirmed prompt delivery. Confirm supported Herdr --wait/--until behaviour and failure/duplicate semantics; reconcile its nudge hunk with #88 if either lands. |
| [#102 — Every project is polled in the tick a remote machine is due (#80)](https://github.com/eliasstravik/herdr-projects/pull/102) | `b400dafd65beae77236f4b087e53ee5780aec82f` | Separate candidate; worthwhile remote scheduling fix | Same-tick eligibility prevents the first project consuming a shared machine’s polling turn. Keep per-machine backoff; add a multi-project ticker scenario rather than relying only on the helper test. |
| [#104 — Read omp's input box so the ticker can nudge an omp coordinator](https://github.com/eliasstravik/herdr-projects/pull/104) | `615f6bef981c10cc6115640819c6fd8bbb56601a` | CHG-003 input | Default OMP editor, italic hints and real screen fixtures. Combine with the pinned hint fix and 90659dd; do not stack competing OMP parser arms. |
| [#105 — A nudge reaches an omp coordinator: its input box is read](https://github.com/eliasstravik/herdr-projects/pull/105) | `2024b4ad98ab2406296ff6af7d7baad4b4a6ee08` | CHG-003 reference; not a standalone import | Alternative bar layout is incorporated as a guarded fallback by 90659dd. It applies alone, but competes with #104/#96 and uses overlapping fixture paths. |
| [#108 — thread read marks the dim text of an input box](https://github.com/eliasstravik/herdr-projects/pull/108) | `2b95f1d1277acce5a96c109a7619398755e384b2` | Separate candidate paired with #109 | Changes default thread read to annotated ANSI-derived input-state output. Shared ghost/typed classification is useful; preserve scrollback behaviour and test quoted markers/suggestions. |
| [#109 — A coordinator sends work only with thread prompt](https://github.com/eliasstravik/herdr-projects/pull/109) | `e6a431d055c8efec602ad3d7155d0c7a21cc16c2` | Separate candidate paired with #108 | Coordinator protocol refers to [dim: …] markers supplied by #108. Do not land that instruction dependency alone. |
| [#110 — A nudge leads with the newest event, not the oldest backlog](https://github.com/eliasstravik/herdr-projects/pull/110) | `7a0a9e952f657c69f6a684d45703efb5e33db6a4` | Separate candidate; worthwhile nudge-content fix | Newest-first summary prevents old backlog hiding new subjects; set-based send deduplication is unchanged. Add >5-subject/tied-timestamp/multi-event coverage before adoption. |
| [#120 — fix: deliver first brief to ready Codex threads](https://github.com/eliasstravik/herdr-projects/pull/120) | `bb1c2fdb5b57abfe1e103a40be9598740574c6eb` | Defer while draft; separate readiness investigation | Still draft at review. Its interactive_ready exception has Codex evidence; it does not establish Hermes readiness. Do not weaken Unknown/blocked/trust handling globally. |

## Dependencies and shared-file risks

- #87 and #93 are alternatives for one bug. Prefer the Claude-top-border-only
  implementation, not a combined broadening of the shared rule predicate.
- #104, #105 and #96 contain competing OMP parser arms and conflicting fixture
  paths. Reconcile supported layouts and fixture provenance; patch application
  in isolation says nothing about their composition.
- #108 adds screen coordinates to Cell while #104 refactors ANSI style state.
  Either can affect a later Hermes parser; preserve both behaviours if selected.
- #109 depends on #108's output markers. Apply/reimplement them as one slice.
- #88 includes nudge bookkeeping unrelated to its branch/path title, overlapping
  #101's confirmed-send change. Split the unrelated hunk and test combined
  delivery/dedup behaviour if selected. #110 changes summary ordering, not the
  send predicate, but scenario expectations still need reconciliation.
- #96 is wider than its title/body's original scope. Its permission policy is
  not a transferable Hermes rule, and its OMP fixtures are not Hermes evidence.
- None of these PRs adds Hermes to the input-box parser. CHG-001 still requires
  real classic-CLI captures and profile/child/virtual-HOME verification.

## Referenced OMP commit

[90659ddf1ae74c2210dbe204fd97fc5d75494f18](https://github.com/afloury/herdr-projects/commit/90659ddf1ae74c2210dbe204fd97fc5d75494f18)
is a child of
[397467d657aa86774eff732cafa7747f319d4715](https://github.com/afloury/herdr-projects/commit/397467d657aa86774eff732cafa7747f319d4715),
whose parent is #104's `615f6bef981c10cc6115640819c6fd8bbb56601a`.
The middle commit handles multi-glyph hint keys. The final commit adds #105's
bar-style editor as a fallback when the default anchor is absent, with an
at-most-one-nonempty-trailing-line guard.

That guard reduces transcript false positives; it does not actually validate
that the trailing line is a status bar and is not a general proof that quoted
text can never be mistaken for input. CHG-003 requires negative cases for
zero/one/multiple trailing transcript lines, stale default anchors, and typed
text next to italic hints before adopting the concept. It is not a standalone
patch against this baseline and should not be stacked with standalone #105.

## Handoff

No application source, installed binary, active profile, remote branch or PR
review was changed. Re-fetch and review the pinned diffs before execution; a
later upstream merge/head update requires fresh applicability checks. The
separate fork-updater concept is assessed under CHG-002, not this package.
