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

# --- Binaries (override via env if needed) -------------------------------
BITCOIND_BIN="${BITCOIND_BIN:-/opt/homebrew/bin/bitcoind}"
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-/opt/homebrew/bin/bitcoin-cli}"
ELECTRS_BIN="${ELECTRS_BIN:-$HOME/.cargo/bin/electrs}"

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
until "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" getblockchaininfo >/dev/null 2>&1; do
  sleep 1
done

echo "[regtest] bitcoind is ready"

# --- Ensure miner wallet is loaded --------------------------------------
if "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" listwallets | grep -q '"miner"'; then
  echo "[regtest] miner wallet already loaded"
else
  if "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" listwalletdir 2>/dev/null | grep -q '"name": "miner"'; then
    echo "[regtest] Loading existing miner wallet"
    "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" loadwallet miner >/dev/null
  else
    echo "[regtest] Creating miner wallet"
    "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" createwallet miner >/dev/null
  fi
fi

# --- Mine initial blocks if chain is empty -------------------------------
BLOCKS=$($BITCOIN_CLI_BIN -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" getblockcount)
if [ "$BLOCKS" -lt 101 ]; then
  echo "[regtest] Mining initial blocks (101)..."
  MINER_ADDR=$($BITCOIN_CLI_BIN -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" -rpcwallet=miner getnewaddress)
  "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" -rpcwallet=miner generatetoaddress 101 "$MINER_ADDR" >/dev/null
fi

# --- Start electrs -------------------------------------------------------
echo "[regtest] Starting electrs..."
if pgrep -f "electrs.*--conf $ELECTRS_CONF_FILE" >/dev/null 2>&1 || nc -z 127.0.0.1 "$ELECTRUM_PORT" >/dev/null 2>&1; then
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

  "$ELECTRS_BIN" \
    --conf "$ELECTRS_CONF_FILE" &
fi

# --- Wait for electrs port ----------------------------------------------
echo "[regtest] Waiting for electrs (port $ELECTRUM_PORT)..."
until nc -z 127.0.0.1 "$ELECTRUM_PORT" >/dev/null 2>&1; do
  sleep 1
done

echo "[regtest] electrs is ready"

echo "[regtest] DONE"