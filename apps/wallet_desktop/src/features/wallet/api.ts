// src/features/wallet/api.ts
import { invokeCommand } from "../../shared/lib/tauri";
import type { WalletSummaryDto, WalletStatusDto, WalletBackendHealthDto } from "../../shared/types/dtos";

export async function getAppInfo(): Promise<string> {
    return invokeCommand<string>("get_app_info");
}

export async function listWallets(): Promise<WalletSummaryDto[]> {
    return invokeCommand<WalletSummaryDto[]>("list_wallets");
}

export async function getWalletStatus(walletName: string): Promise<WalletStatusDto> {
    return invokeCommand<WalletStatusDto>("get_wallet_status", { walletName });
}

export async function syncWallet(walletName: string): Promise<WalletStatusDto> {
    return invokeCommand<WalletStatusDto>("sync_wallet", { walletName });
}

export async function getBackendHealth(walletName: string): Promise<WalletBackendHealthDto> {
    return invokeCommand<WalletBackendHealthDto>("backend_health", { walletName });
}