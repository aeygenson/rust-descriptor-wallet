import type { UtxoSelectionActionBarProps } from "../types";
import { formatBtcFromSats } from "../format";

export function UtxoActionsBar({
  selectedCount,
  selectedValueSat,
  hasLockedSelection,
  hasSpendableSelection,
  disabled,
  onSendFixedSelected,
  onSendMaxSelected,
  onSweepSelected,
  onConsolidateSelected,
  onLockSelected,
  onUnlockSelected,
  onClearSelection,
}: UtxoSelectionActionBarProps) {
  if (selectedCount === 0) return null;

  const formattedBtcValue = formatBtcFromSats(selectedValueSat);
  const formattedSelectedValue = selectedValueSat.toLocaleString();

  return (
    <div className="utxo-actions-bar">
      <div className="utxo-actions-bar__left">
        <div className="utxo-actions-bar__summary">
          <span className="utxo-actions-bar__badge">
            {selectedCount.toLocaleString()} selected
          </span>

          <div className="utxo-actions-bar__values">
            <strong>{formattedSelectedValue} sats</strong>
            <span>{formattedBtcValue}</span>
          </div>
        </div>

        <span className="utxo-actions-bar__hint">
          {hasLockedSelection
            ? "Locked inputs are selected. Unlock them before using spend flows."
            : "Selected inputs can be forwarded directly into Send, Sweep, Consolidation, or future CPFP flows."}
        </span>
      </div>

      <div className="utxo-actions-bar__actions">
        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled || hasLockedSelection || !hasSpendableSelection}
          title="Create a standard payment using the selected inputs"
          onClick={onSendFixedSelected}
        >
          Send
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled || hasLockedSelection || !hasSpendableSelection}
          title="Spend the selected inputs minus fees"
          onClick={onSendMaxSelected}
        >
          Send Max
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled || hasLockedSelection || !hasSpendableSelection}
          title="Sweep the selected inputs into a destination address"
          onClick={onSweepSelected}
        >
          Sweep
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled || hasLockedSelection || !hasSpendableSelection}
          title="Merge selected inputs into fewer wallet-controlled outputs"
          onClick={onConsolidateSelected}
        >
          Consolidate
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn utxo-actions-bar__btn--secondary"
          disabled={disabled || !hasSpendableSelection}
          title="Lock selected spendable inputs"
          onClick={onLockSelected}
        >
          Lock
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn utxo-actions-bar__btn--secondary"
          disabled={disabled || !hasLockedSelection}
          title="Unlock selected locked inputs"
          onClick={onUnlockSelected}
        >
          Unlock
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn utxo-actions-bar__btn--secondary"
          disabled={disabled}
          title="Clear the current UTXO selection"
          onClick={onClearSelection}
        >
          Clear
        </button>
      </div>
    </div>
  );
}