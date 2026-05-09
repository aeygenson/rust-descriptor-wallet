import type { WalletTxDto } from "../../shared/types/dtos";

export function attachTransactionGraph(transactions: WalletTxDto[]): WalletTxDto[] {
  const txids = new Set(transactions.map((tx) => tx.txid));
  const childTxidsByParentTxid = new Map<string, Set<string>>();

  for (const tx of transactions) {
    for (const input of tx.inputs ?? []) {
      const parentTxid = getParentTxidFromOutpoint(input.previous_outpoint);
      if (!parentTxid) {
        continue;
      }
      if (!txids.has(parentTxid)) {
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
        (tx.inputs ?? [])
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

export function buildTransactionIndex(
  transactions: WalletTxDto[],
): Map<string, WalletTxDto> {
  return new Map(transactions.map((tx) => [tx.txid, tx]));
}

export function getKnownParentTxids(
  tx: WalletTxDto,
  transactions: WalletTxDto[],
): string[] {
  const txIndex = buildTransactionIndex(transactions);

  return Array.from(
    new Set(
      (tx.inputs ?? [])
        .map((input) => getParentTxidFromOutpoint(input.previous_outpoint))
        .filter((txid): txid is string => Boolean(txid))
        .filter((txid) => txIndex.has(txid)),
    ),
  );
}

export function getKnownChildTxids(
  tx: WalletTxDto,
  transactions: WalletTxDto[],
): string[] {
  return transactions
    .filter((candidate) =>
      (candidate.inputs ?? []).some(
        (input) => getParentTxidFromOutpoint(input.previous_outpoint) === tx.txid,
      ),
    )
    .map((candidate) => candidate.txid);
}

export function getParentTxidFromOutpoint(
  outpoint: string | null | undefined,
): string | null {
  if (!outpoint) {
    return null;
  }

  const [txid] = outpoint.split(":");
  return txid && txid.length > 0 ? txid : null;
}

function hasHigherFeeRate(
  candidateFeeRate: number | null | undefined,
  baselineFeeRate: number | null | undefined,
): boolean {
  if (candidateFeeRate === null || candidateFeeRate === undefined) {
    return false;
  }

  if (baselineFeeRate === null || baselineFeeRate === undefined) {
    return false;
  }

  return candidateFeeRate > baselineFeeRate;
}

// Heuristic: a tx is a CPFP child if it has a higher fee rate than any of its parents
export function isCpfpChildCandidate(
  tx: WalletTxDto,
  transactions: WalletTxDto[],
): boolean {
  if (
    !tx.parent_txids ||
    tx.parent_txids.length === 0 ||
    tx.fee_rate_sat_per_vb === null ||
    tx.fee_rate_sat_per_vb === undefined
  ) {
    return false;
  }
  const txIndex = buildTransactionIndex(transactions);

  return tx.parent_txids.some((parentTxid) => {
    const parent = txIndex.get(parentTxid);
    return hasHigherFeeRate(
      tx.fee_rate_sat_per_vb,
      parent?.fee_rate_sat_per_vb,
    );
  });
}

// Heuristic: a tx is rate-bumped if any child has a higher fee rate than this tx
export function isRateBumpedByChild(
  tx: WalletTxDto,
  transactions: WalletTxDto[],
): boolean {
  if (
    !tx.child_txids ||
    tx.child_txids.length === 0 ||
    tx.fee_rate_sat_per_vb === null ||
    tx.fee_rate_sat_per_vb === undefined
  ) {
    return false;
  }
  const txIndex = buildTransactionIndex(transactions);

  return tx.child_txids.some((childTxid) => {
    const child = txIndex.get(childTxid);
    return hasHigherFeeRate(
      child?.fee_rate_sat_per_vb,
      tx.fee_rate_sat_per_vb,
    );
  });
}