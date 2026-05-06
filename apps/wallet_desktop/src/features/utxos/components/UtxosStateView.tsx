import type { UtxosStateViewProps } from "../types";

export function UtxosStateView({
  loading,
  error,
  hasData,
  emptyMessage = "No UTXOs found",
}: UtxosStateViewProps) {
  if (loading) {
    return (
      <div className="utxos-state utxos-state--loading">
        <span>Loading UTXOs…</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="utxos-state utxos-state--error">
        <span>Failed to load UTXOs</span>
        <pre className="utxos-state__error">{error}</pre>
      </div>
    );
  }

  if (!hasData) {
    return (
      <div className="utxos-state utxos-state--empty">
        <span>{emptyMessage}</span>
      </div>
    );
  }

  return null;
}