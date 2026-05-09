import type { TransactionIntentBadgeProps } from "../types";
import {
  formatTransactionIntentClass,
  formatTransactionIntentLabel,
} from "../format";

export function TransactionIntentBadge({
  intent,
}: TransactionIntentBadgeProps) {
  const label = formatTransactionIntentLabel(intent);
  const className = formatTransactionIntentClass(intent);
  const title = `Transaction intent: ${label}`;
  const ariaLabel = `Transaction intent ${label}`;
  return (
    <span
      className={className}
      title={title}
      aria-label={ariaLabel}
    >
      {label}
    </span>
  );
}
