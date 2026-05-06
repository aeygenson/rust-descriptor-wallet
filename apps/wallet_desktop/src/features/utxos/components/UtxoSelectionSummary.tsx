import type { UtxoSelectionSummaryProps } from "../types";
import { formatSats } from "../format";

export function UtxoSelectionSummary({
  selectedCount,
  selectedValueSat,
  confirmedCount,
  unconfirmedCount,
  onClearSelection,
}: UtxoSelectionSummaryProps) {
  if (selectedCount === 0) return null;

  return (
    <section className="utxo-selection-summary">
      <div className="utxo-selection-summary__left">
        <span className="utxo-selection-summary__title">
          Selected: {selectedCount.toLocaleString()} UTXO{selectedCount > 1 ? "s" : ""}
        </span>

        <span className="utxo-selection-summary__value">
          {formatSats(selectedValueSat)}
        </span>

        <span className="utxo-selection-summary__meta">
          Confirmed: {confirmedCount.toLocaleString()} · Pending: {unconfirmedCount.toLocaleString()}
        </span>
      </div>

      {onClearSelection && (
        <button
          type="button"
          className="utxo-selection-summary__clear"
          onClick={onClearSelection}
        >
          Clear
        </button>
      )}
    </section>
  );
}