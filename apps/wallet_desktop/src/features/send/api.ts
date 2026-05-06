import { invokeCommand } from "../../shared/lib/tauri";
import type {
  TxBroadcastResultDto,
  WalletCpfpPsbtDto,
  WalletPsbtDto,
  WalletSignedPsbtDto,
} from "../../shared/types/dtos";
import type {
  CreatePsbtInput,
  SignPsbtInput,
  PublishPsbtInput,
  CreatePsbtWithCoinControlInput,
  CreateSendMaxPsbtInput,
  CreateSendMaxPsbtWithCoinControlInput,
  CreateSweepPsbtInput,
  CreateConsolidationPsbtInput,
  BumpFeePsbtInput,
  CpfpPsbtInput,
} from "./types";

export async function createPsbt(input: CreatePsbtInput): Promise<WalletPsbtDto> {
  const request = {
    walletName: input.walletName,
    address: input.toAddress,
    amountSat: input.amountSat,
    feeRateSatVb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    confirmedOnly: Boolean(input.confirmedOnly),
  };

  console.debug("[send/api] create_psbt request", request);

  return invokeCommand<WalletPsbtDto>("create_psbt", {
    request,
  });
}

export async function signPsbt(input: SignPsbtInput): Promise<WalletSignedPsbtDto> {
  return invokeCommand<WalletSignedPsbtDto>("sign_psbt", {
    request: {
      walletName: input.walletName,
      psbtBase64: input.psbtBase64,
    },
  });
}

export async function publishPsbt(input: PublishPsbtInput): Promise<TxBroadcastResultDto> {
  return invokeCommand<TxBroadcastResultDto>("publish_psbt", {
    request: {
      walletName: input.walletName,
      psbtBase64: input.psbtBase64,
    },
  });
}

export async function createPsbtWithCoinControl(
  input: CreatePsbtWithCoinControlInput,
): Promise<WalletPsbtDto> {
  const request = {
    walletName: input.walletName,
    address: input.toAddress,
    amountSat: input.amountSat,
    feeRateSatVb: input.feeRateSatPerVb,
    replaceable: input.replaceable,
    confirmedOnly: Boolean(input.confirmedOnly),
    coinControl: {
      includeOutpoints: input.coinControl.includeOutpoints,
      excludeOutpoints: input.coinControl.excludeOutpoints,
      confirmedOnly: Boolean(input.coinControl.confirmedOnly ?? input.confirmedOnly),
      selectionMode: input.coinControl.selectionMode ?? null,
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
  return invokeCommand<WalletPsbtDto>("create_send_max_psbt", {
    request: {
      walletName: input.walletName,
      address: input.toAddress,
      feeRateSatVb: input.feeRateSatPerVb,
      replaceable: input.replaceable,
    },
  });
}

export async function createSendMaxPsbtWithCoinControl(
  input: CreateSendMaxPsbtWithCoinControlInput,
): Promise<WalletPsbtDto> {
  return invokeCommand<WalletPsbtDto>("create_send_max_psbt_with_coin_control", {
    request: {
      walletName: input.walletName,
      address: input.toAddress,
      feeRateSatVb: input.feeRateSatPerVb,
      replaceable: input.replaceable,
      coinControl: {
        includeOutpoints: input.coinControl.includeOutpoints,
        excludeOutpoints: input.coinControl.excludeOutpoints,
        confirmedOnly: Boolean(input.coinControl.confirmedOnly),
        selectionMode: input.coinControl.selectionMode ?? null,
      },
    },
  });
}

export async function createSweepPsbt(
  input: CreateSweepPsbtInput,
): Promise<WalletPsbtDto> {
  return invokeCommand<WalletPsbtDto>("create_sweep_psbt", {
    request: {
      walletName: input.walletName,
      address: input.toAddress,
      feeRateSatVb: input.feeRateSatPerVb,
      replaceable: input.replaceable,
      coinControl: {
        includeOutpoints: input.coinControl.includeOutpoints,
        excludeOutpoints: input.coinControl.excludeOutpoints,
        confirmedOnly: Boolean(input.coinControl.confirmedOnly),
        selectionMode: input.coinControl.selectionMode ?? null,
      },
    },
  });
}

export async function createConsolidationPsbt(
  input: CreateConsolidationPsbtInput,
): Promise<WalletPsbtDto> {
  return invokeCommand<WalletPsbtDto>("create_consolidation_psbt", {
    request: {
      walletName: input.walletName,
      feeRateSatVb: input.feeRateSatPerVb,
      replaceable: input.replaceable,
      consolidation: {
        includeOutpoints: input.consolidation.includeOutpoints,
        excludeOutpoints: input.consolidation.excludeOutpoints,
        confirmedOnly: Boolean(input.consolidation.confirmedOnly),
        selectionMode: input.consolidation.selectionMode ?? null,
        maxInputCount: input.consolidation.maxInputCount ?? null,
        minInputCount: input.consolidation.minInputCount ?? null,
        minUtxoValueSat: input.consolidation.minUtxoValueSat ?? null,
        maxUtxoValueSat: input.consolidation.maxUtxoValueSat ?? null,
        maxFeePctOfInputValue: input.consolidation.maxFeePctOfInputValue ?? null,
        strategy: input.consolidation.strategy ?? null,
      },
    },
  });
}

export async function bumpFeePsbt(
  input: BumpFeePsbtInput,
): Promise<WalletPsbtDto> {
  return invokeCommand<WalletPsbtDto>("bump_fee_psbt", {
    request: input,
  });
}

export async function cpfpPsbt(
  input: CpfpPsbtInput,
): Promise<WalletCpfpPsbtDto> {
  console.debug("[send/api] cpfp_psbt request", input);

  return invokeCommand<WalletCpfpPsbtDto>("cpfp_psbt", {
    request: input,
  });
}