//! `update`: bring the installed plugin to the newest release, and the cheap
//! "a newer version is available" check `doctor` runs.
//!
//! A release is a `vX.Y.Z` tag on the plugin's `origin`. Two install types:
//! - `herdr plugin install`: Herdr's own managed clone. Re-running the install
//!   runs the manifest's build step in a temporary checkout and swaps it in only
//!   when that passes, at the same plugin root, so the old binary keeps working
//!   on a failure.
//! - `herdr plugin link` to a git checkout: `git pull --ff-only` on `main`, then
//!   the same build step, `scripts/install.sh`. It replaces the binary only when
//!   the download or the build succeeds.
//!
//! The build step downloads the release's prebuilt binary and falls back to
//! `cargo build --release --locked`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::herdr::{self, Herdr, Version};
use crate::paths::{self, Ctx, SessionFlags};
use crate::runner::{Cmd, Runner};

const PLUGIN_ID: &str = "herdr-projects";
const GIT_TIMEOUT: Duration = Duration::from_secs(30);
/// `doctor` must stay fast and quiet when offline.
const CHECK_TIMEOUT: Duration = Duration::from_secs(3);
const BUILD_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const STEP_TIMEOUT: Duration = Duration::from_secs(120);

/// Strict identifier syntax; do not repair a URL/path into an approved source.
pub fn repository_arg(value: &str) -> std::result::Result<String, String> {
    let parts: Vec<_> = value.split('/').collect();
    let component = |s: &str| !s.is_empty() && s != "." && s != ".."
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b));
    if parts.len() < 2 || !parts.iter().all(|s| component(s))
        || parts[0].len() > 39 || parts[0].starts_with('-') || parts[0].ends_with('-')
        || !parts[0].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || parts[1].len() > 100 || parts[1].ends_with(".git")
    {
        return Err("repository must be a canonical GitHub owner/repo[/subdir], not a URL or path".into());
    }
    Ok(value.to_string())
}

