import {
  canTransactionBeRbfBumped,
  canTransactionUseCpfp,
  isConfirmedTransaction,
  isPendingTransaction,
  isReceivedTransaction,
  isSentTransaction,
} from "../lib";

import type {
  TransactionFilter,
  TransactionsFilterBarProps,
} from "../types";

export function TransactionsFilterBar({
  transactions,
  activeFilter,
  onFilterChange,
}: TransactionsFilterBarProps) {
  const counts: Record<TransactionFilter, number> = {
    all: transactions.length,
    pending: transactions.filter(isPendingTransaction).length,
    confirmed: transactions.filter(isConfirmedTransaction).length,
    rbf: transactions.filter(canTransactionBeRbfBumped).length,
    cpfp: transactions.filter(canTransactionUseCpfp).length,
    sent: transactions.filter(isSentTransaction).length,
    received: transactions.filter(isReceivedTransaction).length,
  };

  const renderButton = (key: TransactionFilter, label: string) => (
    <button
      key={key}
      type="button"
      className={activeFilter === key ? "active" : undefined}
      onClick={() => onFilterChange(key)}
    >
      {label} ({counts[key]})
    </button>
  );

  return (
    <div className="transactions-filters" role="group" aria-label="Transaction filters">
      {renderButton("all", "All")}
      {renderButton("pending", "Pending")}
      {renderButton("confirmed", "Confirmed")}
      {renderButton("rbf", "RBF")}
      {renderButton("cpfp", "CPFP")}
      {renderButton("sent", "Sent")}
      {renderButton("received", "Received")}
    </div>
  );
}