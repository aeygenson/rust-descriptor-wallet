import type { WalletReceiveAddressDto } from "../../shared/types/dtos";

export type ReceiveAddressState = {
  loading: boolean;
  error: string | null;
  data: WalletReceiveAddressDto | null;
};

export type ReceiveAddressCardProps = {
  walletName: string;
  address: WalletReceiveAddressDto;
  loading?: boolean;
  onRefresh?: () => void;
  onCopy?: (address: string) => void;
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
