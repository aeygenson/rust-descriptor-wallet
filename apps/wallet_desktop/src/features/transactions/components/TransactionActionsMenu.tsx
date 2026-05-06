import type { MouseEvent } from "react";
import { useEffect, useRef, useState } from "react";
import {
  canTransactionBeRbfBumped,
  canTransactionUseCpfp,
} from "../lib";
import type { TransactionActionsMenuProps } from "../types";

export function TransactionActionsMenu({
  tx,
  onDetails,
  onCopyTxid,
  onBumpFee,
  onCpfp,
}: TransactionActionsMenuProps & {
  onActionMessage?: (message: string, variant?: "info" | "success" | "error") => void;
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const canBump = canTransactionBeRbfBumped(tx);
  const canCpfp = canTransactionUseCpfp(tx);

  const toggle = (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    setOpen((v) => !v);
  };

  const close = () => setOpen(false);

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
        onClick={toggle}
        aria-haspopup="menu"
        aria-expanded={open}
      >
        Actions ▾
      </button>

      {open && (
        <div className="tx-actions__menu" role="menu">
          <button
            type="button"
            role="menuitem"
            onClick={() => {
              onDetails(tx);
              close();
            }}
          >
            Details
          </button>

          <button
            type="button"
            role="menuitem"
            onClick={() => {
              onCopyTxid(tx.txid);
              close();
            }}
          >
            Copy txid
          </button>

          <button
            type="button"
            role="menuitem"
            disabled={!canBump}
            onClick={() => {
              if (!canBump) return;
              onBumpFee(tx);
              close();
            }}
          >
            Bump fee (RBF)
          </button>

          <button
            type="button"
            role="menuitem"
            disabled={!canCpfp}
            onClick={() => {
              if (!canCpfp) return;
              onCpfp(tx);
              close();
            }}
          >
            Accelerate (CPFP)
          </button>
        </div>
      )}
    </div>
  );
}