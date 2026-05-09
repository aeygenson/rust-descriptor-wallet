import type { RbfPsbtWorkflowPanelProps } from "../types";
import {
  formatBooleanLabel,
  formatBtcFromSats,
  formatFeeRate,
  formatSats,
  formatVsize,
  fullOutpoint,
  fullTxid,
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
  const canBroadcast =
    !loading && signedPsbt !== null && broadcastResult === null;

  const originalTxidTitle = fullTxid(psbt.original_txid);
  const replacementTxidTitle = fullTxid(psbt.txid);
  const broadcastTxidTitle = broadcastResult
    ? fullTxid(broadcastResult.txid)
    : undefined;

  const feeBtc = formatBtcFromSats(psbt.fee_sat);

  return (
    <div className="transactions-details-panel">
      <div className="rbf-workflow__header">
        <div>
          <h2>Replacement PSBT</h2>
          {psbt.original_txid && (
            <div className="rbf-workflow__subtitle">
              Replaces{" "}
              <span title={originalTxidTitle}>
                {shortTxid(psbt.original_txid)}
              </span>
            </div>
          )}
          <div className="rbf-workflow__subtitle">
            Review the replacement transaction, sign the PSBT, then broadcast it
            to replace the original pending transaction.
          </div>
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
        <RbfDetailsItem
          label="Replacement txid"
          value={psbt.txid ? shortTxid(psbt.txid) : "—"}
          title={replacementTxidTitle}
        />
        <RbfDetailsItem
          label="Fee"
          value={formatSats(psbt.fee_sat)}
          secondaryValue={feeBtc}
        />
        <RbfDetailsItem
          label="Fee rate"
          value={formatFeeRate(psbt.fee_rate_sat_per_vb)}
        />
        <RbfDetailsItem
          label="Estimated vsize"
          value={formatVsize(psbt.estimated_vsize)}
        />
        <RbfDetailsItem label="Inputs" value={psbt.input_count.toLocaleString()} />
        <RbfDetailsItem
          label="Outputs"
          value={psbt.output_count.toLocaleString()}
        />
        <RbfDetailsItem
          label="Selected UTXOs"
          value={psbt.selected_utxo_count.toLocaleString()}
        />
        <RbfDetailsItem
          label="Replaceable"
          value={formatBooleanLabel(psbt.replaceable)}
        />
        <RbfDetailsItem
          label="Workflow"
          value={signedPsbt ? "signed" : broadcastResult ? "broadcast" : "unsigned"}
        />
      </div>

      {psbt.selected_inputs.length > 0 && (
        <div className="rbf-workflow__section">
          <div className="rbf-workflow__section-title">Selected inputs</div>
          <div className="rbf-workflow__mono-list">
            {psbt.selected_inputs.map((input) => (
              <div key={input} title={fullOutpoint(input)}>
                {shortOutpoint(input)}
              </div>
            ))}
          </div>
          <div className="transactions-panel-muted">
            These inputs are selected by the replacement PSBT. Verify they
            match the original transaction before signing.
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
        <div className="transactions-panel-muted">
          This PSBT is unsigned. Sign it only after verifying fee rate, inputs,
          outputs, and replacement metadata.
        </div>
      </div>

      {signedPsbt && (
        <div className="rbf-workflow__section">
          <div className="rbf-workflow__section-title">
            Signed PSBT{signedPsbt.finalized ? " · finalized" : ""}
          </div>
          <textarea
            className="rbf-workflow__psbt"
            value={signedPsbt.psbt_base64}
            readOnly
            rows={5}
          />
          <div className="transactions-panel-muted">
            Signed replacement PSBT is ready to broadcast.
          </div>
        </div>
      )}

      {broadcastResult && (
        <div className="transactions-action-message">
          Broadcast complete:{" "}
          <span title={broadcastTxidTitle}>
            {shortTxid(broadcastResult.txid)}
          </span>
        </div>
      )}

      <div className="rbf-workflow__actions">
        <button
          type="button"
          className="primary-button"
          title="Sign the replacement PSBT"
          onClick={onSign}
          disabled={!canSign}
        >
          {loading && signedPsbt === null ? "Signing..." : "Sign PSBT"}
        </button>

        <button
          type="button"
          className="primary-button"
          title="Broadcast the signed replacement transaction"
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
  secondaryValue?: string;
  title?: string;
};

function RbfDetailsItem({
  label,
  value,
  secondaryValue,
  title,
}: RbfDetailsItemProps) {
  return (
    <div className="transactions-details-item">
      <div className="transactions-details-label">{label}</div>
      <div className="transactions-details-value" title={title}>
        {value}
        {secondaryValue && (
          <span className="transactions-panel-muted">
            {secondaryValue}
          </span>
        )}
      </div>
    </div>
  );
}