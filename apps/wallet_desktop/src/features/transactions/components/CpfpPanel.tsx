import { useEffect, useMemo, useState } from "react";
import type { CpfpPanelProps } from "../types";
import {
  formatBooleanLabel,
  formatFeeRate,
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

  const hasAvailableOutpoints = availableOutpoints.length > 0;
  const canSubmit =
    !loading && hasAvailableOutpoints && selectedOutpoint.length > 0 && parsedFeeRate !== null;

  const handleSubmit = () => {
    if (!canSubmit || parsedFeeRate === null) {
      return;
    }

    onCreatePsbt({
      walletName,
      parentTxid: tx.txid,
      selectedOutpoint,
      feeRateSatVb: parsedFeeRate,
    });
  };

  return (
    <section className="tx-action-panel transactions-panel-section">
      <div>
        <h3>Accelerate transaction with CPFP</h3>
        <p className="transactions-panel-muted" title={tx.txid}>
          Parent transaction <code>{shortTxid(tx.txid)}</code>
        </p>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">Transaction context</div>
        <div className="transactions-panel-row">
          <span>Current fee rate</span>
          <strong>{tx.fee_rate_sat_per_vb === null ? "n/a" : formatFeeRate(tx.fee_rate_sat_per_vb)}</strong>
        </div>
        <div className="transactions-panel-row">
          <span>Suggested child fee rate</span>
          <strong>{formatFeeRate(suggestedFeeRate)}</strong>
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
        <div className="transactions-panel-section-title">Child transaction inputs</div>
        <label>
          <span>Parent output to spend</span>
          <select
            value={selectedOutpoint}
            disabled={loading || !hasAvailableOutpoints}
            onChange={(event) => setSelectedOutpoint(event.target.value)}
          >
            {availableOutpoints.map((outpoint) => (
              <option key={outpoint} value={outpoint} title={outpoint}>
                {shortOutpoint(outpoint)}
              </option>
            ))}
          </select>
        </label>

        {!hasAvailableOutpoints && (
          <div className="transactions-panel-warning">
            No spendable wallet-owned output is exposed for this parent transaction. Sync the wallet and reopen this transaction from Transactions.
          </div>
        )}

        {selectedOutpoint.length > 0 && (
          <div className="transactions-panel-row">
            <span>Selected outpoint</span>
            <code title={selectedOutpoint}>{shortOutpoint(selectedOutpoint)}</code>
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

        {parsedFeeRate === null && (
          <div className="transactions-panel-warning">Enter a positive fee rate in sat/vB.</div>
        )}
      </div>

      <div className="transactions-panel-actions">
        <button type="button" className="secondary" disabled={loading} onClick={onCancel}>
          Cancel
        </button>
        <button type="button" disabled={!canSubmit} onClick={handleSubmit}>
          {loading ? "Creating CPFP PSBT…" : "Create CPFP PSBT"}
        </button>
      </div>
    </section>
  );
}