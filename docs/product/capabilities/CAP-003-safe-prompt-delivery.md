# CAP-003 — Safe automated prompt delivery

**Status:** implemented
**Primary surface:** none

## Behaviour

- The terminal parser distinguishes empty input from a draft for Claude, Codex, Cursor, Gemini, OpenCode and Pi using per-kind layouts and ANSI styling. Unsupported kinds, including Hermes and OMP, return Unknown.
- Automatic coordinator nudges require an input box observed empty over the quiet interval; drafts and unknown layouts reset the wait. A missing Hermes parser therefore suppresses automatic Hermes coordinator nudges.
- Brief delivery checks agent readiness, trust screens and prompt confirmation state. A known draft is not overwritten; unsupported input layouts take the conservative fallback rather than assuming empty.
- Nudge text summarizes subjects/events without copying message bodies; inbox-set deduplication avoids replaying the same acknowledged nudge set.

## Implementation

- `src/prompt_box.rs` — ANSI cells, per-kind input parsing and empty/draft/unknown classification.
- `src/brief.rs` — brief delivery and confirmation.
- `src/trust_screen.rs` — trust-screen guards.
- `src/steps.rs` — coordinator nudge quiet/dedup gates.

## Rules and boundaries

This CAP owns the automatic delivery protocol and input protection, not the
external agent's UI or Herdr Projects' popup/sidebar design. Unknown must not be
made empty as a compatibility shortcut. Input fixtures must distinguish real
captures from synthetic examples. See [architecture](../../architecture.md).

## Verification

- `src/prompt_box.rs` — tests `every_supported_kind_tells_an_empty_box_from_a_draft`, `a_screen_without_the_box_or_an_unknown_kind_is_unknown`, and `menus_and_multi_line_drafts_count_as_typed`.
- `src/brief.rs` — tests `a_kind_whose_box_cannot_be_read_gets_one_try_then_goes_to_a_person` and `a_draft_in_the_box_or_an_unreadable_screen_after_a_try_waits`.
- `src/scenarios.rs` — tests `a_nudge_waits_while_the_coordinators_box_holds_a_draft_and_goes_out_once_after` and `a_brief_waits_while_a_trust_screen_shows_even_when_herdr_reads_idle`.
- `src/steps.rs` — test `the_nudge_line_names_subjects_and_events_only`.

Run `cargo test --locked`. Hermes fixtures and live nudge evidence do not exist
in this baseline.
