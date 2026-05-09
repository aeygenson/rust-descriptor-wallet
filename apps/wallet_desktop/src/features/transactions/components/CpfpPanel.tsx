import { useEffect, useMemo, useState } from "react";
import type { CpfpPanelProps } from "../types";
import {
  formatBooleanLabel,
  formatFeeRate,
  formatRelativeFeeRate,
  formatRelativeFeeRatePercent,
  fullOutpoint,
  fullTxid,
  shortOutpoint,
  shortTxid,
} from "../format";
import {
  parsePositiveFeeRate,
  suggestCpfpFeeRate,
} from "../lib";

export function CpfpPanel({
  tx,
  walletName,
  loading = false,
  availableOutpoints,
  onCancel,
  onCreatePsbt,
}: CpfpPanelProps) {
  const suggestedFeeRate = useMemo(
    () => suggestCpfpFeeRate(tx.fee_rate_sat_per_vb),
    [tx.fee_rate_sat_per_vb],
  );

  const [feeRateInput, setFeeRateInput] = useState(String(suggestedFeeRate));
  const [selectedOutpoint, setSelectedOutpoint] = useState(
    availableOutpoints[0] ?? "",
  );

  useEffect(() => {
    setFeeRateInput(String(suggestedFeeRate));
  }, [suggestedFeeRate, tx.txid]);

  useEffect(() => {
    if (availableOutpoints.length === 0) {
      setSelectedOutpoint("");
      return;
    }
    setSelectedOutpoint((current) =>
      availableOutpoints.includes(current) ? current : availableOutpoints[0],
    );
  }, [availableOutpoints]);

  const parsedFeeRate = useMemo(
    () => parsePositiveFeeRate(feeRateInput),
    [feeRateInput],
  );

  const selectedOutpointTitle = fullOutpoint(selectedOutpoint);
  const txidTitle = fullTxid(tx.txid);

  const hasAvailableOutpoints = availableOutpoints.length > 0;
  const feeRateDelta = formatRelativeFeeRate(
    parsedFeeRate,
    tx.fee_rate_sat_per_vb,
  );
  const feeRateDeltaPercent = formatRelativeFeeRatePercent(
    parsedFeeRate,
    tx.fee_rate_sat_per_vb,
  );
  const canSubmit =
    !loading &&
    hasAvailableOutpoints &&
    selectedOutpoint.length > 0 &&
    parsedFeeRate !== null;

  const handleSubmit = () => {
    if (!canSubmit || parsedFeeRate === null) {
      return;
    }

    onCreatePsbt({
      walletName,
      parentTxid: tx.txid,
      selectedOutpoint,
      feeRateSatPerVb: parsedFeeRate,
    });
  };

  return (
    <section className="tx-action-panel transactions-panel-section">
      <div>
        <h3>Accelerate transaction with CPFP</h3>
        <p className="transactions-panel-muted" title={txidTitle}>
          Parent transaction <code>{shortTxid(tx.txid)}</code>
        </p>
        <p className="transactions-panel-muted">
          CPFP creates a child transaction that spends one wallet-owned parent
          output with a higher effective fee rate.
        </p>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">
          Transaction context
        </div>
        <div className="transactions-panel-row">
          <span>Current fee rate</span>
          <strong>
            {tx.fee_rate_sat_per_vb === null
              ? "n/a"
              : formatFeeRate(tx.fee_rate_sat_per_vb)}
          </strong>
        </div>
        <div className="transactions-panel-row">
          <span>Suggested child fee rate</span>
          <strong>{formatFeeRate(suggestedFeeRate)}</strong>
        </div>
        <div className="transactions-panel-row">
          <span>Child fee delta</span>
          <strong>
            {feeRateDelta}
            <span className="transactions-panel-muted">
              {feeRateDeltaPercent}
            </span>
          </strong>
        </div>
        <div className="transactions-panel-row">
          <span>Replaceable</span>
          <strong>{formatBooleanLabel(tx.replaceable)}</strong>
        </div>
        <div className="transactions-panel-row">
          <span>Confirmed</span>
          <strong>{formatBooleanLabel(tx.confirmed)}</strong>
        </div>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">
          Child transaction inputs
        </div>
        <label>
          <span>Parent output to spend</span>
          <select
            value={selectedOutpoint}
            disabled={loading || !hasAvailableOutpoints}
            onChange={(event) => setSelectedOutpoint(event.target.value)}
          >
            {availableOutpoints.map((outpoint) => (
              <option
                key={outpoint}
                value={outpoint}
                title={fullOutpoint(outpoint)}
              >
                {shortOutpoint(outpoint)}
              </option>
            ))}
          </select>
        </label>

        {!hasAvailableOutpoints && (
          <div className="transactions-panel-warning">
            No spendable wallet-owned output is exposed for this parent
            transaction. Sync the wallet and reopen this transaction from
            Transactions.
          </div>
        )}

        {selectedOutpoint.length > 0 && (
          <div className="transactions-panel-row">
            <span>Selected outpoint</span>
            <code title={selectedOutpointTitle}>
              {shortOutpoint(selectedOutpoint)}
            </code>
          </div>
        )}

        {selectedOutpoint.length > 0 && (
          <div className="transactions-panel-muted">
            The selected outpoint becomes the child input. The backend will
            build the CPFP PSBT and you can inspect it before signing.
          </div>
        )}

        <label>
          <span>Child fee rate, sat/vB</span>
          <input
            type="number"
            min="1"
            step="1"
            value={feeRateInput}
            disabled={loading}
            onChange={(event) => setFeeRateInput(event.target.value)}
          />
        </label>

        <div className="transactions-panel-muted">
          Suggested child fee rate is based on the parent transaction fee rate
          and should be high enough to make the package attractive to miners.
        </div>

        {parsedFeeRate === null && (
          <div className="transactions-panel-warning">
            Enter a positive fee rate in sat/vB.
          </div>
        )}
      </div>

      <div className="transactions-panel-actions">
        <button
          type="button"
          className="secondary"
          disabled={loading}
          onClick={onCancel}
        >
          Cancel
        </button>
        <button
          type="button"
          disabled={!canSubmit}
          title={
            canSubmit
              ? "Create an unsigned CPFP child PSBT"
              : "Select a spendable parent output and enter a positive fee rate"
          }
          onClick={handleSubmit}
        >
          {loading ? "Creating CPFP PSBT…" : "Create CPFP PSBT"}
        </button>
      </div>
    </section>
  );
}