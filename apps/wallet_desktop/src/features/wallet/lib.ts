

import type {
    DescriptorViewDto,
    WalletDescriptorInfoDto,
} from "../../shared/types/dtos";

export function hasDescriptorWarnings(
    info: WalletDescriptorInfoDto | null | undefined,
): boolean {
    if (!info) {
        return false;
    }

    return info.contains_private_data
        || descriptorHasWarning(info.external)
        || descriptorHasWarning(info.internal);
}

export function descriptorHasWarning(
    descriptor: DescriptorViewDto | null | undefined,
): boolean {
    if (!descriptor) {
        return false;
    }

    return descriptor.has_private_keys;
}

export function hasInternalDescriptor(
    info: WalletDescriptorInfoDto | null | undefined,
): boolean {
    return Boolean(info?.internal);
}

export function getDescriptorBranchCount(
    info: WalletDescriptorInfoDto | null | undefined,
): number {
    if (!info) {
        return 0;
    }

    return info.internal ? 2 : 1;
}

export function getDescriptorSecurityVariant(
    info: WalletDescriptorInfoDto | null | undefined,
): "warning" | "safe" | "unknown" {
    if (!info) {
        return "unknown";
    }

    if (hasDescriptorWarnings(info)) {
        return "warning";
    }

    return "safe";
}

export function getPrimaryScriptType(
    info: WalletDescriptorInfoDto | null | undefined,
): string | null {
    return info?.external.script_type ?? null;
}

export function canCopyDescriptor(
    descriptor: DescriptorViewDto | null | undefined,
): boolean {
    return Boolean(descriptor?.descriptor_redacted.trim());
}