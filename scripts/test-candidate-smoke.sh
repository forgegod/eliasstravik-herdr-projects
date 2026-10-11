#!/bin/sh
# Negative fixtures must fail closed before Docker or scratch creation.
set -eu
cd "$(dirname "$0")/.."
harness="$PWD/scripts/check-candidate-smoke.sh"
reject() {
    label=$1
    shift
    status=0
    output=$("$@" 2>&1) || status=$?
    if [ "$status" != 2 ] || [ "$output" != 'candidate-smoke: unsafe configuration' ]; then
        printf 'not ok - %s (expected safety refusal, exit %s)\n' "$label" "$status"
        printf '%s\n' "$output"
        exit 1
    fi
    printf 'ok - %s\n' "$label"
}
reject 'unsafe mount override' env -i PATH=/usr/bin:/bin sh "$harness" --mount /:/repo:rw
reject 'unsafe runner override' env -i PATH=/usr/bin:/bin sh "$harness" --runner sh
reject 'remote Docker runner' env -i PATH=/usr/bin:/bin DOCKER_HOST=tcp://unsafe.invalid:2375 sh "$harness"
reject 'inherited Docker context' env -i PATH=/usr/bin:/bin DOCKER_CONTEXT=unsafe sh "$harness"
reject 'host Docker credentials' env -i PATH=/usr/bin:/bin DOCKER_CONFIG=/host-credentials sh "$harness"
reject 'repository scratch mount' env -i PATH=/usr/bin:/bin TMPDIR="$PWD" sh "$harness"