pub fn branch_ref_arg(value: &str) -> std::result::Result<String, String> {
    let branch = value.strip_prefix("refs/heads/").unwrap_or(value);
    if branch.is_empty() || branch == "HEAD" || branch.starts_with('-')
        || branch.starts_with("refs/") || branch.contains("..")
        || !branch.bytes().all(|b| b.is_ascii_alphanumeric() || b"-._/".contains(&b))
        || branch.split('/').any(|s| s.is_empty() || s.starts_with('.') || s.ends_with('.') || s.ends_with(".lock"))
    {
        return Err("ref must be an unambiguous branch name or refs/heads/name".into());
    }
    Ok(format!("refs/heads/{branch}"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Install {
    /// Installed with `herdr plugin install OWNER/REPO`.
    Github { root: PathBuf, repo: String },
    /// Linked with `herdr plugin link PATH`.
    Linked { root: PathBuf },
}

impl Install {
    pub fn root(&self) -> &Path {
        match self {
            Install::Github { root, .. } | Install::Linked { root } => root,
        }
    }
}

/// Reads `herdr plugin list --plugin herdr-projects --json`.
pub fn parse_install(json: &str) -> Result<Install> {
    #[derive(Deserialize)]
    struct Reply {
        result: Plugins,
    }
    #[derive(Deserialize)]
    struct Plugins {
        plugins: Vec<Plugin>,
    }
    #[derive(Deserialize)]
    struct Plugin {
        plugin_id: String,
        plugin_root: PathBuf,
        source: Source,
    }
    #[derive(Deserialize)]
    struct Source {
        kind: String,
        owner: Option<String>,
        repo: Option<String>,
        subdir: Option<String>,
    }
    let reply: Reply = serde_json::from_str(json).context("`herdr plugin list --json` output changed")?;
    let Some(plugin) = reply.result.plugins.into_iter().find(|p| p.plugin_id == PLUGIN_ID) else {
        bail!("Herdr has no plugin `{PLUGIN_ID}` installed");
    };
    let root = plugin.plugin_root;
    match plugin.source.kind.as_str() {
        "local" => Ok(Install::Linked { root }),
        "github" => {
            let (Some(owner), Some(repo)) = (plugin.source.owner, plugin.source.repo) else {
                bail!("Herdr does not say which GitHub repository `{PLUGIN_ID}` came from");
            };
            let mut repo = format!("{owner}/{repo}");
            if let Some(subdir) = plugin.source.subdir.filter(|s| !s.is_empty()) {
                repo = format!("{repo}/{subdir}");
            }
            Ok(Install::Github { root, repo })
        }
        other => bail!("`{PLUGIN_ID}` is installed from a `{other}` source, which `update` does not know"),
    }
}

/// `v0.2.3` → 0.2.3; anything else (`v0.3.0-rc1`, `nightly`) is not a release.
pub fn parse_release(tag: &str) -> Option<Version> {
    let core = tag.strip_prefix('v')?;
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    herdr::parse_version(core)
}

/// The highest release in `git ls-remote --tags --refs` output.
pub fn newest_release(ls_remote: &str) -> Option<Version> {
    ls_remote
        .lines()
        .filter_map(|line| line.split('\t').nth(1)?.strip_prefix("refs/tags/"))
        .filter_map(parse_release)
        .max()
}

/// This binary's own release.
pub fn own_version() -> Version {
    herdr::parse_version(env!("CARGO_PKG_VERSION")).expect("Cargo.toml carries an X.Y.Z version")
}

/// The plugin root this binary was built in (`<root>/target/release/herdr-projects`).
pub fn own_root() -> Option<PathBuf> {
    let binary = paths::binary().ok()?;
    Some(binary.parent()?.parent()?.parent()?.to_path_buf())
}

fn git(root: &Path, timeout: Duration) -> Cmd {
    Cmd::new("git", timeout).arg("-C").arg(root.to_string_lossy()).env("GIT_TERMINAL_PROMPT", "0")
}

/// The newest release on the checkout's `origin`; `Ok(None)` when it has no release tags.
fn latest_release(runner: &dyn Runner, root: &Path, timeout: Duration) -> Result<Option<Version>> {
    let out = runner.run(&git(root, timeout).args(["ls-remote", "--tags", "--refs", "origin"]))?;
    if !out.success() {
        bail!("could not list releases on origin: {}", out.error_text());
    }
    Ok(newest_release(&out.stdout))
}

/// For `doctor`: the newer release, when there is one. Never fails: offline,
/// no git, or a binary outside a checkout all mean "nothing to say".
pub fn newer_release(runner: &dyn Runner, root: Option<&Path>) -> Option<Version> {
    let root = root?;
    if !root.join(".git").exists() {
        return None;
    }
    let latest = latest_release(runner, root, CHECK_TIMEOUT).ok()??;
    (latest > own_version()).then_some(latest)
}

fn binary_in(root: &Path) -> PathBuf {
    root.join("target/release/herdr-projects")
}

/// The release of the binary at `binary`, from its `--version`.
fn binary_version(runner: &dyn Runner, binary: &Path) -> Option<Version> {
    let out = runner.run(&Cmd::new(binary.to_string_lossy(), herdr::CALL_TIMEOUT).arg("--version")).ok()?;
    out.success().then(|| herdr::parse_version(&out.stdout)).flatten()
}

/// Why a linked checkout cannot be pulled, or `None` when it can.
fn linked_blocker(runner: &dyn Runner, root: &Path) -> Result<Option<String>> {
    let branch = runner.run(&git(root, GIT_TIMEOUT).args(["rev-parse", "--abbrev-ref", "HEAD"]))?;
    if !branch.success() {
        bail!("{} is not a git checkout: {}", root.display(), branch.error_text());
    }
    let branch = branch.stdout.trim();
    if branch != "main" {
        return Ok(Some(format!(
            "{} is on `{branch}`, not `main`; switch it to main (`git -C {} switch main`) and run update again",
            root.display(),
            root.display()
        )));
    }
    let status = runner.run(&git(root, GIT_TIMEOUT).args(["status", "--porcelain", "--untracked-files=no"]))?;
    if !status.success() {
        bail!("git status failed in {}: {}", root.display(), status.error_text());
    }
    if !status.stdout.trim().is_empty() {
        return Ok(Some(format!(
            "{} has uncommitted changes; commit or stash them and run update again",
            root.display()
        )));
    }
    Ok(None)
}

/// The last lines of a failed step's output.
fn tail(text: &str) -> String {
    let lines: Vec<&str> = text.trim().lines().collect();
    lines[lines.len().saturating_sub(15)..].join("\n")
}

/// Fetches and builds the release. On `Err` the old binary is still in place.
fn fetch_and_build(ctx: &Ctx, herdr: &Herdr, install: &Install, latest: Version) -> Result<()> {
    match install {
        Install::Github { repo, .. } => {
            println!("installing {repo} v{latest} with Herdr (it downloads the prebuilt binary, or builds it when there is none)…");
            let tag = format!("v{latest}");
            let out = ctx.runner.run(&herdr.cmd(BUILD_TIMEOUT).args(["plugin", "install", repo, "--ref", &tag, "--yes"]))?;
            if !out.success() {
                bail!("`herdr plugin install {repo} --ref {tag}` failed:\n{}", tail(&format!("{}\n{}", out.stdout, out.stderr)));
            }
        }
        Install::Linked { root } => {
            println!("pulling main in {}…", root.display());
            let out = ctx.runner.run(&git(root, STEP_TIMEOUT).args(["pull", "--ff-only", "origin", "main"]))?;
            if !out.success() {
                bail!("`git pull --ff-only origin main` failed: {}", out.error_text());
            }
            println!("installing the binary (scripts/install.sh: the prebuilt download, or a source build when there is none)…");
            let out = ctx.runner.run(&Cmd::new("sh", BUILD_TIMEOUT).arg("scripts/install.sh").cwd(root))?;
            if !out.success() {
                bail!("the install failed:\n{}", tail(&format!("{}\n{}", out.stdout, out.stderr)));
            }
            // Its own lines say whether it downloaded or fell back to a build.
            for line in out.stderr.lines().filter(|l| l.starts_with("herdr-projects install:")) {
                println!("{line}");
            }
        }
    }
    Ok(())
}

/// Runs `binary --root <root> <args>` and prints what it said.
fn run_binary(ctx: &Ctx, binary: &Path, args: &[&str]) -> Result<bool> {
    let out = ctx.runner.run(
        &Cmd::new(binary.to_string_lossy(), STEP_TIMEOUT)
            .arg("--root")
            .arg(ctx.root.to_string_lossy())
            .args(args.iter().copied()),
    )?;
    print!("{}", out.stdout);
    eprint!("{}", out.stderr);
    Ok(out.success())
}

fn full_oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn github_remote(value: &str) -> Result<String> {
    let path = value.strip_prefix("https://github.com/")
        .or_else(|| value.strip_prefix("git@github.com:"))
        .or_else(|| value.strip_prefix("ssh://git@github.com/"))
        .context("remote is not a credential-free GitHub URL")?;
    let repo = path.strip_suffix(".git").unwrap_or(path);
    repository_arg(repo).map_err(anyhow::Error::msg)?;
    if repo.split('/').count() != 2 {
        bail!("GitHub remote must identify exactly owner/repo");
    }
    Ok(repo.to_string())
}

fn branch_git(runner: &dyn Runner, root: &Path, args: &[&str]) -> Result<String> {
    let out = runner.run(&git(root, GIT_TIMEOUT).args(args.iter().copied()))?;
    if !out.success() {
        bail!("could not read checkout provenance");
    }
    Ok(out.stdout.trim_end_matches('\n').to_string())
}

/// Only local objects are acquired, into a disposable bare repository. This
/// compares checkout history, never the unverified installed executable.
fn checkout_history(runner: &dyn Runner, root: &Path, candidate: &str) -> Result<String> {
    let head = branch_git(runner, root, &["rev-parse", "--verify", "HEAD"])?;
    if !full_oid(&head) { bail!("invalid checkout commit"); }
    let source = std::fs::canonicalize(root)?;
    let scratch = tempfile::tempdir()?;
    let objects = scratch.path().join("history.git");
    let isolated = |root: &Path| {
        let mut cmd = git(root, GIT_TIMEOUT).own_group()
            .env("HOME", scratch.path().to_string_lossy())
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_COUNT", "0").env("GIT_CONFIG_PARAMETERS", "")
            .env("GIT_NO_REPLACE_OBJECTS", "1").env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_ALLOW_PROTOCOL", "file").env("GIT_OPTIONAL_LOCKS", "0");
        for key in ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE",
            "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_SHALLOW_FILE"] {
            cmd = cmd.env_remove(key);
        }
        cmd
    };
    let out = runner.run(&isolated(scratch.path()).args([
        "clone", "--local", "--no-hardlinks", "--bare", "--no-checkout", "--template=", "--",
        &source.to_string_lossy(), &objects.to_string_lossy(),
    ]))?;
    if !out.success() { bail!("local objects unavailable"); }
    let shallow = runner.run(&isolated(&objects).args(["rev-parse", "--is-shallow-repository"]))?;
    if !shallow.success() || shallow.stdout.trim() != "false" { bail!("incomplete ancestry"); }
    // Validate both complete parent walks before interpreting merge-base's 1
    // as non-ancestry. Missing objects must not become a divergence verdict.
    let walk = runner.run(&isolated(&objects).args(["rev-list", "--parents", &head, candidate, "--"]))?;
    if !walk.success() { bail!("missing ancestry objects"); }
    let ancestor = |older: &str, newer: &str| -> Result<bool> {
        let out = runner.run(&isolated(&objects).args(["merge-base", "--is-ancestor", older, newer]))?;
        match (out.code, out.timed_out) {
            (Some(0), false) => Ok(true),
            (Some(1), false) => Ok(false),
            _ => bail!("ancestry unavailable"),
        }
    };
    let relation = if head == candidate { "equal" }
        else if ancestor(&head, candidate)? { "candidate-ahead" }
        else if ancestor(candidate, &head)? { "checkout-ahead" }
        else { "diverged" };
    Ok(format!("checkout commit: {head}\ncheckout history: {relation}\n"))
}

pub fn check_branch(ctx: &Ctx, repository: &str, reference: &str) -> Result<String> {
    let BranchCandidate { install, repository, reference, commit: candidate } = select_branch(ctx, repository, reference, None)?;
    let history = checkout_history(ctx.runner, install.root(), &candidate).unwrap_or_else(|_| {
        "checkout history: unavailable (local ancestry missing or unreadable; no network acquisition attempted)\n".into()
    });
    Ok(format!("repository: {repository}\nref: {reference}\ninstalled commit: unverified\ncandidate commit: {candidate}\n{history}installed history: unavailable (installed commit unverified)\n"))
}

struct BranchCandidate {
    install: Install,
    repository: String,
    reference: String,
    commit: String,
}

/// Stage-only: no candidate path or executable survives this boundary.
pub fn stage_branch(ctx: &Ctx, repository: &str, reference: &str) -> Result<String> {
    let scratch = tempfile::tempdir()?;
    let verification = (|| -> Result<()> {
        let selected = select_branch(ctx, repository, reference, Some(scratch.path()))?;
        let staged = stage_candidate(ctx.runner, selected.install.root(), &selected.repository, &selected.commit, scratch.path())?;
        staged.build_native(ctx.runner, &selected.repository, &selected.commit)?;
        staged.check_live_before_promotion(ctx.runner)?;
        Ok(())
    })();
    // Explicit close propagates cleanup failures rather than reporting success
    // based on TempDir's best-effort Drop. This also covers failed verification.
    scratch.close().map_err(|_| anyhow::anyhow!("candidate cleanup failed; no installation attempted"))?;
    verification.map_err(|_| anyhow::anyhow!("candidate verification failed; candidate cleaned up; no installation attempted"))?;
    Ok("candidate verification: passed\ncandidate cleanup: complete\ninstallation: unchanged (stage only)\n".into())
}

fn select_branch(ctx: &Ctx, repository: &str, reference: &str, scratch: Option<&Path>) -> Result<BranchCandidate> {
    let repository = repository_arg(repository).map_err(anyhow::Error::msg)?;
    let reference = branch_ref_arg(reference).map_err(anyhow::Error::msg)?;
    let bin = ctx.env.herdr_bin();
    let session = paths::resolve_session(&SessionFlags::default(), ctx.env, ctx.runner)?;
    let herdr = Herdr::new(&bin, &session.socket, ctx.runner);
    let out = ctx.runner.run(&herdr.cmd(herdr::CALL_TIMEOUT).args(["plugin", "list", "--plugin", PLUGIN_ID, "--json"]))?;
    if !out.success() {
        bail!("could not read installation provenance");
    }
    let install = parse_install(&out.stdout)?;
    if let Install::Github { repo, .. } = &install {
        if repo != &repository {
            bail!("selected repository conflicts with managed provenance");
        }
        let json: serde_json::Value = serde_json::from_str(&out.stdout)?;
        let source = &json["result"]["plugins"].as_array().context("missing plugins")?
            .iter().find(|p| p["plugin_id"] == PLUGIN_ID).context("missing plugin")?["source"];
        let requested = source["requested_ref"].as_str().context("missing managed requested_ref")?;
        if branch_ref_arg(requested).map_err(anyhow::Error::msg)? != reference {
            bail!("selected ref conflicts with managed provenance");
        }
        let head = branch_git(ctx.runner, install.root(), &["rev-parse", "--verify", "HEAD"])?;
        let top = branch_git(ctx.runner, install.root(), &["rev-parse", "--show-toplevel"])?;
        if source["resolved_commit"].as_str() != Some(&head) || source["managed_path"].as_str() != Some(&top) {
            bail!("managed commit/path conflicts with checkout provenance");
        }
    }
    let repo = repository.split('/').take(2).collect::<Vec<_>>().join("/");
    let prefix = branch_git(ctx.runner, install.root(), &["rev-parse", "--show-prefix"])?;
    let source = if prefix.is_empty() { repo.clone() } else { format!("{repo}/{}", prefix.strip_suffix('/').context("invalid checkout prefix")?) };
    if source != repository {
        bail!("selected subdirectory conflicts with checkout provenance");
    }
    let branch = ctx.runner.run(&git(install.root(), GIT_TIMEOUT).args(["symbolic-ref", "--quiet", "HEAD"]))?;
    let remote = if branch.success() {
        let branch = branch.stdout.trim_end_matches('\n');
        let branch = branch.strip_prefix("refs/heads/").context("missing checkout branch provenance")?;
        branch_ref_arg(branch).map_err(anyhow::Error::msg)?;
        let merge = branch_git(ctx.runner, install.root(), &["config", "--local", "--no-includes", "--get-all", &format!("branch.{branch}.merge")])?;
        if merge != reference {
            bail!("selected ref conflicts with checkout tracking provenance");
        }
        let name = branch_git(ctx.runner, install.root(), &["config", "--local", "--no-includes", "--get-all", &format!("branch.{branch}.remote")])?;
        if name.is_empty() || name == "." || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)) {
            bail!("missing or ambiguous tracking remote");
        }
        branch_git(ctx.runner, install.root(), &["config", "--local", "--no-includes", "--get-all", &format!("remote.{name}.url")])?
    } else if branch.code == Some(1) && !branch.timed_out && matches!(install, Install::Github { .. }) {
        let urls = branch_git(ctx.runner, install.root(), &["config", "--local", "--no-includes", "--get-regexp", "^remote\\..*\\.url$"])?;
        let lines: Vec<_> = urls.lines().collect();
        if lines.len() != 1 { bail!("missing or ambiguous managed remote provenance"); }
        lines[0].split_once(' ').context("invalid managed remote provenance")?.1.to_string()
    } else {
        bail!("missing branch provenance: detached linked checkout or unreadable HEAD");
    };
    if github_remote(&remote)? != repo {
        bail!("selected repository conflicts with checkout provenance");
    }
    let url = format!("https://github.com/{repo}.git");
    let query = if let Some(home) = scratch {
        candidate_command(git(home, GIT_TIMEOUT), home)
            .args(["-c", "core.hooksPath=/dev/null", "-c", "credential.helper="])
    } else {
        git(install.root(), GIT_TIMEOUT)
    };
    let out = ctx.runner.run(&query.args(["ls-remote", "--heads", "--refs", &url, &reference]))
        .map_err(|_| anyhow::anyhow!("could not query selected branch"))?;
    if !out.success() {
        bail!("could not query selected branch (offline, unavailable, or timed out); no fallback attempted");
    }
    let lines: Vec<_> = out.stdout.lines().collect();
    if lines.len() != 1 { bail!("missing or ambiguous candidate branch"); }
    let (candidate, found_ref) = lines[0].split_once('\t').context("invalid candidate response")?;
    if !full_oid(candidate) || found_ref != reference {
        bail!("invalid candidate commit or ref");
    }
    Ok(BranchCandidate { install, repository, reference, commit: candidate.into() })
}

