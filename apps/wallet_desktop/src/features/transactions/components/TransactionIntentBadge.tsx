import type { TransactionIntentBadgeProps } from "../types";
import {
  formatTransactionIntentClass,
  formatTransactionIntentLabel,
} from "../format";

export function TransactionIntentBadge({ intent }: TransactionIntentBadgeProps) {
  return (
    <span className={formatTransactionIntentClass(intent)}>
      {formatTransactionIntentLabel(intent)}
    </span>
  );
}
