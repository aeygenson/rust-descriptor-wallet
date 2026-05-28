

import type {
    DescriptorBranchViewModel,
} from "../types";

import {
    formatBooleanLabel,
    formatDerivationPath,
    formatDescriptorSecurity,
    formatDescriptorTypeSummary,
    formatThreshold,
} from "../format";

interface DescriptorBranchPanelProps {
    branch: DescriptorBranchViewModel;
}

export function DescriptorBranchPanel(
    props: DescriptorBranchPanelProps,
) {
    const {
        branch,
    } = props;

    const {
        descriptor,
    } = branch;

    async function handleCopy(): Promise<void> {
        if (!branch.copyEnabled) {
            return;
        }

        await navigator.clipboard.writeText(
            descriptor.descriptor_redacted,
        );
    }

    return (
        <div className="descriptor-branch-panel">
            <div className="descriptor-branch-panel__header">
                <div>
                    <h3 className="descriptor-branch-panel__title">
                        {branch.title}
                    </h3>
                    <div className="descriptor-branch-panel__subtitle">
                        {formatDescriptorTypeSummary(descriptor)}
                    </div>
                </div>

                <div
                    className={[
                        "descriptor-branch-panel__security",
                        `descriptor-branch-panel__security--${branch.securityVariant}`,
                    ].join(" ")}
                >
                    {formatDescriptorSecurity(descriptor)}
                </div>
            </div>

            <div className="descriptor-branch-panel__descriptor-box">
                <code>
                    {descriptor.descriptor_redacted}
                </code>
            </div>

            <div className="descriptor-branch-panel__actions">
                <button
                    disabled={!branch.copyEnabled}
                    onClick={() => {
                        void handleCopy();
                    }}
                    type="button"
                >
                    Copy Redacted Descriptor
                </button>
            </div>

            <div className="descriptor-branch-panel__metadata-grid">
                <div>
                    <span>Script Type</span>
                    <strong>
                        {formatDescriptorTypeSummary(descriptor)}
                    </strong>
                </div>

                <div>
                    <span>Private Keys</span>
                    <strong>
                        {formatBooleanLabel(descriptor.has_private_keys)}
                    </strong>
                </div>

                <div>
                    <span>Wildcard Derivation</span>
                    <strong>
                        {formatBooleanLabel(descriptor.has_wildcards)}
                    </strong>
                </div>

                <div>
                    <span>Origin Info</span>
                    <strong>
                        {formatBooleanLabel(descriptor.has_origin_info)}
                    </strong>
                </div>

                <div>
                    <span>Multisig</span>
                    <strong>
                        {formatBooleanLabel(descriptor.is_multisig)}
                    </strong>
                </div>

                <div>
                    <span>Threshold</span>
                    <strong>
                        {formatThreshold(
                            descriptor.threshold,
                            descriptor.participant_count,
                        )}
                    </strong>
                </div>

                <div>
                    <span>Derivation Path</span>
                    <strong>
                        {formatDerivationPath(
                            descriptor.derivation_path,
                        )}
                    </strong>
                </div>
            </div>
        </div>
    );
}