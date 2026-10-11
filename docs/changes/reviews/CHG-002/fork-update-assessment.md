# Fork-aware update assessment

**Status:** review-only

## Reference and applicability

Reviewed [afloury commit 08da209462e736f29b6e9d2325e0cd2b79613f0e](https://github.com/afloury/herdr-projects/commit/08da209462e736f29b6e9d2325e0cd2b79613f0e)
(parent `cfac31e1b4b4d2994c9e70b06a61a69807e4379c`). It adds a fork notice and
`scripts/update-fork.sh`: select fork/origin main, fetch, detach to FETCH_HEAD,
build with Cargo or Docker, stop the ticker, swap the binary, run doctor and restart.

This checkout's origin is `git@github.com:forgegod/eliasstravik-herdr-projects.git`.
The concept is applicable: fork commits can advance without changing the
upstream version, while the current updater checks only greater `vX.Y.Z`
versions. A fork-source update channel and accurate provenance are worthwhile.

The referenced README overstates one point for this baseline: `src/update.rs`
already reads release tags from `origin`, and managed installation derives its
repository from Herdr's installation metadata. `scripts/install.sh` also uses
a recognized GitHub origin for release downloads. Do not claim updates always
install upstream. The risk depends on source metadata/origin mismatch or the
installer fallback, and the definite branch-update gap is version gating.

## Reuse the concept, not the script unchanged

- Keep original project/license attribution; identify this fork without claiming
  upstream is paused unless independently established.
- Provide explicit source/ref/channel selection and print it before mutation.
- Stage and verify the candidate before stopping the ticker or replacing the
  installed executable; use atomic replacement on the target filesystem.
- Preserve operator-owned checkout branches and existing data/permissions.

## Problems requiring a different implementation

1. The script checks out FETCH_HEAD detached before building. Failure leaves the
   source checkout changed and can break existing linked-update expectations.
   The optional build lock does not serialize fetch/checkout/promotion.
2. The cleanliness check ignores untracked files, and has no robust transaction
   against concurrent edits. Staging should avoid touching the user's working tree.
3. `ticker stop || true` ignores inability to stop the old process before doctor
   writes potentially newer state. Stop and replacement need explicit outcomes.
4. Doctor failure is only a warning and there is no complete rollback/receipt
   protocol. Distinguish binary rollback from irreversible doctor/data edits;
   never promise that restoring an executable rolls back user state.
5. Ticker commands omit the operator's explicit project root. Updating from an
   arbitrary shell may stop/start the wrong ticker or consult different config.
6. `rust:1-alpine` is a mutable container reference. Docker-built Linux artifacts
   are not host-native macOS binaries. Require verified host/target compatibility
   and explicit opt-in to a supported build backend rather than guessing.
7. The Docker shell interpolates paths/UID/GID into command text and uses a broad
   `safe.directory '*'`; retain argv-safe boundaries and specific trust scopes.
8. `--version` success does not establish that the candidate is the requested
   commit/source or that build/tests passed for its behaviour.

These are source-review findings, not a live destructive-update experiment.
No patch application or live update was performed.

## Recommended shape

Prefer extending the existing Rust updater with an explicit opt-in fork/branch
channel, reusing Runner, timeouts and install-source discovery. Avoid a parallel
unmanaged shell updater with different safeguards. Keep the release channel
compatible, but fail clearly on inconsistent provenance. Use temporary local
Git remotes/fake runners and disposable binaries to prove failure cases without
updating the active install. Any Docker fallback is optional and separately
verified; it is not necessary for the first safe native-Cargo path.
