

import type {
    DescriptorViewDto,
} from "../../shared/types/dtos";

export function formatScriptType(scriptType: string | null): string {
    if (!scriptType) {
        return "Unknown";
    }

    switch (scriptType) {
        case "tr":
            return "Taproot";
        case "wpkh":
            return "Native SegWit";
        case "sh-wpkh":
            return "Nested SegWit";
        case "wsh":
            return "Native SegWit Script";
        case "sh-wsh":
            return "Nested SegWit Script";
        case "pkh":
            return "Legacy P2PKH";
        case "sh":
            return "P2SH";
        case "pk":
            return "Pay To PubKey";
        case "combo":
            return "Combo";
        case "addr":
            return "Address";
        case "raw":
            return "Raw Script";
        default:
            return scriptType;
    }
}

export function formatBooleanLabel(value: boolean): string {
    return value ? "Yes" : "No";
}

export function formatThreshold(
    threshold: number | null,
    participantCount: number | null,
): string {
    if (threshold == null || participantCount == null) {
        return "n/a";
    }

    return `${threshold}-of-${participantCount}`;
}

export function formatDerivationPath(path: string | null): string {
    return path ?? "n/a";
}

export function formatDescriptorSecurity(
    descriptor: DescriptorViewDto,
): string {
    if (descriptor.has_private_keys) {
        return "Contains Private Keys";
    }

    return "Watch-Only Safe";
}

export function formatDescriptorTypeSummary(
    descriptor: DescriptorViewDto,
): string {
    const scriptType = formatScriptType(descriptor.script_type);

    if (descriptor.is_multisig) {
        return `${scriptType} Multisig`;
    }

    return `${scriptType} Singlesig`;
}