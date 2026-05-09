

import type { WalletReceiveAddressDto } from "../../shared/types/dtos";
import { invokeCommand } from "../../shared/lib/tauri";

export async function getReceiveAddress(
  walletName: string,
): Promise<WalletReceiveAddressDto> {
  const normalizedWalletName = walletName.trim();

  if (!normalizedWalletName) {
    throw new Error("Wallet name is required");
  }

  return invokeCommand<WalletReceiveAddressDto>("get_receive_address", {
    walletName: normalizedWalletName,
  });
}