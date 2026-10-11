# Hermes Agent with Herdr Projects

**Integration status: planned, not yet available in full.** This guide explains
what works at baseline `4e4548c` / 0.2.34, how operators can prepare Hermes
profiles today, and the acceptance boundary for
[CHG-001](changes/active/CHG-001-hermes-agent-integration.md).
Profile preparation does not install the missing Herdr adapter or enable safe
automatic nudges. Do not interpret future acceptance steps as supported CLI flags.

## 1. What is available now

| Capability | Current repository | Planned integration |
| --- | --- | --- |
| Recognize Hermes / build native resume arguments | Yes, through generic agent-kind support | Explicit home/session binding and validation |
| Select Hermes with a Herdr launch preset | Generic preset arguments only | Typed, operator-owned profile binding with managed-use checks |
| Read coordinator AGENTS.md / worker brief | Existing generic instruction path; verify actual launch cwd | Verified profile/cwd/bootstrap matrix |
| Run a progress report from a shell tool | Generic `report` command; depends on correct pane/environment | Verified hook-to-shell identity and record agreement |
| Install Hermes progress adapter and autoproject skill | No | Explicit selected-profile setup, consent, doctor and reversible removal |
| Automatically nudge a Hermes coordinator | No: input-box parser returns Unknown | Captured and tested classic-CLI layouts only |
| Bot Mode or Kanban integration | No bridge | Comparison and coexistence guidance only; a bridge is out of scope |

Current contracts: [harness setup](product/capabilities/CAP-001-harness-setup-and-progress.md),
[launch/resume](product/capabilities/CAP-002-agent-launch-and-resume.md),
[prompt delivery](product/capabilities/CAP-003-safe-prompt-delivery.md).
There is no supported `configure --clients hermes` path at this baseline.

## 2. How the pieces fit

```text
Operator
  owns Hermes provisioning/authentication and Herdr safety settings
    |
Herdr project + coordinator
  owns goal, shared project context, thread assignments and review workflow
    |
Herdr launch preset                    Hermes persistent profile
  harness/model/args/allow-list   --->   selected home/config/skills/memory
    |                                         |
Herdr pane + worker worktree   <---     Hermes CLI conversation
    |                                         |
Herdr thread/report/inbox      <---     shell commands + planned hook adapter
    |
Herdr ticker, sidebar and coordinator nudges
```

The arrows involving explicit profile binding and a Hermes hook adapter are
planned. The runtime remains a Rust CLI talking to Herdr; Hermes executes agent
work. The adapter does not become a second task database or scheduler.

Keep these identities separate:

- **Herdr project:** goal, TASKS.md, project memory, worker records, reports and inbox.
- **Herdr launch preset:** an allowed harness/model/argument recipe. Selecting a
  preset named `hermes` does not select a particular Hermes persistent profile.
- **Hermes profile:** long-lived configuration, persona, skills, memory and
  session state. Two profiles may use the same model but have different roles.
- **Native session:** one conversation in a profile; resuming an ID requires the
  correct profile home. A profile is not a single conversation.
- **Working directory/worktree:** where the task executes and which branch it
  edits. This is not the agent's state home.
- **Machine:** the same profile name on two hosts does not identify the same
  agent. The first integration supports local classic CLI, not remote routing.

### Profile allocation

Use a dedicated coordinator profile and distinct profiles for simultaneously
active independent workers, for example `hp-coordinator`, `hp-worker-01`, and
`hp-worker-02`. These names are examples, not reserved names or a built-in pool.
The operator provisions them; Herdr will bind/reserve them, not clone, allocate
or delete profiles automatically. An occupied binding must fail clearly.

Hermes' profile guide warns against independent agent processes sharing a
home. Different session IDs and worktrees do not isolate automatic memory/skill
writes. The proposed Herdr reservation covers managed use across its projects
and sockets; it is not a machine-wide lock recognized by Desktop, gateways,
Kanban or unrelated shell sessions. Keep those writers off Herdr-reserved homes.
A persistent worker reused sequentially retains its memory; choose dedicated
project identities when that context must not carry into another project.

## 3. HERMES_HOME, HOME and cwd

