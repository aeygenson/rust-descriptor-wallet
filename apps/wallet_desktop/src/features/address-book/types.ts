

import type { AddressBookEntryDto } from "../../shared/types/dtos";

export type AddressBookFormValues = {
  label: string;
  address: string;
  notes: string;
};

export type AddressBookFormProps = {
  walletName: string;
  loading?: boolean;
  onSubmit: (values: AddressBookFormValues) => Promise<void>;
};

export type AddressBookItemProps = {
  entry: AddressBookEntryDto;
  loading?: boolean;
  onCopyAddress?: (address: string) => void;
  onDelete?: (entry: AddressBookEntryDto) => Promise<void>;
};

export type AddressBookListProps = {
  entries: AddressBookEntryDto[];
  loading?: boolean;
  emptyMessage?: string;
  onCopyAddress?: (address: string) => void;
  onDelete?: (entry: AddressBookEntryDto) => Promise<void>;
};

export type AddressBookPageState = {
  entries: AddressBookEntryDto[];
  loading: boolean;
  error: string | null;
};