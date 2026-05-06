import { formatSats } from "../format";
import type { UtxosSummaryCardsProps } from "../types";

export function UtxosSummaryCards({ summary }: UtxosSummaryCardsProps) {
  return (
    <section className="utxos-summary">
      <div className="utxos-summary__grid">
        <div className="utxos-summary__card">
          <div className="utxos-summary__label">Total UTXOs</div>
          <div className="utxos-summary__value">{summary.totalCount}</div>
        </div>

        <div className="utxos-summary__card">
          <div className="utxos-summary__label">Total Value</div>
          <div className="utxos-summary__value">
            {formatSats(summary.totalValue)}
          </div>
        </div>

        <div className="utxos-summary__card">
          <div className="utxos-summary__label">Average Value</div>
          <div className="utxos-summary__value">
            {formatSats(summary.averageValue)}
          </div>
        </div>

        <div className="utxos-summary__card">
          <div className="utxos-summary__label">Confirmed</div>
          <div className="utxos-summary__value">
            {summary.confirmedCount} / {formatSats(summary.confirmedValue)}
          </div>
        </div>

        <div className="utxos-summary__card">
          <div className="utxos-summary__label">Pending</div>
          <div className="utxos-summary__value">
            {summary.pendingCount} / {formatSats(summary.pendingValue)}
          </div>
        </div>

        <div className="utxos-summary__card">
          <div className="utxos-summary__label">Keychains</div>
          <div className="utxos-summary__value">{summary.keychains}</div>
        </div>
      </div>
    </section>
  );
}
