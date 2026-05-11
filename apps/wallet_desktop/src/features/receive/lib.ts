import type { WalletReceiveAddressHistoryDto } from "../../shared/types/dtos";

export function getReceiveAddressString(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string {
  return address?.address?.trim() ?? "";
}

export function getReceiveKeychain(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string | null {
  const keychain = address?.keychain?.trim();

  return keychain && keychain.length > 0 ? keychain : null;
}

export function getReceiveIndex(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): number | null {
  const index = address?.index;

  if (index === null || index === undefined || !Number.isFinite(index)) {
    return null;
  }

  return index;
}

export function getReceiveBitcoinUri(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string {
  return address?.bitcoin_uri?.trim() ?? "";
}

export function getReceiveQrSvg(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string | null {
  const svg = address?.qr_svg?.trim();

  return svg && svg.length > 0 ? svg : null;
}

export function getReceiveLabel(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string | null {
  const label = address?.label?.trim();

  return label && label.length > 0 ? label : null;
}

export function getReceiveCreatedAt(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string | null {
  const createdAt = address?.created_at?.trim();

  return createdAt && createdAt.length > 0 ? createdAt : null;
}

export function getReceiveUpdatedAt(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): string | null {
  const updatedAt = address?.updated_at?.trim();

  return updatedAt && updatedAt.length > 0 ? updatedAt : null;
}

export function hasReceiveAddress(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): boolean {
  return getReceiveAddressString(address).length > 0;
}

export function hasReceiveQrSvg(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): boolean {
  return getReceiveQrSvg(address) !== null;
}

export function hasReceiveMetadata(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): boolean {
  return getReceiveKeychain(address) !== null || getReceiveIndex(address) !== null;
}

export function buildBitcoinUri(
  address: WalletReceiveAddressHistoryDto | string | null | undefined,
): string {
  if (typeof address !== "string") {
    const persistedUri = getReceiveBitcoinUri(address);

    if (persistedUri.length > 0) {
      return persistedUri;
    }
  }

  const addressString =
    typeof address === "string" ? address.trim() : getReceiveAddressString(address);

  if (!addressString) {
    return "";
  }

  return `bitcoin:${addressString}`;
}

export function getReceiveAddressMetadata(
  address: WalletReceiveAddressHistoryDto | null | undefined,
): Array<{ label: string; value: string | number | null }> {
  return [
    { label: "Keychain", value: getReceiveKeychain(address) },
    { label: "Index", value: getReceiveIndex(address) },
    { label: "Label", value: getReceiveLabel(address) },
    { label: "Created", value: getReceiveCreatedAt(address) },
    { label: "Updated", value: getReceiveUpdatedAt(address) },
  ].filter((item) => item.value !== null && item.value !== "");
}
