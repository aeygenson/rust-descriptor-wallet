

import type { SendMode } from "./types";

export function formatSats(value: number): string {
  if (!Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} sats`;
}

export function formatOptionalSats(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—";
  return formatSats(value);
}

export function formatOptionalValue(value: number | string | null | undefined): string {
  if (value === null || value === undefined || value === "") return "—";

  if (typeof value === "number") {
    return Number.isFinite(value) ? value.toLocaleString() : "—";
  }

  return value;
}

export function formatFeeRateSatPerVb(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} sat/vB`;
}

export function formatVsize(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return "—";
  return `${value.toLocaleString()} vB`;
}

export function formatNullableBoolean(value: boolean | null | undefined): string {
  if (value === null || value === undefined) return "—";
  return value ? "Yes" : "No";
}

export function formatSelectedInputCount(count: number): string {
  if (!Number.isFinite(count)) return "—";
  return `${count.toLocaleString()} input${count === 1 ? "" : "s"}`;
}

export function formatSelectedInput(valueSat: number | null | undefined): string {
  if (valueSat === null || valueSat === undefined || !Number.isFinite(valueSat)) {
    return "—";
  }

  return formatSats(valueSat);
}

export function getSendModeDescription(mode: SendMode): string {
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