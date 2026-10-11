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

pub fn check_branch(ctx: &Ctx, repository: &str, reference: &str) -> Result<String> {
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
    let out = ctx.runner.run(&git(install.root(), GIT_TIMEOUT).args(["ls-remote", "--heads", "--refs", &url, &reference]))
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
    Ok(format!("repository: {repository}\nref: {reference}\ninstalled commit: unverified\ncandidate commit: {candidate}\n"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::fake::{FakeRunner, fail, ok};

    const HEAD: &str = "1111111111111111111111111111111111111111";

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
