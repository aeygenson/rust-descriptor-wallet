import {
  formatBtcFromSats,
  formatConfirmationBadge,
  formatSats,
} from "../format";
import type { UtxosSummaryCardsProps } from "../types";

export function UtxosSummaryCards({ summary }: UtxosSummaryCardsProps) {
  const totalValueBtc = formatBtcFromSats(summary.totalValue);
  const averageValueBtc = formatBtcFromSats(summary.averageValue);
  const confirmedRatio =
    summary.totalCount > 0
      ? Math.round((summary.confirmedCount / summary.totalCount) * 100)
      : 0;
  const pendingRatio =
    summary.totalCount > 0
      ? Math.round((summary.pendingCount / summary.totalCount) * 100)
      : 0;
  const keychainCount = summary.keychains.toLocaleString();

  return (
    <section className="utxos-summary">
      <div className="utxos-summary__grid">
        <SummaryCard
          tone="blue"
          label="Total UTXOs"
          value={summary.totalCount.toLocaleString()}
          subvalue="Spendable wallet outputs currently tracked"
          progress={summary.totalCount > 0 ? 100 : 0}
        />

        <SummaryCard
          tone="green"
          label="Total Value"
          value={formatSats(summary.totalValue)}
          subvalue={`${totalValueBtc} available across all UTXOs`}
          progress={summary.totalValue > 0 ? 100 : 0}
        />

        <SummaryCard
          tone="slate"
          label="Average Value"
          value={formatSats(summary.averageValue)}
          subvalue={`${averageValueBtc} average per wallet output`}
        />

        <SummaryCard
          tone="green"
          label="Confirmed"
          value={[
            summary.confirmedCount.toLocaleString(),
            formatSats(summary.confirmedValue),
          ].join(" / ")}
          subvalue={[
            `${confirmedRatio}% confirmed`,
            `${formatConfirmationBadge(6)} ready`,
          ].join(" · ")}
          progress={confirmedRatio}
        />

        <SummaryCard
          tone="amber"
          label="Pending"
          value={[
            summary.pendingCount.toLocaleString(),
            formatSats(summary.pendingValue),
          ].join(" / ")}
          subvalue={
            summary.pendingCount > 0
              ? `${pendingRatio}% of wallet outputs awaiting confirmation`
              : "No pending wallet activity"
          }
          progress={pendingRatio}
        />

        <SummaryCard
          tone="blue"
          label="Keychains"
          value={keychainCount}
          subvalue="External + internal derivation paths"
        />
      </div>
    </section>
  );
}

type SummaryCardTone = "blue" | "green" | "amber" | "slate";

type SummaryCardProps = {
  tone: SummaryCardTone;
  label: string;
  value: string;
  subvalue: string;
  progress?: number;
};

function SummaryCard({
  tone,
  label,
  value,
  subvalue,
  progress,
}: SummaryCardProps) {
  const normalizedProgress =
    progress === undefined ? null : Math.max(0, Math.min(100, progress));

  return (
    <div className={`utxos-summary__card utxos-summary__card--${tone}`}>
      <div className="utxos-summary__card-top">
        <span className="utxos-summary__badge">{label}</span>
      </div>

      <div className="utxos-summary__value">{value}</div>

      <div className="utxos-summary__subvalue">{subvalue}</div>

      {normalizedProgress !== null && (
        <div
          className="utxos-summary__progress"
          aria-label={`${label} progress ${normalizedProgress}%`}
        >
          <span style={{ width: `${normalizedProgress}%` }} />
          <strong>{normalizedProgress}%</strong>
        </div>
      )}
    </div>
  );
}
