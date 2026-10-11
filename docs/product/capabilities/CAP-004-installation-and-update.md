# CAP-004 — Installation and release updates

**Status:** partial
**Primary surface:** none

## Behaviour

- The updater distinguishes a Herdr-managed GitHub installation from a linked local checkout, and rejects unknown installation kinds.
- Release discovery reads `vX.Y.Z` tags from the installation checkout's `origin`; it is not hardcoded to the original upstream repository. Managed installation uses the owner/repository reported by Herdr.
- Update eligibility compares release versions. When the newest version is not greater than the installed version, update returns without pulling. New fork-main commits at the same version are not detected as updates.
- `update --check --channel branch --repository <owner>/<repo>[/<subdir>] --ref <ref>` is a read-only opt-in branch check. It rejects URLs, credentials, partial branch options, source/ref/provenance disagreement, malformed candidate results and unavailable ancestry rather than falling back to release updates.
- The branch check reports the selected repository/ref, candidate commit and a truthful `installed commit: unverified` boundary. It may classify locally available checkout ancestry as equal, checkout-ahead, candidate-ahead or diverged, but never presents checkout history as installed-binary provenance.
- `update --stage --channel branch --repository <owner>/<repo>[/<subdir>] --ref <ref>` verifies one immutable candidate in disposable scratch, reports only bounded verification/cleanup status, and removes the candidate before returning. It cannot combine with `--check` and never promotes, installs, replaces the active binary or changes the live checkout.
- A linked release update requires a tracked-clean checkout on `main`, pulls `origin main` fast-forward-only, and invokes the existing installer. The installer chooses release assets from the GitHub origin, or builds from source when an eligible verified asset is unavailable; an unrecognized/missing origin has an upstream fallback.
- The existing update path stops the ticker before installation and then invokes the installed binary's doctor/repair and ticker start, reporting failed stages. It is not a transactional branch-channel updater.

## Implementation

- `src/update.rs` — installation-source parsing, version discovery and update orchestration.
- `src/cli.rs` — explicit branch-check arguments and release-path preservation.
- `scripts/install.sh` — verified binary download/source-build fallback.
- `scripts/link-command.sh` — owned command link.
- `src/command_link.rs` — link inspection/repair.
- `.github/workflows/release.yml` — published release assets/checksums.

## Rules and boundaries

The `partial` status records an evidence boundary: installation parsing and
release comparison have executable tests, but the full fetch/build/rollback
lifecycle does not have equivalent end-to-end coverage. The baseline has no
automatic fork-main update channel; branch discovery is explicitly check-only.
Origin URL, Herdr installation metadata, branch, checkout ancestry and binary
provenance are distinct facts and may disagree. Branch-history object acquisition
uses a disposable local bare repository, never the live installation.
See [architecture](../../architecture.md).

## Verification

- `src/update.rs` — tests `a_linked_checkout_is_detected`, `a_github_install_is_detected_with_its_repository`, `a_missing_plugin_or_unknown_source_is_an_error`, `release_tags_compare_as_numbers`, `the_newest_release_is_picked_from_ls_remote`, and `doctor_check_is_silent_outside_a_checkout_and_offline`.
- `src/update.rs` — tests `branch_update_reconciles_fork_upstream_url_identities`, `branch_update_requires_matching_subdirectory_and_tracking_ref`, `branch_update_checks_detached_managed_install_without_switching`, `branch_update_offline_is_an_error_without_fallback`, `branch_update_requires_one_exact_candidate_ref_and_full_oid`, `branch_history_classifies_equal_checkout_ahead_and_diverged_without_install_claims`, and `branch_history_refuses_missing_candidate_parent_shallow_and_offline_without_mutation`.
- `src/update.rs` — tests `candidate_staging_preserves_entire_live_checkout_and_active_binary`, `candidate_native_build_verifies_artifact_before_accepting_it`, `candidate_stage_cli_dispatch_verifies_and_cleans_before_returning`, and `candidate_stage_interface_preserves_legacy_update_dispatch`.
- `src/cli.rs` — tests `update_refuses_partial_or_malformed_flags`, `update_repository_is_a_canonical_credential_free_identifier`, and `update_ref_accepts_only_unambiguous_branch_names`.
- `src/command_link.rs` — tests `a_file_or_a_link_to_the_users_own_checkout_is_never_touched` and `the_install_script_links_where_herdr_will_move_the_checkout`.

Run `cargo test --locked update` and `cargo test --locked command_link`.
Existing release-build success is not proof of a live updater transaction.
