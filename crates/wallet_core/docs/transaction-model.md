# Transaction Model

`wallet_core` models wallet-visible transaction data and PSBT build results with typed domain structs in `model.rs`.

## Wallet Transactions

`WalletTxInfo` is the wallet transaction-history model:

- `txid: WalletTxid`
- `confirmed: bool`
- `confirmation_height: Option<BlockHeight>`
- `direction: TxDirection`
- `replaceable: bool`
- `net_value: i64`
- `fee: Option<AmountSat>`
- `fee_rate_sat_per_vb: Option<FeeRateSatPerVb>`

`txs.rs` converts BDK wallet transaction data into this model. Direction is classified from sent, received, and net wallet value.

## Wallet UTXOs

`WalletUtxoInfo` is the spendable-output model:

- `outpoint: WalletOutPoint`
- `value: AmountSat`
- `confirmed: bool`
- `confirmation_height: Option<BlockHeight>`
- `address: Option<String>`
- `keychain: WalletKeychain`

`utxos.rs` converts BDK local outputs into this model and also supports filtering unconfirmed UTXOs by txid for CPFP.

## PSBT Preview

`WalletPsbtInfo` is the common PSBT preview/build result:

- `psbt_base64`
- `txid`
- `original_txid`
- `to_address`
- `amount_sat`
- `fee_sat`
- `fee_rate_sat_per_vb`
- `replaceable`
- `change_amount_sat`
- `selected_utxo_count`
- `selected_inputs`
- `input_count`
- `output_count`
- `recipient_count`
- `estimated_vsize`

Most builders populate all fields from the actual built PSBT. RBF currently uses `from_psbt_minimal`, which preserves PSBT payload, txid, selected inputs, counts, RBF flag, and vsize while using conservative placeholders for UI-oriented payment metadata.

## Signing And Finalization

`WalletSignedPsbtInfo` reports:

- signed PSBT base64
- whether signing modified the PSBT
- whether it finalized the PSBT
- txid

`PsbtSigningStatus` derives `unchanged`, `partially_signed`, or `finalized` from those flags.

`WalletFinalizedTxInfo` reports the broadcast-ready final transaction:

- txid
- raw transaction hex
- replaceability

Broadcast itself is not part of `wallet_core`.

## CPFP Models

`WalletCpfpBuildPlanInfo` represents the internal CPFP child plan:

- selected input outpoint
- input value
- child output value
- fee
- estimated vsize

`WalletCpfpPsbtInfo` is the CPFP PSBT result:

- PSBT base64
- child txid
- parent txid
- selected outpoint
- input value
- child output value
- fee
- fee rate
- replaceability
- estimated vsize

## Selection Models

`WalletInputSelectionConfig` is the shared selection configuration for coin control and consolidation.

`WalletCoinControlInfo` wraps selection config for send/send-max/sweep.

`WalletConsolidationInfo` wraps selection config for consolidation and adds an optional maximum fee percentage of selected input value.

`WalletSendAmountMode` distinguishes fixed-amount sends from send-max/sweep behavior:

- `Fixed(AmountSat)`
- `Max`

## Value Conservation

For every built transaction:

```text
sum(inputs) = sum(outputs) + fee
```

The code computes fee from PSBT input witness UTXOs and unsigned transaction outputs after BDK builds the PSBT. If output value exceeds input value, fee calculation fails.
