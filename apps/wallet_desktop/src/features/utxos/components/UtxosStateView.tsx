import type { UtxosStateViewProps } from "../types";

const EMPTY_UTXO_HINT =
  "Receive funds or sync the wallet to populate spendable outputs.";

const LOCKED_UTXO_HINT =
  "All visible UTXOs are currently locked and unavailable for spending.";

const FILTERED_UTXO_HINT =
  "Adjust the current filters or search query to display additional wallet outputs.";

const LOADING_HINT =
  "Fetching wallet outputs, confirmation state, and spendability.";

const ERROR_HINT =
  "Check backend connectivity, wallet sync status, and Electrum/Bitcoin Core reachability.";

export function UtxosStateView({
  loading,
  error,
  hasData,
  emptyMessage = "No UTXOs found",
  emptyVariant = "empty",
}: UtxosStateViewProps) {
  const emptyHint =
    emptyVariant === "locked"
      ? LOCKED_UTXO_HINT
      : emptyVariant === "filtered"
        ? FILTERED_UTXO_HINT
        : EMPTY_UTXO_HINT;

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

          <span className="utxos-state__hint">{emptyHint}</span>
        </div>
      </div>
    );
  }

  return null;
}