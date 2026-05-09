import { invokeCommand } from "../../shared/lib/tauri";
import type {
    WalletCoinControlDto,
    WalletConsolidationDto,
    WalletReceiveAddressDto,
    WalletUtxoDto,
} from "../../shared/types/dtos";

export async function listUtxos(
    walletName: string,
): Promise<WalletUtxoDto[]> {
    return invokeCommand<WalletUtxoDto[]>("list_utxos", {
        walletName,
    });
}

export async function getReceiveAddress(
    walletName: string,
): Promise<WalletReceiveAddressDto> {
    return invokeCommand<WalletReceiveAddressDto>("get_receive_address", {
        walletName,
    });
}

export function buildCoinControlDto(
    includeOutpoints: string[],
    confirmedOnly = false,
): WalletCoinControlDto {
    return {
        include_outpoints: includeOutpoints,
        exclude_outpoints: [],
        confirmed_only: confirmedOnly,
        selection_mode: includeOutpoints.length > 0 ? "strict-manual" : null,
    };
}

export function buildConsolidationDto(
    includeOutpoints: string[],
): WalletConsolidationDto {
    return {
        include_outpoints: includeOutpoints,
        exclude_outpoints: [],
        confirmed_only: true,
        selection_mode: includeOutpoints.length > 0 ? "strict-manual" : null,
        max_input_count: null,
        min_input_count: null,
        min_utxo_value_sat: null,
        max_utxo_value_sat: null,
        max_fee_pct_of_input_value: null,
        strategy: null,
    };
}