import { useEffect, useMemo, useState } from "react";
import { useWallet } from "../app/providers/useWallet";
import {
  bumpFeePsbt,
  cpfpPsbt as createCpfpPsbt,
  listTransactions,
  publishPsbt,
  signPsbt,
  type BumpFeePsbtInput,
} from "../features/transactions/api";
import { BumpFeePanel } from "../features/transactions/components/BumpFeePanel";
import { CpfpPanel } from "../features/transactions/components/CpfpPanel";
import { CpfpPsbtWorkflowPanel } from "../features/transactions/components/CpfpPsbtWorkflowPanel";
import { RbfPsbtWorkflowPanel } from "../features/transactions/components/RbfPsbtWorkflowPanel";
import { TransactionDetailsModal } from "../features/transactions/components/TransactionDetailsModal";
import { TransactionActionsMenu } from "../features/transactions/components/TransactionActionsMenu";
import { TransactionRelationCell } from "../features/transactions/components/TransactionRelationCell";
import { TransactionsFilterBar } from "../features/transactions/components/TransactionsFilterBar";
import type { CpfpPsbtInput, TransactionFilter } from "../features/transactions/types";
import {
  formatBooleanLabel,
  formatConfirmationHeight,
  formatDirectionClass,
  formatDirectionLabel,
  formatFeeRate,
  formatSats,
  formatSignedSats,
} from "../features/transactions/format";
import {
  canTransactionBeRbfBumped,
  canTransactionUseCpfp,
  getCpfpOutpoints,
  isConfirmedTransaction,
  isPendingTransaction,
  isReceivedTransaction,
  isSentTransaction,
} from "../features/transactions/lib";
import {
  attachTransactionGraph,
  isCpfpChildCandidate,
  isRateBumpedByChild,
} from "../features/transactions/graph";
import type {
  TxBroadcastResultDto,
  WalletCpfpPsbtDto,
  WalletPsbtDto,
  WalletSignedPsbtDto,
  WalletTxDto,
} from "../shared/types/dtos";


