import type {
    DescriptorViewDto,
} from "../../../shared/types/dtos";

import {
    formatBooleanLabel,
    formatDerivationPath,
    formatScriptType,
    formatThreshold,
} from "../format";

interface DescriptorMetadataGridProps {
    descriptor: DescriptorViewDto;
}

interface MetadataItemProps {
    label: string;
    value: string;
}

function MetadataItem(
    props: MetadataItemProps,
) {
    const {
        label,
        value,
    } = props;

    return (
        <div className="descriptor-metadata-grid__item">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    );
}

export function DescriptorMetadataGrid(
    props: DescriptorMetadataGridProps,
) {
    const {
        descriptor,
    } = props;

    return (
        <div className="descriptor-metadata-grid">
            <MetadataItem
                label="Script Type"
                value={formatScriptType(descriptor.script_type)}
            />
            <MetadataItem
                label="Private Keys"
                value={formatBooleanLabel(descriptor.has_private_keys)}
            />
            <MetadataItem
                label="Wildcard Derivation"
                value={formatBooleanLabel(descriptor.has_wildcards)}
            />
            <MetadataItem
                label="Origin Info"
                value={formatBooleanLabel(descriptor.has_origin_info)}
            />
            <MetadataItem
                label="Multisig"
                value={formatBooleanLabel(descriptor.is_multisig)}
            />
            <MetadataItem
                label="Threshold"
                value={formatThreshold(
                    descriptor.threshold,
                    descriptor.participant_count,
                )}
            />
            <MetadataItem
                label="Derivation Path"
                value={formatDerivationPath(descriptor.derivation_path)}
            />
        </div>
    );
}
