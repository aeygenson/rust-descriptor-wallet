import { useMemo } from "react";

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
  const counts = useMemo<Record<TransactionFilter, number>>(
    () => ({
      all: transactions.length,
      pending: transactions.filter(isPendingTransaction).length,
      confirmed: transactions.filter(isConfirmedTransaction).length,
      rbf: transactions.filter(canTransactionBeRbfBumped).length,
      cpfp: transactions.filter(canTransactionUseCpfp).length,
      sent: transactions.filter(isSentTransaction).length,
      received: transactions.filter(isReceivedTransaction).length,
    }),
    [transactions],
  );

  const activeFilterLabel = activeFilter.toUpperCase();
  const activeFilterCount = counts[activeFilter];
  const totalTransactions = transactions.length;

  const renderButton = (key: TransactionFilter, label: string) => {
    const isActive = activeFilter === key;
    const count = counts[key];
    const percentage =
      totalTransactions > 0
        ? Math.round((count / totalTransactions) * 100)
        : 0;
    const title = `${label} transactions (${count} · ${percentage}%)`;
    const ariaLabel = `${label} transactions filter with ${count} transactions`;

    return (
      <button
        key={key}
        type="button"
        className={isActive ? "active" : undefined}
        title={title}
        aria-pressed={isActive}
        aria-label={ariaLabel}
        data-filter={key}
        data-count={count}
        onClick={() => onFilterChange(key)}
      >
        <span className="transactions-filter-label">{label}</span>
        <span
          className="transactions-filter-count"
          title={`${percentage}% of all transactions`}
        >
          {count}
        </span>
      </button>
    );
  };

  return (
    <div
      className="transactions-filters"
      role="group"
      aria-label="Transaction filters"
      data-active-filter={activeFilter}
      data-total-transactions={totalTransactions}
    >
      <div className="transactions-filters__summary">
        <span className="transactions-filters__summary-label">
          Active filter
        </span>
        <span className="transactions-filters__summary-value">
          {activeFilterLabel} · {activeFilterCount.toLocaleString()} /{" "}
          {totalTransactions.toLocaleString()}
        </span>
      </div>
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