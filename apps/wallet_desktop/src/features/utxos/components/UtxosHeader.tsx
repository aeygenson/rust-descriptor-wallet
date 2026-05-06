import type { UtxosHeaderProps } from "../types";
import { UtxosSummaryCards } from "./UtxosSummaryCards";

export function UtxosHeader({ walletName, summary }: UtxosHeaderProps) {
  return (
    <header className="utxos-header">
      <div className="utxos-header__top">
        <h2 className="utxos-header__title">UTXOs</h2>

        {walletName && (
          <span className="utxos-header__wallet-pill">
            {walletName}
          </span>
        )}
      </div>

      <UtxosSummaryCards summary={summary} />
    </header>
  );
}