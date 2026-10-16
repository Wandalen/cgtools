#!/bin/sh
# Checks script/wasm_test_runner.sh without a browser: a stand-in
# wasm-bindgen-test-runner, first on PATH, reports what the shim handed it.
#
# - With WASM_BINDGEN_TEST_TIMEOUT unset, the runner sees 120.
# - With it set by the caller, the runner sees the caller's value.
# - Either way the runner starts in the workspace root, whatever the caller's
#   cwd, and gets the shim's arguments unchanged.
#
# usage:
#   script/wasm_test_runner_test.sh

set -u

root=$( cd "$( dirname "$0" )/.." && pwd )
shim="$root/script/wasm_test_runner.sh"

stub=$( mktemp -d )
trap 'rm -rf "$stub"' EXIT
cat > "$stub/wasm-bindgen-test-runner" <<'STUB'
#!/bin/sh
printf 'timeout=%s|cwd=%s|args=%s\n' "${WASM_BINDGEN_TEST_TIMEOUT-<unset>}" "$( pwd )" "$*"
STUB
chmod +x "$stub/wasm-bindgen-test-runner"

failed=0

# expect <label> <expected> <actual>
expect()
{
  if [ "$2" = "$3" ]
  then
    printf 'ok: %s\n' "$1"
  else
    printf 'FAIL: %s\n  expected: %s\n  actual:   %s\n' "$1" "$2" "$3" >&2
    failed=1
  fi
}

# Run from the stub directory, so a shim that forgot to re-anchor would report it.
actual=$( cd "$stub" && unset WASM_BINDGEN_TEST_TIMEOUT && PATH="$stub:$PATH" "$shim" suite.wasm --nocapture )
expect "unset timeout defaults to 120" "timeout=120|cwd=$root|args=suite.wasm --nocapture" "$actual"

actual=$( cd "$stub" && WASM_BINDGEN_TEST_TIMEOUT=7 PATH="$stub:$PATH" "$shim" suite.wasm )
expect "caller's timeout is kept" "timeout=7|cwd=$root|args=suite.wasm" "$actual"

exit "$failed"
