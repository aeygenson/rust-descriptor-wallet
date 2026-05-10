import type {
  WalletReceiveAddressHistoryDto,
} from "../../shared/types/dtos";
import { invokeCommand } from "../../shared/lib/tauri";

export async function getReceiveAddress(
  walletName: string,
): Promise<WalletReceiveAddressHistoryDto> {
  const normalizedWalletName = walletName.trim();

  if (!normalizedWalletName) {
    throw new Error("Wallet name is required");
  }

  return invokeCommand<WalletReceiveAddressHistoryDto>("get_receive_address", {
    walletName: normalizedWalletName,
  });
}

export async function listReceiveAddresses(
  walletName: string,
): Promise<WalletReceiveAddressHistoryDto[]> {
  const normalizedWalletName = walletName.trim();

  if (!normalizedWalletName) {
    throw new Error("Wallet name is required");
  }

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
  const normalizedWalletName = walletName.trim();
  const normalizedAddress = address.trim();
  const normalizedLabel = label.trim();

  if (!normalizedWalletName) {
    throw new Error("Wallet name is required");
  }

  if (!normalizedAddress) {
    throw new Error("Receive address is required");
  }

  if (!normalizedLabel) {
    throw new Error("Receive address label is required");
  }

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
  const normalizedWalletName = walletName.trim();
  const normalizedAddress = address.trim();

  if (!normalizedWalletName) {
    throw new Error("Wallet name is required");
  }

  if (!normalizedAddress) {
    throw new Error("Receive address is required");
  }

  return invokeCommand<WalletReceiveAddressHistoryDto>(
    "clear_receive_address_label",
    {
      walletName: normalizedWalletName,
      address: normalizedAddress,
    },
  );
}