| Setting | Meaning | Operational rule |
| --- | --- | --- |
| `HERMES_HOME` | Active Hermes configuration/data home | Target a known profile explicitly; do not use the worktree as the home |
| `HOME` | OS-user home used by Git/SSH/gh and Herdr's current configuration lookup | Do not replace it merely to select a Hermes profile |
| `HERMES_REAL_HOME` | Hermes-provided account-home context when tool HOME is isolated | Diagnostic context, not an invitation to override identity/authorization |
| `terminal.cwd` | Hermes tool working directory | For Herdr-managed worktrees, use local launch cwd (`.`), not a fixed unrelated repository |
| Herdr root / config / socket / pane identity | Herdr project data, policy and target pane | Must agree between hook subprocesses and agent-issued shell commands |

On a normal POSIX installation, the default Hermes home is `~/.hermes` and a
named home is `~/.hermes/profiles/<name>`. Custom roots and container/Windows
layouts differ. Use profile/config-path inspection rather than deriving paths
from names alone. Symlink aliases must resolve to one identity for reuse checks.

### Select explicitly, then verify

Prefer `hermes --profile <name> ...` for operator commands. Hermes pre-parses
that selector and resolves the home before importing most runtime modules.
Do not rely on the sticky `hermes profile use` selection for managed launches.
Do not globally export one worker's HERMES_HOME in the shell that starts all
Herdr panes: unrelated children can inherit the same identity.

For automation, pin the intended root and profile per invocation and inspect the
resolved config path. HERMES_HOME alone is not a universal “ignore everything
else” switch: current Hermes CLI startup can still consult the sticky selection
when HERMES_HOME points at a root rather than a named `profiles/<name>` home.
Runtime/profile context can also override a process home in multi-profile hosts.
The integration must validate the resolved home, not just trust its requested argv.

Set HERMES_HOME before starting the process. Do not write it into config.yaml,
a profile `.env`, the repository `.env`, or change it mid-conversation to switch
identities. Start a new explicitly targeted process instead. Preserve the
profile/home/native-session tuple on resume, including after profile renames.

### Recommended initial posture

Use the **local** terminal backend and normal host HOME. If you intentionally
use `terminal.home_mode: profile`, Hermes tools receive
`HOME=<profile-home>/home`; Git/SSH/gh then need their own operator-configured
state there. Today Herdr reads its configuration under HOME, so this mode can
make a hook and a shell `report` consult different settings. Full support must
pass CHG-001's explicit-context tests; do not repair it by copying credentials,
weakening permissions or blindly replacing HOME.

Profiles are not filesystem sandboxes. Shared external memory banks and provider
authentication are separate policies. Check your Hermes version/provider's
credential behavior; do not assume every profile has an independent login or
copy OAuth token files. Configure authentication through Hermes' supported UI/CLI.

## 4. Prepare dedicated Hermes profiles today

