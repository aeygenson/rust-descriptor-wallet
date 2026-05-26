// UTXO formatting helpers (presentation only)

export function formatSats(value: number): string {
  if (!Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} sats`;
}

export function formatBtc(value: number): string {
  if (!Number.isFinite(value)) return "—";
  return `${(value / 1e8).toFixed(8)} BTC`;
}
export function formatOutpointShort(outpoint: string): string {
  if (!outpoint || outpoint.length < 12) return outpoint;
  return `${outpoint.slice(0, 6)}…${outpoint.slice(-6)}`;
}

export function formatOutpointFull(outpoint: string): string {
  if (!outpoint) return "—";
  return outpoint;
}

export function formatTxidShort(txid: string): string {
  if (!txid || txid.length < 12) return txid;
  return `${txid.slice(0, 8)}…${txid.slice(-8)}`;
}

export function formatBtcFromSats(valueSat: number): string {
  if (!Number.isFinite(valueSat)) return "—";
  return formatBtc(valueSat);
}

export function formatCompactBtcFromSats(valueSat: number): string {
  if (!Number.isFinite(valueSat)) return "—";

  return `${(valueSat / 1e8).toFixed(6)} BTC`;
}

export function formatUtxoStatus(confirmed: boolean): string {
  return confirmed ? "Confirmed" : "Pending";
}

export function formatLockState(isLocked: boolean): string {
  return isLocked ? "Locked" : "Spendable";
}

export function formatLockBadge(isLocked: boolean): string {
  return isLocked ? "🔒 Locked" : "Spendable";
}

export function formatLockReason(reason?: string | null): string {
  if (!reason || reason.trim().length === 0) {
    return "No reason";
  }

  return reason;
}
export function formatConfirmations(confirmations?: number | null): string {
  if (confirmations == null) return "—";
  return confirmations === 0 ? "Unconfirmed" : `${confirmations} conf`;
}

export function formatConfirmationBadge(confirmations?: number | null): string {
  if (confirmations == null) return "Unknown";

  if (confirmations <= 0) {
    return "Pending";
  }

  if (confirmations < 6) {
    return `${confirmations} conf`;
  }

  return "Finalized";
}

export function formatNumber(value: number): string {
  if (!Number.isFinite(value)) return "—";
  return value.toLocaleString();
}

export function formatPercentage(value: number): string {
  if (!Number.isFinite(value)) return "—";

  return `${value.toFixed(0)}%`;
}