pub fn run(ctx: &Ctx, check_only: bool) -> Result<()> {
    let bin = ctx.env.herdr_bin();
    let session = paths::resolve_session(&SessionFlags::default(), ctx.env, ctx.runner)?;
    let herdr = Herdr::new(&bin, &session.socket, ctx.runner);
    let out = ctx.runner.run(&herdr.cmd(herdr::CALL_TIMEOUT).args(["plugin", "list", "--plugin", PLUGIN_ID, "--json"]))?;
    if !out.success() {
        bail!("could not ask Herdr how {PLUGIN_ID} is installed: {}", out.error_text());
    }
    let install = parse_install(&out.stdout)?;
    let root = install.root().to_path_buf();
    let binary = binary_in(&root);
    let current = binary_version(ctx.runner, &binary).unwrap_or_else(own_version);
    let latest = latest_release(ctx.runner, &root, GIT_TIMEOUT)?
        .with_context(|| format!("origin of {} has no vX.Y.Z release tags", root.display()))?;

    if check_only {
        println!("installed: {current}");
        println!("latest:    {latest}");
        if latest > current {
            println!("run `herdr-projects update` to update");
        }
        return Ok(());
    }
    if latest <= current {
        println!("herdr-projects {current} is up to date");
        return Ok(());
    }
    if let Install::Linked { root } = &install
        && let Some(reason) = linked_blocker(ctx.runner, root)?
    {
        bail!("not updating: {reason}. Nothing was changed.");
    }

    // An old ticker misreads files a newer `doctor --fix` writes: stop it first.
    crate::ticker::stop(&ctx.root).context("could not stop the ticker; nothing was changed")?;
    let fetched = fetch_and_build(ctx, &herdr, &install, latest);
    // This process is the old binary: the rest runs the one in the plugin root,
    // which is the new one after a successful build and the old one otherwise.
    let fixed = match &fetched {
        Ok(()) => {
            println!("running doctor --fix with the new binary…");
            run_binary(ctx, &binary, &["doctor", "--fix"]).unwrap_or(false)
        }
        Err(_) => true,
    };
    let ticker = run_binary(ctx, &binary, &["ticker", "start"]).unwrap_or(false);
    let ticker_note = if ticker { "" } else { "; `herdr-projects ticker start` failed, run it again" };

    match fetched {
        Ok(()) => {
            let new = binary_version(ctx.runner, &binary).map_or_else(|| "unknown".to_string(), |v| v.to_string());
            println!("updated {current} → {new}");
            if !fixed || !ticker {
                bail!(
                    "updated, but {}{ticker_note}",
                    if fixed { "the ticker did not start" } else { "`doctor --fix` reported problems (above)" }
                );
            }
            Ok(())
        }
        Err(error) => bail!("{error:#}\nupdate failed: {current} is still installed and the ticker was restarted{ticker_note}"),
    }
}

// Internal candidate lifetime; the stage-only CLI never exposes a promotion target.
struct StagedCandidate {
    scratch: tempfile::TempDir,
    source: PathBuf,
    repository: String,
    commit: String,
    live: PathBuf,
    live_state: LiveCandidateState,
}

impl StagedCandidate {
    /// Read-only guard, not a lock or authorization to promote. Phase 3 must
    /// repeat this under installation ownership immediately before promotion.
    fn check_live_before_promotion(&self, runner: &dyn Runner) -> Result<()> {
        let current = live_candidate_state(runner, self.scratch.path(), &self.live)?;
        if !self.live_state.status.is_empty() || !current.status.is_empty() {
            bail!("live checkout is dirty or has untracked collisions");
        }
        if current != self.live_state { bail!("live installation changed concurrently"); }
        Ok(())
    }

