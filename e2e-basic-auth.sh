#!/bin/sh
# End-to-end check of furl's HTTP Basic Authentication support.
#
# This is NOT the project's test suite; run `cargo test --workspace` for that.
# This script exercises the real `furl` binary against a real HTTP server
# (nginx, from docker-compose.yml) that requires Basic Auth and supports Range
# requests, so multi-chunk downloads are covered with every chunk authenticated.
#
# Usage:
#   ./e2e-basic-auth.sh                 # build furl from this workspace and test it
#   ./e2e-basic-auth.sh path/to/furl    # test an existing binary instead
#
# What it does:
#   1. builds furl (unless a binary path is given)
#   2. starts the test server with `docker compose up -d` if it isn't running;
#      the first start generates ~600 MB of random test files in a volume
#   3. downloads every test file with `-u furl:furl-test` and verifies it
#      against the server's SHA256SUMS
#   4. checks that credentials embedded in the url work
#   5. checks that a wrong or missing password fails with 401 Unauthorized
#      and leaves no file behind
#
# The server is left running for faster reruns; stop it and delete the test
# files with `docker compose down -v`.
#
# Environment:
#   FURL_TEST_PORT   port the test server listens on (default: 8080)
#
# Requires: cargo (unless a binary is given), docker compose, curl, and
# sha256sum or shasum. Exits 0 when every check passes, 1 otherwise.
# More details: docker/basic-auth/README.md
set -u

cd "$(dirname "$0")"
BASE="http://localhost:${FURL_TEST_PORT:-8080}"
USER_PASS="furl:furl-test"
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT

if [ $# -gt 0 ]; then
    FURL=$1
else
    echo "building furl..."
    cargo build -q || exit 1
    FURL=target/debug/furl
fi
if [ ! -x "$FURL" ]; then
    echo "furl binary not found at $FURL" >&2
    exit 1
fi

server_ready() { curl -sf -o "$OUT/SHA256SUMS" -u "$USER_PASS" "$BASE/SHA256SUMS"; }
if ! server_ready; then
    echo "starting test server (first run generates ~600 MB of test files)..."
    docker compose up -d || exit 1
    tries=0
    until server_ready; do
        tries=$((tries + 1))
        if [ "$tries" -ge 120 ]; then
            echo "test server did not become ready at $BASE" >&2
            exit 1
        fi
        sleep 1
    done
fi
echo

if command -v sha256sum >/dev/null; then
    sha256() { sha256sum "$1" | cut -d' ' -f1; }
else
    sha256() { shasum -a 256 "$1" | cut -d' ' -f1; }
fi

failures=0
pass() { echo "PASS  $1"; }
fail() { echo "FAIL  $1"; failures=$((failures + 1)); }

# every file, downloaded with credentials, must match its checksum
while read -r expected name; do
    if "$FURL" "$BASE/$name" -u "$USER_PASS" -o "$OUT" -c 1MB -t 8 >"$OUT/log" 2>&1 &&
        [ "$(sha256 "$OUT/$name")" = "$expected" ]; then
        pass "$name ($(grep -o 'Created [0-9]* chunks' "$OUT/log" || echo 'single stream'))"
    else
        fail "$name"
        cat "$OUT/log"
    fi
done <"$OUT/SHA256SUMS"

# credentials embedded in the url
if "$FURL" "http://$USER_PASS@${BASE#http://}/medium.bin" -o "$OUT" -f url.bin >/dev/null 2>&1; then
    pass "credentials in url"
else
    fail "credentials in url"
fi

# bad or missing credentials must fail without leaving a file behind
expect_401() {
    label=$1
    shift
    if "$FURL" "$@" -o "$OUT" -f rejected.bin 2>&1 | grep -q "401 Unauthorized" &&
        [ ! -e "$OUT/rejected.bin" ]; then
        pass "$label"
    else
        fail "$label"
    fi
}
expect_401 "wrong password rejected" "$BASE/large.bin" -u furl:wrong
expect_401 "missing credentials rejected" "$BASE/large.bin"

echo
if [ "$failures" -eq 0 ]; then
    echo "all checks passed"
else
    echo "$failures check(s) failed"
    exit 1
fi
