import type {
  TxBroadcastResultDto,
  WalletPsbtDto,
  WalletSignedPsbtDto,
  WalletTxDto,
  WalletCpfpPsbtDto,
} from "../../shared/types/dtos";

// Derive direction directly from DTO to avoid drift from Rust source of truth
export type WalletTxDirection = WalletTxDto["direction"];

export type TransactionFilter =
  | "all"
  | "pending"
  | "confirmed"
  | "rbf"
  | "cpfp"
  | "sent"
  | "received";

export type TransactionIntent =
  | "fixed"
  | "send_max"
  | "sweep"
  | "consolidation"
  | "rbf"
  | "cpfp"
  | "unknown";

export type TransactionIntentBadgeProps = {
  intent: TransactionIntent;
};

export type TransactionsFilterBarProps = {
  transactions: WalletTxDto[];
  activeFilter: TransactionFilter;
  onFilterChange: (filter: TransactionFilter) => void;
};

export type TransactionRelationCellProps = {
  txids: string[];
  kind: "parents" | "children";
  onOpenTx?: (txid: string) => void;
};

export type TransactionDetailsModalProps = {
  tx: WalletTxDto;
  intent?: TransactionIntent;
  onClose: () => void;
  onOpenTx?: (txid: string) => void;
};

export type TransactionActionsMenuProps = {
  tx: WalletTxDto;
  onDetails: (tx: WalletTxDto) => void;
  onCopyTxid: (txid: string) => void;
  onBumpFee: (tx: WalletTxDto) => void;
  onCpfp: (tx: WalletTxDto) => void;
};

export type BumpFeePsbtInput = {
  walletName: string;
  txid: string;
  feeRateSatPerVb: number;
};

export type BumpFeePanelProps = {
  tx: WalletTxDto;
  walletName: string;
  loading?: boolean;
  onCancel: () => void;
  onCreatePsbt: (input: BumpFeePsbtInput) => Promise<void> | void;
};

export type RbfPsbtWorkflowProps = {
  walletName: string;
  psbtBase64: string;
};

export type SignPsbtInput = {
  walletName: string;
  psbtBase64: string;
};

export type PublishPsbtInput = {
  walletName: string;
  psbtBase64: string;
};

export type RbfPsbtWorkflowPanelProps = {
  psbt: WalletPsbtDto;
  signedPsbt: WalletSignedPsbtDto | null;
  broadcastResult: TxBroadcastResultDto | null;
  loading?: boolean;
  onSign: () => Promise<void> | void;
  onBroadcast: () => Promise<void> | void;
  onClose: () => void;
};

export type CpfpPsbtInput = {
  walletName: string;
  parentTxid: string;
  selectedOutpoint: string;
  feeRateSatVb: number;
};

export type CpfpPanelInput = {
  selectedOutpoint: string;
  feeRateSatVb: string;
};

export type CpfpPanelProps = {
  tx: WalletTxDto;
  walletName: string;
  loading?: boolean;
  availableOutpoints: string[];
  onCancel: () => void;
  onCreatePsbt: (input: CpfpPsbtInput) => Promise<void> | void;
};

export type CpfpPsbtWorkflowPanelProps = {
  psbt: WalletCpfpPsbtDto;
  signedPsbt: WalletSignedPsbtDto | null;
  broadcastResult: TxBroadcastResultDto | null;
  loading?: boolean;
  onSign: () => Promise<void> | void;
  onBroadcast: () => Promise<void> | void;
  onClose: () => void;
};