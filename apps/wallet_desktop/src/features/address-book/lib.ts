

import type { AddressBookEntryDto } from "../../shared/types/dtos";
import type { AddressBookFormValues } from "./types";

export function normalizeAddressBookLabel(label: string): string {
  return label.trim();
}

export function normalizeAddressBookAddress(address: string): string {
  return address.trim();
}

export function normalizeAddressBookNotes(notes: string): string | null {
  const normalized = notes.trim();

  return normalized.length > 0 ? normalized : null;
}

export function isValidAddressBookForm(values: AddressBookFormValues): boolean {
  return (
    normalizeAddressBookLabel(values.label).length > 0 &&
    normalizeAddressBookAddress(values.address).length > 0
  );
}

export function sortAddressBookEntries(
  entries: AddressBookEntryDto[],
): AddressBookEntryDto[] {
  return [...entries].sort((left, right) =>
    left.label.localeCompare(right.label, undefined, {
      sensitivity: "base",
    }),
  );
}

export function findAddressBookEntryByAddress(
  entries: AddressBookEntryDto[],
  address: string,
): AddressBookEntryDto | null {
  const normalizedAddress = normalizeAddressBookAddress(address);

  return (
    entries.find((entry) => entry.address === normalizedAddress) ?? null
  );
}