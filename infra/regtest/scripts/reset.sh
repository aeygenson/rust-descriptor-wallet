#!/usr/bin/env bash
set -euo pipefail

# --- Paths ---------------------------------------------------------------
BASE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BITCOIN_DATA_DIR="$BASE_DIR/bitcoin/data"
ELECTRS_DB_DIR="$BASE_DIR/electrs/db"

STOP_SCRIPT="$BASE_DIR/scripts/stop.sh"

# --- Confirm -------------------------------------------------------------
if [[ "${FORCE:-}" != "1" ]]; then
  echo "[regtest] This will DELETE local regtest data:" >&2
  echo "  - $BITCOIN_DATA_DIR" >&2
  echo "  - $ELECTRS_DB_DIR" >&2
  read -r -p "Type 'yes' to continue: " ans
  if [[ "$ans" != "yes" ]]; then
    echo "[regtest] Aborted"
    exit 1
  fi
fi

# --- Stop services -------------------------------------------------------
if [[ -x "$STOP_SCRIPT" ]]; then
  echo "[regtest] Stopping services..."
  "$STOP_SCRIPT" || true
else
  echo "[regtest] stop.sh not found or not executable, attempting best-effort stop"
  pkill -f "electrs" || true
  pkill -f "bitcoind" || true
fi

# --- Remove data ---------------------------------------------------------
echo "[regtest] Removing Bitcoin data dir: $BITCOIN_DATA_DIR"
rm -rf "$BITCOIN_DATA_DIR"

echo "[regtest] Removing electrs DB dir: $ELECTRS_DB_DIR"
rm -rf "$ELECTRS_DB_DIR"

# Recreate empty dirs to avoid permission issues later
mkdir -p "$BITCOIN_DATA_DIR"
mkdir -p "$ELECTRS_DB_DIR"

echo "[regtest] RESET complete"
