# Coin Selection

Coin selection in `wallet_core` is split between two layers:

- `psbt_coin_selector.rs` selects typed wallet outpoints from a BDK UTXO list for manual/consolidation policies.
- BDK's transaction builder performs final funding for normal automatic sends.

This is important: `wallet_core` does not currently implement a full amount-solving coin selector for every send. It uses typed selection helpers to pin, exclude, filter, and order inputs, then delegates transaction construction to BDK.

## Domain Types

Selection is represented by `WalletInputSelectionConfig`:

- `include_outpoints`
- `exclude_outpoints`
- `confirmed_only`
- `selection_mode`
- `max_input_count`
- `min_input_count`
- `min_utxo_value_sat`
- `max_utxo_value_sat`
- `strategy`

`WalletCoinControlInfo` wraps this config for send/send-max/sweep flows.

`WalletConsolidationInfo` wraps the same config and adds `max_fee_pct_of_input_value`.

## Effective Selection Mode

`common_selection::effective_selection_mode` resolves missing mode values as:

- explicit include set present: `StrictManual`
- no include set: `AutomaticOnly`

Manual auto-completion is opt-in through `ManualWithAutoCompletion`.

## Selection Modes

`StrictManual` means the selected set must come from explicit includes. If the set is empty, exceeds count limits, violates filters, or cannot fund the requested flow, the operation fails.

`ManualWithAutoCompletion` starts with explicitly included inputs and then adds eligible candidates.

`AutomaticOnly` is intended for automatic candidate selection. In the current selector implementation, explicit includes are still validated and seeded before automatic completion if the caller provides them, so callers should avoid sending include sets with `AutomaticOnly` when they expect includes to be ignored.

## Selector Algorithm

`psbt_coin_selector::select_inputs` performs these steps:

1. reject include/exclude overlap
2. build candidates by applying `confirmed_only`, excludes, and value filters
3. resolve included outpoints against the full wallet UTXO list
4. reject included outpoints that violate confirmation or value filters
5. remove already selected inputs from candidates
6. in `StrictManual`, validate count bounds and return selected includes
7. otherwise sort candidates by consolidation strategy
8. add candidates until `max_input_count` is reached or candidates are exhausted
9. validate min/max input count

The returned value is `Vec<WalletOutPoint>`.

## Strategy Ordering

Automatic completion and consolidation can sort candidates by:

- `SmallestFirst`: ascending value, then outpoint
- `LargestFirst`: descending value, then outpoint
- `OldestFirst`: confirmed block height first, unconfirmed last, then outpoint

If no strategy is provided, the selector uses `SmallestFirst`.

## Coin Control Validation

`psbt_coin_control.rs` validates explicit include/exclude intent against wallet UTXOs:

- include/exclude overlap fails
- included outpoint missing from wallet fails
- included unconfirmed outpoint fails when `confirmed_only` is set
- exclusions are returned as typed wallet outpoints

Strict manual send flows turn every non-selected wallet UTXO into an unspendable input for the BDK builder. That prevents silent input expansion.

## Flow Differences

Fixed send:

- amount is fixed
- strict manual pre-check estimates whether selected inputs cover amount plus fee
- non-strict flows let BDK finish funding with available spendable inputs

Send-max:

- uses `WalletSendAmountMode::Max`
- BDK drains the selected or eligible wallet value to the recipient
- final amount is derived from the built PSBT output

Sweep:

- modeled as send-max with explicit coin control
- normally uses strict manual selection through the default include-set behavior

Consolidation:

- selects multiple wallet UTXOs through `select_inputs`
- requires at least two selected inputs
- builds a wallet-internal drain transaction to an internal address
