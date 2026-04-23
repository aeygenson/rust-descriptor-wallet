# Wallet API DTO Boundary

`wallet_api` deliberately exposes DTOs instead of lower-level domain types.

Callers pass wallet names, addresses, txids, outpoints, PSBT base64 strings, satoshi values, and fee-rate values. The API converts those into typed `wallet_core` requests and converts domain results back into stable response DTOs.

## Why The Boundary Exists

The boundary keeps CLI, tests, and the future desktop UI thin.

Callers should not need to know how to parse a Bitcoin outpoint, build a coin-control request, inspect a PSBT, or map low-level wallet errors. Those rules belong in one place: `wallet_api`.

## Main Response DTOs

Wallet metadata and state:

- `WalletSummaryDto`
- `WalletDetailsDto`
- `WalletDescriptorsDto`
- `WalletBackendDto`
- `SyncBackendDto`
- `BroadcastBackendDto`
- `WalletStatusDto`
- `WalletTxDto`
- `WalletUtxoDto`

Transaction and PSBT results:

- `WalletPsbtDto`
- `WalletCpfpPsbtDto`
- `WalletSignedPsbtDto`
- `TxBroadcastResultDto`

Request DTOs:

- `ImportWalletDto`
- `WalletCoinControlDto`
- `WalletConsolidationDto`

Enums:

- `WalletInputSelectionModeDto`
- `WalletConsolidationStrategyDto`

## Coin Control Conversion

`WalletCoinControlDto::try_into_core` converts caller strings into a `wallet_core::transaction::WalletCoinControl`.

It parses `include_outpoints` and `exclude_outpoints`, maps the selection mode, and carries the `confirmed_only` flag.

Malformed outpoints are converted to a core coin-control error, then mapped by `WalletApiError` into `InvalidInput`.

## Consolidation Conversion

`WalletConsolidationDto::try_into_core` converts consolidation filters into a `wallet_core::transaction::WalletConsolidationPolicy`.

It handles:

- explicit include and exclude outpoints
- confirmed-only filtering
- min and max input counts
- min and max UTXO value filters
- maximum fee percentage of input value
- selection strategy
- input selection mode

This keeps consolidation policy parsing outside the CLI and UI layers.

## Preview DTOs

`WalletPsbtDto` is the common preview response for most transaction builders. It includes:

- base64 PSBT payload
- txid and optional original txid
- destination address
- amount and fee in satoshis
- fee rate in sat/vB
- replaceability
- optional change amount
- selected input count and selected input outpoints
- transaction input, output, and recipient counts
- estimated virtual size

`WalletCpfpPsbtDto` is CPFP-specific and includes the parent txid, selected child input outpoint, input value, child output value, fee, fee rate, replaceability, and estimated virtual size.

## Error Boundary

DTO parsing is part of the API error boundary.

Examples:

- invalid outpoint strings become `WalletApiError::InvalidInput`
- invalid txids become transaction or input errors at the API layer
- invalid destination addresses become `WalletApiError::InvalidDestinationAddress`
- malformed PSBT base64 becomes a PSBT encoding or structure error

Integration tests in `crates/wallet_api/tests/regtest_flow.rs` assert this caller-visible behavior, including invalid coin-control outpoint handling.
