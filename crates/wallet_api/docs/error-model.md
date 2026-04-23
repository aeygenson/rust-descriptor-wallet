# Wallet API Error Model

`WalletApiError` is the caller-facing error boundary for `wallet_api`.

It normalizes failures from storage, wallet-core domain logic, sync backends, broadcast backends, DTO parsing, and PSBT handling into a stable set of API errors.

## Error Sources

Errors can originate from:

- `wallet_storage`: wallet registry and imported metadata failures
- `wallet_core`: wallet loading, transaction building, signing, PSBT validation, RBF, CPFP, coin control, and consolidation policy
- `wallet_sync`: Esplora, Electrum, and Bitcoin Core RPC sync or broadcast failures
- `wallet_api`: API-level parsing and orchestration checks

Callers should handle `WalletApiError`, not lower-level crate errors.

## Main Categories

Input and request errors:

- `InvalidInput`
- `InvalidAmount`
- `InvalidFeeRate`
- `InvalidDestinationAddress`
- `DestinationNetworkMismatch`

Wallet and state lookup:

- `Storage`
- `NotFound`
- `TransactionNotFound`

Transaction eligibility:

- `TransactionAlreadyConfirmed`
- `TransactionNotReplaceable`
- `FeeRateTooLowForBump`

PSBT construction and signing:

- `PsbtBuildFailed`
- `FeeCalculationFailedWithReason`
- `FeeCalculationFailed`
- `InvalidPsbtEncoding`
- `InvalidPsbtStructure`
- `InvalidPsbtSemantic`
- `InvalidPsbt`
- `SignPsbtFailed`
- `WatchOnlyCannotSign`
- `PsbtNotFinalized`
- `SendNotFinalized`
- `ExtractTxFailed`

Maintenance transaction builders:

- `FeeBumpBuildFailed`
- `CpfpBuildFailed`

Backend and broadcast:

- `Sync`
- `InvalidBackend`
- `BackendUnavailable`
- `BroadcastTransport`
- `BroadcastFailed`
- `BroadcastMempoolConflict`
- `BroadcastAlreadyConfirmed`
- `BroadcastMissingInputs`
- `BroadcastInsufficientFee`

Low-level fallback:

- `Core`

Most important `wallet_core` errors are mapped into specific API errors. The `Core` variant remains as a fallback for lower-level failures that do not yet have a more stable API category.

## Coin Control And Consolidation Mapping

Coin-control and consolidation policy errors are intentionally mapped to `InvalidInput` because they are caller-actionable request problems.

Examples:

- malformed outpoint
- selected outpoint not found
- selected outpoint not spendable
- selected outpoint not confirmed while `confirmed_only` is set
- include/exclude conflict
- strict manual selection cannot fund the transaction
- consolidation has too few eligible inputs
- consolidation fee exceeds the requested ceiling
- consolidation filters leave no eligible UTXOs

This keeps CLI and UI behavior predictable: the caller can fix the request and retry.

## Sync And Broadcast Mapping

`WalletSyncError` is mapped into API-level backend and broadcast variants.

Examples:

- transport failures become `BroadcastTransport`
- mempool conflicts become `BroadcastMempoolConflict`
- missing inputs become `BroadcastMissingInputs`
- insufficient relay fee becomes `BroadcastInsufficientFee`
- invalid backend configuration becomes `InvalidBackend`
- unavailable backend configuration becomes `BackendUnavailable`

Unexpected sync failures are collapsed into `Sync(String)`.

## RBF And CPFP Mapping

RBF errors are surfaced with transaction-specific variants:

- original transaction missing: `TransactionNotFound`
- transaction already confirmed: `TransactionAlreadyConfirmed`
- transaction not replaceable: `TransactionNotReplaceable`
- requested fee rate too low: `FeeRateTooLowForBump`
- replacement build failed: `FeeBumpBuildFailed`

CPFP build errors use `CpfpBuildFailed` unless they are better represented by another domain-specific variant.

## Caller Guidance

Callers should:

- display `InvalidInput` as a user-correctable request error
- display backend and broadcast errors as operational failures
- keep preview failures visible instead of silently changing the request
- avoid parsing error strings for control flow when a structured variant exists

The integration tests assert the important caller-visible cases, including invalid outpoint conversion to `WalletApiError::InvalidInput`.
