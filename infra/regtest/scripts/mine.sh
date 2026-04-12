

#!/usr/bin/env bash
set -euo pipefail

# --- RPC config ----------------------------------------------------------
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-/opt/homebrew/bin/bitcoin-cli}"
RPC_USER="${BITCOIN_RPC_USER:-bitcoin}"
RPC_PASS="${BITCOIN_RPC_PASS:-bitcoin}"
RPC_PORT="${BITCOIN_RPC_PORT:-18443}"

# --- Arguments -----------------------------------------------------------
BLOCK_COUNT="${1:-1}"

if ! [[ "$BLOCK_COUNT" =~ ^[0-9]+$ ]] || [[ "$BLOCK_COUNT" -lt 1 ]]; then
  echo "Usage: $0 [block_count]" >&2
  echo "Example: $0 1" >&2
  echo "Example: $0 101" >&2
  exit 1
fi

# --- Ensure miner wallet exists -----------------------------------------
if ! "$BITCOIN_CLI_BIN" -regtest -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcport="$RPC_PORT" listwallets | grep -q '"miner"'; then
  echo "[regtest] miner wallet is not loaded" >&2
  echo "Run start.sh first to create/load the miner wallet" >&2
  exit 1
fi

# --- Generate mining address --------------------------------------------
MINER_ADDR=$(
  "$BITCOIN_CLI_BIN" \
    -regtest \
    -rpcuser="$RPC_USER" \
    -rpcpassword="$RPC_PASS" \
    -rpcport="$RPC_PORT" \
    -rpcwallet=miner \
    getnewaddress
)

echo "[regtest] Mining $BLOCK_COUNT block(s) to $MINER_ADDR"

# --- Mine blocks ---------------------------------------------------------
"$BITCOIN_CLI_BIN" \
  -regtest \
  -rpcuser="$RPC_USER" \
  -rpcpassword="$RPC_PASS" \
  -rpcport="$RPC_PORT" \
  -rpcwallet=miner \
  generatetoaddress "$BLOCK_COUNT" "$MINER_ADDR"

echo "[regtest] Mining complete"