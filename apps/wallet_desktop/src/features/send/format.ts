import type { SendMode } from "./types";

export function formatSats(value: number): string {
  if (!Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} sats`;
}

export function formatBtc(valueSat: number | null | undefined): string {
  if (
    valueSat === null ||
    valueSat === undefined ||
    !Number.isFinite(valueSat)
  ) {
    return "—";
  }

  return `${(valueSat / 100_000_000).toFixed(8)} BTC`;
}

export function formatOptionalSats(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—";
  return formatSats(value);
}

export function formatOptionalValue(
  value: number | string | null | undefined,
): string {
  if (value === null || value === undefined || value === "") return "—";

  if (typeof value === "number") {
    return Number.isFinite(value) ? value.toLocaleString() : "—";
  }

  return value;
}

export function formatFeeRateSatPerVb(
  value: number | null | undefined,
): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} sat/vB`;
}

export function formatRelativeFeeRate(
  current: number | null | undefined,
  baseline: number | null | undefined,
): string {
  if (
    current === null ||
    current === undefined ||
    baseline === null ||
    baseline === undefined
  ) {
    return "—";
  }

  const delta = current - baseline;
  const sign = delta > 0 ? "+" : "";

  return `${sign}${delta.toFixed(1)} sat/vB`;
}

export function formatPercent(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) {
    return "—";
  }

  return `${value.toLocaleString()}%`;
}

export function formatRelativePercent(
  current: number | null | undefined,
  baseline: number | null | undefined,
): string {
  if (
    current === null ||
    current === undefined ||
    baseline === null ||
    baseline === undefined ||
    baseline === 0
  ) {
    return "—";
  }

  const deltaPercent = ((current - baseline) / baseline) * 100;
  const sign = deltaPercent > 0 ? "+" : "";

  return `${sign}${deltaPercent.toFixed(0)}%`;
}

export function formatTxid(txid: string | null | undefined): string {
  if (!txid) {
    return "—";
  }

  if (txid.length <= 16) {
    return txid;
  }

  return `${txid.slice(0, 8)}…${txid.slice(-8)}`;
}

export function formatOutpoint(outpoint: string | null | undefined): string {
  if (!outpoint) {
    return "—";
  }

  const [txid, vout] = outpoint.split(":");

  if (!txid || vout === undefined) {
    return outpoint;
  }

  return `${formatTxid(txid)}:${vout}`;
}

export function formatVsize(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} vB`;
}

export function formatNullableBoolean(
  value: boolean | null | undefined,
): string {
  if (value === null || value === undefined) return "—";
  return value ? "Yes" : "No";
}

export function formatSelectedInputCount(count: number): string {
  if (!Number.isFinite(count)) return "—";
  return `${count.toLocaleString()} input${count === 1 ? "" : "s"}`;
}

export function formatSelectedInput(
  valueSat: number | null | undefined,
): string {
  if (valueSat === null || valueSat === undefined || !Number.isFinite(valueSat)) {
    return "—";
  }

  return formatSats(valueSat);
}

export function formatSelectedInputWithBtc(
  valueSat: number | null | undefined,
): string {
  if (
    valueSat === null ||
    valueSat === undefined ||
    !Number.isFinite(valueSat)
  ) {
    return "—";
  }

  return `${formatSats(valueSat)} · ${formatBtc(valueSat)}`;
}

export function getSendModeDescription(
  mode: SendMode,
): string {
  switch (mode) {
    case "fixed":
      return "Send a specific amount to one recipient.";
    case "send_max":
      return "Spend the maximum available amount after fees.";
    case "sweep":
      return "Spend only explicitly selected UTXOs to a recipient.";
    case "consolidate":
      return "Merge selected UTXOs into a new wallet-controlled output.";
    default:
      return "";
  }
}