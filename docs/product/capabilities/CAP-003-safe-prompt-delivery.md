# CAP-003 — Safe automated prompt delivery

**Status:** implemented
**Primary surface:** none

## Behaviour

- The terminal parser distinguishes empty input from a draft for Claude, Codex, Cursor, Gemini, OpenCode and Pi using per-kind layouts and ANSI styling. OMP supports the default `╰─ ` editor and its continuation/menu rows. Unsupported kinds, including Hermes, return Unknown.
- OMP ignores only a whole padded first-row hint with the exact default `⇧⇥` or nerd `U+F0636 U+F0312` key and italic `to change thinking effort` label. Text beside that hint remains intact, including one-space token chains; dim and italic drafts and continuation/menu text count as Typed. Other hint labels, unstyled lookalikes and narrow/wrapped partial hints remain Typed rather than being stripped.
- OMP rejects incomplete/malformed or unsupported ANSI sequences and unclosed italic styling as Unknown. Its ANSI validation is adapter-local; other kinds retain their existing classification policy. An empty editor on a working screen describes input only, not readiness to send.
- Parser support (`check`/`box_text`/`knows`) does not authorize automatic delivery. The separate `delivery_enabled` policy disables OMP coordinator nudges, ticker brief sends/retries/confirmation, and automatic PR-routine follow-ups. OMP Empty does not start or retain a nudge quiet interval and cannot suppress trust-screen detection. Explicit brief/adoption and follow-up commands remain operator actions, with draft and trust checks.
- Automatic coordinator nudges for enabled kinds require an input box observed empty over the quiet interval; drafts and unknown layouts reset the wait. A missing Hermes parser therefore suppresses automatic Hermes coordinator nudges.
- Brief delivery checks agent readiness, trust screens and prompt confirmation state. A known draft is not overwritten; unsupported input layouts take the conservative fallback rather than assuming empty.
- Nudge text summarizes subjects/events without copying message bodies; inbox-set deduplication avoids replaying the same acknowledged nudge set.

## Implementation

- `src/prompt_box.rs` — ANSI cells, per-kind input parsing and empty/draft/unknown classification.
- `src/brief.rs` — brief delivery and confirmation.
- `src/threads.rs` — separates explicit follow-ups from delivery-gated automatic follow-ups.
- `src/trust_screen.rs` — trust-screen guards.
- `src/steps.rs` — coordinator nudge quiet/dedup gates.

## Rules and boundaries

This CAP owns the automatic delivery protocol and input protection, not the
external agent's UI or Herdr Projects' popup/sidebar design. Unknown must not be
made empty as a compatibility shortcut. Input fixtures must distinguish real
captures from synthetic examples. See [architecture](../../architecture.md).

OMP evidence is limited to labeled synthetic inline fixtures in
`src/prompt_box.rs`, informed by the pinned default-layout patches describing
OMP 18.4.10 and 18.6.1. No runtime version or terminal size is live-certified.
The alternate `▎` layout is unsupported. Unknown hint labels conservatively
block empty detection. Screen text/styling cannot authenticate an editor:
an exact transcript or styled-draft imitation is not a proven safe capture.
Automatic OMP delivery remains disabled rather than accepting that risk.
Missing anchors and incomplete ANSI do not imply empty input.

## Verification

- `src/prompt_box.rs` — tests `every_supported_kind_tells_an_empty_box_from_a_draft`, `a_screen_without_the_box_or_an_unknown_kind_is_unknown`, and `menus_and_multi_line_drafts_count_as_typed`.
- `src/prompt_box.rs` — synthetic OMP tests `omp_default_empty_editor_is_readable`, `omp_default_hints_do_not_consume_adjacent_drafts`, `omp_default_preserves_styled_drafts_and_continuations`, and `omp_default_malformed_ansi_is_unknown` prove default-editor classification, bounded default/nerd hints, draft retention, busy/menu/missing-editor cases and narrow/wrapped/truncated ANSI negatives.
- `src/brief.rs` — tests `a_kind_whose_box_cannot_be_read_gets_one_try_then_goes_to_a_person` and `a_draft_in_the_box_or_an_unreadable_screen_after_a_try_waits`.
- `src/scenarios.rs` — tests `a_nudge_waits_while_the_coordinators_box_holds_a_draft_and_goes_out_once_after` and `a_brief_waits_while_a_trust_screen_shows_even_when_herdr_reads_idle`.
- `src/steps.rs` — test `the_nudge_line_names_subjects_and_events_only`.
- `src/scenarios.rs` — `omp_default_empty_cannot_advance_the_nudge_quiet_gate` exercises ticker discovery, the real quiet gate and a persisted old interval; `omp_default_empty_does_not_enable_automatic_pr_follow_up` proves routine refusal and preserved explicit follow-ups.
- `src/brief.rs` — `omp_parser_support_does_not_enable_automatic_brief_delivery` proves automatic first send, retry, Enter and transcript confirmation remain disabled.
- `src/trust_screen.rs` — `omp_parser_empty_does_not_dismiss_a_trust_screen` keeps a trust phrase actionable despite a synthetic empty OMP anchor.

Run `cargo test --locked`. Hermes fixtures and live nudge evidence do not exist
in this baseline.
