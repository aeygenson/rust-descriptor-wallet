

import type { WalletUtxoDto } from "../../shared/types/dtos";
import type {
  CoinControlSelection,
  CoinControlUtxoOption,
  ConsolidationSelection,
} from "./types";

export function sanitizeIntegerInput(value: string): string {
  return value.replace(/[^0-9]/g, "");
}

export function sanitizeDecimalInput(value: string): string {
  const normalized = value.replace(/[^0-9.]/g, "");
  const firstDotIndex = normalized.indexOf(".");

  if (firstDotIndex === -1) {
    return normalized;
  }

  return `${normalized.slice(0, firstDotIndex + 1)}${normalized
    .slice(firstDotIndex + 1)
    .replace(/\./g, "")}`;
}

export function normalizeSelectedOutpoints(outpoints: string[]): string[] {
  const seen = new Set<string>();

  return outpoints
    .map((outpoint) => outpoint.trim())
    .filter((outpoint) => {
      if (!outpoint || seen.has(outpoint)) {
        return false;
      }

      seen.add(outpoint);
      return true;
    });
}

export function getValidSelectedOutpoints(
  selectedOutpoints: string[],
  availableUtxos: CoinControlUtxoOption[]
): string[] {
  const availableSet = new Set(availableUtxos.map((utxo) => utxo.outpoint));

  return normalizeSelectedOutpoints(selectedOutpoints).filter((outpoint) =>
    availableSet.has(outpoint)
  );
}

export function sumSelectedInputValue(
  utxos: CoinControlUtxoOption[],
  selectedOutpoints: string[]
): number {
  const selectedSet = new Set(normalizeSelectedOutpoints(selectedOutpoints));

  return utxos.reduce((sum, utxo) => {
    if (!selectedSet.has(utxo.outpoint)) {
      return sum;
    }

    const value = Number(utxo.valueSat ?? 0);
    return Number.isFinite(value) ? sum + value : sum;
  }, 0);
}

export function buildSelectedCoinControl(
  selectedOutpoints: string[],
  availableUtxos: CoinControlUtxoOption[]
): CoinControlSelection {
  return {
    includeOutpoints: getValidSelectedOutpoints(selectedOutpoints, availableUtxos),
    excludeOutpoints: [],
    confirmedOnly: true,
    selectionMode: "strict-manual",
  };
}

export function buildSelectedConsolidation(
  selectedOutpoints: string[],
  availableUtxos: CoinControlUtxoOption[]
): ConsolidationSelection {
  return {
    ...buildSelectedCoinControl(selectedOutpoints, availableUtxos),
    strategy: "smallest-first",
  };
}

export function mapUtxosForCoinControl(utxos: WalletUtxoDto[]): CoinControlUtxoOption[] {
  return utxos.map(toCoinControlUtxoOption);
}

export function toCoinControlUtxoOption(utxo: WalletUtxoDto): CoinControlUtxoOption {
  return {
    outpoint: String(utxo.outpoint),
    valueSat: parseNumberOrZero(utxo.value),
    label: String(utxo.outpoint),
    address: utxo.address ?? null,
    confirmations: parseNullableNumber(utxo.confirmation_height),
    confirmed: parseNullableBoolean(utxo.confirmed),
  };
}


export function findOutpointsForTxid(
  utxos: CoinControlUtxoOption[],
  txid: string | null | undefined
): string[] {
  if (!txid) return [];

  return utxos
    .map((utxo) => utxo.outpoint)
    .filter((outpoint) => outpoint.startsWith(`${txid}:`));
}

export function pickDtoNumber(
  source: Record<string, unknown>,
  snakeCaseKey: string,
  camelCaseKey: string
): number | null {
  const value = source[snakeCaseKey] ?? source[camelCaseKey];

  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }

  if (typeof value === "string") {
    const parsed = Number(value.trim());
    return Number.isFinite(parsed) ? parsed : null;
  }

  return null;
}

export function pickDtoString(
  source: Record<string, unknown>,
  snakeCaseKey: string,
  camelCaseKey: string
): string | null {
  const value = source[snakeCaseKey] ?? source[camelCaseKey];
  return typeof value === "string" ? value : null;
}

export function readReplaceableFromForm(form: { replaceable?: unknown }): boolean {
  return form.replaceable === undefined ? true : Boolean(form.replaceable);
}

export function parseNumberOrZero(value: unknown): number {
  const parsed = Number(value ?? 0);
  return Number.isFinite(parsed) ? parsed : 0;
}

export function parseNullableNumber(value: unknown): number | null {
  if (value === null || value === undefined) return null;

  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

export function parseNullableBoolean(value: unknown): boolean | null {
  if (value === null || value === undefined) return null;

  return Boolean(value);
}