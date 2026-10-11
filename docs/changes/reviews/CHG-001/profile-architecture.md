# Hermes profiles versus Herdr launch presets

**Status:** review-only

## Operator discussion and choices

The operator selected explicit bindings to operator-provisioned Hermes profiles
with reuse checks, and classic-CLI integration including safe nudges. Ink TUI
support and automatic profile provisioning are excluded.

The [operator guide](../../../hermes-agent.md) now owns the maintained setup
recipe, HERMES_HOME/HOME/cwd explanation, and Bot Mode/Kanban comparison. This
review remains rationale, not a competing setup manual or progress record.

The initial analysis correctly identified missing hooks and skill linking, but
installation-path awareness alone is not enough for concurrent workers. The
current Hermes profile documentation explicitly discourages separate independent
agents sharing one home because persistent memory writes can contaminate one
another. A fresh conversation or Git worktree is not a fresh Hermes identity.

| Object | Owns | Does not imply |
| --- | --- | --- |
| Herdr launch preset | Harness/model/effort/arguments and role allow-list selection | Exclusive persistent home |
| Hermes profile | Config, persona, memory, skills, plugin state and session storage | Worktree cwd, filesystem sandbox or independent OAuth login |
| Herdr project | Goal, deliberate shared project memory, threads and inbox | A Hermes profile |
| Worker worktree | Checkout and branch isolation | Isolated agent memory/session database |
| Native session | A conversation within a Hermes home | Safe concurrent independent processes on that home |

## Recommended ownership model

- Operator provisions and authenticates profile identities. Herdr references,
  validates and reserves them for managed use; it does not clone or delete them.
- A coordinator and each simultaneously active worker need distinct explicitly
  bound homes. Reusing one named launch preset that points at an occupied home
  must fail clearly, rather than silently allocate another identity. No pool
  scheduler is included in the first implementation.
- Resolve canonical homes and persist them with native session IDs. Preserve
  bindings on restart; a missing/renamed home is not permission to resume or
  silently start fresh in a different profile.
- Scope setup/doctor/removal to the selected target. HERMES_HOME from this coding
  session is not authorization to configure all project agents or the default
  profile. A sticky `hermes profile use` default must not affect managed launches.
- Reuse checks must cover managed coordinators/workers in other projects,
  sockets and data roots for the same account. State the limitation for external
  processes; no unsupported claim of machine-wide sandbox/exclusive locking.

## Home, cwd and shared state hazards

Hermes normally scopes state through HERMES_HOME. With `terminal.home_mode:
profile`, tool subprocesses instead see HOME under the profile; HERMES_REAL_HOME
identifies the account home. Herdr currently derives its config directory from
HOME. A hook launched by Hermes and a `report` launched by a terminal tool can
therefore consult different Herdr configuration even within the same pane.

Propagate explicit Herdr root/config identity to both paths and test it. Do not
just substitute HERMES_REAL_HOME everywhere: it is contextual data, and the
selected Herdr configuration must remain explicit and validated. A profile's
terminal.cwd/backend may also redirect shell tools away from the assigned
worktree; diagnose incompatibility rather than overwrite operator settings.

OAuth is not necessarily isolated: some provider refresh credentials are
shared via Hermes' root credential store. External memory providers can also
share banks across profiles. The integration must not read/copy credentials or
promise memory independence where the operator intentionally configured a
shared provider. Profile cloning is excluded precisely because it can carry
memory, plugin configuration, credentials and inherited provider-bank bindings.

## Hook architecture

Prefer a thin Python plugin adapter and a common Rust progress core. Install the
shared autoproject skill into the selected profile without changing its normal
invocation to a namespaced plugin-only command.

Use `pre_llm_call` for startup/per-turn context; it fires once per user turn.
`on_session_start` and `post_tool_call` are observers whose returns are ignored.
Use a supported transform path for in-turn reminders, preserve original tool
output, and verify composition with other plugins. Never copy a Claude JSONC
schema or shell redirection suffix into Hermes' direct subprocess commands.

Profile consent is separate from pane ownership. Hook checks must authorize a
managed pane and top-level session, not merely an inherited environment or a
matching profile. Native delegated children are not Herdr worker processes;
they must not reset/report the parent's progress. A turn ending is not proof
that the user's task is complete.

## Surface and verification boundaries

Classic CLI only. A Hermes profile can select Ink TUI, so interface detection
must not assume every `hermes` process uses one layout. Unknown remains safe.
The CAPs here describe headless configuration/protocol/automation behaviour,
not a new primary human-facing screen. If an interactive profile picker is
introduced, the implementation must add its own canonical visual handoff.

Required evidence is in CHG phase gates: isolated installation/removal,
concurrent bindings, failed launch/stale ownership, resume/reset/compaction,
managed/unmanaged/child hooks, virtual HOME/cwd, true mid-turn injection,
and real classic-CLI empty/draft/menu/busy captures plus a live nudge test.

## Sources

- [Hermes profiles](https://hermes-agent.nousresearch.com/docs/user-guide/profiles)
- [Profile commands](https://hermes-agent.nousresearch.com/docs/reference/profile-commands)
- [Hook contracts](https://hermes-agent.nousresearch.com/docs/user-guide/features/hooks)
- [Observer contract](https://hermes-agent.nousresearch.com/docs/developer-guide/observer-hooks)
- [Plugins and opt-in](https://hermes-agent.nousresearch.com/docs/user-guide/features/plugins)
- [Skill discovery](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills)

Documentation was retrieved during planning on 2026-10-11. Recheck the supported
runtime version during implementation; documentation is not a live parity test.
