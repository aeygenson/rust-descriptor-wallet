import type { UtxosStateViewProps } from "../types";

const EMPTY_UTXO_HINT =
  "Receive funds or sync the wallet to populate spendable outputs.";

const LOADING_HINT =
  "Fetching wallet outputs, confirmation state, and spendability.";

const ERROR_HINT =
  "Check backend connectivity, wallet sync status, and Electrum/Bitcoin Core reachability.";

export function UtxosStateView({
  loading,
  error,
  hasData,
  emptyMessage = "No UTXOs found",
}: UtxosStateViewProps) {
  if (loading) {
    return (
      <div
        className="utxos-state utxos-state--loading"
        role="status"
        aria-live="polite"
      >
        <div className="utxos-state__icon" aria-hidden="true">
          ⟳
        </div>

        <div className="utxos-state__content">
          <strong className="utxos-state__title">Loading UTXOs…</strong>

          <span className="utxos-state__hint">{LOADING_HINT}</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className="utxos-state utxos-state--error"
        role="alert"
      >
        <div className="utxos-state__icon" aria-hidden="true">
          !
        </div>

        <div className="utxos-state__content">
          <strong className="utxos-state__title">
            Failed to load UTXOs
          </strong>

          <span className="utxos-state__hint">{ERROR_HINT}</span>

          <pre className="utxos-state__error">{error}</pre>
        </div>
      </div>
    );
  }

  if (!hasData) {
    return (
      <div className="utxos-state utxos-state--empty">
        <div className="utxos-state__icon" aria-hidden="true">
          ₿
        </div>

        <div className="utxos-state__content">
          <strong className="utxos-state__title">{emptyMessage}</strong>

          <span className="utxos-state__hint">{EMPTY_UTXO_HINT}</span>
        </div>
      </div>
    );
  }

  return null;
}