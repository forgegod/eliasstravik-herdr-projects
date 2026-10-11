# CAP-004 — Installation and release updates

**Status:** partial
**Primary surface:** none

## Behaviour

- The updater distinguishes a Herdr-managed GitHub installation from a linked local checkout, and rejects unknown installation kinds.
- Release discovery reads `vX.Y.Z` tags from the installation checkout's `origin`; it is not hardcoded to the original upstream repository. Managed installation uses the owner/repository reported by Herdr.
- Update eligibility compares release versions. When the newest version is not greater than the installed version, update returns without pulling. New fork-main commits at the same version are not detected as updates.
- A linked release update requires a tracked-clean checkout on `main`, pulls `origin main` fast-forward-only, and invokes the existing installer. The installer chooses release assets from the GitHub origin, or builds from source when an eligible verified asset is unavailable; an unrecognized/missing origin has an upstream fallback.
- The existing update path stops the ticker before installation and then invokes the installed binary's doctor/repair and ticker start, reporting failed stages. It is not a transactional branch-channel updater.

## Implementation

- `src/update.rs` — installation-source parsing, version discovery and update orchestration.
- `scripts/install.sh` — verified binary download/source-build fallback.
- `scripts/link-command.sh` — owned command link.
- `src/command_link.rs` — link inspection/repair.
- `.github/workflows/release.yml` — published release assets/checksums.

## Rules and boundaries

The `partial` status records an evidence boundary: installation parsing and
release comparison have executable tests, but the full fetch/build/rollback
lifecycle does not have equivalent end-to-end coverage. The baseline has no
explicit fork-main update channel. Origin URL, Herdr installation metadata,
branch and binary provenance are distinct facts and may disagree.
See [architecture](../../architecture.md).

## Verification

- `src/update.rs` — tests `a_linked_checkout_is_detected`, `a_github_install_is_detected_with_its_repository`, `a_missing_plugin_or_unknown_source_is_an_error`, `release_tags_compare_as_numbers`, `the_newest_release_is_picked_from_ls_remote`, and `doctor_check_is_silent_outside_a_checkout_and_offline`.
- `src/command_link.rs` — tests `a_file_or_a_link_to_the_users_own_checkout_is_never_touched` and `the_install_script_links_where_herdr_will_move_the_checkout`.

Run `cargo test --locked update` and `cargo test --locked command_link`.
Existing release-build success is not proof of a live updater transaction.
