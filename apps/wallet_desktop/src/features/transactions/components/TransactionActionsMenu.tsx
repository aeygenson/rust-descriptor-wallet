import type { MouseEvent } from "react";
import { useEffect, useRef, useState } from "react";

type ActionMessageVariant = "info" | "success" | "error";

import {
  copyTransactionTxid,
  getTransactionRowActions,
} from "../lib";
import type { TransactionActionsMenuProps } from "../types";

export function TransactionActionsMenu({
  tx,
  onDetails,
  onCopyTxid,
  onBumpFee,
  onCpfp,
  onActionMessage,
}: TransactionActionsMenuProps & {
  onActionMessage?: (
    message: string,
    variant?: ActionMessageVariant,
  ) => void;
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const actions = getTransactionRowActions(tx);
  const menuId = `tx-actions-menu-${tx.txid}`;

  const toggle = (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    setOpen((v) => !v);
  };

  const close = () => setOpen(false);

  const handleDetails = () => {
    onDetails(tx);
    close();
  };

  const handleCopyTxid = async () => {
    try {
      await copyTransactionTxid(tx);
      onCopyTxid(tx.txid);
      onActionMessage?.("Transaction id copied", "success");
    } catch (error) {
      onActionMessage?.(
        error instanceof Error
          ? error.message
          : "Failed to copy transaction id",
        "error",
      );
    } finally {
      close();
    }
  };

  const handleBumpFee = () => {
    if (!actions.canBumpFee) {
      onActionMessage?.(
        "RBF is only available for pending replaceable transactions",
        "info",
      );
      return;
    }

    onBumpFee(tx);
    close();
  };

  const handleCpfp = () => {
    if (!actions.canCpfp) {
      onActionMessage?.(
        "CPFP needs a pending transaction with a wallet-owned output",
        "info",
      );
      return;
    }

    onCpfp(tx);
    close();
  };

  // Close on outside click or Escape
  useEffect(() => {
    if (!open) return;

    const handleClick = (e: globalThis.MouseEvent) => {
      if (!containerRef.current) return;
      if (!containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpen(false);
      }
    };

    window.addEventListener("click", handleClick);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("click", handleClick);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open]);

  return (
    <div className="tx-actions" ref={containerRef}>
      <button
        className="tx-actions__button"
        type="button"
        title="Open transaction actions"
        aria-label={`Open actions for transaction ${tx.txid}`}
        onClick={toggle}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={open ? menuId : undefined}
      >
        Actions ▾
      </button>

      {open && (
        <div
          id={menuId}
          className="tx-actions__menu"
          role="menu"
          aria-label={`Actions for transaction ${tx.txid}`}
        >
          <button
            type="button"
            role="menuitem"
            title="Open transaction details"
            onClick={handleDetails}
          >
            Details
          </button>

          <button
            type="button"
            role="menuitem"
            disabled={!actions.canCopyTxid}
            title="Copy transaction id to clipboard"
            onClick={() => void handleCopyTxid()}
          >
            Copy txid
          </button>

          <button
            type="button"
            role="menuitem"
            disabled={!actions.canBumpFee}
            title={
              actions.canBumpFee
                ? "Create a replacement PSBT for this pending transaction"
                : "RBF unavailable: transaction is confirmed or not replaceable"
            }
            onClick={handleBumpFee}
          >
            Bump fee (RBF)
          </button>

          <button
            type="button"
            role="menuitem"
            disabled={!actions.canCpfp}
            title={
              actions.canCpfp
                ? "Create a child-pays-for-parent PSBT"
                : "CPFP unavailable: no pending wallet-owned output found"
            }
            onClick={handleCpfp}
          >
            Accelerate (CPFP)
          </button>
        </div>
      )}
    </div>
  );
}