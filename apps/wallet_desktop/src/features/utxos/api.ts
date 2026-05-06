import { invokeCommand } from "../../shared/lib/tauri";
import type { WalletUtxoDto } from "../../shared/types/dtos";

export async function listUtxos(walletName: string): Promise<WalletUtxoDto[]> {
    return invokeCommand<WalletUtxoDto[]>("list_utxos", { walletName });
}