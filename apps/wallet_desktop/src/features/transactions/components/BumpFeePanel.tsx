import { useEffect, useMemo, useState } from "react";
import type { BumpFeePanelProps } from "../types";
import {
  fullTxid,
  shortTxid,
  formatFeeRate,
  formatRelativeFeeRate,
  formatRelativeFeeRatePercent,
} from "../format";
import {
  clampNumber,
  parsePositiveFeeRate,
  suggestNextFeeRate,
} from "../lib";

const MIN_FEE_RATE_SAT_PER_VB = 1;
const MAX_FEE_RATE_SAT_PER_VB = 10_000;

export function BumpFeePanel({
  tx,
  walletName,
  loading = false,
  onCancel,
  onCreatePsbt,
}: BumpFeePanelProps) {
  const [feeRateInput, setFeeRateInput] = useState<string>("");

  const currentFeeRate = tx.fee_rate_sat_per_vb ?? null;

  const suggested = useMemo(
    () => suggestNextFeeRate(currentFeeRate),
    [currentFeeRate],
  );

  useEffect(() => {
    // initialize input with suggestion when tx changes
    setFeeRateInput(String(suggested));
  }, [suggested, tx.txid]);

  const parsedFeeRate = useMemo(() => {
    const parsed = parsePositiveFeeRate(feeRateInput);

    if (parsed === null) {
      return null;
    }

    return clampNumber(
      parsed,
      MIN_FEE_RATE_SAT_PER_VB,
      MAX_FEE_RATE_SAT_PER_VB,
    );
  }, [feeRateInput]);

  const isHigherThanCurrentFeeRate =
    parsedFeeRate !== null &&
    (currentFeeRate === null || parsedFeeRate > currentFeeRate);

  const feeRateDelta = formatRelativeFeeRate(parsedFeeRate, currentFeeRate);
  const feeRateDeltaPercent = formatRelativeFeeRatePercent(
    parsedFeeRate,
    currentFeeRate,
  );
  const txidTitle = fullTxid(tx.txid);

  const canSubmit = !loading && isHigherThanCurrentFeeRate;

  return (
    <div className="bump-fee">
      <div className="bump-fee__header">
        <div className="bump-fee__title">Bump Fee (RBF)</div>
        <div className="bump-fee__txid" title={txidTitle}>
          {shortTxid(tx.txid)}
        </div>
      </div>

      <div className="bump-fee__body">
        <div className="bump-fee__row">
          <span className="bump-fee__label">Current fee rate</span>
          <span className="bump-fee__value">
            {formatFeeRate(currentFeeRate)}
          </span>
        </div>

        <div className="bump-fee__row">
          <span className="bump-fee__label">Suggested</span>
          <span className="bump-fee__value">{formatFeeRate(suggested)}</span>
        </div>

        <div className="bump-fee__row">
          <span className="bump-fee__label">Delta</span>
          <span className="bump-fee__value">
            {feeRateDelta}
            <span className="bump-fee__value-secondary">
              {feeRateDeltaPercent}
            </span>
          </span>
        </div>

        <div className="bump-fee__field">
          <label className="field__label" htmlFor="bump-fee-rate">
            New fee rate (sat/vB)
          </label>
          <input
            id="bump-fee-rate"
            className="field__input"
            type="number"
            inputMode="decimal"
            min={MIN_FEE_RATE_SAT_PER_VB}
            max={MAX_FEE_RATE_SAT_PER_VB}
            step={1}
            value={feeRateInput}
            onChange={(e) => setFeeRateInput(e.target.value)}
            disabled={loading}
          />
          <div className="field__hint">
            RBF replacement must pay a higher fee rate than the current
            transaction and remain valid under backend policy.
          </div>
          <div className="field__hint">
            A replacement PSBT is created first so you can inspect it before
            signing and broadcasting.
          </div>
          {parsedFeeRate !== null && !isHigherThanCurrentFeeRate && (
            <div className="field__error">
              New fee rate must be higher than the current fee rate.
            </div>
          )}
        </div>
      </div>

      <div className="bump-fee__actions">
        <button
          type="button"
          className="secondary-button"
          onClick={onCancel}
          disabled={loading}
        >
          Cancel
        </button>

        <button
          type="button"
          className="primary-button"
          disabled={!canSubmit}
          title={
            canSubmit
              ? "Create an unsigned RBF replacement PSBT"
              : "Enter a fee rate higher than the current transaction fee rate"
          }
          onClick={() => {
            if (parsedFeeRate === null) return;
            onCreatePsbt({
              walletName,
              txid: tx.txid,
              feeRateSatPerVb: parsedFeeRate,
            });
          }}
        >
          {loading ? "Creating Replacement PSBT…" : "Create Replacement PSBT"}
        </button>
      </div>
    </div>
  );
}