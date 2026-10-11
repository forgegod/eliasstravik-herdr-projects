#!/bin/sh
# Explicit manual gate: fixed local Docker runner, never an installer.
set -eu
unsafe() { printf '%s\n' 'candidate-smoke: unsafe configuration' >&2; exit 2; }
[ "$#" = 0 ] || unsafe
[ -z "${DOCKER_HOST-}${DOCKER_CONTEXT-}${DOCKER_CONFIG-}${DOCKER_TLS_VERIFY-}${DOCKER_CERT_PATH-}" ] || unsafe
repo=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd -P)
[ -n "${TMPDIR-}" ] && [ -d "$TMPDIR" ] || unsafe
base=$(CDPATH= cd -- "$TMPDIR" && pwd -P)
case "$repo$base" in *','*|*'
'*) unsafe ;; esac
case "$base/" in "$repo/"*) unsafe ;; esac
case "$repo/" in "$base/"*) unsafe ;; esac
uid=$(id -u)
gid=$(id -g)
[ "$uid" != 0 ] && [ "$(stat -c %u "$base")" = "$uid" ] || unsafe
[ -x /usr/bin/docker ] && [ -S /var/run/docker.sock ] || {
    printf '%s\n' 'candidate-smoke: local Docker runner unavailable' >&2
    exit 1
}
umask 077
scratch=$(mktemp -d "$base/candidate-smoke.XXXXXX")
name="herdr-$(basename "$scratch")"
docker_local() {
    env -i PATH=/usr/bin:/bin HOME="$scratch/home" DOCKER_CONFIG="$scratch/docker" \
        /usr/bin/docker --host unix:///var/run/docker.sock "$@"
}
cleanup() {
    status=$?
    trap - 0 HUP INT TERM
    docker_local rm -f "$name" >/dev/null 2>&1 || :
    rm -rf -- "$scratch"
    exit "$status"
}
trap cleanup 0
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir "$scratch/home" "$scratch/docker" "$scratch/cargo-home" "$scratch/target" "$scratch/tmp"
# Previously reviewed Rust 1.89 / Node 22 / rsync image; never pull or build it.
image=sha256:32083d2ba467a8af4f60e97857f7e50f7dc74c7633a670b4963bb425f8d18237
run() {
    network=$1
    shift
    docker_local run --rm --pull never --name "$name" --network "$network" \
        --read-only --cap-drop ALL --security-opt no-new-privileges \
        --user "$uid:$gid" \
        --mount "type=bind,src=$repo,dst=/repo,readonly" \
        --mount "type=bind,src=$scratch,dst=/scratch" \
        --workdir /repo --entrypoint /usr/bin/env "$image" -i \
        PATH=/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin \
        RUSTUP_HOME=/usr/local/rustup HOME=/scratch/home \
        CARGO_HOME=/scratch/cargo-home CARGO_TARGET_DIR=/scratch/target \
        CARGO_BUILD_JOBS=2 TMPDIR=/scratch/tmp \
        GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
        GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=safe.directory GIT_CONFIG_VALUE_0=/repo \
        GIT_TERMINAL_PROMPT=0 "$@"
}
# Only dependency acquisition has networking; no compilation/tests in this step.
run bridge sh -ec '
    rustc --version
    cargo --version
    node --version
    rsync --version
    cargo fetch --locked
'
run none sh -ec '
    export CARGO_NET_OFFLINE=true
    sh scripts/test-candidate-smoke.sh
    for name in candidate_native_smoke_builds_and_cleans_without_live_mutation candidate_native_smoke_invalid_executable_cleans_without_success; do
        test="update::tests::$name"
        listing=$(cargo test --offline --locked --bin herdr-projects "$test" -- --exact --list)
        case "$listing" in *"$test: test"*) ;; *) printf "Missing smoke test: %s\n" "$test" >&2; exit 1 ;; esac
        cargo test --offline --locked --bin herdr-projects "$test" -- --exact
    done
    sh scripts/check-records.sh
'
printf '%s\n' 'candidate-smoke: PASS'
