import type { WalletReceiveAddressDto } from "../../shared/types/dtos";

export function getReceiveAddressString(
  address: WalletReceiveAddressDto | null | undefined,
): string {
  return address?.address?.trim() ?? "";
}

export function getReceiveKeychain(
  address: WalletReceiveAddressDto | null | undefined,
): string | null {
  const keychain = address?.keychain?.trim();

  return keychain && keychain.length > 0 ? keychain : null;
}

export function getReceiveIndex(
  address: WalletReceiveAddressDto | null | undefined,
): number | null {
  const index = address?.index;

  if (index === null || index === undefined || !Number.isFinite(index)) {
    return null;
  }

  return index;
}

export function hasReceiveAddress(
  address: WalletReceiveAddressDto | null | undefined,
): boolean {
  return getReceiveAddressString(address).length > 0;
}

export function hasReceiveMetadata(
  address: WalletReceiveAddressDto | null | undefined,
): boolean {
  return getReceiveKeychain(address) !== null || getReceiveIndex(address) !== null;
}

export function buildBitcoinUri(
  address: WalletReceiveAddressDto | string | null | undefined,
): string {
  const addressString =
    typeof address === "string" ? address.trim() : getReceiveAddressString(address);

  if (!addressString) {
    return "";
  }

  return `bitcoin:${addressString}`;
}

export function getReceiveAddressMetadata(
  address: WalletReceiveAddressDto | null | undefined,
): Array<{ label: string; value: string | number | null }> {
  return [
    { label: "Keychain", value: getReceiveKeychain(address) },
    { label: "Index", value: getReceiveIndex(address) },
  ].filter((item) => item.value !== null && item.value !== "");
}
