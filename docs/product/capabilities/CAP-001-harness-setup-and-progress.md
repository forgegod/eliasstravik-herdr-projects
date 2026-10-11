# CAP-001 — Harness setup and progress reporting

**Status:** implemented
**Primary surface:** none

## Behaviour

- `configure` installs hooks for Claude, Codex, Droid, Gemini and Copilot, selecting installed configuration directories unless clients are explicitly named. Hermes is not accepted by configure/hook's client parser.
- The bundled autoproject skill is linked for Claude and Codex only. Foreign skills are left alone; owned links can be repaired or removed.
- Hook/configuration edits are journaled, idempotent, dry-runnable and reversible. Unconfigure preserves unrelated later edits; symbolic-link configuration files are refused.
- Any shell-capable agent can run `report` in its Herdr pane. Records identify the socket, pane and terminal; activities are bounded and sanitized. A 100-percent report becomes Done.
- Supported hook events inject initial guidance, clear an answered waiting state and throttle tool-boundary reminders. Recognized subagent/foreign-session events and the reporter's own command are excluded. Outside Herdr, hooks are silent.

## Implementation

- `src/setup.rs` — harness registry, configuration ownership and skill linking.
- `src/progress.rs` — report records, event eligibility and context responses.
- `src/doctor.rs` — owned configuration and skill diagnostics/repair.
- `src/cli.rs` — accepted client values and CLI entry points.

## Rules and boundaries

This is a configuration and machine/agent protocol contract, not the sidebar's
visual layout. Current hook eligibility is not restricted to managed project
panes. There is no Hermes installation adapter or profile-aware ownership scope.
Manual guidance in briefs is not proof of automatic Hermes hook delivery.
See [architecture](../../architecture.md).

## Verification

- `src/setup.rs` — tests `existing_hooks_comments_and_user_edits_survive`, `configure_and_unconfigure_round_trip_byte_for_byte_and_keep_user_additions`, and `the_skill_is_linked_once_into_a_shared_skills_dir_and_foreign_ones_are_left_alone`.
- `src/progress.rs` — tests `helpers_and_the_reporters_own_call_do_not_trigger`, `reminders_are_throttled_to_once_a_minute_and_stop_at_done`, `native_events_are_normalized_and_answered_in_each_harness_shape`, and `records_round_trip_per_session_and_stale_terminals_are_ignored`.
- `src/doctor.rs` — test `fix_links_the_skill_only_for_a_configured_harness_and_never_over_a_foreign_one`.

Run `cargo test --locked`. Existing tests passed on the adoption baseline; no
live Hermes integration is claimed.
