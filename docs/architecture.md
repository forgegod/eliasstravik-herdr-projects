# Herdr Projects architecture

## Implemented system

Herdr Projects is a Rust CLI and Herdr plugin, not an agent runtime. `src/main.rs`
loads `src/cli.rs`; commands use `paths::Ctx`, injected `Env`, and `Runner`.
`herdr.rs` wraps bounded calls to the external Herdr CLI. The Herdr server owns
panes, workspaces, agent detection and terminal transport; this repository owns
project orchestration and files. See [operations](operations.md).

- **Project data:** `project.rs` owns project Markdown/TOML, priming and safety
  loading. Projects hold goals, tasks, shared memory, thread reports and inbox.
  Default data root is `~/.herdr-projects`; safety/profile settings are in
  `~/.config/herdr-projects/config.toml`, outside project working folders.
- **Launch and identity:** `profiles.rs` resolves named launch presets and
  allow-lists; `agents.rs` knows Herdr kinds/resume arguments. `coordinator.rs`,
  `threads.rs` and `thread.rs` create/discover agents and persist thread state.
  `remote.rs` resolves profiles on the target machine and quotes SSH commands.
- **Progress and setup:** `progress.rs` records per-pane self-reports. `setup.rs`
  installs supported harness hooks and links the bundled autoproject skill,
  journaling edits for reversible removal. `doctor.rs` checks/repairs owned data.
- **Automation:** `ticker.rs` drives `steps.rs`, routines, PR checks, reports,
  cleanup and notifications. `brief.rs` and `prompt_box.rs` protect prompt
  delivery against typed drafts, unknown layouts and trust screens.
- **Human surfaces:** `popup.rs`, `overview.rs`, `sidebar.rs` and `grouping.rs`
  expose projects through Herdr's terminal UI and metadata. The initial CAP
  inventory is narrower than this product and does not claim complete UI coverage.
- **Installation/update:** `update.rs` discovers installation source and origin
  release tags; `scripts/install.sh` verifies release downloads or builds from
  source. Same-version fork-main commits are not a separate update channel.
  [CHG-002](changes/active/CHG-002-fork-aware-updates.md) proposes that extension;
  it is not implemented by this documentation adoption.
- **Agent protocols:** coordinator and thread instructions are product assets;
  agents execute actual work. Herdr Projects does not infer successful task
  completion from elapsed time or tool counts.

## Current Hermes boundary

Hermes is an accepted generic kind and has native resume arguments. It is not a
configured hook harness or bundled-skill installation target. The classic CLI's
input box is not recognized, so automatic coordinator nudges fail safe.
Existing launch presets can carry arbitrary operator-owned arguments, but there
is no typed Hermes-home binding, profile lease, or profile-aware resume guard.
Current `Env` derives Herdr's configuration location from `HOME`; it does not
separate a Hermes tool subprocess's virtual HOME from the account home.

The accepted future direction is recorded in [design decisions](design-decisions.md)
and [CHG-001](changes/active/CHG-001-hermes-agent-integration.md). It is not shipped.
Do not infer shared-profile safety from worktree isolation or generic launch support.

The [Hermes operator guide](hermes-agent.md) owns the setup/support matrix,
home-selection examples and lifecycle walkthrough. Bot Mode is Hermes' profile
roster/chat surface; Kanban worker lanes are executors under a distinct task/run
dispatcher. Neither is a Herdr integration bridge. Keep independent Bot Chat,
cron and Kanban writers off Herdr-reserved profile homes. Herdr's planned
managed-use reservation cannot fence external schedulers.

Kanban descendant task/claim scope is not conveyed merely by selecting a profile.
Unsupported nested dispatch must be diagnosed rather than removing lineage
metadata. The current scope does not mirror boards, complete cards, synchronize
Bot Chats or replace Herdr's coordinator with a terminal-disabled Kanban
orchestrator profile. A future bridge needs an explicit lifecycle/authority design.

## Safety and ownership

Safety settings and protocol text are soft controls, not a sandbox. Agents can
execute shell commands. Preserve allow-lists, permission prompts, explicit human
approval and bounded pane identity checks; see [operations](operations.md#what-the-safety-settings-do-and-dont-stop).
Global hooks currently act in matching Herdr panes even outside managed projects;
managed-project scoping is proposed work, not an existing guarantee.

Herdr project memory is deliberate shared task context. A Hermes profile's
persistent memory/skills/sessions are a separate state boundary. External memory
provider banks and some OAuth credentials may be shared across Hermes profiles;
profiles neither guarantee independent authentication nor sandbox filesystem access.

## Maintenance architecture

Root/child AGENTS.md files are the DOX hierarchy. CAPs record current behaviour;
active CHGs alone track material implementation progress. The dependency-free
Node validator and its synthetic tests are development/CI tools only. They do
not alter the Cargo runtime, installation artifacts or release mechanism.

## Non-goals

- No replacement Herdr server, terminal multiplexer, Hermes runtime or credential manager.
- No silent credential cloning, global hook auto-accept, or automated profile deletion.
- No new hosted service/database or second implementation-progress tracker.
- No implied support for Hermes gateway/Desktop/Ink TUI/remote sandboxes from a
  local classic-CLI integration; those require separately verified scope.

## Verification

[Development](development.md) defines the credential-free gate. Code and tests
remain decisive; CAP structure and wireframe inventories cannot prove runtime
behaviour, visual freshness, or live third-party compatibility.
