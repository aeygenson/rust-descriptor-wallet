# Wallet API DTO Boundary

`wallet_api` deliberately exposes DTOs instead of lower-level domain types.

Callers pass wallet names, addresses, txids, outpoints, PSBT base64 strings, satoshi values, and fee-rate values. The API normalizes those into canonical request DTOs, converts those into typed `wallet_core` requests, and converts domain results back into stable response DTOs.

## Why The Boundary Exists

The boundary keeps CLI, tests, and the future desktop UI thin.

Callers should not need to know how to parse a Bitcoin outpoint, build a coin-control request, inspect a PSBT, or map low-level wallet errors. Those rules belong in one place: `wallet_api`.

The canonical request DTOs also give the crate one stable internal request shape per workflow. `WalletApi` convenience methods can stay ergonomic while service modules and tests use explicit request objects.

## Main Response DTOs

Wallet metadata and state:

- `WalletSummaryDto`
- `WalletDetailsDto`
- `WalletDescriptorsDto`
- `WalletBackendDto`
- `SyncBackendDto`
- `BroadcastBackendDto`
- `WalletStatusDto`
- `WalletReceiveAddressHistoryDto`
- `AddressBookEntryDto`
- `WalletLockedUtxoDto`
- `WalletLockedUtxosDto`
- `WalletTxDto`
- `WalletUtxoDto`

Transaction and PSBT results:

- `WalletPsbtDto`
- `WalletCpfpPsbtDto`
- `WalletSignedPsbtDto`
- `WalletReplacementDto`
- `WalletBroadcastCandidateDto`
- `WalletSelectionResultDto`
- `WalletBackendCapabilitiesDto`
- `TxBroadcastResultDto`

Request DTOs:

- `CreatePsbtRequestDto`
- `SendRequestDto`
- `SendMaxRequestDto`
- `SweepRequestDto`
- `ConsolidationRequestDto`
- `SignPsbtRequestDto`
- `PublishPsbtRequestDto`
- `BumpFeeRequestDto`
- `CpfpRequestDto`
- `WalletTransactionsRequestDto`
- `WalletUtxosRequestDto`
- `WalletAddressRequestDto`
- `WalletReceiveAddressesRequestDto`
- `LabelReceiveAddressRequestDto`
- `ClearReceiveAddressLabelRequestDto`
- `CreateAddressBookEntryRequestDto`
- `ListAddressBookEntriesRequestDto`
- `GetAddressBookEntryRequestDto`
- `DeleteAddressBookEntryRequestDto`
- `WalletLockUtxosRequestDto`
- `WalletUnlockUtxosRequestDto`
- `WalletLockedUtxosRequestDto`
- `ImportWalletRequestDto`
- `DeleteWalletRequestDto`
- `GetWalletRequestDto`
- `WalletCoinControlDto`
- `WalletConsolidationDto`

Enums:

- `WalletInputSelectionModeDto`
- `WalletConsolidationStrategyDto`

## Coin Control Conversion

`WalletCoinControlDto::try_into_core` converts caller strings into `WalletCoinControlInfo`.

It parses `include_outpoints` and `exclude_outpoints`, maps the selection mode, and carries the `confirmed_only` flag.

Malformed outpoints are converted to a core coin-control error, then mapped by `WalletApiError` into `InvalidInput`.

## Canonical Request DTOs

PSBT and transaction workflows now have one canonical request DTO each:

- `CreatePsbtRequestDto`
- `SendRequestDto`
- `SendMaxRequestDto`
- `SweepRequestDto`
- `ConsolidationRequestDto`
- `BumpFeeRequestDto`
- `CpfpRequestDto`
- `SignPsbtRequestDto`
- `PublishPsbtRequestDto`

Wallet-read and registry operations also use request DTOs:

- `WalletAddressRequestDto`
- `WalletTransactionsRequestDto`
- `WalletUtxosRequestDto`
- `ImportWalletRequestDto`
- `DeleteWalletRequestDto`
- `GetWalletRequestDto`

These DTOs are the canonical service boundary inside `wallet_api`. They reduce signature drift between `WalletApi`, service modules, integration tests, and future UI callers.

## Consolidation Conversion

`WalletConsolidationDto::try_into_core` converts consolidation filters into `WalletConsolidationInfo`.

It handles:

- explicit include and exclude outpoints
- confirmed-only filtering
- min and max input counts
- min and max UTXO value filters
- maximum fee percentage of input value
- selection strategy
- input selection mode

This keeps consolidation policy parsing outside the CLI and UI layers.

## Address And Inspection DTOs

`WalletReceiveAddressHistoryDto` returns:

- `address`
- `keychain`
- `index`
- `bitcoin_uri`
- `qr_svg`
- `label`
- `created_at`
- `updated_at`

`AddressBookEntryDto` returns:

- `wallet_name`
- `network`
- `label`
- `address`
- `notes`
- `created_at`
- `updated_at`

`WalletLockedUtxoDto` returns:

- `wallet_name`
- `outpoint`
- `reason`
- `locked_at`
- `updated_at`

`WalletUtxoDto` now also carries:

- `derivation_index`
- `is_locked`
- `lock_reason`
- `locked_at`

This lets callers render wallet-owned address metadata, persisted receive-history rows, and lock state without reaching into `wallet_core` or `wallet_storage`.

## Preview DTOs

`WalletPsbtDto` is the common preview response for most transaction builders. It includes:

- base64 PSBT payload
- txid and optional original txid
- optional replacement lineage
- destination address
- amount and fee in satoshis
- fee rate in sat/vB
- replaceability
- optional change amount
- selected input count and selected input outpoints
- transaction input, output, and recipient counts
- estimated virtual size

When replacement metadata is available, `WalletPsbtDto.replacement` contains:

- `replaced_txid`
- `replacement_txid`
- `replacement_depth`
- `replacement_chain`

`WalletCpfpPsbtDto` is CPFP-specific and includes the parent txid, selected child input outpoint, input value, child output value, fee, fee rate, replaceability, and estimated virtual size.

`WalletBroadcastCandidateDto` is the finalized-transaction analysis DTO used before broadcast. It carries the extracted tx hex together with optional fee, fee-rate, vsize, and mempool ancestry metadata.

## Error Boundary

DTO parsing is part of the API error boundary.

Examples:

- invalid outpoint strings become `WalletApiError::InvalidInput`
- invalid txids become transaction or input errors at the API layer
- invalid destination addresses become `WalletApiError::InvalidDestinationAddress`
- malformed PSBT base64 becomes a PSBT encoding or structure error
- attempts to spend a locked outpoint become `WalletApiError::LockedUtxo`

The regtest integration suite asserts this caller-visible behavior, including invalid coin-control outpoint handling, replacement metadata, and consolidation selection behavior.
