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

# --- Ensure dirs ---------------------------------------------------------
mkdir -p "$BITCOIN_DATA_DIR"
mkdir -p "$ELECTRS_DB_DIR"

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

# --- Start electrs -------------------------------------------------------
echo "[regtest] Starting electrs..."
if pgrep -f "electrs.*--conf $ELECTRS_CONF_FILE" >/dev/null 2>&1 || lsof -nP -iTCP:"$ELECTRUM_PORT" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "[regtest] electrs already running or port $ELECTRUM_PORT is already in use"
else
  cat > "$ELECTRS_CONF_FILE" <<EOF
network = "regtest"
db_dir = "$ELECTRS_DB_DIR"
daemon_dir = "$BITCOIN_DATA_DIR"
daemon_rpc_addr = "127.0.0.1:$RPC_PORT"
electrum_rpc_addr = "127.0.0.1:$ELECTRUM_PORT"
auth = "$RPC_USER:$RPC_PASS"
EOF

  echo "[regtest] electrs log: $ELECTRS_LOG_FILE"
  nohup "$ELECTRS_BIN" \
    --conf "$ELECTRS_CONF_FILE" \
    >>"$ELECTRS_LOG_FILE" 2>&1 &
  ELECTRS_PID=$!
  disown "$ELECTRS_PID" 2>/dev/null || true
fi

# --- Wait for electrs port ----------------------------------------------
echo "[regtest] Waiting for electrs (port $ELECTRUM_PORT)..."
ELECTRS_READY=0
for _ in {1..60}; do
  if lsof -nP -iTCP:"$ELECTRUM_PORT" -sTCP:LISTEN >/dev/null 2>&1; then
    ELECTRS_READY=1
    break
  fi
  sleep 1
done

if [[ "$ELECTRS_READY" != "1" ]]; then
  echo "[regtest] electrs did not open port $ELECTRUM_PORT" >&2
  echo "[regtest] electrs config: $ELECTRS_CONF_FILE" >&2
  echo "[regtest] Last electrs.log lines:" >&2
  tail -n 80 "$ELECTRS_LOG_FILE" >&2 || true
  exit 1
fi

echo "[regtest] electrs is ready"

echo "[regtest] DONE"
