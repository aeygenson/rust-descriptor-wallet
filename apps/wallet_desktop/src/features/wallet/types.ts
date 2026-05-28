

import type {
    DescriptorViewDto,
    WalletDescriptorInfoDto,
} from "../../shared/types/dtos";

// Frontend-specific descriptor presentation models.
//
// These types intentionally extend the backend DTO layer with UI-only fields
// and presentation state. Security-sensitive descriptor sanitization remains
// backend-owned.

export interface DescriptorBranchViewModel {
    id: "external" | "internal";
    title: string;
    descriptor: DescriptorViewDto;
    copyEnabled: boolean;
    securityVariant: "safe" | "warning";
}

export interface DescriptorInfoViewModel {
    walletName: string;
    network: string;
    isWatchOnly: boolean;
    containsPrivateData: boolean;
    branchCount: number;
    primaryScriptType: string | null;
    securityVariant: "safe" | "warning" | "unknown";
    branches: DescriptorBranchViewModel[];
    source: WalletDescriptorInfoDto;
}