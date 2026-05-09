import type { UtxosHeaderProps } from "../types";
import { formatBtcFromSats } from "../format";
import { UtxosSummaryCards } from "./UtxosSummaryCards";

export function UtxosHeader({ walletName, summary }: UtxosHeaderProps) {
  const totalValueSat = summary.totalValue ?? 0;
  const formattedBtcValue = formatBtcFromSats(totalValueSat);
  const hasPending = summary.pendingCount > 0;

  return (
    <header className="utxos-header">
      <div className="utxos-header__top">
        <div className="utxos-header__hero">
          <div className="utxos-header__title-block">
            <h2 className="utxos-header__title">UTXOs</h2>

            <p className="utxos-header__subtitle">
              Inspect spendable outputs, confirmation state, wallet distribution, and coin selection readiness.
            </p>
          </div>

          {walletName && (
            <div className="utxos-header__wallet-pill-wrap">
              <span className="utxos-header__wallet-pill">
                {walletName}
              </span>
            </div>
          )}
        </div>

        <div className="utxos-header__meta">
          <span className="utxos-header__value">
            {totalValueSat.toLocaleString()} sats
          </span>

          <span className="utxos-header__value-secondary">
            {formattedBtcValue}
          </span>

          <span
            className={`utxos-header__status ${hasPending ? "utxos-header__status--pending" : "utxos-header__status--ready"}`}
            title={hasPending ? "Wallet contains pending UTXOs" : "All visible UTXOs are confirmed"}
          >
            {hasPending ? "Pending activity" : "Ready"}
          </span>
        </div>
      </div>

      <div className="utxos-header__hint">
        Selected UTXOs can be forwarded directly into Send, Send Max, Sweep, Consolidation, RBF, and future CPFP flows.
      </div>

      <UtxosSummaryCards summary={summary} />
    </header>
  );
}