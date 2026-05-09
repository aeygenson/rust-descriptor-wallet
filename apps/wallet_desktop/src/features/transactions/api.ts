// src/features/transactions/api.ts

import { invokeCommand } from "../../shared/lib/tauri";

import type {
  BumpFeeRequestDto,
  CpfpRequestDto,
  TxBroadcastResultDto,
  WalletCpfpPsbtDto,
  WalletPsbtDto,
  WalletSignedPsbtDto,
  WalletTxDto,
} from "../../shared/types/dtos";

import type { PublishPsbtInput, SignPsbtInput } from "./types";

export async function listTransactions(
  walletName: string,
): Promise<WalletTxDto[]> {
  const transactions = await invokeCommand<WalletTxDto[]>("list_transactions", {
    walletName,
  });

  console.debug(
    "[transactions/api] list_transactions response",
    transactions.map((tx) => ({
      txid: tx.txid,
      confirmed: tx.confirmed,
      replaceable: tx.replaceable,
      outputs: tx.outputs?.length ?? 0,
      walletOwnedOutputs:
        tx.outputs?.filter((output) => output.is_mine).length ?? 0,
    })),
  );

  return transactions;
}

export type BumpFeePsbtInput = {
  walletName: string;
  txid: string;
  feeRateSatPerVb: number;
};

export type CpfpPsbtInput = {
  walletName: string;
  parentTxid: string;
  selectedOutpoint: string;
  feeRateSatPerVb: number;
};

function toBumpFeeRequestDto(
  input: BumpFeePsbtInput,
): BumpFeeRequestDto {
  return {
    name: input.walletName,
    txid: input.txid,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
  };
}

function toCpfpRequestDto(
  input: CpfpPsbtInput,
): CpfpRequestDto {
  return {
    name: input.walletName,
    parent_txid: input.parentTxid,
    selected_outpoint: input.selectedOutpoint,
    fee_rate_sat_per_vb: input.feeRateSatPerVb,
  };
}

export async function bumpFeePsbt(
  input: BumpFeePsbtInput,
): Promise<WalletPsbtDto> {
  const request = toBumpFeeRequestDto(input);

  console.debug("[transactions/api] bump_fee_psbt request", request);

  return invokeCommand<WalletPsbtDto>("bump_fee_psbt", {
    request,
  });
}

export async function cpfpPsbt(
  input: CpfpPsbtInput,
): Promise<WalletCpfpPsbtDto> {
  const request = toCpfpRequestDto(input);

  console.debug("[transactions/api] cpfp_psbt request", request);

  return invokeCommand<WalletCpfpPsbtDto>("cpfp_psbt", {
    request,
  });
}

// Optional direct-send helper (non-PSBT flow)
export async function bumpFee(
  input: BumpFeePsbtInput,
): Promise<{ txid: string }> {
  console.debug("[transactions/api] bump_fee request", input);

  return invokeCommand<{ txid: string }>("bump_fee", {
    request: toBumpFeeRequestDto(input),
  });
}

export async function cpfp(input: CpfpPsbtInput): Promise<TxBroadcastResultDto> {
  console.debug("[transactions/api] cpfp request", input);

  return invokeCommand<TxBroadcastResultDto>("cpfp", {
    request: toCpfpRequestDto(input),
  });
}

export async function signPsbt(
  input: SignPsbtInput,
): Promise<WalletSignedPsbtDto> {
  const request = {
    walletName: input.walletName,
    psbtBase64: input.psbtBase64,
  };

  console.debug("[transactions/api] sign_psbt request", request);

  return invokeCommand<WalletSignedPsbtDto>("sign_psbt", {
    request,
  });
}

export async function publishPsbt(
  input: PublishPsbtInput,
): Promise<TxBroadcastResultDto> {
  const request = {
    walletName: input.walletName,
    psbtBase64: input.psbtBase64,
  };

  console.debug("[transactions/api] publish_psbt request", request);

  return invokeCommand<TxBroadcastResultDto>("publish_psbt", {
    request,
  });
}