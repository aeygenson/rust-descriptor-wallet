import type { WalletPsbtDto } from "../../shared/types/dtos";

export type SendMode = "fixed" | "send_max" | "sweep" | "consolidate";

export const SEND_MODE_LABELS: Record<SendMode, string> = {
  fixed: "Fixed amount",
  send_max: "Send max",
  sweep: "Sweep UTXOs",
  consolidate: "Consolidate UTXOs",
};

export interface SendModeSelectorProps {
  mode: SendMode;
  disabled?: boolean;
  onModeChange: (mode: SendMode) => void;
}

export interface SendPageNavigationState {
  mode?: SendMode;
  selectedOutpoints?: string[];
}

// Raw form state (strings because of inputs)
export type FixedSendFormState = {
  toAddress: string;
  amountSat: string;
  feeRateSatPerVb: string;
  replaceable: boolean;
  confirmedOnly: boolean;
};

export interface FixedSendFormProps {
  disabled?: boolean;
  onSubmit: (form: FixedSendFormState) => void;
}

export type SendMaxFormState = {
  toAddress: string;
  feeRateSatPerVb: string;
  replaceable: boolean;
};

export interface SendMaxFormProps {
  disabled?: boolean;
  onSubmit: (form: SendMaxFormState) => void;
}

export type SweepFormState = {
  toAddress: string;
  feeRateSatPerVb: string;
  replaceable: boolean;
};

export interface SweepFormProps {
  disabled?: boolean;
  selectedUtxoCount?: number;
  onSubmit: (form: SweepFormState) => void;
}

export type ConsolidationFormState = {
  feeRateSatPerVb: string;
  replaceable: boolean;
};

export interface ConsolidationFormProps {
  disabled?: boolean;
  selectedUtxoCount?: number;
  onSubmit: (form: ConsolidationFormState) => void;
}

// Validated input passed to backend
export type CreatePsbtInput = {
  walletName: string;
  toAddress: string;
  amountSat: number;
  feeRateSatPerVb: number;
  replaceable: boolean;
  confirmedOnly: boolean;
};

export type SignPsbtInput = {
  walletName: string;
  psbtBase64: string;
};

export type PublishPsbtInput = {
  walletName: string;
  psbtBase64: string;
};

export type CoinControlMode = "auto" | "manual";

export type InputSelectionMode =
  | "strict-manual"
  | "manual-with-auto-completion"
  | "automatic-only";

export type CoinControlUtxoOption = {
  outpoint: string;
  valueSat: number;
  label?: string;
  address?: string | null;
  confirmations?: number | null;
  confirmed?: boolean | null;
};

export type CoinControlSelection = {
  includeOutpoints: string[];
  excludeOutpoints: string[];
  confirmedOnly: boolean;
  selectionMode?: InputSelectionMode | null;
};

export type CreatePsbtWithCoinControlInput = CreatePsbtInput & {
  coinControl: CoinControlSelection;
};

export type CreateSendMaxPsbtInput = {
  walletName: string;
  toAddress: string;
  feeRateSatPerVb: number;
  replaceable: boolean;
};

export type CreateSendMaxPsbtWithCoinControlInput = CreateSendMaxPsbtInput & {
  coinControl: CoinControlSelection;
};

export type CreateSweepPsbtInput = {
  walletName: string;
  toAddress: string;
  feeRateSatPerVb: number;
  replaceable: boolean;
  coinControl: CoinControlSelection;
};

export type ConsolidationStrategy = "largest-first" | "smallest-first" | "oldest-first";

export type ConsolidationSelection = CoinControlSelection & {
  maxInputCount?: number | null;
  minInputCount?: number | null;
  minUtxoValueSat?: number | null;
  maxUtxoValueSat?: number | null;
  maxFeePctOfInputValue?: number | null;
  strategy?: ConsolidationStrategy | null;
};

export type CreateConsolidationPsbtInput = {
  walletName: string;
  feeRateSatPerVb: number;
  replaceable: boolean;
  consolidation: ConsolidationSelection;
};

export type BumpFeePsbtInput = {
  walletName: string;
  txid: string;
  feeRateSatPerVb: number;
};

