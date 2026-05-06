#!/usr/bin/env bash
set -euo pipefail

# --- Paths ---------------------------------------------------------------
BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BITCOIN_DIR="$BASE_DIR/bitcoin"
BITCOIN_DATA_DIR="$BITCOIN_DIR/data"
BITCOIN_CONF="$BITCOIN_DIR/bitcoin.conf"
ELECTRS_DIR="$BASE_DIR/electrs"
ELECTRS_DB_DIR="$ELECTRS_DIR/db"
ELECTRS_CONF_FILE="$ELECTRS_DIR/electrs.toml"
ELECTRS_LOG_FILE="$ELECTRS_DIR/electrs.log"
ELECTRS_LOG_DIR="$ELECTRS_DIR/logs"

# --- Binaries (override via env if needed) -------------------------------
BITCOIND_BIN="${BITCOIND_BIN:-$(command -v bitcoind || true)}"
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-$(command -v bitcoin-cli || true)}"
ELECTRS_BIN="${ELECTRS_BIN:-$(command -v electrs || true)}"

# --- RPC credentials (must match bitcoin.conf) ---------------------------
RPC_USER="${BITCOIN_RPC_USER:-bitcoin}"
RPC_PASS="${BITCOIN_RPC_PASS:-bitcoin}"

# --- Ports ---------------------------------------------------------------
RPC_PORT="${BITCOIN_RPC_PORT:-18443}"
P2P_PORT="${BITCOIN_P2P_PORT:-18444}"
ELECTRUM_PORT="${ELECTRUM_PORT:-60401}"
MONITORING_PORT="${ELECTRS_MONITORING_PORT:-24224}"

# --- Ensure dirs ---------------------------------------------------------
mkdir -p "$BITCOIN_DATA_DIR"
mkdir -p "$ELECTRS_DB_DIR"
mkdir -p "$ELECTRS_LOG_DIR"

if [[ -z "$BITCOIND_BIN" || ! -x "$BITCOIND_BIN" ]]; then
  echo "[regtest] bitcoind not found. Set BITCOIND_BIN or install Bitcoin Core." >&2
  exit 1
fi

if [[ -z "$BITCOIN_CLI_BIN" || ! -x "$BITCOIN_CLI_BIN" ]]; then
  echo "[regtest] bitcoin-cli not found. Set BITCOIN_CLI_BIN or install Bitcoin Core." >&2
  exit 1
fi

if [[ -z "$ELECTRS_BIN" || ! -x "$ELECTRS_BIN" ]]; then
  echo "[regtest] electrs not found. Set ELECTRS_BIN or install electrs." >&2
  exit 1
fi

bitcoin_cli() {
  "$BITCOIN_CLI_BIN" \
    -regtest \
    -rpcuser="$RPC_USER" \
    -rpcpassword="$RPC_PASS" \
    -rpcport="$RPC_PORT" \
    "$@"
}

warn_if_known_risky_version_pair() {
  local electrs_version="$1"
  local bitcoind_version="$2"
  local electrs_major electrs_minor bitcoind_major

  electrs_major="$(sed -E 's/^v?([0-9]+).*/\1/' <<<"$electrs_version")"
  electrs_minor="$(sed -E 's/^v?[0-9]+\.([0-9]+).*/\1/' <<<"$electrs_version")"
  bitcoind_major="$(sed -E 's/^([0-9]+).*/\1/' <<<"$bitcoind_version")"

  if [[ "$electrs_major" =~ ^[0-9]+$ && "$electrs_minor" =~ ^[0-9]+$ && "$bitcoind_major" =~ ^[0-9]+$ ]]; then
    if (( electrs_major == 0 && electrs_minor <= 11 && bitcoind_major >= 31 )); then
      echo "[regtest] WARNING: detected electrs $electrs_version with Bitcoin Core $bitcoind_version" >&2
      echo "[regtest] WARNING: this combination is known to be risky in this environment." >&2
      echo "[regtest] WARNING: if electrs drops after startup, upgrade electrs before debugging wallet code." >&2
    fi
  fi
}

probe_electrs_rpc() {
  local response
  response="$(
    printf '{"id":1,"method":"server.version","params":["regtest-health","1.4"]}\n' \
      | nc -w 2 127.0.0.1 "$ELECTRUM_PORT" 2>/dev/null || true
  )"
  grep -q '"id":1' <<<"$response"
}

# --- Start bitcoind ------------------------------------------------------
echo "[regtest] Starting bitcoind..."
if pgrep -f "bitcoind.*-datadir=$BITCOIN_DATA_DIR" >/dev/null 2>&1; then
  echo "[regtest] bitcoind already running for this datadir"
else
  "$BITCOIND_BIN" \
    -conf="$BITCOIN_CONF" \
    -datadir="$BITCOIN_DATA_DIR" \
    -regtest=1 \
    -server=1 \
    -txindex=1 \
    -fallbackfee=0.0002 \
    -rpcuser="$RPC_USER" \
    -rpcpassword="$RPC_PASS" \
    -rpcport="$RPC_PORT" \
    -port="$P2P_PORT" \
    -daemon
fi

# --- Wait for RPC --------------------------------------------------------
echo "[regtest] Waiting for bitcoind RPC..."
BITCOIND_READY=0
for _ in {1..60}; do
  if bitcoin_cli getblockchaininfo >/dev/null 2>&1; then
    BITCOIND_READY=1
    break
  fi
  sleep 1
done

if [[ "$BITCOIND_READY" != "1" ]]; then
  echo "[regtest] bitcoind RPC did not become ready on port $RPC_PORT" >&2
  echo "[regtest] Last debug.log lines:" >&2
  tail -n 80 "$BITCOIN_DATA_DIR/regtest/debug.log" >&2 || true
  exit 1
