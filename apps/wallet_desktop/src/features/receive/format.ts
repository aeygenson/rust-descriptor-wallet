

export function formatReceiveAddress(address: string): string {
  const normalized = address.trim();

  if (normalized.length <= 18) {
    return normalized;
  }

  return `${normalized.slice(0, 10)}…${normalized.slice(-8)}`;
}

export function formatAddressIndex(index: number | null | undefined): string {
  if (
    index === null ||
    index === undefined ||
    !Number.isFinite(index)
  ) {
    return "—";
  }

  return `#${index.toLocaleString()}`;
}

export function formatAddressUsageLabel(
  used: boolean | null | undefined,
): string {
  if (used === null || used === undefined) {
    return "Unknown";
  }

  return used ? "Used" : "Unused";
}

export function formatReceiveTimestamp(
  timestamp: string | null | undefined,
): string {
  if (!timestamp) {
    return "—";
  }

  const parsed = new Date(timestamp);

  if (Number.isNaN(parsed.getTime())) {
    return "—";
  }

  return parsed.toLocaleString();
}

export function formatReceiveLabel(
  label: string | null | undefined,
): string {
  const normalized = label?.trim();

  if (!normalized) {
    return "—";
  }

  return normalized;
}

export function formatBitcoinUri(
  uri: string | null | undefined,
): string {
  const normalized = uri?.trim();

  if (!normalized) {
    return "—";
  }

  if (normalized.length <= 36) {
    return normalized;
  }

  return `${normalized.slice(0, 18)}…${normalized.slice(-14)}`;
}

export function formatDerivationPath(
  path: string | null | undefined,
): string {
  const normalized = path?.trim();

  if (!normalized) {
    return "—";
  }

  return normalized;
}