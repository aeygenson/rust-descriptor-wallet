import type { CpfpPsbtWorkflowPanelProps } from "../types";
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

export function CpfpPsbtWorkflowPanel({
  psbt,
  signedPsbt,
  broadcastResult,
  loading = false,
  onSign,
  onBroadcast,
  onClose,
}: CpfpPsbtWorkflowPanelProps) {
  const canSign = !loading && signedPsbt === null && broadcastResult === null;
  const canBroadcast =
    !loading && signedPsbt !== null && broadcastResult === null;

  const childTxidTitle = fullTxid(psbt.txid);
  const parentTxidTitle = fullTxid(psbt.parent_txid);
  const selectedOutpointTitle = fullOutpoint(psbt.selected_outpoint);
  const broadcastTxidTitle = broadcastResult
    ? fullTxid(broadcastResult.txid)
    : undefined;

  const childOutputBtc = formatBtcFromSats(psbt.child_output_value_sat);
  const feeBtc = formatBtcFromSats(psbt.fee_sat);

  return (
    <section className="tx-action-panel transactions-panel-section">
      <div>
        <h3>CPFP Transaction</h3>
        <p className="transactions-panel-muted" title={childTxidTitle}>
          Child transaction <code>{shortTxid(psbt.txid)}</code>
        </p>
        <p className="transactions-panel-muted">
          Review the CPFP child transaction, sign the PSBT, then broadcast it
          to raise the effective package fee rate.
        </p>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">
          Transaction links
        </div>
        <div className="transactions-panel-row">
          <span>Parent tx</span>
          <code title={parentTxidTitle}>{shortTxid(psbt.parent_txid)}</code>
        </div>
        <div className="transactions-panel-row">
          <span>Selected outpoint</span>
          <code title={selectedOutpointTitle}>
            {shortOutpoint(psbt.selected_outpoint)}
          </code>
        </div>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">Relation</div>
        <div className="transactions-panel-muted">
          This child transaction spends the selected outpoint from the parent
          transaction.
        </div>
        <div className="transactions-panel-row">
          <span>Spends</span>
          <code title={selectedOutpointTitle}>
            {shortOutpoint(psbt.selected_outpoint)}
          </code>
        </div>
        <div className="transactions-panel-row">
          <span>From parent</span>
          <code title={parentTxidTitle}>{shortTxid(psbt.parent_txid)}</code>
        </div>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">
          Child transaction economics
        </div>
        <div className="transactions-panel-row">
          <span>Input value</span>
          <strong>{formatSats(psbt.input_value_sat)}</strong>
        </div>
        <div className="transactions-panel-row">
          <span>Child output value</span>
          <strong>
            {formatSats(psbt.child_output_value_sat)}
            <span className="transactions-panel-muted">{childOutputBtc}</span>
          </strong>
        </div>
        <div className="transactions-panel-row">
          <span>Fee</span>
          <strong>
            {formatSats(psbt.fee_sat)}
            <span className="transactions-panel-muted">{feeBtc}</span>
          </strong>
        </div>
        <div className="transactions-panel-row">
          <span>Fee rate</span>
          <strong>{formatFeeRate(psbt.fee_rate_sat_per_vb)}</strong>
        </div>
        <div className="transactions-panel-row">
          <span>Replaceable</span>
          <strong>{formatBooleanLabel(psbt.replaceable)}</strong>
        </div>
        <div className="transactions-panel-row">
          <span>Estimated vsize</span>
          <strong>{formatVsize(psbt.estimated_vsize)}</strong>
        </div>
        <div className="transactions-panel-muted">
          Child output value equals selected input value minus the child fee.
          Broadcasting the child can help miners include the parent and child
          together.
        </div>
      </div>

      <div className="transactions-panel-section">
        <div className="transactions-panel-section-title">Unsigned PSBT</div>
        <label>
          <span>PSBT (base64)</span>
          <textarea readOnly value={psbt.psbt_base64} rows={4} />
        </label>
        <div className="transactions-panel-muted">
          This PSBT is unsigned. Sign it only after verifying the parent
          transaction, selected outpoint, fee, and output value.
        </div>
      </div>

      {signedPsbt && (
        <div className="transactions-panel-section">
          <div className="transactions-panel-section-title">Signed PSBT</div>
          <label>
            <span>Signed PSBT (base64)</span>
            <textarea readOnly value={signedPsbt.psbt_base64} rows={4} />
          </label>
          <div className="transactions-panel-muted">
            Signed PSBT is ready to broadcast. Broadcasting will publish the
            CPFP child transaction.
          </div>
        </div>
      )}

      {broadcastResult && (
        <div className="transactions-action-message success">
          Broadcasted:{" "}
          <code title={broadcastTxidTitle}>
            {shortTxid(broadcastResult.txid)}
          </code>
        </div>
      )}

      <div className="transactions-panel-actions">
        <button
          type="button"
          className="secondary"
          onClick={onClose}
        >
          Close
        </button>
        <button
          type="button"
          disabled={!canSign}
          title="Sign the CPFP child PSBT"
          onClick={onSign}
        >
          {loading ? "Signing…" : "Sign"}
        </button>
        <button
          type="button"
          disabled={!canBroadcast}
          title="Broadcast the signed CPFP child transaction"
          onClick={onBroadcast}
        >
          {loading ? "Broadcasting…" : "Broadcast"}
        </button>
      </div>
    </section>
  );
}