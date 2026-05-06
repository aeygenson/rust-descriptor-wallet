import type { WalletUtxoDto } from "../../shared/types/dtos";
import type { UtxoOutpoint, UtxoSelectionSummary } from "./types";

export function getUtxoOutpoint(utxo: WalletUtxoDto): UtxoOutpoint {
  return String(utxo.outpoint);
}

export function getUtxoValueSat(utxo: WalletUtxoDto): number {
  const value = Number(utxo.value ?? 0);
  return Number.isFinite(value) ? value : 0;
}

export function isUtxoConfirmed(utxo: WalletUtxoDto): boolean {
  return Boolean(utxo.confirmed);
}

export function normalizeSelectedOutpoints(
  selectedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[] {
  const seen = new Set<UtxoOutpoint>();

  return selectedOutpoints
    .map((outpoint) => outpoint.trim())
    .filter((outpoint) => {
      if (!outpoint || seen.has(outpoint)) {
        return false;
      }

      seen.add(outpoint);
      return true;
    });
}

export function calculateSelectedUtxoValueSat(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): number {
  const selectedSet = new Set(normalizeSelectedOutpoints(selectedOutpoints));

  return utxos.reduce((total, utxo) => {
    const outpoint = getUtxoOutpoint(utxo);

    if (!selectedSet.has(outpoint)) {
      return total;
    }

    return total + getUtxoValueSat(utxo);
  }, 0);
}

export function filterConfirmedUtxos(utxos: WalletUtxoDto[]): WalletUtxoDto[] {
  return utxos.filter(isUtxoConfirmed);
}

export function buildUtxoSelectionSummary(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): UtxoSelectionSummary {
  const selectedSet = new Set(normalizeSelectedOutpoints(selectedOutpoints));

  const summary: UtxoSelectionSummary = {
    selectedCount: 0,
    selectedValueSat: 0,
    confirmedCount: 0,
    unconfirmedCount: 0,
  };

  for (const utxo of utxos) {
    const outpoint = getUtxoOutpoint(utxo);
    if (!selectedSet.has(outpoint)) continue;

    summary.selectedCount += 1;
    summary.selectedValueSat += getUtxoValueSat(utxo);

    if (isUtxoConfirmed(utxo)) {
      summary.confirmedCount += 1;
    } else {
      summary.unconfirmedCount += 1;
    }
  }

  return summary;
}

export function getValidSelectedOutpoints(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[] {
  const availableSet = new Set(utxos.map(getUtxoOutpoint));
  const seen = new Set<UtxoOutpoint>();

  return normalizeSelectedOutpoints(selectedOutpoints).filter((outpoint) => {
    if (!availableSet.has(outpoint) || seen.has(outpoint)) return false;
    seen.add(outpoint);
    return true;
  });
}

export function toggleSelectedOutpoint(
  selectedOutpoints: UtxoOutpoint[],
  outpoint: UtxoOutpoint
): UtxoOutpoint[] {
  return selectedOutpoints.includes(outpoint)
    ? selectedOutpoints.filter((candidate) => candidate !== outpoint)
    : [...selectedOutpoints, outpoint];
}

export function selectAllVisibleOutpoints(utxos: WalletUtxoDto[]): UtxoOutpoint[] {
  return Array.from(new Set(utxos.map(getUtxoOutpoint)));
}

export function areAllVisibleUtxosSelected(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): boolean {
  if (utxos.length === 0) return false;

  const selectedSet = new Set(selectedOutpoints);
  return utxos.every((utxo) => selectedSet.has(getUtxoOutpoint(utxo)));
}

export function areSomeVisibleUtxosSelected(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): boolean {
  if (utxos.length === 0) return false;

  const selectedSet = new Set(selectedOutpoints);
  return utxos.some((utxo) => selectedSet.has(getUtxoOutpoint(utxo)));
}