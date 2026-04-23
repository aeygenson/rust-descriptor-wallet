# PSBT Flows

`wallet_core` builds and manipulates PSBTs through `WalletService`.

The core handles creation, signing, and final transaction extraction. Actual broadcast is outside `wallet_core` and is performed by `wallet_sync` through `wallet_api`.

## Creation Methods

`psbt_create.rs` provides:

- `create_psbt`
- `create_psbt_with_coin_control`
- `create_send_max_psbt`
- `create_send_max_psbt_with_coin_control`
- `create_sweep_psbt`
- `create_sweep_psbt_with_optional_coin_control`

All creation methods validate destination network, fee rate, and amount semantics before calling BDK's transaction builder.

## Fixed Amount

`create_psbt` and `create_psbt_with_coin_control` use `WalletSendAmountMode::Fixed`.

Behavior:

- add one recipient output for the requested amount
- apply the requested fee rate
- optionally enable RBF by setting the exact RBF sequence
- optionally pin selected inputs and mark excluded inputs unspendable
- return `WalletPsbtInfo`

`WalletPsbtInfo` includes the PSBT base64, txid, amount, fee, fee rate, replaceability, change amount, selected inputs, counts, and estimated vsize.

## Send-Max

`create_send_max_psbt` and `create_send_max_psbt_with_coin_control` use `WalletSendAmountMode::Max`.

Behavior:

- call BDK `drain_wallet`
- drain to the recipient script
- compute the actual recipient amount from the built PSBT output
- fail with `SendMaxAmountTooSmall` when the resulting recipient amount is zero

## Sweep

Sweep is represented as send-max plus coin control.

`create_sweep_psbt` passes `WalletSendAmountMode::Max` with explicit `WalletCoinControlInfo`. Because include sets default to `StrictManual`, sweep normally drains only the selected outpoints and rejects silent input expansion.

## Consolidation

`psbt_consolidation.rs` provides `create_consolidation_psbt`.

Behavior:

- default policy is confirmed-only plus `SmallestFirst`
- select candidates through `select_inputs`
- require at least two selected inputs
- enforce optional min/max input count and UTXO value filters
- estimate fee before building
- optionally enforce `max_fee_pct_of_input_value`
- drain selected inputs to the next internal wallet address
- return `WalletPsbtInfo`

The consolidation output is wallet-internal and is reported as both `amount_sat` and `change_amount_sat`.

## RBF

`psbt_rbf.rs` provides `bump_fee_psbt`.

Behavior:

- parse and locate the original txid
- require the original transaction to exist in wallet state
- reject confirmed transactions
- require the original transaction to be RBF-enabled
- estimate the original fee rate
- require the requested fee rate to be strictly higher
- call BDK's fee-bump builder
- return a minimal `WalletPsbtInfo` with `original_txid` populated

Because BDK builds the replacement PSBT, some preview fields are conservative placeholders in the minimal conversion path.

## CPFP

`psbt_cpfp.rs` provides `create_cpfp_psbt`.

Behavior:

- require non-empty parent txid
- require positive fee rate
- reject watch-only wallets because the current CPFP path expects a signing-capable wallet
- find an unconfirmed wallet UTXO matching the selected parent outpoint
- estimate a one-input, one-output child transaction size
- fail when the fee consumes the selected input value
- build a child PSBT manually selecting only the parent output
- send the child output to an internal wallet address
- return `WalletCpfpPsbtInfo`

## Signing

`psbt_sign.rs` provides `sign_psbt`.

Behavior:

- parse `PsbtBase64`
- reject watch-only wallets
- sign with the wallet's configured signers
- return `WalletSignedPsbtInfo`

`WalletSignedPsbtInfo::signing_status` classifies the result as `unchanged`, `partially_signed`, or `finalized`.

## Finalization For Broadcast

`psbt_publish.rs` provides `finalize_psbt_for_broadcast`.

Behavior:

- parse `PsbtBase64`
- require finalized inputs
- extract the final transaction
- return `WalletFinalizedTxInfo` with txid, raw transaction hex, and replaceability

Despite the module name, this method does not broadcast. It prepares the finalized transaction for a broadcast backend.
