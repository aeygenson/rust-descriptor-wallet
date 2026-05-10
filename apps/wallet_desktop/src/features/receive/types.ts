import type { WalletReceiveAddressHistoryDto } from "../../shared/types/dtos";

export type ReceiveAddressState = {
  loading: boolean;
  error: string | null;
  data: WalletReceiveAddressHistoryDto | null;
};

export type ReceiveAddressCardProps = {
  walletName: string;
  address: WalletReceiveAddressHistoryDto;
  loading?: boolean;
  onRefresh?: () => void;
  onCopy?: (address: string) => void;
};


export type ReceiveAddressActionsProps = {
  address: string;
  bitcoinUri: string;
  loading?: boolean;
  onRefresh?: () => void;
  onCopyAddress?: (address: string) => void;
  onCopyBitcoinUri?: (bitcoinUri: string) => void;
};

export type ReceiveAddressMetadataProps = {
  items: ReceiveAddressMetadataItem[];
};

export type ReceiveAddressHistoryListProps = {
  walletName: string;
  addresses: WalletReceiveAddressHistoryDto[];
  loading?: boolean;
  onSelect?: (address: WalletReceiveAddressHistoryDto) => void;
};

export type ReceiveAddressHistoryItemProps = {
  walletName: string;
  address: WalletReceiveAddressHistoryDto;
  selected?: boolean;
  onSelect?: (address: WalletReceiveAddressHistoryDto) => void;
};

export type ReceiveAddressLabelEditorProps = {
  address: WalletReceiveAddressHistoryDto;
  loading?: boolean;
  onSave?: (label: string) => void;
  onClear?: () => void;
};

export type ReceiveQrCodeProps = {
  value: string;
  size?: number;
};

export type ReceiveEmptyStateProps = {
  walletName: string;
  loading?: boolean;
  error?: string | null;
  onGenerate?: () => void;
};

export type ReceiveAddressMetadataItem = {
  label: string;
  value: string | number | null;
};
