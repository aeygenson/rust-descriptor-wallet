// === Wallet Summary ===
export interface WalletSummaryDto {
    name: string;
    network: string;
    is_watch_only: boolean;
}

// === Wallet Details ===
export interface WalletDescriptorsDto {
    external: string;
    internal: string;
}

export type SyncBackendDto =
    | { kind: "esplora"; url: string }
    | { kind: "electrum"; url: string };

export type BroadcastBackendDto =
    | { kind: "esplora"; url: string }
    | { kind: "rpc"; url: string; rpc_user: string; rpc_pass: string };

export interface WalletBackendDto {
    sync: SyncBackendDto;
    broadcast: BroadcastBackendDto | null;
}

export interface WalletDetailsDto {
    name: string;
    network: string;
    descriptors: WalletDescriptorsDto;
    backend: WalletBackendDto;
    is_watch_only: boolean;
}

// === Wallet Status ===
export interface WalletStatusDto {
    balance: number;
    utxo_count: number;
    last_block_height: number | null;
}

// === Input Selection Mode ===
export type WalletInputSelectionModeDto =
    | "strict-manual"
    | "manual-with-auto-completion"
    | "automatic-only";

// === Backend Health ===
export interface WalletBackendHealthDto {
    sync_backend_reachable: boolean;
    bitcoin_tip_reachable: boolean;
    broadcast_backend_reachable: boolean;
    tip_height: number | null;
    message: string | null;
}

// === Transactions ===
export interface WalletTxInputDto {
    previous_outpoint: string;
}

export interface WalletTxOutputDto {
    outpoint: string;
    value_sat: number;
    address: string | null;
    is_mine: boolean;
    keychain: string | null;
}

export interface WalletTxDto {
    txid: string;
    confirmed: boolean;
    confirmation_height: number | null;
    direction: string;
    replaceable: boolean;
    net_value: number;
    fee: number | null;
    fee_rate_sat_per_vb: number | null;
    inputs: WalletTxInputDto[];
    outputs: WalletTxOutputDto[];
    // derived graph helpers (optional, may be empty)
    parent_txids?: string[];
    child_txids?: string[];
}

// === UTXOs ===
export interface WalletUtxoDto {
    outpoint: string;
    value: number;
    confirmed: boolean;
    confirmation_height: number | null;
    address: string | null;
    keychain: string;
}

// === Coin Control ===
export interface WalletCoinControlDto {
    include_outpoints: string[];
    exclude_outpoints: string[];
    confirmed_only: boolean;
    selection_mode: WalletInputSelectionModeDto | null;
}

// === Consolidation ===
export type WalletConsolidationStrategyDto =
    | "smallest-first"
    | "largest-first"
    | "oldest-first";

export interface WalletConsolidationDto {
    include_outpoints: string[];
    exclude_outpoints: string[];
    confirmed_only: boolean;
    max_input_count: number | null;
    min_input_count: number | null;
    min_utxo_value_sat: number | null;
    max_utxo_value_sat: number | null;
    max_fee_pct_of_input_value: number | null;
    strategy: WalletConsolidationStrategyDto | null;
    selection_mode: WalletInputSelectionModeDto | null;
}

// === PSBT ===
export interface WalletPsbtDto {
    psbt_base64: string;
    txid: string;
    original_txid: string | null;
    to_address: string;
    amount_sat: number;
    fee_sat: number;
    fee_rate_sat_per_vb: number;
    replaceable: boolean;
    change_amount_sat: number | null;
    selected_utxo_count: number;
    selected_inputs: string[];
    input_count: number;
    output_count: number;
    recipient_count: number;
    estimated_vsize: number;
}

// === CPFP PSBT ===
export interface WalletCpfpPsbtDto {
    psbt_base64: string;
    txid: string;
    parent_txid: string;
    selected_outpoint: string;
    input_value_sat: number;
    child_output_value_sat: number;
    fee_sat: number;
    fee_rate_sat_per_vb: number;
    replaceable: boolean;
    estimated_vsize: number;
}

// === Signed PSBT ===
export interface WalletSignedPsbtDto {
    psbt_base64: string;
    modified: boolean;
    finalized: boolean;
    txid: string;
    signing_status: string;
}

// === Broadcast Result ===
export interface TxBroadcastResultDto {
    txid: string;
    replaceable: boolean | null;
}
