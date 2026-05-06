import type { UtxoSelectionActionBarProps } from "../types";

export function UtxoActionsBar({
  selectedCount,
  selectedValueSat,
  disabled,
  onSendFixedSelected,
  onSendMaxSelected,
  onSweepSelected,
  onConsolidateSelected,
  onClearSelection,
}: UtxoSelectionActionBarProps) {
  if (selectedCount === 0) return null;

  return (
    <div className="utxo-actions-bar">
      <div className="utxo-actions-bar__left">
        <span className="utxo-actions-bar__label">
          Actions for {selectedCount} selected ({selectedValueSat.toLocaleString()} sats)
        </span>
      </div>

      <div className="utxo-actions-bar__actions">
        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled}
          onClick={onSendFixedSelected}
        >
          Send Fixed
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled}
          onClick={onSendMaxSelected}
        >
          Send Max
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled}
          onClick={onSweepSelected}
        >
          Sweep
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn"
          disabled={disabled}
          onClick={onConsolidateSelected}
        >
          Consolidate
        </button>

        <button
          type="button"
          className="utxo-actions-bar__btn utxo-actions-bar__btn--secondary"
          onClick={onClearSelection}
        >
          Clear
        </button>
      </div>
    </div>
  );
}