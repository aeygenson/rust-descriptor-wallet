

import type {
    WalletDescriptorInfoDto,
} from "../../../shared/types/dtos";
import type {
    DescriptorBranchViewModel,
    DescriptorInfoViewModel,
} from "../types";

import {
    canCopyDescriptor,
    descriptorHasWarning,
    getDescriptorBranchCount,
    getDescriptorSecurityVariant,
    getPrimaryScriptType,
} from "../lib";
import {
    formatBooleanLabel,
    formatScriptType,
} from "../format";
import {
    DescriptorBranchPanel,
} from "./DescriptorBranchPanel";

interface DescriptorInfoCardProps {
    info: WalletDescriptorInfoDto | null;
    loading?: boolean;
    error?: string | null;
    onRefresh?: () => void;
}

function toBranchViewModel(
    id: "external" | "internal",
    title: string,
    descriptor: WalletDescriptorInfoDto["external"],
): DescriptorBranchViewModel {
    return {
        id,
        title,
        descriptor,
        copyEnabled: canCopyDescriptor(descriptor),
        securityVariant: descriptorHasWarning(descriptor) ? "warning" : "safe",
    };
}

function toViewModel(
    info: WalletDescriptorInfoDto,
): DescriptorInfoViewModel {
    const branches: DescriptorBranchViewModel[] = [
        toBranchViewModel(
            "external",
            "External / Receive Descriptor",
            info.external,
        ),
    ];

    if (info.internal) {
        branches.push(
            toBranchViewModel(
                "internal",
                "Internal / Change Descriptor",
                info.internal,
            ),
        );
    }

    return {
        walletName: info.wallet_name,
        network: info.network,
        isWatchOnly: info.is_watch_only,
        containsPrivateData: info.contains_private_data,
        branchCount: getDescriptorBranchCount(info),
        primaryScriptType: getPrimaryScriptType(info),
        securityVariant: getDescriptorSecurityVariant(info),
        branches,
        source: info,
    };
}

export function DescriptorInfoCard(
    props: DescriptorInfoCardProps,
) {
    const {
        info,
        loading = false,
        error = null,
        onRefresh,
    } = props;

    if (loading) {
        return (
            <section className="descriptor-info-card">
                <div className="descriptor-info-card__header">
                    <div>
                        <h2>Descriptor Info</h2>
                        <p>Loading descriptor metadata…</p>
                    </div>
                </div>
            </section>
        );
    }

    if (error) {
        return (
            <section className="descriptor-info-card descriptor-info-card--error">
                <div className="descriptor-info-card__header">
                    <div>
                        <h2>Descriptor Info</h2>
                        <p>{error}</p>
                    </div>

                    {onRefresh ? (
                        <button
                            onClick={onRefresh}
                            type="button"
                        >
                            Retry
                        </button>
                    ) : null}
                </div>
            </section>
        );
    }

    if (!info) {
        return (
            <section className="descriptor-info-card descriptor-info-card--empty">
                <div className="descriptor-info-card__header">
                    <div>
                        <h2>Descriptor Info</h2>
                        <p>Select a wallet to inspect safe descriptor metadata.</p>
                    </div>
                </div>
            </section>
        );
    }

    const viewModel = toViewModel(info);

    return (
        <section className="descriptor-info-card">
            <div className="descriptor-info-card__header">
                <div>
                    <h2>Descriptor Info</h2>
                    <p>
                        Safe descriptor inspection for {viewModel.walletName}.
                    </p>
                </div>

                <div
                    className={[
                        "descriptor-info-card__security",
                        `descriptor-info-card__security--${viewModel.securityVariant}`,
                    ].join(" ")}
                >
                    {viewModel.securityVariant === "warning"
                        ? "Private Data Detected"
                        : "Safe Redacted View"}
                </div>
            </div>

            <div className="descriptor-info-card__summary-grid">
                <div>
                    <span>Network</span>
                    <strong>{viewModel.network}</strong>
                </div>

                <div>
                    <span>Watch-only</span>
                    <strong>
                        {formatBooleanLabel(viewModel.isWatchOnly)}
                    </strong>
                </div>

                <div>
                    <span>Primary Script</span>
                    <strong>
                        {formatScriptType(viewModel.primaryScriptType)}
                    </strong>
                </div>

                <div>
                    <span>Descriptor Branches</span>
                    <strong>{viewModel.branchCount}</strong>
                </div>

                <div>
                    <span>Private Data</span>
                    <strong>
                        {formatBooleanLabel(viewModel.containsPrivateData)}
                    </strong>
                </div>
            </div>

            <div className="descriptor-info-card__branches">
                {viewModel.branches.map((branch) => (
                    <DescriptorBranchPanel
                        branch={branch}
                        key={branch.id}
                    />
                ))}
            </div>
        </section>
    );
}