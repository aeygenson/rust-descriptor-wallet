import type { RbfPsbtWorkflowPanelProps } from "../types";
import {
  formatBooleanLabel,
  formatFeeRate,
  formatSats,
  formatVsize,
  shortOutpoint,
  shortTxid,
} from "../format";

export function RbfPsbtWorkflowPanel({
  psbt,
  signedPsbt,
  broadcastResult,
  loading = false,
  onSign,
  onBroadcast,
  onClose,
}: RbfPsbtWorkflowPanelProps) {
  const canSign = !loading && signedPsbt === null && broadcastResult === null;
  const canBroadcast = !loading && signedPsbt !== null && broadcastResult === null;

  return (
    <div className="transactions-details-panel">
      <div className="rbf-workflow__header">
        <div>
          <h2>Replacement PSBT</h2>
          {psbt.original_txid && (
            <div className="rbf-workflow__subtitle">
              Replaces <span title={psbt.original_txid}>{shortTxid(psbt.original_txid)}</span>
            </div>
          )}
        </div>
        <button
          type="button"
          className="secondary-button"
          onClick={onClose}
          disabled={loading}
        >
          Close
        </button>
      </div>

      <div className="transactions-details-grid">
        <RbfDetailsItem label="Replacement txid" value={psbt.txid ? shortTxid(psbt.txid) : "—"} title={psbt.txid || undefined} />
        <RbfDetailsItem label="Fee" value={formatSats(psbt.fee_sat)} />
        <RbfDetailsItem label="Fee rate" value={formatFeeRate(psbt.fee_rate_sat_per_vb)} />
        <RbfDetailsItem label="Estimated vsize" value={formatVsize(psbt.estimated_vsize)} />
        <RbfDetailsItem label="Inputs" value={psbt.input_count.toLocaleString()} />
        <RbfDetailsItem label="Outputs" value={psbt.output_count.toLocaleString()} />
        <RbfDetailsItem label="Selected UTXOs" value={psbt.selected_utxo_count.toLocaleString()} />
        <RbfDetailsItem
          label="Replaceable"
          value={formatBooleanLabel(psbt.replaceable)}
        />
      </div>

      {psbt.selected_inputs.length > 0 && (
        <div className="rbf-workflow__section">
          <div className="rbf-workflow__section-title">Selected inputs</div>
          <div className="rbf-workflow__mono-list">
            {psbt.selected_inputs.map((input) => (
              <div key={input} title={input}>
                {shortOutpoint(input)}
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="rbf-workflow__section">
        <div className="rbf-workflow__section-title">PSBT base64</div>
        <textarea
          className="rbf-workflow__psbt"
          value={psbt.psbt_base64}
          readOnly
          rows={5}
        />
      </div>

      {signedPsbt && (
        <div className="transactions-action-message">
          Signed PSBT ready{signedPsbt.finalized ? " and finalized" : ""}.
        </div>
      )}

      {broadcastResult && (
        <div className="transactions-action-message">
          Broadcast complete: <span title={broadcastResult.txid}>{shortTxid(broadcastResult.txid)}</span>
        </div>
      )}

      <div className="rbf-workflow__actions">
        <button
          type="button"
          className="primary-button"
          onClick={onSign}
          disabled={!canSign}
        >
          {loading && signedPsbt === null ? "Signing..." : "Sign PSBT"}
        </button>

        <button
          type="button"
          className="primary-button"
          onClick={onBroadcast}
          disabled={!canBroadcast}
        >
          {loading && signedPsbt !== null ? "Broadcasting..." : "Broadcast"}
        </button>
      </div>
    </div>
  );
}

type RbfDetailsItemProps = {
  label: string;
  value: string;
  title?: string;
};

function RbfDetailsItem({ label, value, title }: RbfDetailsItemProps) {
  return (
    <div className="transactions-details-item">
      <div className="transactions-details-label">{label}</div>
      <div className="transactions-details-value" title={title}>{value}</div>
    </div>
  );
}