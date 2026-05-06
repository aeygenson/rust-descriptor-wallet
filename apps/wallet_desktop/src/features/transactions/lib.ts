import type {
  WalletTxDto,
  WalletTxOutputDto,
} from "../../shared/types/dtos";

export function clampNumber(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function suggestNextFeeRate(current: number | null): number {
  if (current === null || !Number.isFinite(current) || current <= 0) {
    return 2;
  }

  return Math.max(current + 1, Math.ceil(current * 1.25));
}

export function suggestCpfpFeeRate(current: number | null): number {
  if (current === null || !Number.isFinite(current) || current <= 0) {
    return 2;
  }

  return Math.max(current + 1, Math.ceil(current * 1.5));
}

export function parsePositiveFeeRate(value: string): number | null {
  const parsed = Number(value);

  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }

  return parsed;
}

export function isPendingTransaction(tx: WalletTxDto): boolean {
  return !tx.confirmed;
}

export function isConfirmedTransaction(tx: WalletTxDto): boolean {
  return tx.confirmed;
}

export function isSentTransaction(tx: WalletTxDto): boolean {
  return tx.direction === "sent";
}

export function isReceivedTransaction(tx: WalletTxDto): boolean {
  return tx.direction === "received";
}

export function isReplaceableTransaction(tx: WalletTxDto): boolean {
  return tx.replaceable;
}

export function canTransactionBeRbfBumped(tx: WalletTxDto): boolean {
  return isPendingTransaction(tx) && isReplaceableTransaction(tx);
}

export function hasWalletOwnedOutput(tx: WalletTxDto): boolean {
  return (tx.outputs ?? []).some(isWalletOwnedOutput);
}

export function canTransactionUseCpfp(tx: WalletTxDto): boolean {
  return isPendingTransaction(tx) && hasWalletOwnedOutput(tx);
}

export function isWalletOwnedOutput(output: WalletTxOutputDto): boolean {
  return output.is_mine;
}

export function extractParentTxids(tx: WalletTxDto): string[] {
  return Array.from(
    new Set(
      (tx.inputs ?? [])
        .map((input) => input.previous_outpoint.split(":")[0])
        .filter(Boolean)
    )
  );
}

export function getCpfpOutpoints(tx: WalletTxDto): string[] {
  return (tx.outputs ?? [])
    .filter(isWalletOwnedOutput)
    .map((output) => output.outpoint)
    .filter(Boolean);
}

export function getTransactionDirection(
  tx: WalletTxDto
): string {
  return tx.direction;
}