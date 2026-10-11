# CAP-002 — Agent launch presets and resume

**Status:** implemented
**Primary surface:** none

## Behaviour

- Every recognized Herdr agent kind has a built-in launch preset. User-owned named presets specify harness, model, supported effort and extra arguments; per-role allow-lists constrain selection.
- Built-ins are listed when an executable is found and the known harness-specific sign-in heuristic passes. Kinds without a known sign-in check, including Hermes, count as signed in once installed; this is not an authentication probe.
- Agents may request model changes through validated model arguments, not arbitrary privilege-expanding launch flags.
- Hermes is a recognized kind with `--resume <id>` arguments. Empty and flag-like session identifiers are refused by the resume-argument helper.
- Coordinator and worker launch/restart paths preserve the existing project/thread lifecycle and generic resume behaviour; a Herdr preset is reusable, not a leased persistent agent home.

## Implementation

- `src/agents.rs` — kind/resume tables and agent-requested model arguments.
- `src/profiles.rs` — user presets, defaults, allow-lists and detection.
- `src/coordinator.rs` — coordinator launch/discovery/resume.
- `src/threads.rs` — worker start/restart.
- `src/paths.rs` — environment and Herdr configuration resolution.

## Rules and boundaries

No typed Hermes-profile binding, concurrent-profile reuse guard or pairing of a
Hermes home with its session ID exists. Raw preset arguments do not establish
profile isolation. Herdr configuration currently follows HOME, including when a
caller changes that environment variable. Worktrees isolate checkout state, not
Hermes memory/skills/sessions. This CAP excludes the profile-editor popup UI.
See [architecture](../../architecture.md).

## Verification

- `src/agents.rs` — tests `resume_arguments_follow_herdrs_table` and `agent_args_carry_only_a_model_flag`; Hermes-specific resume is table-backed but not independently live-tested.
- `src/profiles.rs` — tests `profiles_resolve_to_their_harness_flags`, `allow_lists_refuse_other_profiles`, `built_ins_are_the_installed_and_signed_in_kinds`.
- `src/scenarios.rs` — tests `a_thread_launches_with_its_profile_and_fails_closed_once_disallowed`, `thread_start_and_open_refuse_profiles_off_the_allow_list`, and `open_in_a_pane_resumes_the_recorded_session_and_starts_fresh_when_that_fails`.
- `src/paths.rs` — test `root_order_flag_env_config_default`.

Run `cargo test --locked`; generic scenario evidence is not a Hermes live-session certification.
