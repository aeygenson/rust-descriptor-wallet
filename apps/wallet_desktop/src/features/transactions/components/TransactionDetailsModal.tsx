import {
  formatBooleanLabel,
  formatConfirmationHeight,
  formatDirectionLabel,
  formatFeeRate,
  formatOwnershipLabel,
  formatSats,
  formatSignedSats,
  shortTxid,
} from "../format";
import { extractParentTxids } from "../lib";
import type { TransactionDetailsModalProps } from "../types";
import { TransactionIntentBadge } from "./TransactionIntentBadge";

export function TransactionDetailsModal({
  tx,
  intent = "unknown",
  onClose,
  onOpenTx,
}: TransactionDetailsModalProps) {
  return (
    <div className="transactions-details-overlay" onClick={onClose}>
      <div
        className="transactions-details-panel"
        onClick={(e) => e.stopPropagation()}
        style={{ position: "relative" }}
      >
        <div className="transactions-details-header">
          <h2>Transaction details</h2>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close transaction details"
            style={{
              position: "absolute",
              top: 12,
              right: 12,
              background: "transparent",
              border: "none",
              color: "#e5e7eb",
              fontSize: 20,
              fontWeight: 700,
              cursor: "pointer",
            }}
          >
            ✕
          </button>
        </div>

        <div className="transactions-details-grid">
          <TransactionDetailsItem label="Txid" value={shortTxid(tx.txid)} title={tx.txid} />
          <TransactionDetailsItem label="Confirmed" value={formatBooleanLabel(tx.confirmed)} />
          <TransactionDetailsItem
            label="Confirmation height"
            value={formatConfirmationHeight(tx.confirmation_height)}
          />
          <TransactionDetailsItem label="Direction" value={formatDirectionLabel(tx.direction)} />
          <div className="transactions-details-item">
            <div className="transactions-details-label">Intent</div>
            <div className="transactions-details-value">
              <TransactionIntentBadge intent={intent} />
            </div>
          </div>
          <TransactionDetailsItem
            label="Replaceable"
            value={formatBooleanLabel(tx.replaceable)}
          />
          <TransactionDetailsItem label="Net value" value={formatSignedSats(tx.net_value)} />
          <TransactionDetailsItem label="Fee" value={formatSats(tx.fee)} />
          <TransactionDetailsItem label="Fee rate" value={formatFeeRate(tx.fee_rate_sat_per_vb)} />
        </div>

        {tx.inputs && tx.inputs.length > 0 && (
          <div className="transactions-panel-section" style={{ marginTop: 16 }}>
            <div className="transactions-panel-section-title">Inputs</div>
            {tx.inputs.map((input) => (
              <div key={input.previous_outpoint} className="transactions-panel-row">
                <span title={input.previous_outpoint}>
                  {input.previous_outpoint}
                </span>
              </div>
            ))}
          </div>
        )}

        {tx.outputs && tx.outputs.length > 0 && (
          <div className="transactions-panel-section" style={{ marginTop: 16 }}>
            <div className="transactions-panel-section-title">Outputs</div>
            {tx.outputs.map((output) => (
              <div key={output.outpoint} className="transactions-panel-row">
                <div style={{ display: "flex", flexDirection: "column" }}>
                  <span title={output.outpoint}>
                    {output.outpoint}
                  </span>
                  {output.address && (
                    <span style={{ color: "#94a3b8", fontSize: 12 }}>
                      {output.address}
                    </span>
                  )}
                </div>
                <div style={{ textAlign: "right" }}>
                  <strong>{formatSats(output.value_sat)}</strong>
                  <div style={{ fontSize: 12, color: output.is_mine ? "#22c55e" : "#94a3b8" }}>
                    {formatOwnershipLabel(output.is_mine)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {tx.inputs && tx.inputs.length > 0 && (
          <div className="transactions-panel-section" style={{ marginTop: 16 }}>
            <div className="transactions-panel-section-title">Parents</div>
            {extractParentTxids(tx).map((parentTxid) => (
              <div key={parentTxid} className="transactions-panel-row">
                <button
                  type="button"
                  className="transactions-graph-link"
                  title={parentTxid}
                  disabled={!onOpenTx}
                  onClick={() => onOpenTx?.(parentTxid)}
                >
                  {shortTxid(parentTxid)}
                </button>
              </div>
            ))}
          </div>
        )}

        {tx.child_txids && tx.child_txids.length > 0 && (
          <div className="transactions-panel-section" style={{ marginTop: 16 }}>
            <div className="transactions-panel-section-title">Children</div>
            {tx.child_txids.map((child) => (
              <div key={child} className="transactions-panel-row">
                <button
                  type="button"
                  className="transactions-graph-link"
                  title={child}
                  disabled={!onOpenTx}
                  onClick={() => onOpenTx?.(child)}
                >
                  {shortTxid(child)}
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

type TransactionDetailsItemProps = {
  label: string;
  value: string;
  title?: string;
};

function TransactionDetailsItem({ label, value, title }: TransactionDetailsItemProps) {
  return (
    <div className="transactions-details-item">
      <div className="transactions-details-label">{label}</div>
      <div className="transactions-details-value" title={title}>{value}</div>
    </div>
  );
}