

import { describe, expect, it } from "vitest";
import type { WalletTxDto } from "../../shared/types/dtos";
import {
  attachTransactionGraph,
  getParentTxidFromOutpoint,
  isCpfpChildCandidate,
  isRateBumpedByChild,
} from "./graph";

function tx(overrides: Partial<WalletTxDto>): WalletTxDto {
  return {
    txid: overrides.txid ?? "tx",
    confirmed: overrides.confirmed ?? false,
    confirmation_height: overrides.confirmation_height ?? null,
    direction: overrides.direction ?? "sent",
    replaceable: overrides.replaceable ?? false,
    net_value: overrides.net_value ?? 0,
    fee: overrides.fee ?? null,
    fee_rate_sat_per_vb: overrides.fee_rate_sat_per_vb ?? null,
    inputs: overrides.inputs ?? [],
    outputs: overrides.outputs ?? [],
    parent_txids: overrides.parent_txids,
    child_txids: overrides.child_txids,
  };
}

describe("transaction graph helpers", () => {
  it("extracts parent txid from txid:vout outpoints", () => {
    expect(getParentTxidFromOutpoint("abc123:0")).toBe("abc123");
    expect(getParentTxidFromOutpoint("")).toBeNull();
  });

  it("attaches parent and child txids from transaction inputs", () => {
    const parent = tx({
      txid: "parent",
      outputs: [
        {
          outpoint: "parent:1",
          value_sat: 10_000,
          address: null,
          is_mine: true,
          keychain: "internal",
        },
      ],
    });

    const child = tx({
      txid: "child",
      inputs: [{ previous_outpoint: "parent:1" }],
    });

    const [parentWithGraph, childWithGraph] = attachTransactionGraph([parent, child]);

    expect(parentWithGraph.child_txids).toEqual(["child"]);
    expect(parentWithGraph.parent_txids).toEqual([]);
    expect(childWithGraph.parent_txids).toEqual(["parent"]);
    expect(childWithGraph.child_txids).toEqual([]);
  });

  it("deduplicates parent txids when several inputs spend the same parent tx", () => {
    const parent = tx({ txid: "parent" });
    const child = tx({
      txid: "child",
      inputs: [
        { previous_outpoint: "parent:0" },
        { previous_outpoint: "parent:1" },
      ],
    });

    const [, childWithGraph] = attachTransactionGraph([parent, child]);

    expect(childWithGraph.parent_txids).toEqual(["parent"]);
  });

  it("detects CPFP child and rate-bumped parent by fee-rate relationship", () => {
    const graph = attachTransactionGraph([
      tx({ txid: "parent", fee_rate_sat_per_vb: 1 }),
      tx({
        txid: "child",
        fee_rate_sat_per_vb: 5,
        inputs: [{ previous_outpoint: "parent:0" }],
      }),
    ]);

    const parent = graph.find((item) => item.txid === "parent");
    const child = graph.find((item) => item.txid === "child");

    expect(parent).toBeDefined();
    expect(child).toBeDefined();
    expect(isRateBumpedByChild(parent!, graph)).toBe(true);
    expect(isCpfpChildCandidate(child!, graph)).toBe(true);
  });

  it("does not mark equal or lower-fee children as CPFP/rate bump candidates", () => {
    const graph = attachTransactionGraph([
      tx({ txid: "parent", fee_rate_sat_per_vb: 5 }),
      tx({
        txid: "child",
        fee_rate_sat_per_vb: 5,
        inputs: [{ previous_outpoint: "parent:0" }],
      }),
    ]);

    const parent = graph.find((item) => item.txid === "parent");
    const child = graph.find((item) => item.txid === "child");

    expect(parent).toBeDefined();
    expect(child).toBeDefined();
    expect(isRateBumpedByChild(parent!, graph)).toBe(false);
    expect(isCpfpChildCandidate(child!, graph)).toBe(false);
  });
});