fi

echo "[regtest] bitcoind is ready"

BITCOIND_VERSION="$(
  bitcoin_cli getnetworkinfo \
    | sed -nE 's#.*"subversion":[[:space:]]*"/?Satoshi:([0-9]+\.[0-9]+\.[0-9]+)/?".*#\1#p' \
    | head -n 1
)"
ELECTRS_VERSION="$("$ELECTRS_BIN" --version 2>/dev/null || true)"
if [[ -n "$BITCOIND_VERSION" && -n "$ELECTRS_VERSION" ]]; then
  echo "[regtest] bitcoind version: $BITCOIND_VERSION"
  echo "[regtest] electrs version: $ELECTRS_VERSION"
  warn_if_known_risky_version_pair "$ELECTRS_VERSION" "$BITCOIND_VERSION"
fi

# --- Ensure miner wallet is loaded --------------------------------------
if bitcoin_cli listwallets | grep -q '"miner"'; then
  echo "[regtest] miner wallet already loaded"
else
  if bitcoin_cli listwalletdir 2>/dev/null | grep -q '"name": "miner"'; then
    echo "[regtest] Loading existing miner wallet"
    bitcoin_cli loadwallet miner >/dev/null
  else
    echo "[regtest] Creating miner wallet"
    bitcoin_cli createwallet miner >/dev/null
  fi
fi

# --- Mine initial blocks if chain is empty -------------------------------
BLOCKS=$(bitcoin_cli getblockcount)
if [ "$BLOCKS" -lt 101 ]; then
  echo "[regtest] Mining initial blocks (101)..."
  MINER_ADDR=$(bitcoin_cli -rpcwallet=miner getnewaddress)
  bitcoin_cli -rpcwallet=miner generatetoaddress 101 "$MINER_ADDR" >/dev/null
fi

# On regtest, a stale tip can keep Bitcoin Core in initial block download mode
# across restarts. Electrs waits on that state by default, so refresh the tip
# before starting Electrum sync services.
if bitcoin_cli getblockchaininfo | grep -q '"initialblockdownload": true'; then
  echo "[regtest] Refreshing regtest tip to exit initial block download..."
  MINER_ADDR=$(bitcoin_cli -rpcwallet=miner getnewaddress)
  bitcoin_cli -rpcwallet=miner generatetoaddress 1 "$MINER_ADDR" >/dev/null
fi

# --- Start electrs -------------------------------------------------------
# Always write electrs config so env/port/path changes are reflected.
cat > "$ELECTRS_CONF_FILE" <<EOF
network = "regtest"
db_dir = "$ELECTRS_DB_DIR"
daemon_dir = "$BITCOIN_DATA_DIR"
daemon_rpc_addr = "127.0.0.1:$RPC_PORT"
daemon_p2p_addr = "127.0.0.1:$P2P_PORT"
electrum_rpc_addr = "127.0.0.1:$ELECTRUM_PORT"
monitoring_addr = "127.0.0.1:$MONITORING_PORT"
auth = "$RPC_USER:$RPC_PASS"
skip_block_download_wait = true
EOF

echo "[regtest] Starting electrs..."
if pgrep -f "electrs.*--conf $ELECTRS_CONF_FILE" >/dev/null 2>&1 || lsof -nP -iTCP:"$ELECTRUM_PORT" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "[regtest] electrs already running or port $ELECTRUM_PORT is already in use"
  ELECTRS_PID=""
else
  RUN_TS="$(date +%Y%m%d%H%M%S)"
  RUN_ELECTRS_LOG_FILE="$ELECTRS_LOG_DIR/electrs-$RUN_TS.log"
  : > "$RUN_ELECTRS_LOG_FILE"
  ln -sf "$RUN_ELECTRS_LOG_FILE" "$ELECTRS_LOG_FILE"
  echo "[regtest] electrs log: $RUN_ELECTRS_LOG_FILE"
  nohup "$ELECTRS_BIN" \
    --conf "$ELECTRS_CONF_FILE" \
    >>"$RUN_ELECTRS_LOG_FILE" 2>&1 &
  ELECTRS_PID=$!
  echo "[regtest] electrs pid: $ELECTRS_PID"
  disown "$ELECTRS_PID" 2>/dev/null || true
fi

# --- Wait for electrs port / process health -----------------------------
echo "[regtest] Waiting for electrs (port $ELECTRUM_PORT)..."
ELECTRS_READY=0
for _ in {1..60}; do
  if [[ -n "${ELECTRS_PID:-}" ]] && ! kill -0 "$ELECTRS_PID" >/dev/null 2>&1; then
    echo "[regtest] electrs process exited before becoming ready" >&2
    echo "[regtest] electrs config: $ELECTRS_CONF_FILE" >&2
    echo "[regtest] Last electrs.log lines:" >&2
    tail -n 120 "$ELECTRS_LOG_FILE" >&2 || true
    exit 1
  fi

  if lsof -nP -iTCP:"$ELECTRUM_PORT" -sTCP:LISTEN >/dev/null 2>&1; then
    if probe_electrs_rpc; then
      ELECTRS_READY=1
      break
    fi
  fi
  sleep 1
done

if [[ "$ELECTRS_READY" != "1" ]]; then
  echo "[regtest] electrs did not become RPC-ready on port $ELECTRUM_PORT" >&2
  echo "[regtest] electrs config: $ELECTRS_CONF_FILE" >&2
  echo "[regtest] electrs monitoring port: $MONITORING_PORT" >&2
  echo "[regtest] Last electrs.log lines:" >&2
  tail -n 120 "$ELECTRS_LOG_FILE" >&2 || true
  exit 1
fi

echo "[regtest] electrs RPC is ready"

echo "[regtest] DONE"
