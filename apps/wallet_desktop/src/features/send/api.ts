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
  TxBroadcastResultDto,
  WalletCpfpPsbtDto,
  WalletPsbtDto,
  WalletSignedPsbtDto,
} from "../../shared/types/dtos";
import type {
  BumpFeePsbtInput,
  CpfpPsbtInput,
  CreateConsolidationPsbtInput,
  CreatePsbtInput,
  CreatePsbtWithCoinControlInput,
  CreateSendMaxPsbtInput,
  CreateSendMaxPsbtWithCoinControlInput,
  CreateSweepPsbtInput,
  PublishPsbtInput,
  SignPsbtInput,
} from "./types";

function toCoinControlDto(
  input: CreatePsbtWithCoinControlInput["coinControl"],
) {
  return {
    include_outpoints: input.includeOutpoints,
    exclude_outpoints: input.excludeOutpoints,
    confirmed_only: Boolean(input.confirmedOnly),
    selection_mode: input.selectionMode ?? null,
  };
}

export async function createPsbt(
  input: CreatePsbtInput,
): Promise<WalletPsbtDto> {
  const request: CreatePsbtRequestDto = {
    name: input.walletName,
    to_address: input.toAddress,
    amount_sat: input.amountSat,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    coin_control: input.confirmedOnly
      ? {
          include_outpoints: [],
          exclude_outpoints: [],
          confirmed_only: true,
          selection_mode: null,
        }
      : null,
  };

  console.debug("[send/api] create_psbt request", request);

  return invokeCommand<WalletPsbtDto>("create_psbt", {
    request,
  });
}

export async function signPsbt(
  input: SignPsbtInput,
): Promise<WalletSignedPsbtDto> {
  const request: SignPsbtRequestDto = {
    name: input.walletName,
    psbt_base64: input.psbtBase64,
  };

  return invokeCommand<WalletSignedPsbtDto>("sign_psbt", {
    request,
  });
}

export async function publishPsbt(
  input: PublishPsbtInput,
): Promise<TxBroadcastResultDto> {
  const request: PublishPsbtRequestDto = {
    name: input.walletName,
    psbt_base64: input.psbtBase64,
  };

  return invokeCommand<TxBroadcastResultDto>("publish_psbt", {
    request,
  });
}

export async function createPsbtWithCoinControl(
  input: CreatePsbtWithCoinControlInput,
): Promise<WalletPsbtDto> {
  const request: CreatePsbtRequestDto = {
    name: input.walletName,
    to_address: input.toAddress,
    amount_sat: input.amountSat,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    coin_control: {
      ...toCoinControlDto(input.coinControl),
      confirmed_only: Boolean(
        input.coinControl.confirmedOnly ?? input.confirmedOnly,
      ),
    },
  };

  console.debug("[send/api] create_psbt_with_coin_control request", request);

  return invokeCommand<WalletPsbtDto>("create_psbt_with_coin_control", {
    request,
  });
}

export async function createSendMaxPsbt(
  input: CreateSendMaxPsbtInput,
): Promise<WalletPsbtDto> {
  const request: SendMaxRequestDto = {
    name: input.walletName,
    to_address: input.toAddress,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    coin_control: null,
  };

  return invokeCommand<WalletPsbtDto>("create_send_max_psbt", {
    request,
  });
}

export async function createSendMaxPsbtWithCoinControl(
  input: CreateSendMaxPsbtWithCoinControlInput,
): Promise<WalletPsbtDto> {
  const request: SendMaxRequestDto = {
    name: input.walletName,
    to_address: input.toAddress,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    coin_control: toCoinControlDto(input.coinControl),
  };

  return invokeCommand<WalletPsbtDto>(
    "create_send_max_psbt_with_coin_control",
    {
      request,
    },
  );
}

export async function createSweepPsbt(
  input: CreateSweepPsbtInput,
): Promise<WalletPsbtDto> {
  const request: SweepRequestDto = {
    name: input.walletName,
    to_address: input.toAddress,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    coin_control: toCoinControlDto(input.coinControl),
  };

  return invokeCommand<WalletPsbtDto>("create_sweep_psbt", {
    request,
  });
}

export async function createConsolidationPsbt(
  input: CreateConsolidationPsbtInput,
): Promise<WalletPsbtDto> {
  const request: ConsolidationRequestDto = {
    name: input.walletName,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    consolidation: {
      include_outpoints: input.consolidation.includeOutpoints,
      exclude_outpoints: input.consolidation.excludeOutpoints,
      confirmed_only: Boolean(input.consolidation.confirmedOnly),
      selection_mode: input.consolidation.selectionMode ?? null,
      max_input_count: input.consolidation.maxInputCount ?? null,
      min_input_count: input.consolidation.minInputCount ?? null,
      min_utxo_value_sat: input.consolidation.minUtxoValueSat ?? null,
      max_utxo_value_sat: input.consolidation.maxUtxoValueSat ?? null,
      max_fee_pct_of_input_value: input.consolidation.maxFeePctOfInputValue ?? null,
      strategy: input.consolidation.strategy ?? null,
    },
  };

  return invokeCommand<WalletPsbtDto>("create_consolidation_psbt", {
    request,
  });
}

export async function bumpFeePsbt(
  input: BumpFeePsbtInput,
): Promise<WalletPsbtDto> {
  const request: BumpFeeRequestDto = {
    name: input.walletName,
    txid: input.txid,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
  };

  return invokeCommand<WalletPsbtDto>("bump_fee_psbt", {
    request,
  });
}

export async function cpfpPsbt(
  input: CpfpPsbtInput,
): Promise<WalletCpfpPsbtDto> {
  const request: CpfpRequestDto = {
    name: input.walletName,
    parent_txid: input.parentTxid,
    selected_outpoint: input.selectedOutpoint,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
  };

  console.debug("[send/api] cpfp_psbt request", request);

  return invokeCommand<WalletCpfpPsbtDto>("cpfp_psbt", {
    request,
  });
}