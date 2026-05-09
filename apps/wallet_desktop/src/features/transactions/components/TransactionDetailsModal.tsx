import {
  formatBooleanLabel,
  formatConfirmationHeight,
  formatConfirmationState,
  formatConfirmationStateClass,
  formatDirectionLabel,
  formatFeeRate,
  formatOwnershipLabel,
  formatSats,
  formatSignedBtc,
  formatSignedSats,
  fullOutpoint,
  fullTxid,
  shortOutpoint,
  shortTxid,
} from "../format";
import {
  extractParentTxids,
  getTransactionConfirmationState,
  getTransactionDisplayDirection,
  getTransactionNetAmountSat,
} from "../lib";
import type { TransactionDetailsModalProps } from "../types";
import { TransactionIntentBadge } from "./TransactionIntentBadge";

export function TransactionDetailsModal({
  tx,
  intent = "unknown",
  onClose,
  onOpenTx,
}: TransactionDetailsModalProps) {
  const confirmationState = getTransactionConfirmationState(tx);
  const netAmountSat = getTransactionNetAmountSat(tx);
  const parentTxids = extractParentTxids(tx);
  const childTxids = tx.child_txids ?? [];
  const hasParents = parentTxids.length > 0;
  const hasChildren = childTxids.length > 0;

  return (
    <div className="transactions-details-overlay" onClick={onClose}>
      <div
        className="transactions-details-panel"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="transactions-details-header">
          <h2>Transaction details</h2>
          <button
            type="button"
            className="transactions-details-close"
            onClick={onClose}
            aria-label="Close transaction details"
          >
            ✕
          </button>
        </div>

        <div className="transactions-details-grid">
          <TransactionDetailsItem
            label="Txid"
            value={shortTxid(tx.txid)}
            title={fullTxid(tx.txid)}
          />
          <TransactionDetailsItem label="Confirmed" value={formatBooleanLabel(tx.confirmed)} />
          <div className="transactions-details-item">
            <div className="transactions-details-label">State</div>
            <div className="transactions-details-value">
              <span className={formatConfirmationStateClass(confirmationState)}>
                {formatConfirmationState(confirmationState)}
              </span>
            </div>
          </div>
          <TransactionDetailsItem
            label="Confirmation height"
            value={formatConfirmationHeight(tx.confirmation_height)}
          />
          <TransactionDetailsItem
            label="Direction"
            value={formatDirectionLabel(getTransactionDisplayDirection(tx))}
          />
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
          <TransactionDetailsItem
            label="Net value"
            value={`${formatSignedSats(tx.net_value_sat)} · ${formatSignedBtc(netAmountSat)}`}
          />
          <TransactionDetailsItem label="Fee" value={formatSats(tx.fee_sat)} />
          <TransactionDetailsItem
            label="Fee rate"
            value={formatFeeRate(tx.fee_rate_sat_per_vb)}
          />
        </div>

        {tx.inputs && tx.inputs.length > 0 && (
          <div className="transactions-panel-section transactions-panel-section--spaced">
            <div className="transactions-panel-section-title">Inputs</div>
            {tx.inputs.map((input) => (
              <div key={input.previous_outpoint} className="transactions-panel-row">
                <span title={fullOutpoint(input.previous_outpoint)}>
                  {shortOutpoint(input.previous_outpoint)}
                </span>
              </div>
            ))}
          </div>
        )}

        {tx.outputs && tx.outputs.length > 0 && (
          <div className="transactions-panel-section transactions-panel-section--spaced">
            <div className="transactions-panel-section-title">Outputs</div>
            {tx.outputs.map((output) => (
              <div key={output.outpoint} className="transactions-panel-row">
                <div className="transactions-output-cell">
                  <span title={fullOutpoint(output.outpoint)}>
                    {shortOutpoint(output.outpoint)}
                  </span>
                  {output.address && (
                    <span className="transactions-output-address">
                      {output.address}
                    </span>
                  )}
                </div>
                <div className="transactions-output-value">
                  <strong>{formatSats(output.value_sat)}</strong>
                  <div
                    className={
                      output.is_mine
                        ? "transactions-output-ownership transactions-output-ownership--mine"
                        : "transactions-output-ownership"
                    }
                  >
                    {formatOwnershipLabel(output.is_mine)}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {hasParents && (
          <div className="transactions-panel-section transactions-panel-section--spaced">
            <div className="transactions-panel-section-title">Parents</div>
            {parentTxids.map((parentTxid) => (
              <div key={parentTxid} className="transactions-panel-row">
                <button
                  type="button"
                  className="transactions-graph-link"
                  title={fullTxid(parentTxid)}
                  disabled={!onOpenTx}
                  onClick={() => onOpenTx?.(parentTxid)}
                >
                  {shortTxid(parentTxid)}
                </button>
              </div>
            ))}
          </div>
        )}

        {hasChildren && (
          <div className="transactions-panel-section transactions-panel-section--spaced">
            <div className="transactions-panel-section-title">Children</div>
            {childTxids.map((child) => (
              <div key={child} className="transactions-panel-row">
                <button
                  type="button"
                  className="transactions-graph-link"
                  title={fullTxid(child)}
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

function TransactionDetailsItem({
  label,
  value,
  title,
}: TransactionDetailsItemProps) {
  return (
    <div className="transactions-details-item">
      <div className="transactions-details-label">{label}</div>
      <div className="transactions-details-value" title={title}>
        {value}
      </div>
    </div>
  );
}