These are **operator-run Hermes commands**, not an installation of the planned
Herdr integration. Run them from an ordinary OS terminal with your real account
HOME, outside an assigned Kanban/delegated worker. Do not reuse an active personal
profile. Check for existing names before creation; use different names if needed.
See the official [installation guide](https://hermes-agent.nousresearch.com/docs/getting-started/installation)
if Hermes is not installed.

### Inspect the intended root and create identities

```sh
# Shell variable used by these examples, not a Hermes configuration key.
hermes_root="$HOME/.hermes"
HERMES_HOME="$hermes_root" hermes --profile default profile list
HERMES_HOME="$hermes_root" hermes --profile default profile create hp-coordinator
HERMES_HOME="$hermes_root" hermes --profile default profile create hp-worker-01
HERMES_HOME="$hermes_root" hermes --profile default profile create hp-worker-02
```

For an intentionally separate installation root, replace `hermes_root` with an
operator-owned absolute directory and use it consistently on **every** command.
The example uses named profiles plus an explicit root to avoid accidentally
provisioning beneath an inherited personal profile or a different installation.
Do not create additional profiles just to evade an existing task/lineage fence.

Fresh profiles are the simplest starting point. `--clone` can carry persona,
skills, memory and external memory-provider configuration; a new directory may
still point at the same memory bank. Do not treat clone as “empty independent
worker.” Profile distributions are an optional Hermes provisioning mechanism,
not a feature Herdr must implement. Review shared plugins, skills and memory
policies before adopting a distribution.

### Configure each profile deliberately

```sh
# Repeat with each intended worker profile; no authentication material goes in docs.
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator setup
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator config set terminal.backend local
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator config set terminal.cwd .
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator config set display.interface cli
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator config path
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator config get terminal.cwd
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator config get terminal.home_mode
HERMES_HOME="$hermes_root" hermes --profile hp-coordinator doctor
```

An unset home-mode value may mean the runtime default; check that version's
behavior rather than inventing a value. Confirm config path identifies the
coordinator, not the default/another profile. Run setup/model selection for
workers too. Use `config set` rather than hand-rewriting YAML. Do not share
configuration dumps containing secrets when diagnosing paths.

Choose tools for the role:

- The Herdr coordinator must be able to call the Herdr Projects CLI, read its
  digest and maintain project coordination files. It is instructed not to do
  worker implementation itself, but it still needs shell access to the product.
- Workers need the tools/authentication appropriate to their assigned repository
  and checks. Keep ordinary approval prompts and Herdr role allow-lists.
- The coordinator needs discovery of the bundled autoproject skill when using
  that workflow. The intended adapter/skill enablement must be verified in each
  selected profile; discovery in your personal profile proves nothing about workers.
- Do not add `--yolo`, global hook auto-accept, `--safe-mode` or `--ignore-rules`
  as integration shortcuts. The last two can disable the very plugins/context
  this workflow needs. Do not add Hermes `--worktree` when Herdr already owns
  worker worktree creation; nested worktrees change the expected task location.

A standalone profile smoke test can start classic CLI from a scratch workspace:

```sh
# Run from the intended scratch cwd, not from the profile directory.
HERMES_HOME="$hermes_root" hermes --profile hp-worker-01 --cli
```

Ask it to run a harmless `pwd` and report only the non-secret selected profile
home; inspect actual tool output, not only the model's answer. End that process
before allocating the profile to Herdr. This validates Hermes preparation only,
not safe Herdr launch/resume, profile locking or automatic progress/nudges.

## 5. Completion checklist for the planned integration

CHG-001 must replace this checklist with tested, exact Herdr setup commands when
the CLI/schema exists. Do not publish guessed `--hermes-home`/profile-binding
flags or pretend copying a skill enables hooks.

1. Record supported Herdr/Hermes/Python versions and how to force classic CLI.
2. Create allowed Herdr presets bound to the already-provisioned coordinator and
   worker homes; show the real schema and validated selection precedence.
3. Preview configuration for each selected profile, install only owned adapter
   and skill artifacts, and complete that profile's explicit consent/enablement.
4. Verify doctor reports the selected home, bound preset, adapter version,
   enablement, skill discovery and any unsupported HOME/cwd/backend conditions.
5. Open the coordinator and two independent workers: verify different homes,
   intended worktrees, correct shared project context and separate session state.
6. Prove an actual agent shell `report` updates the same pane record as the hook.
   Wait for a real due reminder; a successful callback registration is not proof
   the model received context. Unmanaged panes and delegated children stay silent.
7. Verify a quiet coordinator receives a nudge, drafts/menus are preserved, and
   Ink TUI/unknown layouts remain unsupported rather than being treated as empty.
8. Verify same-home conflict refusal, recovery after a failed launch, and resume
   in the original home. Hermes can restore a resumed session's recorded cwd;
   test the supported `--in`/`--no-restore-cwd` behavior rather than let resume
   quietly move a worker away from its assigned worktree.
9. Update/relink after moving the checkout; test uninstall/repair after user
   edits. Removing integration must not remove Hermes profiles/history/memory.
10. Confirm other dispatchers cannot silently reuse these profiles under the
    documented operating posture. A Herdr lease is not a cross-product lock.

## 6. Comparison with Hermes Bot Mode

[Bot Mode](https://hermes-agent.nousresearch.com/docs/user-guide/bot-mode) is a
Desktop roster over Hermes profiles, with a canonical persistent Bot Chat,
profile personas/models/skills, bot/group messaging and scheduled routines.
It is not a separate kind of credential/home, and it is not Herdr Projects.
A plain CLI profile need not be a Bot; opening `hermes --profile NAME` selects
the identity but does not by itself prove it resumed the canonical Bot Chat.

| Concern | Herdr Projects + planned Hermes adapter | Hermes Bot Mode |
| --- | --- | --- |
| Main operator surface | Herdr terminal panes, project sidebar and coordinator | Desktop bot roster, Bot Chat and groups |
| Coordination unit | Project and worker thread/worktree | Persistent profile presented as a Bot; per-room conversations |
| Shared task context | Project brief, memory, task/report files and inbox | Bot/room conversation and configured profile memory |
| Recurring work | Herdr project routines/ticker | Hermes cron-backed Bot routines |
| Completion | Herdr report/review/resolve workflow | Chat/routine outcomes; not a Herdr thread resolution |
| Parallel capacity | Herdr settings plus available explicit profile bindings | Desktop warm-backend pool and its own dispatch behavior |
| Profile visibility | Binding/diagnostics planned; no roster synchronization | Profile-backed roster, including source-qualified remote Bots |

You can configure a dedicated profile through Hermes' management surfaces while
it is idle. Do not also drive its Bot Chat, group turns, routines or gateway
work while an independent Herdr CLI worker owns it. **Hiding a Bot or turning off
the Bot Mode UI does not stop its routines or remove its profile.** Allocate
separate identities or explicitly quiesce the external writer.

There is no Bot Chat/Herdr inbox synchronization, bot-DM-to-thread bridge or
Desktop backend lease coordination in CHG-001. To preserve the current design,
use Bot Mode for a separate personal/research fleet, and Herdr for the project
fleet. A future shared roster can be designed separately with host-qualified
identity and explicit run ownership; sharing HERMES_HOME is not that design.

## 7. Comparison with Hermes Kanban worker lanes

[Kanban worker lanes](https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban-worker-lanes)
are executors dispatched for assigned cards. The **Kanban kernel** owns task/run
lifecycle and audit truth; an executor does not own that lifecycle just because
its process exited. A Hermes profile lane normally runs
`hermes -p <assignee> chat -q <prompt>` in a pinned workspace.

| Concern | Herdr Projects | Hermes Kanban profile lane |
| --- | --- | --- |
| Dispatch owner | Herdr coordinator/ticker thread lifecycle | Kanban dispatcher/kernel |
| Work identity | Herdr project/thread + pane/socket + native session | Board/task/run + claim lock + assignee profile |
| Workspace | Herdr-created/selected worker directory/worktree | Dispatcher-pinned task workspace |
| Normal executor | Interactive agent in a Herdr pane | Assigned one-shot/profile worker |
| Run handoff | Thread report, next steps and Herdr review/resolve | `kanban_complete`, `kanban_request_review` or `kanban_block` |
| Failure handling | Herdr launch/observation/brief policies | Claims, heartbeats, crash/reclaim/runtime limits and run IDs |
| Progress hook | Advisory self-report; not task lifecycle authority | Must not replace a terminating lifecycle tool call |
| External CLI support | Herdr runs multiple agent kinds | A non-Hermes lane needs a designed spawn/lifecycle adapter; not a paved automatic bridge |

A Kanban worker exiting without the required lifecycle handoff can be classified
as failed even if it wrote code. Conversely, a 100-percent Herdr report does not
complete a Kanban card. Kanban same-card review and downstream review-card
patterns also have different termination rules; do not substitute the
`/autoproject` review protocol for either.

Do not copy the Kanban **orchestrator profile** tool policy verbatim: that role
can exclude implementation tools including terminal, but the Herdr coordinator
needs terminal access to call `herdr-projects`. Similar role names are not
interchangeable configurations.

### One owner per piece of work

For the planned integration, **Herdr owns dispatch of its project threads**.
Do not concurrently assign those same profiles/tasks/worktrees to Kanban. Use
different profiles for separate Kanban work. Herdr routines and Hermes cron are
also separate schedulers; creating the same job in both duplicates work.

If running multiple Hermes gateways for a separate Kanban fleet, follow the
[one-dispatcher guidance](https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban-multi-gateway):
only the dispatch-owning gateway enables `kanban.dispatch_in_gateway`; others
disable it. That setting only controls gateway Kanban dispatch. It is not a
Herdr lock, a stop for cron/Bot Chat, or a universal ban on CLI dispatch.

### Lineage and environment boundaries

A Kanban worker carries task, run, claim, board/database/workspace and profile
identity (`HERMES_KANBAN_TASK`, `HERMES_KANBAN_RUN_ID`,
`HERMES_KANBAN_CLAIM_LOCK`, `HERMES_KANBAN_DB`, `HERMES_KANBAN_BOARD`,
`HERMES_KANBAN_WORKSPACE`, `HERMES_KANBAN_WORKSPACES_ROOT`, `HERMES_PROFILE`,
and sometimes `HERMES_TENANT`). Shell tools, hooks and delegated descendants
are not automatically owners of that assigned run. Hermes carries cooperative
non-owner lineage fencing into descendant processes; profile/home changes are
not authorization to terminate or take over the parent's task.

CHG-001 must diagnose/refuse unsupported nested-dispatch contexts rather than
strip task/claim/fence metadata to make a Herdr launch appear unrestricted.
Do not set those variables manually to impersonate a worker. A future bridge
would need the documented spawn/descendant-scope contract, ownership of each
run, review handoff, workspace mapping, cancellation and idempotent recovery.
It would require separate approval and tests, not another profile setting.

### Repository records are not a competing queue

CAPs describe implemented product behavior; CHGs describe this repository's
material-change progress and verification gates. Kanban, if separately used,
can own an assigned execution run without replacing those contracts. A future
bridge must reference the owning CHG/phase and define how evidence returns;
`kanban_complete` must not automatically archive a CHG or bypass its review/gates.
No such bridge or synchronization is implemented here.

## 8. Troubleshooting

| Symptom | Check / action |
| --- | --- |
| Wrong persona, skills or history | Inspect explicit root/profile with `config path`; check inherited HERMES_HOME and sticky selection before starting another process |
| Tools run in another repo | Check `terminal.cwd`, local backend, launch directory and resumed-cwd behavior; do not change HERMES_HOME to the repo |
| Hook sees the pane but shell report does not | Compare non-secret Herdr root/config/socket/pane and HOME context; a sandbox may not reach the Herdr socket |
| No autoproject / no reminders | Baseline lacks the Hermes installer; after implementation check the selected profile's skill discovery and plugin enablement/consent |
| Coordinator never receives nudges | Baseline parser is unsupported; after implementation confirm classic CLI, known layout, quiet empty input and project nudge policy |
| Same identity writes conflicting memory | Stop conflicting writers; allocate separate profile homes, not just sessions/worktrees; review shared external memory settings |
| Kanban card is crashed/blocked after a Herdr task | These lifecycles are not bridged; use the original dispatcher/handoff contract, never manufacture completion from a progress percentage |
| Profile removed/renamed or checkout moved | Do not resume in a fallback identity; verify binding/session provenance and repair only owned integration artifacts |
| Bot hidden but work still starts | Hide is display-only; inspect Bot routines, cron, gateway and dispatch activity independently |

Never paste `.env`, credential files, complete environment dumps or unredacted
logs into a bug report. Share versions, selected paths, relevant non-secret
settings and a minimal sanitized reproduction instead.

## Sources and verification status

Compared with official Hermes documentation retrieved on 2026-10-11:

- [Profiles](https://hermes-agent.nousresearch.com/docs/user-guide/profiles)
- [Bot Mode](https://hermes-agent.nousresearch.com/docs/user-guide/bot-mode)
- [Kanban worker lanes](https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban-worker-lanes)
- [Kanban multi-gateway](https://hermes-agent.nousresearch.com/docs/user-guide/features/kanban-multi-gateway)
- [Environment variables](https://hermes-agent.nousresearch.com/docs/reference/environment-variables)
- [Hooks](https://hermes-agent.nousresearch.com/docs/user-guide/features/hooks)
- [Plugins](https://hermes-agent.nousresearch.com/docs/user-guide/features/plugins)
- [CLI reference](https://hermes-agent.nousresearch.com/docs/reference/cli-commands)
- [CLI profile pre-parsing source](https://github.com/NousResearch/hermes-agent/blob/main/hermes_cli/main.py)
- [Home/root resolution source](https://github.com/NousResearch/hermes-agent/blob/main/hermes_constants.py)

Profile-selection caveats were also checked against upstream CLI startup and
`get_default_hermes_root` source. Local CLI help confirmed the documented
profile-create/config operations and `--cli`, `--in`, `--no-restore-cwd` flags.
Preparation examples were not executed against real user profiles. A live
compatibility matrix and clean-profile walkthrough remain CHG-001 completion
gates; documentation and command-help inspection are not proof of full integration.
