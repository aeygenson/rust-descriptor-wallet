import type {
  WalletReceiveAddressHistoryDto,
} from "../../shared/types/dtos";
import { invokeCommand } from "../../shared/lib/tauri";

function requireWalletName(walletName: string): string {
  const normalizedWalletName = walletName.trim();

  if (!normalizedWalletName) {
    throw new Error("Wallet name is required");
  }

  return normalizedWalletName;
}

function requireReceiveAddress(address: string): string {
  const normalizedAddress = address.trim();

  if (!normalizedAddress) {
    throw new Error("Receive address is required");
  }

  return normalizedAddress;
}

function requireReceiveAddressLabel(label: string): string {
  const normalizedLabel = label.trim();

  if (!normalizedLabel) {
    throw new Error("Receive address label is required");
  }

  return normalizedLabel;
}

export async function getReceiveAddress(
  walletName: string,
): Promise<WalletReceiveAddressHistoryDto> {
  const normalizedWalletName = requireWalletName(walletName);

  return invokeCommand<WalletReceiveAddressHistoryDto>("get_receive_address", {
    walletName: normalizedWalletName,
  });
}

export async function listReceiveAddresses(
  walletName: string,
): Promise<WalletReceiveAddressHistoryDto[]> {
  const normalizedWalletName = requireWalletName(walletName);

  return invokeCommand<WalletReceiveAddressHistoryDto[]>(
    "list_receive_addresses",
    {
      walletName: normalizedWalletName,
    },
  );
}

export async function labelReceiveAddress(
  walletName: string,
  address: string,
  label: string,
): Promise<WalletReceiveAddressHistoryDto> {
  const normalizedWalletName = requireWalletName(walletName);
  const normalizedAddress = requireReceiveAddress(address);
  const normalizedLabel = requireReceiveAddressLabel(label);

  return invokeCommand<WalletReceiveAddressHistoryDto>(
    "label_receive_address",
    {
      walletName: normalizedWalletName,
      address: normalizedAddress,
      label: normalizedLabel,
    },
  );
}

export async function clearReceiveAddressLabel(
  walletName: string,
  address: string,
): Promise<WalletReceiveAddressHistoryDto> {
  const normalizedWalletName = requireWalletName(walletName);
  const normalizedAddress = requireReceiveAddress(address);

  return invokeCommand<WalletReceiveAddressHistoryDto>(
    "clear_receive_address_label",
    {
      walletName: normalizedWalletName,
      address: normalizedAddress,
    },
  );
}