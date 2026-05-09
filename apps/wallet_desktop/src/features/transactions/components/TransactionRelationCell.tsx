import { fullTxid, shortTxid } from "../format";
import type { TransactionRelationCellProps } from "../types";

export function TransactionRelationCell({
  txids,
  kind,
  onOpenTx,
}: TransactionRelationCellProps) {
  if (!txids || txids.length === 0) {
    return <span className="transactions-muted">—</span>;
  }

  const previewTxids = txids.slice(0, 2);
  const hiddenCount = txids.length - previewTxids.length;
  const relationLabel = kind === "parents" ? "Parent" : "Child";
  const relationAriaLabel = `${kind} transaction relations`;
  const relationPreviewAriaLabel = `${kind} transaction ids`;

  const relationDescription =
    kind === "parents"
      ? "Transactions referenced as inputs"
      : "Transactions spending outputs from this transaction";

  const badgeClass =
    kind === "parents"
      ? "transactions-badge transactions-badge--final"
      : "transactions-badge transactions-badge--replaceable";

  const badgeTitle = `${txids.length} ${kind} · ${relationDescription}`;
  const badgeAriaLabel = `${txids.length} ${kind}`;

  return (
    <div
      className="transactions-relations-cell"
      data-kind={kind}
      aria-label={relationAriaLabel}
    >
      <span
        className={badgeClass}
        title={badgeTitle}
        aria-label={badgeAriaLabel}
      >
        {txids.length}
      </span>

      <div
        className="transactions-relations-preview"
        aria-label={relationPreviewAriaLabel}
        title={relationDescription}
      >
        {previewTxids.map((txid) => (
          <button
            key={txid}
            type="button"
            title={fullTxid(txid)}
            aria-label={`${relationLabel} transaction ${txid}`}
            className="transactions-graph-link"
            data-txid={txid}
            disabled={!onOpenTx}
            data-clickable={Boolean(onOpenTx)}
            onClick={() => onOpenTx?.(txid)}
          >
            {shortTxid(txid)}
          </button>
        ))}

        {hiddenCount > 0 && (
          <span
            className="transactions-muted"
            title={`${hiddenCount} additional ${kind}`}
            aria-label={`${hiddenCount} additional ${kind}`}
          >
            +{hiddenCount}
          </span>
        )}
      </div>
    </div>
  );
}