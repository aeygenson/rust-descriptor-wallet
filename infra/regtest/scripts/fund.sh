#!/usr/bin/env bash
set -euo pipefail

# --- RPC config ----------------------------------------------------------
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-$(command -v bitcoin-cli || true)}"
RPC_USER="${BITCOIN_RPC_USER:-bitcoin}"
RPC_PASS="${BITCOIN_RPC_PASS:-bitcoin}"
RPC_PORT="${BITCOIN_RPC_PORT:-18443}"

if [[ -z "$BITCOIN_CLI_BIN" || ! -x "$BITCOIN_CLI_BIN" ]]; then
  echo "[regtest] bitcoin-cli not found. Set BITCOIN_CLI_BIN or install Bitcoin Core." >&2
  exit 1
fi

# --- Arguments -----------------------------------------------------------
if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <address> [amount_btc]" >&2
  echo "Example: $0 bcrt1q..." >&2
  echo "Example: $0 bcrt1q... 1.0" >&2
  exit 1
fi

DEST_ADDR="$1"
AMOUNT_BTC="${2:-1.0}"

# --- Ensure miner wallet exists -----------------------------------------
if ! "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" listwallets | grep -q '"miner"'; then
  echo "[regtest] miner wallet is not loaded" >&2
  echo "Run start.sh first to create/load the miner wallet" >&2
  exit 1
fi

# --- Send funds ----------------------------------------------------------
echo "[regtest] Funding $DEST_ADDR with $AMOUNT_BTC BTC"
TXID=$(
  "$BITCOIN_CLI_BIN" \
    -regtest \
    -rpcuser="$RPC_USER" \
    -rpcpassword="$RPC_PASS" \
    -rpcport="$RPC_PORT" \
    -rpcwallet=miner \
    sendtoaddress "$DEST_ADDR" "$AMOUNT_BTC"
)

echo "[regtest] Funding txid: $TXID"

# --- Confirm funding -----------------------------------------------------
MINER_ADDR=$(
  "$BITCOIN_CLI_BIN" \
    -regtest \
    -rpcuser="$RPC_USER" \
    -rpcpassword="$RPC_PASS" \
    -rpcport="$RPC_PORT" \
    -rpcwallet=miner \
    getnewaddress
)

echo "[regtest] Mining 1 confirmation block to $MINER_ADDR"
"$BITCOIN_CLI_BIN" \
  -regtest \
  -rpcuser="$RPC_USER" \
  -rpcpassword="$RPC_PASS" \
  -rpcport="$RPC_PORT" \
  -rpcwallet=miner \
  generatetoaddress 1 "$MINER_ADDR" >/dev/null

echo "[regtest] Funding confirmed"