    fn build_native(&self, runner: &dyn Runner, repository: &str, commit: &str) -> Result<PathBuf> {
        if repository != self.repository || commit != self.commit {
            bail!("candidate source/commit provenance mismatch");
        }
        if candidate_git(runner, self.scratch.path(), &self.source, &["rev-parse", "HEAD"])? != commit {
            bail!("candidate commit mismatch");
        }
        let subdir = repository.split('/').skip(2).collect::<Vec<_>>().join("/");
        let package = std::fs::canonicalize(self.source.join(subdir))?;
        if !package.starts_with(&self.source) { bail!("candidate package escapes source"); }
        let manifest: toml::Value = toml::from_str(&std::fs::read_to_string(package.join("Cargo.toml"))?)?;
        if manifest["package"]["name"].as_str() != Some(PLUGIN_ID) {
            bail!("candidate package identity mismatch");
        }
        let version = manifest["package"]["version"].as_str().context("candidate version missing")?;
        if parse_release(&format!("v{version}")).is_none() { bail!("invalid candidate version"); }
        if !package.join("Cargo.lock").is_file() { bail!("candidate lockfile missing"); }
        let target = self.scratch.path().join("target");
        let cmd = candidate_command(Cmd::new("cargo", BUILD_TIMEOUT), self.scratch.path())
            .cwd(&package)
            .env("CARGO_HOME", self.scratch.path().join("cargo-home").to_string_lossy())
            .env("CARGO_TARGET_DIR", target.to_string_lossy())
            .env_remove("CARGO_BUILD_TARGET").env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS").env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTC_WORKSPACE_WRAPPER").env_remove("CARGO_REGISTRY_TOKEN")
            .args(["build", "--release", "--locked", "--bin", PLUGIN_ID, "--target-dir"])
            .arg(target.to_string_lossy());
        if !runner.run(&cmd)?.success() { bail!("native candidate build failed; no fallback attempted"); }
        let binary = target.join("release/herdr-projects");
        let checksum = candidate_checksum(&binary)?;
        let out = runner.run(&candidate_command(Cmd::new(binary.to_string_lossy(), STEP_TIMEOUT), self.scratch.path())
            .cwd(self.scratch.path()).arg("--version"))?;
        if !out.success() { bail!("candidate executable cannot run"); }
        let reported = out.stdout.trim().strip_prefix("herdr-projects ").context("candidate executable identity mismatch")?;
        let (found_version, build) = reported.split_once('+').context("candidate build provenance missing")?;
        let (hash, timestamp) = build.split_once('.').context("candidate build provenance missing")?;
        if found_version != version || hash.len() < 7 || !commit.starts_with(hash)
            || timestamp.is_empty() || !timestamp.bytes().all(|b| b.is_ascii_digit()) {
            bail!("candidate version/commit provenance mismatch");
        }
        if candidate_checksum(&binary)? != checksum { bail!("candidate checksum mismatch"); }
        if !candidate_git(runner, self.scratch.path(), &self.source, &["status", "--porcelain", "--untracked-files=all"])?.is_empty() {
            bail!("candidate source changed during build");
        }
        Ok(binary)
    }
}

fn candidate_checksum(path: &Path) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    if !std::fs::symlink_metadata(path)?.is_file() { bail!("candidate must be a regular file"); }
    Ok(Sha256::digest(std::fs::read(path)?).to_vec())
}

#[derive(PartialEq)]
struct LiveCandidateState {
    head: String,
    branch: String,
    status: String,
    remotes: String,
    binary: Vec<u8>,
}

fn live_candidate_state(runner: &dyn Runner, home: &Path, live: &Path) -> Result<LiveCandidateState> {
    let read = |args: &[&str]| candidate_git(runner, home, live, args);
    Ok(LiveCandidateState {
        head: read(&["rev-parse", "--verify", "HEAD"] )?,
        branch: read(&["rev-parse", "--abbrev-ref", "HEAD"] )?,
        status: read(&["status", "--porcelain", "--untracked-files=all"] )?,
        remotes: read(&["config", "--local", "--no-includes", "--get-regexp", "^remote\\..*\\.url$"] )?,
        binary: candidate_checksum(&binary_in(live))?,
    })
}

fn stage_candidate(
    runner: &dyn Runner, live: &Path, repository: &str, commit: &str, scratch: &Path,
) -> Result<StagedCandidate> {
    repository_arg(repository).map_err(anyhow::Error::msg)?;
    if !full_oid(commit) { bail!("candidate requires an explicitly selected full commit"); }
    let live = std::fs::canonicalize(live)?;
    let parent = std::fs::canonicalize(scratch)?;
    if parent.starts_with(&live) { bail!("candidate scratch must be outside live checkout"); }
    let scratch = tempfile::tempdir_in(parent)?;
    let live_state = live_candidate_state(runner, scratch.path(), &live)?;
    // This internal slice accepts only unambiguous checkout source evidence.
    // Managed/tracking selection remains the phase-1 caller's responsibility.
    let urls: Vec<_> = live_state.remotes.lines().collect();
    let repo = repository.split('/').take(2).collect::<Vec<_>>().join("/");
    if urls.len() != 1 || github_remote(urls[0].split_once(' ').context("missing source provenance")?.1)? != repo {
        bail!("candidate source conflicts with live provenance");
    }
    let source = scratch.path().join("source");
    std::fs::create_dir(&source)?;
    let url = format!("https://github.com/{repo}.git");
    for args in [
        vec!["init", "--template="],
        vec!["fetch", "--no-tags", "--no-recurse-submodules", "--", &url, commit],
        vec!["checkout", "--detach", commit, "--"],
    ] {
        candidate_git(runner, scratch.path(), &source, &args)?;
    }
    if candidate_git(runner, scratch.path(), &source, &["rev-parse", "HEAD"])? != commit {
        bail!("candidate commit mismatch");
    }
    Ok(StagedCandidate { scratch, source, repository: repository.into(), commit: commit.into(), live, live_state })
}

fn candidate_command(cmd: Cmd, home: &Path) -> Cmd {
    let mut cmd = cmd.own_group()
        .env("HOME", home.to_string_lossy())
        .env("XDG_CONFIG_HOME", home.to_string_lossy())
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "0").env("GIT_CONFIG_PARAMETERS", "")
        .env("GIT_NO_REPLACE_OBJECTS", "1").env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_ALLOW_PROTOCOL", "https").env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    for key in ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES", "GIT_SHALLOW_FILE",
        "GIT_ASKPASS", "SSH_ASKPASS", "SSH_AUTH_SOCK"] {
        cmd = cmd.env_remove(key);
    }
    cmd
}

