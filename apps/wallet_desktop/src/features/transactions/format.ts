import type { WalletTxDirection } from "./types";

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
  value: number | null | undefined
): string {
  if (value === null || value === undefined) {
    return "pending";
  }

  return value.toLocaleString();
}

export function formatOwnershipLabel(isMine: boolean): string {
  return isMine ? "wallet" : "external";
}

export function formatDirectionLabel(direction: WalletTxDirection | string): string {
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

export function formatDirectionClass(direction: WalletTxDirection | string): string {
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

export function shortTxid(txid: string): string {
  if (txid.length <= 16) {
    return txid;
  }

  return `${txid.slice(0, 8)}…${txid.slice(-8)}`;
}

export function shortOutpoint(outpoint: string): string {
  const separatorIndex = outpoint.lastIndexOf(":");
  if (separatorIndex === -1) {
    return shortTxid(outpoint);
  }

  const txid = outpoint.slice(0, separatorIndex);
  const vout = outpoint.slice(separatorIndex + 1);
  return `${shortTxid(txid)}:${vout}`;
}