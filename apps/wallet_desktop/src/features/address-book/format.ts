export function formatAddressBookLabel(label: string | null | undefined): string {
  const normalized = label?.trim();

  return normalized && normalized.length > 0 ? normalized : "—";
}

export function formatAddressBookAddress(address: string | null | undefined): string {
  const normalized = address?.trim();

  if (!normalized) {
    return "—";
  }

  if (normalized.length <= 24) {
    return normalized;
  }

  return `${normalized.slice(0, 12)}…${normalized.slice(-10)}`;
}

export function formatAddressBookNetwork(network: string | null | undefined): string {
  const normalized = network?.trim();

  return normalized && normalized.length > 0 ? normalized : "—";
}

export function formatAddressBookNotes(notes: string | null | undefined): string {
  const normalized = notes?.trim();

  return normalized && normalized.length > 0 ? normalized : "—";
}

export function formatAddressBookTimestamp(timestamp: string | null | undefined): string {
  const normalized = timestamp?.trim();

  if (!normalized) {
    return "—";
  }

  const date = new Date(normalized);

  if (Number.isNaN(date.getTime())) {
    return normalized;
  }

  return date.toLocaleString();
}
