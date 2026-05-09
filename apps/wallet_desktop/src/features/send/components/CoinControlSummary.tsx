import type { FC } from "react";
import type { CoinControlSummaryProps } from "../types";
import { CoinControlSelector } from "./CoinControlSelector";
import {
  formatOptionalSats,
  formatOutpoint,
  formatSelectedInputWithBtc,
} from "../format";
import { normalizeSelectedOutpoints } from "../lib";

export const CoinControlSummary: FC<CoinControlSummaryProps> = ({
  selectedInputCount,
  selectedValueSat = 0,
  utxos = [],
  selectedUtxos = [],
  requiredSat = null,
  estimatedFeeSat = null,
  changeSat = null,
  remainingSat = null,
  selectionMode = "auto",
  onSelectionModeChange,
  onUtxoSelectionChange,
  onClearSelection,
}) => {
  const isManual = selectionMode === "manual";
  const normalizedSelectedUtxos = normalizeSelectedOutpoints(selectedUtxos);
  const resolvedSelectedInputCount =
    selectedInputCount ?? normalizedSelectedUtxos.length;
  const hasSelection = resolvedSelectedInputCount > 0;
  const selectedOutpointPreview = normalizedSelectedUtxos.slice(0, 3);
  const hiddenSelectedOutpointCount = Math.max(
    normalizedSelectedUtxos.length - selectedOutpointPreview.length,
    0,
  );
  const isInsufficient = remainingSat !== null && remainingSat < 0;

  const selectedValueLabel = formatSelectedInputWithBtc(selectedValueSat);
  const selectedValueTitle = formatOptionalSats(selectedValueSat);

  const modeToggleTitle = onSelectionModeChange
    ? isManual
      ? "Switch to automatic coin selection"
      : "Switch to manual coin selection"
    : "Mode switching is not wired yet";

  const canRenderSelector =
    isManual && Boolean(onUtxoSelectionChange) && utxos.length > 0;

  const handleModeToggle = () => {
    onSelectionModeChange?.(isManual ? "auto" : "manual");
  };

  const handleUtxoSelectionChange = (nextSelectedUtxos: string[]) => {
    onUtxoSelectionChange?.(nextSelectedUtxos);
  };

  return (
    <section className="coin-control">
      <div className="coin-control__header">
        <h3 className="coin-control__title">Coin Control</h3>
        <div className="coin-control__actions">
          <button
            className={`coin-control__badge coin-control__badge--clickable ${
              isManual ? "coin-control__badge--manual" : ""
            }`}
            type="button"
            onClick={handleModeToggle}
            disabled={!onSelectionModeChange}
            title={modeToggleTitle}
            aria-pressed={isManual}
          >
            {isManual ? "Manual" : "Auto"}
          </button>
          {!isManual && onSelectionModeChange ? (
            <button
              className="coin-control__button"
              type="button"
              onClick={() => onSelectionModeChange("manual")}
              title="Switch to manual coin selection"
            >
              Enable manual selection
            </button>
          ) : null}
        </div>
      </div>

      <div className="coin-control__grid">
        <div className="coin-control__item">
          <div className="coin-control__label">Selected inputs</div>
          <div className="coin-control__value">
            {resolvedSelectedInputCount.toLocaleString()}
          </div>
        </div>

        <div className="coin-control__item">
          <div className="coin-control__label">Selected value</div>
          <div className="coin-control__value" title={selectedValueTitle}>
            {selectedValueLabel}
          </div>
        </div>

        <div className="coin-control__item">
          <div className="coin-control__label">Required</div>
          <div className="coin-control__value">
            {formatOptionalSats(requiredSat)}
          </div>
        </div>

        <div className="coin-control__item">
          <div className="coin-control__label">Estimated fee</div>
          <div className="coin-control__value">
            {formatOptionalSats(estimatedFeeSat)}
          </div>
        </div>

        <div className="coin-control__item">
          <div className="coin-control__label">Estimated change</div>
          <div className="coin-control__value">
            {formatOptionalSats(changeSat)}
          </div>
        </div>

        <div className="coin-control__item">
          <div className="coin-control__label">Remaining after send</div>
          <div
            className={`coin-control__value ${
              isInsufficient ? "coin-control__value--error" : ""
            }`}
          >
            {formatOptionalSats(remainingSat)}
          </div>
        </div>
      </div>

      <div className="coin-control__hint">
        {isManual
          ? hasSelection
            ? "Manual coin control is enabled. Only selected inputs will be spent."
            : "Manual coin control is enabled, but no inputs are selected yet."
          : "Coin control is in auto mode. Inputs are selected automatically by the backend."}
      </div>

      {isManual && hasSelection ? (
        <div className="coin-control__selected-outpoints">
          <div className="coin-control__label">Selected outpoints</div>
          <div className="coin-control__chips">
            {selectedOutpointPreview.map((outpoint) => (
              <span
                className="coin-control__chip"
                key={outpoint}
                title={outpoint}
              >
                {formatOutpoint(outpoint)}
              </span>
            ))}
            {hiddenSelectedOutpointCount > 0 ? (
              <span className="coin-control__chip">
                +{hiddenSelectedOutpointCount} more
              </span>
            ) : null}
          </div>
        </div>
      ) : null}

      {isManual && hasSelection && onClearSelection ? (
        <button
          className="coin-control__clear"
          type="button"
          onClick={onClearSelection}
          title="Clear selected inputs"
        >
          Clear selected inputs
        </button>
      ) : null}

      {canRenderSelector ? (
        <div className="coin-control__selector">
          <CoinControlSelector
            utxos={utxos}
            selectedUtxos={normalizedSelectedUtxos}
            onSelectionChange={handleUtxoSelectionChange}
          />
        </div>
      ) : null}
    </section>
  );
};