export function TransactionsPage() {
  const { selectedWalletName } = useWallet();
  const [transactions, setTransactions] = useState<WalletTxDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [detailsTx, setDetailsTx] = useState<WalletTxDto | null>(null);
  const [bumpFeeTx, setBumpFeeTx] = useState<WalletTxDto | null>(null);
  const [bumpFeeLoading, setBumpFeeLoading] = useState(false);
  const [rbfPsbt, setRbfPsbt] = useState<WalletPsbtDto | null>(null);
  const [actionMessage, setActionMessage] = useState<{
    text: string;
    variant: "info" | "success" | "error";
  } | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [rbfSignedPsbt, setRbfSignedPsbt] = useState<WalletSignedPsbtDto | null>(null);
  const [rbfBroadcastResult, setRbfBroadcastResult] = useState<TxBroadcastResultDto | null>(null);
  const [rbfActionLoading, setRbfActionLoading] = useState(false);
  const [cpfpTx, setCpfpTx] = useState<WalletTxDto | null>(null);
  const [cpfpPsbtDto, setCpfpPsbtDto] = useState<WalletCpfpPsbtDto | null>(null);
  const [cpfpSignedPsbt, setCpfpSignedPsbt] = useState<WalletSignedPsbtDto | null>(null);
  const [cpfpBroadcastResult, setCpfpBroadcastResult] = useState<TxBroadcastResultDto | null>(null);
  const [cpfpLoading, setCpfpLoading] = useState(false);
  const [cpfpActionLoading, setCpfpActionLoading] = useState(false);
  const [transactionFilter, setTransactionFilter] = useState<TransactionFilter>("all");

  useEffect(() => {
    let cancelled = false;

    if (!selectedWalletName) {
      setTransactions([]);
      setLoading(false);
      setError(null);
      return () => {
        cancelled = true;
      };
    }

    setLoading(true);
    setError(null);

    listTransactions(selectedWalletName)
      .then((data) => {
        if (!cancelled) {
          setTransactions(data);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          const msg = e instanceof Error ? e.message : String(e);
          setTransactions([]);
          setError(msg);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedWalletName]);

  useEffect(() => {
    setDetailsTx(null);
    setBumpFeeTx(null);
    setRbfPsbt(null);
    setRbfSignedPsbt(null);
    setRbfBroadcastResult(null);
    setRbfActionLoading(false);
    setActionMessage(null);
    setActionError(null);
    setCpfpTx(null);
    setCpfpPsbtDto(null);
    setCpfpSignedPsbt(null);
    setCpfpBroadcastResult(null);
    setCpfpLoading(false);
    setCpfpActionLoading(false);
  }, [selectedWalletName]);

  const refreshTransactions = async () => {
    if (!selectedWalletName) {
      setTransactions([]);
      return;
    }

    const data = await listTransactions(selectedWalletName);
    setTransactions(data);
  };

  const showActionMessage = (
    message: string,
    variant: "info" | "success" | "error" = "info",
  ) => {
    setActionMessage({ text: message, variant });
  };

  const transactionsWithGraph = useMemo(() => {
    return attachTransactionGraph(transactions);
  }, [transactions]);

  const handleDetails = (tx: WalletTxDto) => {
    setActionError(null);
    setDetailsTx(tx);
  };

  const handleOpenTxById = (txid: string) => {
    const tx = transactionsWithGraph.find((item) => item.txid === txid);
    if (!tx) {
      showActionMessage("Related transaction is not loaded in the current wallet history", "error");
      return;
    }

    setActionError(null);
    setDetailsTx(tx);
  };

  const handleCopyTxid = async (txid: string) => {
    try {
      setActionError(null);
      await navigator.clipboard.writeText(txid);
      showActionMessage("Copied txid");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(`Could not copy txid: ${msg}`);
    }
  };

  const handleBumpFee = (tx: WalletTxDto) => {
    if (!canTransactionBeRbfBumped(tx)) {
      showActionMessage("This transaction cannot be bumped");
      return;
    }

    setActionError(null);
    setActionMessage(null);
    setBumpFeeTx(tx);
    setRbfPsbt(null);
    setRbfSignedPsbt(null);
    setRbfBroadcastResult(null);
    setBumpFeeLoading(false);
    setRbfActionLoading(false);
    setCpfpTx(null);
    setCpfpPsbtDto(null);
    setCpfpSignedPsbt(null);
    setCpfpBroadcastResult(null);
    setCpfpLoading(false);
    setCpfpActionLoading(false);
  };

  const handleCpfp = (tx: WalletTxDto) => {
    if (!canTransactionUseCpfp(tx)) {
      showActionMessage("CPFP is only available for unconfirmed transactions");
      return;
    }

    setActionError(null);
    setActionMessage(null);
    setCpfpTx(tx);
    setCpfpPsbtDto(null);
    setCpfpSignedPsbt(null);
    setCpfpBroadcastResult(null);
    setCpfpLoading(false);
    setCpfpActionLoading(false);
    setBumpFeeTx(null);
    setRbfPsbt(null);
    setRbfSignedPsbt(null);
    setRbfBroadcastResult(null);
    setBumpFeeLoading(false);
    setRbfActionLoading(false);
  };

  const handleCreateBumpFeePsbt = async (input: BumpFeePsbtInput) => {
    try {
      setBumpFeeLoading(true);
      setActionError(null);
      setError(null);
      const result = await bumpFeePsbt(input);
      setRbfPsbt(result);
      setRbfSignedPsbt(null);
      setRbfBroadcastResult(null);
      showActionMessage("Replacement PSBT created. Review, sign, and broadcast it next.");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(msg);
    } finally {
      setBumpFeeLoading(false);
    }
  };

  const handleCreateCpfpPsbt = async (input: CpfpPsbtInput) => {
    try {
      setCpfpLoading(true);
      setActionError(null);
      setError(null);
      const result = await createCpfpPsbt({
        walletName: input.walletName,
        parentTxid: input.parentTxid,
        selectedOutpoint: input.selectedOutpoint,
        feeRateSatPerVb: input.feeRateSatVb,
      });
      setCpfpPsbtDto(result);
      setCpfpSignedPsbt(null);
      setCpfpBroadcastResult(null);
      showActionMessage("CPFP PSBT created. Review, sign, and broadcast it next.");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(msg);
    } finally {
      setCpfpLoading(false);
    }
  };

  const handleCancelBumpFee = () => {
    setBumpFeeTx(null);
    setRbfPsbt(null);
    setRbfSignedPsbt(null);
    setRbfBroadcastResult(null);
    setRbfActionLoading(false);
    setActionError(null);
  };

  const handleCancelCpfp = () => {
    setCpfpTx(null);
    setCpfpPsbtDto(null);
    setCpfpSignedPsbt(null);
    setCpfpBroadcastResult(null);
    setCpfpLoading(false);
    setCpfpActionLoading(false);
    setActionError(null);
  };

  const handleSignRbfPsbt = async () => {
    if (!selectedWalletName || !rbfPsbt) {
      setActionError("No replacement PSBT is available to sign");
      return;
    }

    try {
      setRbfActionLoading(true);
      setActionError(null);
      const signed = await signPsbt({
        walletName: selectedWalletName,
        psbtBase64: rbfPsbt.psbt_base64,
      });
      setRbfSignedPsbt(signed);
      setRbfBroadcastResult(null);
      showActionMessage("Replacement PSBT signed. Broadcast it next.");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(msg);
    } finally {
      setRbfActionLoading(false);
    }
  };

  const handleBroadcastRbfPsbt = async () => {
    if (!selectedWalletName || !rbfSignedPsbt) {
      setActionError("No signed replacement PSBT is available to broadcast");
      return;
    }

    try {
      setRbfActionLoading(true);
      setActionError(null);
      const result = await publishPsbt({
        walletName: selectedWalletName,
        psbtBase64: rbfSignedPsbt.psbt_base64,
      });
      setRbfBroadcastResult(result);
      showActionMessage("Replacement transaction broadcast");
      await refreshTransactions();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(msg);
    } finally {
      setRbfActionLoading(false);
    }
  };

  const handleSignCpfpPsbt = async () => {
    if (!selectedWalletName || !cpfpPsbtDto) {
      setActionError("No CPFP PSBT is available to sign");
      return;
    }

    try {
      setCpfpActionLoading(true);
      setActionError(null);
      const signed = await signPsbt({
        walletName: selectedWalletName,
        psbtBase64: cpfpPsbtDto.psbt_base64,
      });
      setCpfpSignedPsbt(signed);
      setCpfpBroadcastResult(null);
      showActionMessage("CPFP PSBT signed. Broadcast it next.");
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(msg);
    } finally {
      setCpfpActionLoading(false);
    }
  };

  const handleBroadcastCpfpPsbt = async () => {
    if (!selectedWalletName || !cpfpSignedPsbt) {
      setActionError("No signed CPFP PSBT is available to broadcast");
      return;
    }

    try {
      setCpfpActionLoading(true);
      setActionError(null);
      const result = await publishPsbt({
        walletName: selectedWalletName,
        psbtBase64: cpfpSignedPsbt.psbt_base64,
      });
      setCpfpBroadcastResult(result);
      showActionMessage("CPFP transaction broadcast");
      await refreshTransactions();
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setActionError(msg);
    } finally {
      setCpfpActionLoading(false);
    }
  };

  // --- Transaction summary calculations ---
  const totalTransactions = transactionsWithGraph.length;
  const incomingCount = transactionsWithGraph.filter(isReceivedTransaction).length;
  const outgoingCount = transactionsWithGraph.filter(isSentTransaction).length;
  const replaceableCount = transactionsWithGraph.filter(canTransactionBeRbfBumped).length;
  const filteredTransactions = useMemo(() => {
    switch (transactionFilter) {
      case "pending":
        return transactionsWithGraph.filter(isPendingTransaction);
      case "confirmed":
        return transactionsWithGraph.filter(isConfirmedTransaction);
      case "rbf":
        return transactionsWithGraph.filter(canTransactionBeRbfBumped);
      case "cpfp":
        return transactionsWithGraph.filter(canTransactionUseCpfp);
      case "sent":
        return transactionsWithGraph.filter(isSentTransaction);
      case "received":
        return transactionsWithGraph.filter(isReceivedTransaction);
      case "all":
      default:
        return transactionsWithGraph;
    }
  }, [transactionsWithGraph, transactionFilter]);
  const cpfpOutpoints = useMemo(() => {
    return cpfpTx ? getCpfpOutpoints(cpfpTx) : [];
  }, [cpfpTx]);

  // const confirmedCount = transactions.filter((tx) => tx.confirmed).length;

  return (
    <section className="transactions-page">
      <div className="transactions-page__header transactions-page__header--compact">
        <div className="transactions-wallet-pill">
          Wallet: <span>{selectedWalletName || "—"}</span>
        </div>

        <div className="transactions-summary">
          <div className="transactions-summary-card">
            <span className="transactions-summary-card__icon" aria-hidden="true">▧</span>
            <span className="transactions-summary-card__label">Total</span>
            <strong className="transactions-summary-card__value">{totalTransactions}</strong>
          </div>

          <div className="transactions-summary-card">
            <span className="transactions-summary-card__icon" aria-hidden="true">↓</span>
            <span className="transactions-summary-card__label">Incoming</span>
            <strong className="transactions-summary-card__value">{incomingCount}</strong>
          </div>

          <div className="transactions-summary-card">
            <span className="transactions-summary-card__icon" aria-hidden="true">↑</span>
            <span className="transactions-summary-card__label">Outgoing</span>
            <strong className="transactions-summary-card__value">{outgoingCount}</strong>
          </div>

          <div className="transactions-summary-card">
            <span className="transactions-summary-card__icon" aria-hidden="true">⟲</span>
            <span className="transactions-summary-card__label">Replaceable</span>
            <strong className="transactions-summary-card__value">{replaceableCount}</strong>
          </div>
        </div>
      </div>

      {loading && <div className="transactions-state">Loading transactions...</div>}
      {error && <div className="transactions-error">Error: {error}</div>}

      {actionMessage && (
        <div
          className={`transactions-action-message transactions-action-message--${actionMessage.variant}`}
          role={actionMessage.variant === "error" ? "alert" : "status"}
        >
          <span>{actionMessage.text}</span>
          <button
            type="button"
            className="transactions-action-message__dismiss"
            onClick={() => setActionMessage(null)}
            aria-label="Dismiss transaction action message"
          >
            ×
          </button>
        </div>
      )}
      {actionError && <div className="transactions-action-error">{actionError}</div>}

      <TransactionsFilterBar
        transactions={transactionsWithGraph}
        activeFilter={transactionFilter}
        onFilterChange={setTransactionFilter}
      />

      {detailsTx && (
        <TransactionDetailsModal
          tx={detailsTx}
          onOpenTx={handleOpenTxById}
          onClose={() => setDetailsTx(null)}
        />
      )}

      {bumpFeeTx && selectedWalletName && (
        <div className="transactions-workflow-panel">
          <h2>Bump fee / RBF</h2>
          <BumpFeePanel
            tx={bumpFeeTx}
            walletName={selectedWalletName}
            loading={bumpFeeLoading}
            onCancel={handleCancelBumpFee}
            onCreatePsbt={handleCreateBumpFeePsbt}
          />
          {rbfPsbt && (
            <RbfPsbtWorkflowPanel
              psbt={rbfPsbt}
              signedPsbt={rbfSignedPsbt}
              broadcastResult={rbfBroadcastResult}
              loading={rbfActionLoading}
              onSign={handleSignRbfPsbt}
              onBroadcast={handleBroadcastRbfPsbt}
              onClose={handleCancelBumpFee}
            />
          )}
        </div>
      )}

      {cpfpTx && selectedWalletName && (
        <div className="transactions-workflow-panel">
          <h2>Child Pays For Parent / CPFP</h2>
          {cpfpOutpoints.length === 0 && (
            <div className="transactions-action-error">
              CPFP needs a spendable wallet-owned output from this parent transaction. No selectable wallet-owned outputs are currently exposed for this transaction.
            </div>
          )}
          <CpfpPanel
            tx={cpfpTx}
            walletName={selectedWalletName}
            availableOutpoints={cpfpOutpoints}
            loading={cpfpLoading}
            onCancel={handleCancelCpfp}
            onCreatePsbt={handleCreateCpfpPsbt}
          />
          {cpfpPsbtDto && (
            <CpfpPsbtWorkflowPanel
              psbt={cpfpPsbtDto}
              signedPsbt={cpfpSignedPsbt}
              broadcastResult={cpfpBroadcastResult}
              loading={cpfpActionLoading}
              onSign={handleSignCpfpPsbt}
              onBroadcast={handleBroadcastCpfpPsbt}
              onClose={handleCancelCpfp}
            />
          )}
        </div>
      )}

      {!loading && !error && filteredTransactions.length === 0 && (
        <div className="transactions-state">
          {transactionsWithGraph.length === 0 ? "No transactions found." : "No transactions match this filter."}
        </div>
      )}

      {filteredTransactions.length > 0 && (
        <div className="transactions-table-card">
          <div className="transactions-table-scroll">
            <table className="transactions-table">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Txid</th>
                  <th>Direction</th>
                  <th>Net Value</th>
                  <th>Fee</th>
                  <th>Fee Rate</th>
                  <th>RBF</th>
                  <th>Status</th>
                  <th>Height</th>
                  <th>Parents</th>
                  <th>Children</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredTransactions.map((tx, index) => {
                  return (
                    <tr key={`${tx.txid}-${index}`}>
                      <td>
                        <span className="transactions-row-index">{index + 1}</span>
                      </td>
                      <td>
                        <div className="transactions-txid-cell">
                          <span className="transactions-txid" title={tx.txid}>
                            {tx.txid}
                          </span>
                          {renderGraphBadges(tx, transactionsWithGraph)}
                        </div>
                      </td>
                      <td>
                        <span
                          className={`transactions-direction transactions-direction--${formatDirectionClass(tx.direction)}`}
                        >
                          {formatDirectionLabel(tx.direction)}
                        </span>
                      </td>
                      <td
                        className={
                          tx.net_value > 0
                            ? "transactions-value transactions-value--positive"
                            : tx.net_value < 0
                              ? "transactions-value transactions-value--negative"
                              : "transactions-value"
                        }
                      >
                        {formatSignedSats(tx.net_value)}
                      </td>
                      <td className="transactions-value">{formatSats(tx.fee)}</td>
                      <td className="transactions-value">{formatFeeRate(tx.fee_rate_sat_per_vb)}</td>
                      <td>
                        <span
                          className={
                            tx.replaceable
                              ? "transactions-badge transactions-badge--replaceable"
                              : "transactions-badge transactions-badge--final"
                          }
                        >
                          {formatBooleanLabel(tx.replaceable)}
                        </span>
                      </td>
                      <td>
                        <span
                          className={
                            tx.confirmed
                              ? "transactions-badge transactions-badge--confirmed"
                              : "transactions-badge transactions-badge--pending"
                          }
                        >
                          {tx.confirmed ? "confirmed" : "pending"}
                        </span>
                      </td>
                      <td>{formatConfirmationHeight(tx.confirmation_height)}</td>
                      <td>
                        <TransactionRelationCell
                          txids={tx.parent_txids ?? []}
                          kind="parents"
                          onOpenTx={handleOpenTxById}
                        />
                      </td>
                      <td>
                        <TransactionRelationCell
                          txids={tx.child_txids ?? []}
                          kind="children"
                          onOpenTx={handleOpenTxById}
                        />
                      </td>
                      <td>
                        <TransactionActionsMenu
                          tx={tx}
                          onDetails={handleDetails}
                          onCopyTxid={handleCopyTxid}
                          onBumpFee={handleBumpFee}
                          onCpfp={handleCpfp}
                          onActionMessage={showActionMessage}
                        />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </section>
  );
}

function renderGraphBadges(tx: WalletTxDto, transactions: WalletTxDto[]) {
  const isCpfpChild = isCpfpChildCandidate(tx, transactions);
  const isRateBumped = isRateBumpedByChild(tx, transactions);

  if (!isCpfpChild && !isRateBumped) {
    return null;
  }

  return (
    <div className="transactions-graph-badges">
      {isCpfpChild && (
        <span className="transactions-badge transactions-badge--cpfp-child">
          CPFP child
        </span>
      )}
      {isRateBumped && (
        <span className="transactions-badge transactions-badge--rate-bumped">
          Rate bumped
        </span>
      )}
    </div>
  );
}
