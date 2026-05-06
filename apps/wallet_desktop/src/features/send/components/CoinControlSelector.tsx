import type { CoinControlSelectorProps } from "../types";
import { formatSats } from "../format";
import { normalizeSelectedOutpoints, sumSelectedInputValue } from "../lib";


export function CoinControlSelector({
  utxos,
  selectedUtxos,
  onSelectionChange,
}: CoinControlSelectorProps) {
  const normalizedSelectedUtxos = normalizeSelectedOutpoints(selectedUtxos);
  const selectedSet = new Set(normalizedSelectedUtxos);

  const visibleUtxos = utxos.filter((utxo, index, allUtxos) => {
    const outpoint = utxo.outpoint.trim();
    return (
      outpoint.length > 0 &&
      allUtxos.findIndex((candidate) => candidate.outpoint.trim() === outpoint) === index
    );
  });

  const selectedValueSat = sumSelectedInputValue(visibleUtxos, normalizedSelectedUtxos);

  const toggleUtxo = (outpoint: string) => {
    const normalizedOutpoint = outpoint.trim();

    if (normalizedOutpoint.length === 0) {
      return;
    }

    if (selectedSet.has(normalizedOutpoint)) {
      onSelectionChange(
        normalizedSelectedUtxos.filter((selected) => selected !== normalizedOutpoint),
      );
      return;
    }

    onSelectionChange([...normalizedSelectedUtxos, normalizedOutpoint]);
  };

  const clearSelection = () => {
    onSelectionChange([]);
  };

  return (
    <section className="coin-selector">
      <div className="coin-selector__header">
        <div>
          <h3 className="coin-selector__title">Input Selection</h3>
          <p className="coin-selector__subtitle">
            Select exactly which existing UTXOs this transaction may spend.
          </p>
        </div>

        <div className="coin-selector__summary">
          <strong>{normalizedSelectedUtxos.length}</strong>
          <span>selected</span>
          <strong>{formatSats(selectedValueSat)}</strong>
        </div>
      </div>

      {visibleUtxos.length === 0 ? (
        <div className="coin-selector__empty">No spendable UTXOs found for this wallet.</div>
      ) : (
        <div className="coin-selector__list">
          {visibleUtxos.map((utxo, index) => {
            const checked = selectedSet.has(utxo.outpoint);
            const rowId = `coin-selector-${utxo.outpoint.replace(/[^a-zA-Z0-9_-]/g, "-")}-${index}`;
            const confirmationLabel =
              utxo.confirmations === null || utxo.confirmations === undefined
                ? utxo.confirmed === true
                  ? "confirmed"
                  : utxo.confirmed === false
                    ? "unconfirmed"
                    : "confirmation status unknown"
                : `${utxo.confirmations.toLocaleString()} confirmations`;

            return (
              <label
                key={utxo.outpoint}
                className={`coin-selector__row ${
                  checked ? "coin-selector__row--selected" : ""
                }`}
                htmlFor={rowId}
              >
                <input
                  type="checkbox"
                  id={rowId}
                  checked={checked}
                  aria-label={`Select UTXO ${utxo.outpoint}`}
                  onChange={() => toggleUtxo(utxo.outpoint)}
                />

                <span className="coin-selector__main">
                  <span className="coin-selector__outpoint">{utxo.outpoint}</span>
                  <span className="coin-selector__meta">
                    {utxo.label ? `${utxo.label} · ` : ""}
                    {confirmationLabel}
                  </span>
                  {utxo.address ? (
                    <span className="coin-selector__address">{utxo.address}</span>
                  ) : null}
                </span>

                <strong className="coin-selector__value">{formatSats(utxo.valueSat)}</strong>
              </label>
            );
          })}
        </div>
      )}

      {normalizedSelectedUtxos.length > 0 ? (
        <button className="coin-selector__clear" type="button" onClick={clearSelection}>
          Clear selected inputs
        </button>
      ) : null}
    </section>
  );
}