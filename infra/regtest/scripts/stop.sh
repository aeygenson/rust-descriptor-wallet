#!/usr/bin/env bash
set -euo pipefail

# --- Paths ---------------------------------------------------------------
BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BITCOIN_DATA_DIR="$BASE_DIR/bitcoin/data"
ELECTRS_DIR="$BASE_DIR/electrs"
ELECTRS_CONF_FILE="$ELECTRS_DIR/electrs.toml"

# --- Binaries / ports ----------------------------------------------------
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-$(command -v bitcoin-cli || true)}"
RPC_USER="${BITCOIN_RPC_USER:-bitcoin}"
RPC_PASS="${BITCOIN_RPC_PASS:-bitcoin}"
RPC_PORT="${BITCOIN_RPC_PORT:-18443}"
ELECTRUM_PORT="${ELECTRUM_PORT:-60401}"
MONITORING_PORT="${ELECTRS_MONITORING_PORT:-24224}"

if [[ -z "$BITCOIN_CLI_BIN" || ! -x "$BITCOIN_CLI_BIN" ]]; then
  echo "[regtest] bitcoin-cli not found. Set BITCOIN_CLI_BIN or install Bitcoin Core." >&2
  exit 1
fi

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
    echo "[regtest] Force releasing port $PORT"
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

# Force-kill only this regtest datadir if graceful RPC stop failed.
if pgrep -f "bitcoind.*-datadir=$BITCOIN_DATA_DIR" >/dev/null 2>&1; then
  echo "[regtest] Force stopping bitcoind for $BITCOIN_DATA_DIR"
  pkill -f "bitcoind.*-datadir=$BITCOIN_DATA_DIR" >/dev/null 2>&1 || true
  sleep 1
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
