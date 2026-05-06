

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

export function formatUtxoStatus(confirmed: boolean): string {
  return confirmed ? "Confirmed" : "Pending";
}

export function formatConfirmations(confirmations?: number | null): string {
  if (confirmations == null) return "—";
  return confirmations === 0 ? "Unconfirmed" : `${confirmations} conf`;
}

export function formatNumber(value: number): string {
  if (!Number.isFinite(value)) return "—";
  return value.toLocaleString();
}