#!/usr/bin/env bash
# Smoke-test the built MCP endpoint without a session.
# Starts the daemon on spare loopback ports, calls tools/list and tools/call
# on /mcp, then stops the daemon by PID.
# Usage (from repo root): daemon/scripts/mcp-smoke.sh [path/to/centy-daemon]
set -euo pipefail

DAEMON_BIN="${1:-target/release/centy-daemon}"
CARGO_TOML="$(dirname "$0")/../Cargo.toml"
TIMEOUT_SECS=30
LOG="$(mktemp)"
DAEMON_PID=""

cleanup() {
  status=$?
  if [ -n "$DAEMON_PID" ] && kill -0 "$DAEMON_PID" 2>/dev/null; then
    kill "$DAEMON_PID" 2>/dev/null || true
    wait "$DAEMON_PID" 2>/dev/null || true
  fi
  if [ "$status" -ne 0 ]; then
    echo "::group::daemon log"
    cat "$LOG"
    echo "::endgroup::"
  fi
  rm -f "$LOG"
  exit "$status"
}
trap cleanup EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

free_port() {
  python3 -I -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'
}

GRPC_PORT="$(free_port)"
MCP_PORT="$(free_port)"
EXPECTED_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$CARGO_TOML" | head -n 1)"
[ -n "$EXPECTED_VERSION" ] || fail "no version in $CARGO_TOML"

"$DAEMON_BIN" --addr "127.0.0.1:$GRPC_PORT" --mcp-addr "127.0.0.1:$MCP_PORT" >"$LOG" 2>&1 &
DAEMON_PID=$!
echo "started centy-daemon pid=$DAEMON_PID grpc=$GRPC_PORT mcp=$MCP_PORT"

# Wait until the MCP port answers any HTTP request, bounded by TIMEOUT_SECS.
for _ in $(seq "$TIMEOUT_SECS"); do
  kill -0 "$DAEMON_PID" 2>/dev/null || fail "daemon exited before MCP port was ready"
  if curl --silent --output /dev/null --max-time 2 "http://127.0.0.1:$MCP_PORT/"; then
    break
  fi
  sleep 1
done
curl --silent --output /dev/null --max-time 2 "http://127.0.0.1:$MCP_PORT/" \
  || fail "MCP port $MCP_PORT did not answer within ${TIMEOUT_SECS}s"

mcp_post() {
  curl --silent --show-error --fail --max-time 10 -X POST "http://127.0.0.1:$MCP_PORT/mcp" \
    -H 'content-type: application/json' \
    -H 'accept: application/json, text/event-stream' \
    --data "$1"
}

TOOLS_LIST='{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
CALL='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"centy_v1_CentyDaemon_GetDaemonInfo","arguments":{}}}'

LIST_RESPONSE="$(mcp_post "$TOOLS_LIST")" || fail "tools/list request failed"
echo "tools/list: $(echo "$LIST_RESPONSE" | jq -c '.result.tools | length') tools"
echo "$LIST_RESPONSE" | jq -e '(.error == null) and ((.result.tools | length) > 0)' >/dev/null \
  || fail "tools/list returned no tools: $LIST_RESPONSE"

CALL_RESPONSE="$(mcp_post "$CALL")" || fail "tools/call request failed"
echo "tools/call GetDaemonInfo: $CALL_RESPONSE"
echo "$CALL_RESPONSE" | jq -e --arg v "$EXPECTED_VERSION" \
  '(.error == null) and (.result.isError == false) and (.result.structuredContent.version == $v)' >/dev/null \
  || fail "tools/call did not return version $EXPECTED_VERSION: $CALL_RESPONSE"

echo "PASS: MCP tools/call returned version $EXPECTED_VERSION"