fn candidate_git(runner: &dyn Runner, home: &Path, root: &Path, args: &[&str]) -> Result<String> {
    let cmd = candidate_command(git(root, GIT_TIMEOUT), home)
        .args(["-c", "core.hooksPath=/dev/null", "-c", "credential.helper="])
        .args(args.iter().copied());
    let out = runner.run(&cmd)?;
    if !out.success() { bail!("candidate Git step failed: {}", args[0]); }
    Ok(out.stdout.trim_end_matches('\n').into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::fake::{FakeRunner, fail, ok};

    const HEAD: &str = "1111111111111111111111111111111111111111";

    fn staging_fixture() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir, String, FakeRunner) {
        let (remote, _, _, _) = local_history();
        std::fs::write(remote.path().join("Cargo.toml"), "[package]\nname = 'herdr-projects'\nversion = '0.2.34'\n").unwrap();
        std::fs::write(remote.path().join("Cargo.lock"), "# synthetic locked dependencies\n").unwrap();
        local_git(remote.path(), &["add", "Cargo.toml", "Cargo.lock"]);
        local_git(remote.path(), &["commit", "-m", "candidate"]);
        let commit = local_git(remote.path(), &["rev-parse", "HEAD"]);
        let (live, _, _, _) = local_history();
        local_git(live.path(), &["remote", "add", "origin", "https://github.com/forgegod/eliasstravik-herdr-projects.git"]);
        std::fs::create_dir_all(live.path().join("target/release")).unwrap();
        std::fs::write(binary_in(live.path()), "active executable, never replace").unwrap();
        std::fs::write(live.path().join("notes"), "untracked operator notes").unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let runner = FakeRunner::new();
        let live_path = live.path().to_path_buf();
        let remote_path = remote.path().to_path_buf();
        runner.on_fn(|c| c.program == "git", move |cmd| {
            assert!(cmd.own_group && cmd.timeout <= GIT_TIMEOUT);
            if Path::new(&cmd.args[1]) == live_path {
                for forbidden in ["fetch", "checkout", "reset", "stash", "remote", "pull"] {
                    assert!(!cmd.args.iter().any(|a| a == forbidden), "live mutation: {}", cmd.display());
                }
            }
            let mut local = cmd.clone();
            if let Some(i) = local.args.iter().position(|a| a == "https://github.com/forgegod/eliasstravik-herdr-projects.git") {
                assert!(local.args.iter().any(|a| a == "fetch"));
                local.args[i] = remote_path.to_string_lossy().into_owned();
                local = local.env("GIT_ALLOW_PROTOCOL", "file");
            }
            crate::runner::RealRunner.run(&local)
        });
        (live, remote, scratch, commit, runner)
    }

    #[test]
    fn candidate_stage_interface_preserves_legacy_update_dispatch() {
        for check in [false, true] {
            let home = tempfile::tempdir().unwrap();
            let env = paths::Env::for_test(home.path(), &[("HERDR_SOCKET_PATH", "/synthetic/socket")]);
            let runner = FakeRunner::new();
            runner.on("plugin list", ok(&list(r#"{"kind":"local"}"#)));
            runner.on("--version", ok("herdr-projects 0.2.34\n"));
            runner.on("ls-remote --tags --refs origin", ok("abc\trefs/tags/v999.0.0\n"));
            runner.on("rev-parse --abbrev-ref HEAD", ok("operator-branch\n"));
            let ctx = Ctx { env: &env, root: home.path().join("projects"), config_dir: env.config_dir(), runner: &runner, detached_ticker: false };
            let mut args = vec!["herdr-projects", "update"];
            if check { args.push("--check"); }
            let result = crate::cli::dispatch_test(&ctx, &args);
            if check {
                result.unwrap();
                assert_eq!(runner.count("rev-parse"), 0);
            } else {
                assert!(result.unwrap_err().to_string().contains("not `main`"));
            }
            assert_eq!(runner.count("ls-remote --tags --refs origin"), 1);
            assert_eq!(runner.count("--version"), 1);
            assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
            for forbidden in ["fetch", "cargo", "plugin install", "scripts/install", "doctor", "ticker"] {
                assert_eq!(runner.count(forbidden), 0);
            }
        }
    }

    #[test]
    fn candidate_stage_cli_dispatch_verifies_and_cleans_before_returning() {
        for case in ["valid", "subdir", "build-failure", "dirty"] {
            let (live, remote, _scratch, mut commit, staging) = staging_fixture();
            std::fs::remove_file(live.path().join("notes")).unwrap();
            std::fs::write(live.path().join(".git/info/exclude"), "/target/\n").unwrap();
            let package = if case == "subdir" { "plugin" } else { "" };
            if case == "subdir" {
                std::fs::create_dir(remote.path().join(package)).unwrap();
                for file in ["Cargo.toml", "Cargo.lock"] {
                    local_git(remote.path(), &["mv", file, &format!("{package}/{file}")]);
                }
                local_git(remote.path(), &["commit", "-m", "nested package"]);
                commit = local_git(remote.path(), &["rev-parse", "HEAD"]);
            }
            if case == "dirty" { std::fs::write(live.path().join("Cargo.toml"), "operator edit").unwrap(); }
            let before = checkout_snapshot(live.path());
            let runner = FakeRunner::new();
            runner.on("plugin list", ok(&list(r#"{"kind":"local"}"#).replace("/p/root", live.path().to_str().unwrap())));
            runner.on("rev-parse --show-prefix", ok(if package.is_empty() { "" } else { "plugin/\n" }));
            runner.on("symbolic-ref --quiet HEAD", ok("refs/heads/main\n"));
            runner.on("branch.main.merge", ok("refs/heads/main\n"));
            runner.on("branch.main.remote", ok("origin\n"));
            runner.on("remote.origin.url", ok("https://github.com/forgegod/eliasstravik-herdr-projects.git\n"));
            let selected = commit.clone();
            let live_root = live.path().to_path_buf();
            runner.on_fn(|c| c.args.iter().any(|a| a == "ls-remote"), move |cmd| {
                assert_ne!(Path::new(&cmd.args[1]), live_root, "stage selection must not use live Git credentials/config");
                assert!(cmd.env.contains(&("GIT_CONFIG_GLOBAL".into(), "/dev/null".into())));
                assert!(cmd.args.iter().any(|a| a == "credential.helper="));
                Ok(ok(&format!("{selected}\trefs/heads/main\n")))
            });
            runner.on_fn(|c| c.program == "git", move |cmd| staging.run(cmd));
            let built = std::rc::Rc::new(std::cell::RefCell::new(None));
            let seen = built.clone();
            runner.on_fn(|c| c.program == "cargo", move |cmd| {
                let target = PathBuf::from(cmd.args.last().unwrap());
                *seen.borrow_mut() = Some(target.parent().unwrap().to_path_buf());
                assert_eq!(cmd.cwd.as_ref().unwrap().file_name().unwrap(), if package.is_empty() { "source" } else { package });
                if case == "build-failure" { return Ok(fail(1, "synthetic build failure")); }
                std::fs::create_dir_all(target.join("release"))?;
                std::fs::write(target.join("release/herdr-projects"), "synthetic candidate")?;
                Ok(ok(""))
            });
            runner.on_fn(|c| c.args == ["--version"], move |_| Ok(ok(&format!("herdr-projects 0.2.34+{}.123\n", &commit[..7]))));
            let home = tempfile::tempdir().unwrap();
            let env = paths::Env::for_test(home.path(), &[("HERDR_SOCKET_PATH", "/synthetic/socket")]);
            let ctx = Ctx { env: &env, root: home.path().join("projects"), config_dir: env.config_dir(), runner: &runner, detached_ticker: false };
            let repository = format!("forgegod/eliasstravik-herdr-projects{}", if package.is_empty() { "" } else { "/plugin" });
            let result = crate::cli::dispatch_test(&ctx, &["herdr-projects", "update", "--stage", "--channel", "branch", "--repository", &repository, "--ref", "main"]);
            assert_eq!(result.is_ok(), matches!(case, "valid" | "subdir"), "{case}: {result:?}");
            let scratch = built.borrow().clone().expect("stage dispatch did not build the candidate");
            assert!(!scratch.exists(), "candidate survived dispatch: {case}");
            assert_eq!(before, checkout_snapshot(live.path()), "live mutation: {case}");
            assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0);
            for forbidden in ["plugin install", "scripts/install", "doctor", "ticker", "docker", "--tags"] {
                assert_eq!(runner.count(forbidden), 0, "unexpected {forbidden}");
            }
        }
    }

    #[test]
    fn candidate_staging_refuses_conflicting_source_before_fetching() {
        let (live, _remote, scratch, commit, runner) = staging_fixture();
        local_git(live.path(), &["remote", "set-url", "origin", "https://github.com/upstream/other.git"]);
        let before = checkout_snapshot(live.path());
        let result = stage_candidate(&runner, live.path(), "forgegod/eliasstravik-herdr-projects", &commit, scratch.path());
        assert!(result.is_err(), "staged a source conflicting with live provenance");
        assert_eq!(runner.count("fetch"), 0);
        assert_eq!(before, checkout_snapshot(live.path()));
        assert_eq!(std::fs::read_dir(scratch.path()).unwrap().count(), 0);
    }

    #[test]
    fn candidate_guard_rejects_dirty_colliding_or_concurrently_changed_live_state() {
        for case in ["dirty", "collision", "concurrent-file", "concurrent-head", "concurrent-branch", "concurrent-remote", "concurrent-binary", "clean"] {
            let (live, _remote, scratch, commit, runner) = staging_fixture();
            std::fs::remove_file(live.path().join("notes")).unwrap();
            std::fs::write(live.path().join(".git/info/exclude"), "/target/\n").unwrap();
            if case == "dirty" { std::fs::write(live.path().join("Cargo.toml"), "operator changes").unwrap(); }
            if case == "collision" { std::fs::write(live.path().join("Cargo.lock"), "operator lockfile").unwrap(); }
            let staged = stage_candidate(&runner, live.path(), "forgegod/eliasstravik-herdr-projects", &commit, scratch.path()).unwrap();
            match case {
                "concurrent-file" => { std::fs::write(live.path().join("Cargo.toml"), "concurrent edit").unwrap(); }
                "concurrent-head" => {
                    let tree = local_git(live.path(), &["rev-parse", "HEAD^{tree}"]);
                    let next = local_git(live.path(), &["commit-tree", &tree, "-p", "HEAD", "-m", "concurrent commit"]);
                    local_git(live.path(), &["update-ref", "refs/heads/main", &next]);
                }
                "concurrent-branch" => { local_git(live.path(), &["branch", "other"]); local_git(live.path(), &["symbolic-ref", "HEAD", "refs/heads/other"]); }
                "concurrent-remote" => { local_git(live.path(), &["remote", "set-url", "origin", "https://github.com/upstream/other.git"]); }
                "concurrent-binary" => { std::fs::write(binary_in(live.path()), "replaced by another installer").unwrap(); }
                _ => {}
            }
            let before = checkout_snapshot(live.path());
            let result = staged.check_live_before_promotion(&runner);
            assert_eq!(result.is_ok(), case == "clean", "guard accepted {case}: {result:?}");
            assert_eq!(before, checkout_snapshot(live.path()), "guard mutated live state: {case}");
        }
    }

    #[test]
    fn candidate_native_build_verifies_artifact_before_accepting_it() {
        for case in ["valid", "version", "provenance", "not-runnable", "checksum", "build-timeout", "source", "commit"] {
            let (live, _remote, scratch, commit, runner) = staging_fixture();
            let before = checkout_snapshot(live.path());
            let staged = stage_candidate(&runner, live.path(), "forgegod/eliasstravik-herdr-projects", &commit, scratch.path()).unwrap();
            let stage_root = staged.scratch.path().to_path_buf();
            let live_root = live.path().to_path_buf();
            runner.on_fn(|c| c.program == "cargo", move |cmd| {
                assert_eq!(cmd.args, ["build", "--release", "--locked", "--bin", "herdr-projects", "--target-dir", stage_root.join("target").to_str().unwrap()]);
                assert!(cmd.own_group && cmd.timeout == BUILD_TIMEOUT);
                assert!(!cmd.cwd.as_ref().unwrap().starts_with(&live_root));
                assert!(cmd.env.contains(&("CARGO_HOME".into(), stage_root.join("cargo-home").to_string_lossy().into_owned())));
                if case == "build-timeout" { return Ok(crate::runner::fake::timeout()); }
                let binary = stage_root.join("target/release/herdr-projects");
                std::fs::create_dir_all(binary.parent().unwrap())?;
                std::fs::write(binary, "synthetic candidate artifact")?;
                Ok(ok(""))
            });
            let oid = commit.clone();
            runner.on_fn(|c| c.args == ["--version"], move |cmd| {
                assert!(cmd.own_group && cmd.timeout <= STEP_TIMEOUT);
                if case == "not-runnable" { bail!("Exec format error"); }
                if case == "checksum" { std::fs::write(&cmd.program, "tampered artifact")?; }
                let version = if case == "version" { "9.9.9" } else { "0.2.34" };
                let hash = if case == "provenance" { "0000000" } else { &oid[..7] };
                Ok(ok(&format!("herdr-projects {version}+{hash}.123\n")))
            });
            let repository = if case == "source" { "upstream/other" } else { "forgegod/eliasstravik-herdr-projects" };
            let selected = if case == "commit" { HEAD } else { &commit };
            let result = staged.build_native(&runner, repository, selected);
            if case == "valid" {
                assert!(result.is_ok(), "valid native candidate rejected: {result:?}");
                assert!(result.unwrap().is_file());
            } else {
                assert!(result.is_err(), "accepted {case}");
            }
            assert_eq!(before, checkout_snapshot(live.path()), "live mutation: {case}");
            let path = staged.scratch.path().to_path_buf();
            drop(staged);
            assert!(!path.exists());
        }
    }

    #[test]
    fn candidate_staging_preserves_entire_live_checkout_and_active_binary() {
        let (live, _remote, scratch, commit, runner) = staging_fixture();
        let before = checkout_snapshot(live.path());
        let staged = stage_candidate(&runner, live.path(), "forgegod/eliasstravik-herdr-projects", &commit, scratch.path());
        assert_eq!(before, checkout_snapshot(live.path()), "HEAD, branch, tracked/untracked files, remotes or binary changed");
        let staged = staged.expect("selected immutable candidate must stage without changing live state");
        assert!(!staged.source.starts_with(live.path()));
        assert_eq!(local_git(&staged.source, &["rev-parse", "HEAD"]), commit);
        assert!(staged.source.join("Cargo.lock").is_file());
        let path = staged.scratch.path().to_path_buf();
        drop(staged);
        assert!(!path.exists(), "scratch was not cleaned up");
        assert_eq!(before, checkout_snapshot(live.path()));
    }

    struct HistoryRunner {
        root: PathBuf,
        candidate: String,
    }

    impl Runner for HistoryRunner {
        fn run(&self, cmd: &Cmd) -> Result<crate::runner::Output> {
            use crate::runner::RealRunner;
            if cmd.program != "git" {
                return Ok(ok(&list(r#"{"kind":"local"}"#).replace("/p/root", self.root.to_str().unwrap())));
            }
            if cmd.args.iter().any(|a| a == "ls-remote") {
                if self.candidate == "offline" { return Ok(fail(128, "offline")); }
                return Ok(ok(&format!("{}\trefs/heads/main\n", self.candidate)));
            }
            if cmd.args.iter().any(|a| a == "config") {
                return branch_runner().run(cmd);
            }
            for forbidden in ["fetch", "pull", "checkout", "switch", "remote"] {
                assert!(!cmd.args.iter().any(|a| a == forbidden), "unexpected command: {}", cmd.display());
            }
            if cmd.args.iter().any(|a| a == "clone") {
                assert_ne!(Path::new(&cmd.args[1]), self.root, "clone runs in live installation");
            }
            RealRunner.run(cmd)
        }

        fn socket_request(&self, _: &Path, _: &str, _: Duration) -> Result<String> {
            panic!("unexpected socket request")
        }

        fn run_foreground(&self, _: &Cmd, _: &mut dyn FnMut() -> bool) -> Result<Option<i32>> {
            panic!("unexpected foreground command")
        }
    }

    fn local_git(root: &Path, args: &[&str]) -> String {
        let out = crate::runner::RealRunner.run(&git(root, GIT_TIMEOUT)
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Test").env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_NAME", "Test").env("GIT_COMMITTER_EMAIL", "test@example.invalid")
            .args(args.iter().copied())).unwrap();
        assert!(out.success(), "{}", out.error_text());
        out.stdout.trim().to_string()
    }

    fn local_history() -> (tempfile::TempDir, String, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        local_git(root, &["init", "--initial-branch=main"]);
        std::fs::write(root.join("Cargo.toml"), "version = '0.2.34'\n").unwrap();
        local_git(root, &["add", "Cargo.toml"]);
        local_git(root, &["commit", "-m", "base"]);
        let base = local_git(root, &["rev-parse", "HEAD"]);
        let tree = local_git(root, &["rev-parse", "HEAD^{tree}"]);
        let next = local_git(root, &["commit-tree", &tree, "-p", &base, "-m", "same version"]);
        let fork = local_git(root, &["commit-tree", &tree, "-p", &base, "-m", "fork"]);
        (dir, base, next, fork)
    }

    fn checkout_snapshot(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
        fn walk(root: &Path, path: &Path, files: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() { walk(root, &path, files); }
                else { files.insert(path.strip_prefix(root).unwrap().into(), std::fs::read(path).unwrap()); }
            }
        }
        let mut files = std::collections::BTreeMap::new();
        walk(root, root, &mut files);
        files
    }

    #[test]
    fn branch_history_same_version_candidate_ahead_needs_ancestry_not_sha_inequality() {
        let (dir, base, next, _) = local_history();
        assert_ne!(base, next);
        assert_eq!(local_git(dir.path(), &["show", &format!("{base}:Cargo.toml")]),
            local_git(dir.path(), &["show", &format!("{next}:Cargo.toml")]));
        let before = checkout_snapshot(dir.path());
        let runner = HistoryRunner { root: dir.path().into(), candidate: next };
        let report = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").unwrap();
        assert_eq!(before, checkout_snapshot(dir.path()), "live checkout mutated");
        assert!(report.contains("checkout history: candidate-ahead\n"), "{report}");
        assert!(report.contains("installed commit: unverified\n"), "{report}");
        assert!(report.contains("installed history: unavailable (installed commit unverified)\n"), "{report}");
    }

    #[test]
    fn branch_history_classifies_equal_checkout_ahead_and_diverged_without_install_claims() {
        let (dir, base, next, fork) = local_history();
        for (head, candidate, expected) in [
            (&base, &base, "equal"),
            (&next, &base, "checkout-ahead"),
            (&base, &next, "candidate-ahead"),
            (&next, &fork, "diverged"),
        ] {
            // All three unequal pairs have the same SHA inequality, but
            // different ancestry. HEAD is only a checkout, not binary proof.
            local_git(dir.path(), &["update-ref", "refs/heads/main", head]);
            std::fs::write(dir.path().join("untracked"), "preserve me").unwrap();
            std::fs::write(dir.path().join("Cargo.toml"), "dirty, preserve me").unwrap();
            let before = checkout_snapshot(dir.path());
            let runner = HistoryRunner { root: dir.path().into(), candidate: candidate.clone() };
            let report = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").unwrap();
            assert_eq!(before, checkout_snapshot(dir.path()), "live checkout mutated");
            assert!(report.contains(&format!("checkout history: {expected}\n")), "{report}");
            assert!(report.contains("installed commit: unverified\n"), "{report}");
            assert!(report.contains("installed history: unavailable (installed commit unverified)\n"), "{report}");
            assert!(!report.contains("installed-ahead"), "invented binary provenance: {report}");
        }
    }

    #[test]
    fn branch_history_refuses_missing_candidate_parent_shallow_and_offline_without_mutation() {
        for case in ["missing-candidate", "missing-parent", "shallow", "offline"] {
            let (dir, base, next, fork) = local_history();
            local_git(dir.path(), &["update-ref", "refs/heads/main", &next]);
            let candidate = match case {
                "missing-candidate" => {
                    std::fs::remove_file(dir.path().join(".git/objects").join(&fork[..2]).join(&fork[2..])).unwrap();
                    fork
                }
                "missing-parent" => {
                    std::fs::remove_file(dir.path().join(".git/objects").join(&base[..2]).join(&base[2..])).unwrap();
                    fork
                }
                "shallow" => {
                    std::fs::write(dir.path().join(".git/shallow"), format!("{next}\n")).unwrap();
                    fork
                }
                _ => "offline".into(),
            };
            let before = checkout_snapshot(dir.path());
            let runner = HistoryRunner { root: dir.path().into(), candidate };
            let result = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main");
            assert_eq!(before, checkout_snapshot(dir.path()), "live checkout mutated: {case}");
            if case == "offline" {
                assert!(result.unwrap_err().to_string().contains("could not query selected branch"));
            } else {
                let report = result.unwrap();
                assert!(report.contains("checkout history: unavailable (local ancestry missing or unreadable; no network acquisition attempted)\n"), "{case}: {report}");
                assert!(report.contains("installed commit: unverified\n"), "{report}");
                assert!(report.contains("installed history: unavailable (installed commit unverified)\n"), "{report}");
            }
        }
    }

    fn branch_runner() -> FakeRunner {
        branch_runner_with(|_| {})
    }

    fn branch_runner_with(setup: impl FnOnce(&FakeRunner)) -> FakeRunner {
        let runner = FakeRunner::new();
        setup(&runner);
        runner.on("plugin list", ok(&list(r#"{"kind":"local"}"#)));
        runner.on("rev-parse --show-prefix", ok(""));
        runner.on("rev-parse --show-toplevel", ok("/p/root\n"));
        runner.on("rev-parse --verify HEAD", ok(HEAD));
        runner.on("symbolic-ref --quiet HEAD", ok("refs/heads/main\n"));
        runner.on("config --local --no-includes --get-all branch.main.remote", ok("origin\n"));
        runner.on("config --local --no-includes --get-all branch.main.merge", ok("refs/heads/main\n"));
        runner.on("config --local --no-includes --get-all remote.origin.url", ok("https://github.com/forgegod/eliasstravik-herdr-projects.git\n"));
        runner.on("ls-remote", ok(&format!("{HEAD}\trefs/heads/main\n")));
        runner
    }

    fn branch_check(runner: &dyn Runner, repository: &str, reference: &str) -> Result<String> {
        let home = tempfile::tempdir().unwrap();
        let env = paths::Env::for_test(home.path(), &[("HERDR_SOCKET_PATH", "/synthetic/socket")]);
        let ctx = Ctx { env: &env, root: home.path().join("projects"), config_dir: env.config_dir(), runner, detached_ticker: false };
        let result = check_branch(&ctx, repository, reference);
        assert_eq!(std::fs::read_dir(home.path()).unwrap().count(), 0, "check wrote user state");
        result
    }

    #[test]
    fn branch_update_installed_commit_is_unverified_not_checkout_or_metadata() {
        for source in [r#"{"kind":"local"}"#.to_string(), format!(r#"{{"kind":"github","owner":"forgegod","repo":"eliasstravik-herdr-projects","requested_ref":"main","resolved_commit":"{HEAD}","managed_path":"/p/root"}}"#)] {
            let runner = branch_runner_with(|r| { r.on("plugin list", ok(&list(&source))); });
            let report = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").unwrap();
            assert!(report.contains("installed commit: unverified\n"), "{report}");
            assert!(!report.contains(&format!("installed commit: {HEAD}")));
        }
    }

    #[test]
    fn branch_update_requires_one_exact_candidate_ref_and_full_oid() {
        for response in [
            "".to_string(), format!("{HEAD}\trefs/heads/other\n"),
            "abc\trefs/heads/main\n".into(), format!("{}\trefs/heads/main\n", "z".repeat(40)),
            format!("{HEAD}\trefs/heads/main\n{HEAD}\trefs/heads/main\n"),
            format!("{HEAD}\trefs/heads/main extra\n"),
        ] {
            let runner = branch_runner_with(|r| { r.on("ls-remote", ok(&response)); });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err(), "accepted {response:?}");
        }
    }

    #[test]
    fn branch_update_offline_is_an_error_without_fallback() {
        for output in [fail(128, "offline"), crate::runner::fake::timeout(), crate::runner::Output { code: Some(128), stdout: format!("{HEAD}\trefs/heads/main\n"), ..Default::default() }] {
            let runner = branch_runner_with(|r| { r.on("ls-remote", output); });
            let error = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").unwrap_err();
            assert!(error.to_string().contains("could not query selected branch"), "{error}");
            assert_eq!(runner.count("ls-remote"), 1);
            assert_eq!(runner.count("--tags"), 0);
        }
    }

    #[test]
    fn branch_update_checks_detached_managed_install_without_switching() {
        let source = format!(r#"{{"kind":"github","owner":"forgegod","repo":"eliasstravik-herdr-projects","requested_ref":"refs/heads/main","resolved_commit":"{HEAD}","managed_path":"/p/root"}}"#);
        let runner = branch_runner_with(|r| {
            r.on("plugin list", ok(&list(&source)));
            r.on("symbolic-ref --quiet HEAD", fail(1, ""));
            r.on("--get-regexp", ok("remote.renamed.url git@github.com:forgegod/eliasstravik-herdr-projects.git\n"));
        });
        let report = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").unwrap();
        assert!(report.contains("candidate commit:"));
        assert_eq!(runner.count("switch"), 0);
        assert_eq!(runner.count("checkout"), 0);
        let linked = branch_runner_with(|r| { r.on("symbolic-ref --quiet HEAD", fail(1, "")); });
        assert!(branch_check(&linked, "forgegod/eliasstravik-herdr-projects", "main").is_err());
        assert_eq!(linked.count("ls-remote"), 0);
        for urls in ["", "remote.origin.url https://github.com/upstream/repo.git", "remote.a.url https://github.com/forgegod/eliasstravik-herdr-projects.git\nremote.b.url https://github.com/upstream/repo.git"] {
            let runner = branch_runner_with(|r| {
                r.on("plugin list", ok(&list(&source)));
                r.on("symbolic-ref --quiet HEAD", fail(1, ""));
                r.on("--get-regexp", ok(urls));
            });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err());
        }
    }

    #[test]
    fn branch_update_refuses_managed_provenance_mismatch() {
        let source = serde_json::json!({"kind":"github", "owner":"forgegod", "repo":"eliasstravik-herdr-projects", "requested_ref":"main", "resolved_commit":HEAD, "managed_path":"/p/root"});
        let runner = branch_runner_with(|r| { r.on("plugin list", ok(&list(&source.to_string()))); });
        assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_ok());
        for (field, value) in [
            ("owner", "upstream"), ("repo", "other"), ("subdir", "plugin"),
            ("requested_ref", "other"), ("requested_ref", ""),
            ("resolved_commit", "2222222222222222222222222222222222222222"),
            ("resolved_commit", ""), ("managed_path", "/elsewhere"),
        ] {
            let mut bad = source.clone();
            bad[field] = value.into();
            let runner = branch_runner_with(|r| { r.on("plugin list", ok(&list(&bad.to_string()))); });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err(), "accepted {field}={value}");
            assert_eq!(runner.count("ls-remote"), 0);
        }
        for field in ["requested_ref", "resolved_commit", "managed_path"] {
            let mut bad = source.clone();
            bad.as_object_mut().unwrap().remove(field);
            let runner = branch_runner_with(|r| { r.on("plugin list", ok(&list(&bad.to_string()))); });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err(), "missing {field}");
        }
    }

    #[test]
    fn branch_update_follows_tracking_identity_not_remote_name() {
        let runner = branch_runner_with(|r| {
            r.on("branch.main.remote", ok("my-fork\n"));
            r.on("remote.my-fork.url", ok("git@github.com:forgegod/eliasstravik-herdr-projects.git\n"));
            r.on("remote.origin.url", ok("https://github.com/eliasstravik/herdr-projects.git\n"));
        });
        assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_ok());
        assert_eq!(runner.count("remote.origin.url"), 0);
        for remote in ["", ".", "origin\nother", "../origin"] {
            let runner = branch_runner_with(|r| { r.on("branch.main.remote", ok(remote)); });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err());
            assert_eq!(runner.count("ls-remote"), 0);
        }
        let runner = branch_runner_with(|r| { r.on("branch.main.remote", fail(1, "missing")); });
        assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err());
    }

    #[test]
    fn branch_update_requires_matching_subdirectory_and_tracking_ref() {
        for (prefix, merge, selected, reference, valid) in [
            ("plugin/sub/", "refs/heads/main", "forgegod/eliasstravik-herdr-projects/plugin/sub", "main", true),
            ("", "refs/heads/main", "forgegod/eliasstravik-herdr-projects/plugin/sub", "main", false),
            ("plugin/", "refs/heads/main", "forgegod/eliasstravik-herdr-projects", "main", false),
            ("", "refs/heads/other", "forgegod/eliasstravik-herdr-projects", "main", false),
            ("", "", "forgegod/eliasstravik-herdr-projects", "main", false),
            ("", "refs/heads/main\nrefs/heads/other", "forgegod/eliasstravik-herdr-projects", "main", false),
        ] {
            let runner = branch_runner_with(|r| {
                r.on("rev-parse --show-prefix", ok(prefix));
                r.on("branch.main.merge", ok(merge));
            });
            assert_eq!(branch_check(&runner, selected, reference).is_ok(), valid, "{prefix:?} {merge:?}");
            if !valid { assert_eq!(runner.count("ls-remote"), 0); }
        }
    }

    #[test]
    fn branch_update_reconciles_fork_upstream_url_identities() {
        for url in [
            "https://github.com/forgegod/eliasstravik-herdr-projects.git",
            "git@github.com:forgegod/eliasstravik-herdr-projects.git",
            "ssh://git@github.com/forgegod/eliasstravik-herdr-projects.git",
        ] {
            let runner = branch_runner_with(|r| { r.on("remote.origin.url", ok(url)); });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_ok());
            assert!(branch_check(&runner, "eliasstravik/herdr-projects", "main").is_err(), "source switch allowed for {url}");
        }
        for url in [
            "https://github.com/eliasstravik/herdr-projects.git",
            "https://github.com.evil/forgegod/eliasstravik-herdr-projects.git",
            "https://token@github.com/forgegod/eliasstravik-herdr-projects.git",
            "file:///forgegod/eliasstravik-herdr-projects", "", "../repo",
        ] {
            let runner = branch_runner_with(|r| { r.on("remote.origin.url", ok(url)); });
            assert!(branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").is_err(), "accepted {url}");
            assert_eq!(runner.count("ls-remote"), 0);
        }
    }

    #[test]
    fn branch_update_checks_without_release_tags() {
        let runner = branch_runner();
        let report = branch_check(&runner, "forgegod/eliasstravik-herdr-projects", "main").unwrap();
        assert!(report.contains("repository: forgegod/eliasstravik-herdr-projects\n"));
        assert!(report.contains("ref: refs/heads/main\n"));
        assert!(report.contains(&format!("candidate commit: {HEAD}\n")));
        assert_eq!(runner.count("--tags"), 0);
        assert_eq!(runner.count("--version"), 0);
        assert!(runner.calls.borrow().iter().all(|c| c.timeout <= GIT_TIMEOUT));
        for forbidden in ["fetch", "pull", "checkout", "switch", "install", "doctor", "ticker", "remote set"] {
            assert_eq!(runner.count(forbidden), 0, "mutating call {forbidden}");
        }
    }

    fn list(kind_source: &str) -> String {
        format!(
            r#"{{"id":"cli:plugin","result":{{"plugins":[{{"plugin_id":"herdr-projects","plugin_root":"/p/root","version":"0.2.2","source":{kind_source}}}],"type":"plugin_list"}}}}"#
        )
    }

    #[test]
    fn a_linked_checkout_is_detected() {
        let install = parse_install(&list(r#"{"kind":"local"}"#)).unwrap();
        assert_eq!(install, Install::Linked { root: "/p/root".into() });
    }

    #[test]
    fn a_github_install_is_detected_with_its_repository() {
        let json = list(r#"{"kind":"github","owner":"eliasstravik","repo":"herdr-projects","managed_path":"/p/root","resolved_commit":"abc","requested_ref":"v0.2.2"}"#);
        let install = parse_install(&json).unwrap();
        assert_eq!(install, Install::Github { root: "/p/root".into(), repo: "eliasstravik/herdr-projects".into() });
    }

    #[test]
    fn a_missing_plugin_or_unknown_source_is_an_error() {
        let empty = r#"{"id":"cli:plugin","result":{"plugins":[],"type":"plugin_list"}}"#;
        assert!(parse_install(empty).unwrap_err().to_string().contains("no plugin"));
        assert!(parse_install(&list(r#"{"kind":"archive"}"#)).is_err());
    }

    #[test]
    fn release_tags_compare_as_numbers() {
        assert_eq!(parse_release("v0.2.3"), Some(Version(0, 2, 3)));
        assert_eq!(parse_release("0.2.3"), None);
        assert_eq!(parse_release("v0.3.0-rc1"), None);
        assert_eq!(parse_release("v1.2"), None);
        assert!(parse_release("v0.10.0") > parse_release("v0.9.9"));
        assert!(parse_release("v1.0.0") > parse_release("v0.99.99"));
    }

    #[test]
    fn the_newest_release_is_picked_from_ls_remote() {
        let out = "aaa\trefs/tags/v0.2.0\nbbb\trefs/tags/v0.10.1\nccc\trefs/tags/v0.9.0\nddd\trefs/tags/v1.0.0-rc1\neee\trefs/tags/nightly\n";
        assert_eq!(newest_release(out), Some(Version(0, 10, 1)));
        assert_eq!(newest_release(""), None);
    }

    #[test]
    fn doctor_check_is_silent_outside_a_checkout_and_offline() {
        use crate::runner::fake::{FakeRunner, fail, ok};
        let dir = tempfile::tempdir().unwrap();
        let runner = FakeRunner::new();
        assert_eq!(newer_release(&runner, None), None);
        assert_eq!(newer_release(&runner, Some(dir.path())), None);
        assert_eq!(runner.count("ls-remote"), 0);

        std::fs::create_dir(dir.path().join(".git")).unwrap();
        runner.on("ls-remote", fail(128, "could not resolve host"));
        assert_eq!(newer_release(&runner, Some(dir.path())), None);

        let runner = FakeRunner::new();
        runner.on("ls-remote", ok("aaa\trefs/tags/v999.0.0\n"));
        assert_eq!(newer_release(&runner, Some(dir.path())), Some(Version(999, 0, 0)));
        let runner = FakeRunner::new();
        runner.on("ls-remote", ok(&format!("aaa\trefs/tags/v{}\n", env!("CARGO_PKG_VERSION"))));
        assert_eq!(newer_release(&runner, Some(dir.path())), None);
    }
}
