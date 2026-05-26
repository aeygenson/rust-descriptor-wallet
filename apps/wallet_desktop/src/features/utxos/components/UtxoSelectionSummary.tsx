import type { UtxoSelectionSummaryProps } from "../types";
import { formatBtcFromSats, formatSats } from "../format";

export function UtxoSelectionSummary({
  selectedCount,
  selectedValueSat,
  confirmedCount,
  unconfirmedCount,
  lockedCount,
  spendableCount,
  onClearSelection,
}: UtxoSelectionSummaryProps) {
  if (selectedCount === 0) return null;

  const formattedBtcValue = formatBtcFromSats(selectedValueSat);
  const confirmedOnly = unconfirmedCount === 0;

  const formattedSelectedCount = selectedCount.toLocaleString();
  const formattedConfirmedCount = confirmedCount.toLocaleString();
  const formattedPendingCount = unconfirmedCount.toLocaleString();
  const formattedLockedCount = lockedCount.toLocaleString();
  const formattedSpendableCount = spendableCount.toLocaleString();
  const hasLockedSelection = lockedCount > 0;

  return (
    <section className="utxo-selection-summary">
      <div className="utxo-selection-summary__left">
        <div className="utxo-selection-summary__headline">
          <span className="utxo-selection-summary__badge">
            {formattedSelectedCount} selected
          </span>

          <span className="utxo-selection-summary__title">
            UTXO Selection Summary
          </span>
        </div>

        <div className="utxo-selection-summary__value-group">
          <strong className="utxo-selection-summary__value-primary">
            {formatSats(selectedValueSat)}
          </strong>

          <span className="utxo-selection-summary__value-secondary">
            {formattedBtcValue}
          </span>
        </div>

        <span className="utxo-selection-summary__meta">
          Confirmed: {formattedConfirmedCount} · Pending: {formattedPendingCount} · Locked: {formattedLockedCount} · Spendable: {formattedSpendableCount}
          {hasLockedSelection
            ? " · Locked inputs must be unlocked before spending"
            : confirmedOnly
              ? " · Ready for consolidation, Send, and RBF flows"
              : " · Contains pending inputs"}
        </span>
      </div>

      <div className="utxo-selection-summary__hint">
        {hasLockedSelection
          ? "Selection contains locked inputs that cannot currently be used in spend flows."
          : "Selected inputs can be reused directly in Send, Send Max, Sweep, Consolidation, RBF, and future CPFP flows."}
      </div>

      {onClearSelection && (
        <button
          type="button"
          className="utxo-selection-summary__clear"
          aria-label="Clear selected UTXOs"
          title="Clear the current UTXO selection"
          onClick={onClearSelection}
        >
          Clear
        </button>
      )}
    </section>
  );
}