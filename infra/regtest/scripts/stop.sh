

#!/usr/bin/env bash
set -euo pipefail

# --- Paths ---------------------------------------------------------------
BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ELECTRS_DIR="$BASE_DIR/electrs"
ELECTRS_CONF_FILE="$ELECTRS_DIR/electrs.toml"

# --- Binaries / ports ----------------------------------------------------
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-/opt/homebrew/bin/bitcoin-cli}"
RPC_USER="${BITCOIN_RPC_USER:-bitcoin}"
RPC_PASS="${BITCOIN_RPC_PASS:-bitcoin}"
RPC_PORT="${BITCOIN_RPC_PORT:-18443}"
ELECTRUM_PORT="${ELECTRUM_PORT:-50001}"
MONITORING_PORT="${ELECTRS_MONITORING_PORT:-24224}"

# --- Stop electrs --------------------------------------------------------
echo "[regtest] Stopping electrs..."
if pgrep -f "electrs.*--conf $ELECTRS_CONF_FILE" >/dev/null 2>&1; then
  pkill -f "electrs.*--conf $ELECTRS_CONF_FILE" || true
  sleep 1
else
  echo "[regtest] electrs not running via expected config"
fi

# Fallback: free electrs ports if something still holds them
for PORT in "$ELECTRUM_PORT" "$MONITORING_PORT"; do
  if lsof -ti tcp:"$PORT" >/dev/null 2>&1; then
    echo "[regtest] Releasing port $PORT"
    lsof -ti tcp:"$PORT" | xargs kill -9 || true
  fi
done

# --- Stop bitcoind -------------------------------------------------------
echo "[regtest] Stopping bitcoind..."
if "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" stop >/dev/null 2>&1; then
  echo "[regtest] bitcoind stop requested"
else
  echo "[regtest] bitcoind RPC stop failed or node not running"
fi

# Wait briefly for bitcoind to stop accepting RPC
for _ in {1..15}; do
  if "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" getblockchaininfo >/dev/null 2>&1; then
    sleep 1
  else
    break
  fi
done

echo "[regtest] STOP complete"