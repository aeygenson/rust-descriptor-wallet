import type { WalletUtxoDto } from "../../shared/types/dtos";
import type {
  UtxoFilterState,
  UtxoOutpoint,
  UtxoSelectionPreview,
  UtxoSelectionSummary,
} from "./types";

export function getUtxoOutpoint(utxo: WalletUtxoDto): UtxoOutpoint {
  return String(utxo.outpoint);
}

export function getUtxoValueSat(utxo: WalletUtxoDto): number {
  const value = Number(utxo.value_sat ?? 0);
  return Number.isFinite(value) ? value : 0;
}

export function isUtxoConfirmed(utxo: WalletUtxoDto): boolean {
  return Boolean(utxo.confirmed);
}

export function isUtxoLocked(utxo: WalletUtxoDto): boolean {
  return Boolean(utxo.is_locked);
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

export function filterLockedUtxos(utxos: WalletUtxoDto[]): WalletUtxoDto[] {
  return utxos.filter(isUtxoLocked);
}

export function filterSpendableUtxos(utxos: WalletUtxoDto[]): WalletUtxoDto[] {
  return utxos.filter((utxo) => !isUtxoLocked(utxo));
}

export function filterUtxos(
  utxos: WalletUtxoDto[],
  filter: UtxoFilterState
): WalletUtxoDto[] {
  const search = filter.search?.trim().toLowerCase() ?? "";

  return utxos.filter((utxo) => {
    const valueSat = getUtxoValueSat(utxo);
    const confirmed = isUtxoConfirmed(utxo);
    const outpoint = getUtxoOutpoint(utxo).toLowerCase();

    if (filter.status === "confirmed" && !confirmed) {
      return false;
    }

    if (filter.status === "pending" && confirmed) {
      return false;
    }

    if (filter.status === "locked" && !isUtxoLocked(utxo)) {
      return false;
    }

    if (filter.status === "spendable" && isUtxoLocked(utxo)) {
      return false;
    }

    if (
      filter.minValueSat !== null &&
      filter.minValueSat !== undefined &&
      valueSat < filter.minValueSat
    ) {
      return false;
    }

    if (
      filter.maxValueSat !== null &&
      filter.maxValueSat !== undefined &&
      valueSat > filter.maxValueSat
    ) {
      return false;
    }

    if (search && !outpoint.includes(search)) {
      return false;
    }

    return true;
  });
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
    lockedCount: 0,
    spendableCount: 0,
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

    if (isUtxoLocked(utxo)) {
      summary.lockedCount += 1;
    } else {
      summary.spendableCount += 1;
    }
  }

  return summary;
}

export function buildUtxoSelectionPreview(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): UtxoSelectionPreview {
  const selectedOutpointsNormalized = getValidSelectedOutpoints(utxos, selectedOutpoints);
  const selectedSummary = buildUtxoSelectionSummary(utxos, selectedOutpointsNormalized);

  return {
    selectedOutpoints: selectedOutpointsNormalized,
    selectedValueSat: selectedSummary.selectedValueSat,
    selectedCount: selectedSummary.selectedCount,
    confirmedOnly: selectedSummary.unconfirmedCount === 0,
  };
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

export function getConfirmedSelectedOutpoints(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[] {
  const confirmedSet = new Set(filterConfirmedUtxos(utxos).map(getUtxoOutpoint));

  return getValidSelectedOutpoints(utxos, selectedOutpoints).filter((outpoint) =>
    confirmedSet.has(outpoint)
  );
}

export function getLockedSelectedOutpoints(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[] {
  const lockedSet = new Set(filterLockedUtxos(utxos).map(getUtxoOutpoint));

  return getValidSelectedOutpoints(utxos, selectedOutpoints).filter((outpoint) =>
    lockedSet.has(outpoint)
  );
}

export function getSpendableSelectedOutpoints(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): UtxoOutpoint[] {
  const lockedSet = new Set(filterLockedUtxos(utxos).map(getUtxoOutpoint));

  return getValidSelectedOutpoints(utxos, selectedOutpoints).filter(
    (outpoint) => !lockedSet.has(outpoint)
  );
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
  return Array.from(new Set(filterSpendableUtxos(utxos).map(getUtxoOutpoint)));
}

export function areAllVisibleUtxosSelected(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): boolean {
  const spendableUtxos = filterSpendableUtxos(utxos);
  if (spendableUtxos.length === 0) return false;

  const selectedSet = new Set(normalizeSelectedOutpoints(selectedOutpoints));
  return spendableUtxos.every((utxo) => selectedSet.has(getUtxoOutpoint(utxo)));
}

export function areSomeVisibleUtxosSelected(
  utxos: WalletUtxoDto[],
  selectedOutpoints: UtxoOutpoint[]
): boolean {
  if (utxos.length === 0) return false;

  const selectedSet = new Set(normalizeSelectedOutpoints(selectedOutpoints));
  return utxos.some((utxo) => selectedSet.has(getUtxoOutpoint(utxo)));
}