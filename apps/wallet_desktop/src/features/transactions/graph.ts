

import type { WalletTxDto } from "../../shared/types/dtos";

export function attachTransactionGraph(transactions: WalletTxDto[]): WalletTxDto[] {
  const childTxidsByParentTxid = new Map<string, Set<string>>();

  for (const tx of transactions) {
    for (const input of tx.inputs) {
      const parentTxid = getParentTxidFromOutpoint(input.previous_outpoint);
      if (!parentTxid) {
        continue;
      }

      const children = childTxidsByParentTxid.get(parentTxid) ?? new Set<string>();
      children.add(tx.txid);
      childTxidsByParentTxid.set(parentTxid, children);
    }
  }

  return transactions.map((tx) => {
    const parentTxids = Array.from(
      new Set(
        tx.inputs
          .map((input) => getParentTxidFromOutpoint(input.previous_outpoint))
          .filter((txid): txid is string => Boolean(txid)),
      ),
    );

    const childTxids = Array.from(childTxidsByParentTxid.get(tx.txid) ?? []);

    return {
      ...tx,
      parent_txids: parentTxids,
      child_txids: childTxids,
    };
  });
}

export function getParentTxidFromOutpoint(outpoint: string): string | null {
  const [txid] = outpoint.split(":");
  return txid && txid.length > 0 ? txid : null;
}

// Heuristic: a tx is a CPFP child if it has a higher fee rate than any of its parents
export function isCpfpChildCandidate(tx: WalletTxDto, transactions: WalletTxDto[]): boolean {
  if (!tx.parent_txids || tx.parent_txids.length === 0 || tx.fee_rate_sat_per_vb == null) {
    return false;
  }

  return tx.parent_txids.some((parentTxid) => {
    const parent = transactions.find((t) => t.txid === parentTxid);
    return parent?.fee_rate_sat_per_vb != null && tx.fee_rate_sat_per_vb! > parent.fee_rate_sat_per_vb;
  });
}

// Heuristic: a tx is rate-bumped if any child has a higher fee rate than this tx
export function isRateBumpedByChild(tx: WalletTxDto, transactions: WalletTxDto[]): boolean {
  if (!tx.child_txids || tx.child_txids.length === 0 || tx.fee_rate_sat_per_vb == null) {
    return false;
  }

  return tx.child_txids.some((childTxid) => {
    const child = transactions.find((t) => t.txid === childTxid);
    return child?.fee_rate_sat_per_vb != null && child.fee_rate_sat_per_vb > tx.fee_rate_sat_per_vb!;
  });
}