import { describe, expect, it } from "vitest";
import {
  canCopyDescriptor,
  descriptorHasWarning,
  getDescriptorBranchCount,
  getDescriptorSecurityVariant,
  getPrimaryScriptType,
  hasDescriptorWarnings,
  hasInternalDescriptor,
} from "./lib";
import type { WalletDescriptorInfoDto } from "../../shared/types/dtos";

const watchOnlyDescriptor: WalletDescriptorInfoDto = {
  wallet_name: "watch-only",
  network: "regtest",
  is_watch_only: true,
  contains_private_data: false,
  external: {
    descriptor_redacted: "wpkh([abcd1234/84h/1h/0h]xpub.../0/*)",
    script_type: "wpkh",
    has_private_keys: false,
    has_wildcards: true,
    has_origin_info: true,
    is_multisig: false,
    threshold: null,
    participant_count: null,
    derivation_path: "/84h/1h/0h",
  },
  internal: null,
};

const signingDescriptor: WalletDescriptorInfoDto = {
  ...watchOnlyDescriptor,
  wallet_name: "signing",
  contains_private_data: true,
  internal: {
    descriptor_redacted: "wpkh([abcd1234/84h/1h/0h]tprv...<redacted>/1/*)",
    script_type: "wpkh",
    has_private_keys: true,
    has_wildcards: true,
    has_origin_info: true,
    is_multisig: false,
    threshold: null,
    participant_count: null,
    derivation_path: "/84h/1h/0h",
  },
};

describe("wallet descriptor helpers", () => {
  it("recognizes safe watch-only descriptor state", () => {
    expect(hasDescriptorWarnings(watchOnlyDescriptor)).toBe(false);
    expect(descriptorHasWarning(watchOnlyDescriptor.external)).toBe(false);
    expect(hasInternalDescriptor(watchOnlyDescriptor)).toBe(false);
    expect(getDescriptorBranchCount(watchOnlyDescriptor)).toBe(1);
    expect(getDescriptorSecurityVariant(watchOnlyDescriptor)).toBe("safe");
    expect(getPrimaryScriptType(watchOnlyDescriptor)).toBe("wpkh");
    expect(canCopyDescriptor(watchOnlyDescriptor.external)).toBe(true);
  });

  it("escalates warning state when private descriptor material is present", () => {
    expect(hasDescriptorWarnings(signingDescriptor)).toBe(true);
    expect(descriptorHasWarning(signingDescriptor.internal)).toBe(true);
    expect(hasInternalDescriptor(signingDescriptor)).toBe(true);
    expect(getDescriptorBranchCount(signingDescriptor)).toBe(2);
    expect(getDescriptorSecurityVariant(signingDescriptor)).toBe("warning");
  });
});
