# CHG-003 — Reconciled OMP input-box compatibility

**Status:** planned
**External request:** Direct operator request: verify also open PRs and the https://github.com/afloury/herdr-projects/commit/90659dd patch if it make sense to apply them to the CHGs
**Impacts:** CAP-003
**Baseline:** `4e4548c` — release `0.2.34`; OMP remains an unsupported input-box kind

| # | Phase | Status | Verification gate |
| --- | --- | --- | --- |
| 1 | Default OMP editor with draft-safe hint handling | pending | `cargo test --locked prompt_box && cargo test --locked brief && sh scripts/check-records.sh` exits 0; default-layout fixture matrix passes |
| 2 | Guarded alternate editor and nudge integration | pending | `cargo test --locked prompt_box && cargo test --locked scenarios && sh scripts/check-records.sh` exits 0; alternate-layout and transcript-negative matrix passes |
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

**Verification gate:** `cargo test --locked prompt_box && cargo test --locked brief && sh scripts/check-records.sh` exits 0; default-layout fixture matrix above passes. Record matched test names/counts.

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
