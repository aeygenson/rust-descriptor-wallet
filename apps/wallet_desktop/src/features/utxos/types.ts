import type { WalletUtxoDto } from "../../shared/types/dtos";

export type UtxoOutpoint = string;

export type UtxoSelectionAction = "fixed" | "send_max" | "sweep" | "consolidate";

export interface UtxoSelectionState {
  selectedOutpoints: UtxoOutpoint[];
}

export interface UtxoSelectionSummary {
  selectedCount: number;
  selectedValueSat: number;
  confirmedCount: number;
  unconfirmedCount: number;
}

export interface UtxosSummary {
  totalCount: number;
  totalValue: number;
  averageValue: number;
  confirmedCount: number;
  confirmedValue: number;
  pendingCount: number;
  pendingValue: number;
  keychains: string;
}

export interface UtxosSummaryCardsProps {
  summary: UtxosSummary;
}

export interface UtxosHeaderProps {
  walletName?: string | null;
  summary: UtxosSummary;
}

export interface UtxoSelectionActionBarProps {
  selectedCount: number;
  selectedValueSat: number;
  disabled?: boolean;
  onSendFixedSelected: () => void;
  onSendMaxSelected: () => void;
  onSweepSelected: () => void;
  onConsolidateSelected: () => void;
  onClearSelection: () => void;
}

export interface UtxoSelectionSummaryProps {
  selectedCount: number;
  selectedValueSat: number;
  confirmedCount: number;
  unconfirmedCount: number;
  onClearSelection?: () => void;
}

export interface UtxoTableSelectionProps {
  selectedOutpoints: UtxoOutpoint[];
  onToggleOutpoint: (outpoint: UtxoOutpoint) => void;
  onSelectAllVisible: () => void;
  onClearSelection: () => void;
}

export interface UtxosTableProps {
  utxos: WalletUtxoDto[];
  selectedOutpoints?: UtxoOutpoint[];
  onToggleOutpoint?: (outpoint: UtxoOutpoint) => void;
  onSelectAllVisible?: () => void;
  onClearSelection?: () => void;
}

export interface UtxosStateViewProps {
  loading: boolean;
  error: string | null;
  hasData: boolean;
  emptyMessage?: string;
}

export interface UtxosPageNavigationActionState {
  mode: "fixed" | "send_max" | "sweep" | "consolidate";
  selectedOutpoints: UtxoOutpoint[];
}

export type UtxoSortKey = "outpoint" | "value" | "status" | "height" | "keychain";

export type UtxoSortDirection = "asc" | "desc";

export interface UtxoSortState {
  key: UtxoSortKey;
  direction: UtxoSortDirection;
}