export type CpfpPsbtInput = {
  walletName: string;
  parentTxid: string;
  feeRateSatPerVb: number;
};

export interface CoinControlSummaryProps {
  selectedInputCount?: number;
  selectedValueSat?: number;
  utxos?: CoinControlUtxoOption[];
  selectedUtxos?: string[];
  requiredSat?: number | null;
  estimatedFeeSat?: number | null;
  changeSat?: number | null;
  remainingSat?: number | null;
  selectionMode?: CoinControlMode;
  onSelectionModeChange?: (mode: CoinControlMode) => void;
  onUtxoSelectionChange?: (selectedUtxos: string[]) => void;
  onClearSelection?: () => void;
}

export interface CoinControlSelectorProps {
  utxos: CoinControlUtxoOption[];
  selectedUtxos: string[];
  onSelectionChange: (selectedUtxos: string[]) => void;
}

export interface PsbtPreviewPanelProps {
  psbt: WalletPsbtDto | null;
}

// UI state for Send flow
export type SendFlowState =
  | { type: "idle" }
  | { type: "loading" }
  | { type: "preview"; psbt: WalletPsbtDto }
  | { type: "error"; message: string };

// Helper to convert form → backend input
export function toCreatePsbtInput(
  form: FixedSendFormState,
  walletName: string
): CreatePsbtInput {
  const amountSat = Number(form.amountSat);
  const feeRateSatPerVb = Number(form.feeRateSatPerVb);

  if (!form.toAddress.trim()) {
    throw new Error("Address is required");
  }

  if (!Number.isFinite(amountSat) || amountSat <= 0) {
    throw new Error("Invalid amount");
  }

  if (!Number.isFinite(feeRateSatPerVb) || feeRateSatPerVb <= 0) {
    throw new Error("Invalid fee rate");
  }

  return {
    walletName,
    toAddress: form.toAddress.trim(),
    amountSat,
    feeRateSatPerVb,
    replaceable: form.replaceable,
    confirmedOnly: form.confirmedOnly,
  };
}

export function toCreateSendMaxPsbtInput(
  form: SendMaxFormState,
  walletName: string
): CreateSendMaxPsbtInput {
  const feeRateSatPerVb = Number(form.feeRateSatPerVb);

  if (!form.toAddress.trim()) {
    throw new Error("Address is required");
  }

  if (!Number.isFinite(feeRateSatPerVb) || feeRateSatPerVb <= 0) {
    throw new Error("Invalid fee rate");
  }

  return {
    walletName,
    toAddress: form.toAddress.trim(),
    feeRateSatPerVb,
    replaceable: form.replaceable,
  };
}

export function toCreateSweepPsbtInput(
  form: SweepFormState,
  walletName: string,
  coinControl: CoinControlSelection
): CreateSweepPsbtInput {
  const feeRateSatPerVb = Number(form.feeRateSatPerVb);

  if (!form.toAddress.trim()) {
    throw new Error("Address is required");
  }

  if (!Number.isFinite(feeRateSatPerVb) || feeRateSatPerVb <= 0) {
    throw new Error("Invalid fee rate");
  }

  if (coinControl.includeOutpoints.length === 0) {
    throw new Error("Select at least one UTXO to sweep");
  }

  return {
    walletName,
    toAddress: form.toAddress.trim(),
    feeRateSatPerVb,
    replaceable: form.replaceable,
    coinControl,
  };
}

export function toCreateConsolidationPsbtInput(
  form: ConsolidationFormState,
  walletName: string,
  consolidation: ConsolidationSelection
): CreateConsolidationPsbtInput {
  const feeRateSatPerVb = Number(form.feeRateSatPerVb);

  if (!Number.isFinite(feeRateSatPerVb) || feeRateSatPerVb <= 0) {
    throw new Error("Invalid fee rate");
  }

  if (consolidation.includeOutpoints.length < 2) {
    throw new Error("Select at least two UTXOs to consolidate");
  }

  return {
    walletName,
    feeRateSatPerVb,
    replaceable: form.replaceable,
    consolidation,
  };
}