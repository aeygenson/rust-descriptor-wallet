import { shortTxid } from "../format";
import type { TransactionRelationCellProps } from "../types";

export function TransactionRelationCell({ txids, kind, onOpenTx }: TransactionRelationCellProps) {
  if (!txids || txids.length === 0) {
    return <span className="transactions-muted">—</span>;
  }

  const previewTxids = txids.slice(0, 2);

  const badgeClass =
    kind === "parents"
      ? "transactions-badge transactions-badge--final"
      : "transactions-badge transactions-badge--replaceable";

  return (
    <div className="transactions-relations-cell">
      <span className={badgeClass}>{txids.length}</span>

      <div className="transactions-relations-preview" aria-label={`${kind} transaction ids`}>
        {previewTxids.map((txid) => (
          <button
            key={txid}
            type="button"
            title={txid}
            className="transactions-graph-link"
            disabled={!onOpenTx}
            onClick={() => onOpenTx?.(txid)}
            style={{ cursor: onOpenTx ? "pointer" : "default" }}
          >
            {shortTxid(txid)}
          </button>
        ))}

        {txids.length > previewTxids.length && (
          <span className="transactions-muted">
            +{txids.length - previewTxids.length}
          </span>
        )}
      </div>
    </div>
  );
}