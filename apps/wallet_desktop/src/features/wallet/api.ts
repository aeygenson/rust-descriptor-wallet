// src/features/wallet/api.ts
import { invokeCommand } from "../../shared/lib/tauri";
import type {
    BumpFeeRequestDto,
    ConsolidationRequestDto,
    CpfpRequestDto,
    CreatePsbtRequestDto,
    PublishPsbtRequestDto,
    SendMaxRequestDto,
    SignPsbtRequestDto,
    SweepRequestDto,
    WalletBackendHealthDto,
    WalletStatusDto,
    WalletSummaryDto,
} from "../../shared/types/dtos";

export async function getAppInfo(): Promise<string> {
    return invokeCommand<string>("get_app_info");
}

export async function listWallets(): Promise<WalletSummaryDto[]> {
    return invokeCommand<WalletSummaryDto[]>("list_wallets");
}

export async function getWalletStatus(walletName: string): Promise<WalletStatusDto> {
    return invokeCommand<WalletStatusDto>("get_wallet_status", {
        walletName,
    });
}

export async function syncWallet(walletName: string): Promise<WalletStatusDto> {
    return invokeCommand<WalletStatusDto>("sync_wallet", {
        walletName,
    });
}

export async function getBackendHealth(walletName: string): Promise<WalletBackendHealthDto> {
    return invokeCommand<WalletBackendHealthDto>("backend_health", {
        walletName,
    });
}

export async function createPsbt(
    request: CreatePsbtRequestDto,
) {
    return invokeCommand("create_psbt", {
        request,
    });
}

export async function createSendMaxPsbt(
    request: SendMaxRequestDto,
) {
    return invokeCommand("create_send_max_psbt", {
        request,
    });
}

export async function createSweepPsbt(
    request: SweepRequestDto,
) {
    return invokeCommand("create_sweep_psbt", {
        request,
    });
}

export async function createConsolidationPsbt(
    request: ConsolidationRequestDto,
) {
    return invokeCommand("create_consolidation_psbt", {
        request,
    });
}

export async function signPsbt(
    request: SignPsbtRequestDto,
) {
    return invokeCommand("sign_psbt", {
        request,
    });
}

export async function publishPsbt(
    request: PublishPsbtRequestDto,
) {
    return invokeCommand("publish_psbt", {
        request,
    });
}

export async function bumpFeePsbt(
    request: BumpFeeRequestDto,
) {
    return invokeCommand("bump_fee_psbt", {
        request,
    });
}

export async function cpfpPsbt(
    request: CpfpRequestDto,
) {
    return invokeCommand("cpfp_psbt", {
        request,
    });
}