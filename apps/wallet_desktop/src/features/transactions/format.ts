import type {
  TransactionConfirmationState,
  TransactionIntent,
  WalletTxDirection,
} from "./types";

export function formatSats(value: number | null | undefined): string {
  if (value === null || value === undefined) {
    return "—";
  }

  return `${value.toLocaleString()} sats`;
}

export function formatSignedSats(value: number): string {
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toLocaleString()} sats`;
}

export function formatBtcFromSats(
  valueSat: number | null | undefined,
): string {
  if (valueSat === null || valueSat === undefined) {
    return "—";
  }

  return `${(valueSat / 100_000_000).toFixed(8)} BTC`;
}

export function formatFeeRate(value: number | null | undefined): string {
  if (value === null || value === undefined) {
    return "—";
  }

  return `${value.toLocaleString(undefined, {
    maximumFractionDigits: 2,
  })} sat/vB`;
}

export function formatBooleanLabel(value: boolean | null | undefined): string {
  if (value === null || value === undefined) {
    return "—";
  }

  return value ? "yes" : "no";
}

export function formatVsize(value: number | null | undefined): string {
  if (value === null || value === undefined) {
    return "—";
  }

  return `${value.toLocaleString()} vB`;
}

export function formatConfirmationHeight(
  value: number | null | undefined,
): string {
  if (value === null || value === undefined) {
    return "pending";
  }

  return value.toLocaleString();
}

export function formatConfirmationState(
  state: TransactionConfirmationState,
): string {
  switch (state) {
    case "pending":
      return "Pending";
    case "confirmed":
      return "Confirmed";
    case "finalized":
      return "Finalized";
    default:
      return "Unknown";
  }
}

export function formatConfirmationStateClass(
  state: TransactionConfirmationState,
): string {
  return `transactions-confirmation transactions-confirmation--${state}`;
}

export function formatOwnershipLabel(isMine: boolean): string {
  return isMine ? "wallet" : "external";
}

export function formatTransactionIntentLabel(
  intent: TransactionIntent,
): string {
  switch (intent) {
    case "fixed":
      return "Fixed send";
    case "send_max":
      return "Send Max";
    case "sweep":
      return "Sweep";
    case "consolidation":
      return "Consolidation";
    case "rbf":
      return "RBF";
    case "cpfp":
      return "CPFP";
    case "unknown":
    default:
      return "Standard";
  }
}

export function formatTransactionIntentClass(
  intent: TransactionIntent,
): string {
  return `transactions-intent transactions-intent--${intent}`;
}

export function formatDirectionLabel(
  direction: WalletTxDirection | string,
): string {
  switch (direction) {
    case "received":
      return "Received";
    case "sent":
      return "Sent";
    case "self_transfer":
      return "Self transfer";
    default:
      return direction;
  }
}

export function formatDirectionClass(
  direction: WalletTxDirection | string,
): string {
  switch (direction) {
    case "received":
      return "received";
    case "sent":
      return "sent";
    case "self_transfer":
      return "self";
    default:
      return "unknown";
  }
}

export function formatSignedBtc(
  valueSat: number | null | undefined,
): string {
  if (valueSat === null || valueSat === undefined) {
    return "—";
  }

  const btc = valueSat / 100_000_000;
  const sign = btc > 0 ? "+" : "";

  return `${sign}${btc.toFixed(8)} BTC`;
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

  return `${sign}${delta.toFixed(2)} sat/vB`;
}

export function formatRelativeFeeRatePercent(
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

export function shortTxid(txid: string): string {
  if (txid.length <= 16) {
    return txid;
  }

  return `${txid.slice(0, 8)}…${txid.slice(-8)}`;
}

export function fullTxid(txid: string | null | undefined): string {
  return txid ?? "—";
}

export function shortOutpoint(
  outpoint: string | null | undefined,
): string {
  if (!outpoint) {
    return "—";
  }

  const separatorIndex = outpoint.lastIndexOf(":");

  if (separatorIndex === -1) {
    return shortTxid(outpoint);
  }

  const txid = outpoint.slice(0, separatorIndex);
  const vout = outpoint.slice(separatorIndex + 1);
  return `${shortTxid(txid)}:${vout}`;
}

export function fullOutpoint(
  outpoint: string | null | undefined,
): string {
  return outpoint ?? "—";
}