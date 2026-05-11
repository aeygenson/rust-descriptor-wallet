

import { invoke } from "@tauri-apps/api/core";

import type {
  AddressBookEntryDto,
  CreateAddressBookEntryRequestDto,
  DeleteAddressBookEntryRequestDto,
  GetAddressBookEntryRequestDto,
  ListAddressBookEntriesRequestDto,
} from "../../shared/types/dtos";

export async function createAddressBookEntry(
  request: CreateAddressBookEntryRequestDto,
): Promise<AddressBookEntryDto> {
  return invoke<AddressBookEntryDto>("create_address_book_entry", {
    walletName: request.name,
    label: request.label,
    address: request.address,
    notes: request.notes,
  });
}

export async function listAddressBookEntries(
  request: ListAddressBookEntriesRequestDto,
): Promise<AddressBookEntryDto[]> {
  return invoke<AddressBookEntryDto[]>("list_address_book_entries", {
    walletName: request.name,
  });
}

export async function getAddressBookEntry(
  request: GetAddressBookEntryRequestDto,
): Promise<AddressBookEntryDto | null> {
  return invoke<AddressBookEntryDto | null>("get_address_book_entry", {
    walletName: request.name,
    address: request.address,
  });
}

export async function deleteAddressBookEntry(
  request: DeleteAddressBookEntryRequestDto,
): Promise<boolean> {
  return invoke<boolean>("delete_address_book_entry", {
    walletName: request.name,
    address: request.address,
  });
}