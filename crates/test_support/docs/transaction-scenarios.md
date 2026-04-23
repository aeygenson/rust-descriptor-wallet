# Transaction Scenarios

This document describes the wallet behaviors that the regtest integration suite is designed to validate.

The scenarios are implemented through production APIs, with `test_support` providing local chain setup, funding, mining, mempool inspection, and PSBT input decoding.

## Scenario Pattern

Most scenarios follow the same pattern:

1. Start or reuse regtest infrastructure.
2. Prepare a wallet through `wallet_api`.
3. Fund the wallet with miner-wallet RPC.
4. Mine blocks when confirmed funds are required.
5. Sync wallet state.
6. Create, sign, publish, or inspect a wallet transaction.
7. Assert preview, mempool, or post-sync state.

This pattern is deliberately explicit so failures are easier to diagnose.

## Current `regtest_flow.rs` Coverage

The current `crates/wallet_api/tests/regtest_flow.rs` suite covers these named groups:

- receive and sync
- self-send with change
- RBF replacement
- CPFP PSBT creation, requested parent outpoint selection, broadcast, confirmation, confirmed-parent rejection, and missing-parent rejection
- fixed-amount coin-control PSBT creation and one-shot send
- invalid outpoint rejection as `WalletApiError::InvalidInput`
- include/exclude conflict rejection
- insufficient selected-input rejection
- confirmed-only rejection for unconfirmed selected inputs
- send-max PSBT creation, one-shot send-max, no-change invariants, and fee-consumed-input rejection
- sweep PSBT creation, one-shot sweep, missing selected outpoint rejection, conflict rejection, confirmed-only rejection, and fee-consumed-input rejection
- consolidation PSBT creation, one-shot consolidation, missing selected outpoint rejection, conflict rejection, confirmed-only rejection, minimum input-count rejection, fee-consumed-input rejection, min/max UTXO value filters, fee percentage limit, smallest-first and largest-first strategies, recipient/change consistency, core invariants, and fuzz-style invariant preservation

## Receive and Sync

Purpose:
- prove that a wallet can derive an address
- fund that address on regtest
- sync the wallet
- observe balance and spendable UTXOs

Useful helpers:
- `RegtestEnv::fund_sats`
- `RegtestEnv::mine`
- `parse_regtest_address`

Expected assertions:
- balance increases after sync
- UTXO count reflects funding
- block height is observed after sync

## Fixed-Amount Send

Purpose:
- validate normal payment construction
- verify fee and change behavior
- confirm broadcast and follow-up sync

Expected assertions:
- PSBT creation succeeds
- selected inputs cover amount and fee
- fee and vsize are populated
- transaction appears in mempool after publish
- wallet state changes after confirmation and sync

## Coin Control: Strict Manual

Purpose:
- prove exact selected-input behavior
- reject insufficient manual selections
- prevent silent auto-completion

Useful helper:
- `decode_psbt_inputs`

Expected assertions:
- final PSBT inputs match included outpoints
- extra inputs are not added
- insufficient selected inputs return an API error

## Coin Control: Manual With Auto-Completion

Purpose:
- prove selected inputs are pinned
- allow backend completion when selected value is insufficient

Expected assertions:
- included outpoints are present in final inputs
- extra inputs may be present when needed
- preview data reports final selected input count

## Coin Control: Automatic Only

Purpose:
- validate backend-owned input selection
- keep simple send paths free from manual input requirements

Expected assertions:
- transaction is valid without includes
- excluded outpoints are not selected
- selected inputs are surfaced in preview output

## Include and Exclude Conflicts

Purpose:
- reject impossible or contradictory coin-control policies

Typical cases:
- same outpoint is included and excluded
- requested outpoint does not exist in the wallet
- confirmed-only removes included unconfirmed inputs
- exclusions remove all viable candidates
- invalid outpoint strings are rejected at the API boundary

Expected assertions:
- errors are returned, not panics
- invalid outpoint format maps to `WalletApiError::InvalidInput`
- error messages identify invalid policy or input data clearly enough for caller handling

## Send-Max

Purpose:
- validate max-spend amount calculation after fees
- prove the amount is backend-derived

Expected assertions:
- recipient amount is computed by the backend
- fee is subtracted from available selected value
- input policy still applies
- preview exposes selected inputs and output count
- strict selected-input send-max has no change output
- recipient amount plus fee does not exceed available selected value
- inputs that cannot pay the fee are rejected

## Sweep

Purpose:
- drain explicitly selected UTXOs to a destination
- preserve max-style selected-input semantics

Expected assertions:
- included outpoints drive the spend
- strict-manual sweep does not add extra wallet inputs
- no-change expectations are visible in preview when applicable
- one-shot sweep publishes successfully when signed and funded
- missing selected outpoints fail
- selected inputs that cannot cover fees fail

## Consolidation

Purpose:
- validate wallet-internal UTXO maintenance
- merge eligible inputs into a wallet-owned output

Expected assertions:
- selected inputs satisfy include, exclude, count, value-range, fee-ceiling, and strategy constraints
- output remains wallet-internal
- consolidation is not modeled as an external recipient payment
- one-shot consolidation publishes when policy permits
- `min-input-count` is enforced
- `min-utxo-value-sat` and `max-utxo-value-sat` are enforced
- `max-fee-pct` is enforced
- `smallest-first` and `largest-first` choose the expected candidates
- recipient count, output count, and change amount remain consistent
- core invariants hold across deterministic and fuzz-style cases

## RBF Replacement

Purpose:
- replace an unconfirmed replaceable transaction with a higher-fee transaction

Useful helpers:
- `mempool_contains`
- `parse_txid`

Expected assertions:
- original transaction is visible before replacement
- replacement transaction is created with the requested higher fee rate
- replacement is visible in the mempool after publish
- original transaction is no longer visible after successful replacement
- post-confirmation sync reflects the replacement

## CPFP

Purpose:
- accelerate an unconfirmed parent by spending a wallet-owned parent output in a child transaction

Useful helpers:
- `mempool_contains`
- `outpoint_txid`

Expected assertions:
- parent transaction is visible in the mempool
- selected parent outpoint is wallet-spendable
- child PSBT references the selected outpoint
- child transaction publishes successfully
- both parent and child leave the mempool after confirmation

## Multi-Input Behavior

Purpose:
- validate fee and change behavior when several inputs are needed or intentionally selected

Expected assertions:
- input count matches policy
- selected inputs are all surfaced in preview
- change and output count remain consistent
- fees scale with transaction size

## Confirmed-Only Behavior

Purpose:
- prevent accidental spending of unconfirmed UTXOs when the caller requests confirmed-only selection

Expected assertions:
- unconfirmed candidates are excluded
- transactions fail when confirmed-only leaves insufficient value
- confirmed inputs work after mining and sync

## Scenario Quality Rules

Good scenarios should:
- use real funding and mining
- sync before asserting wallet state
- assert final selected inputs, not just request values
- check mempool behavior for publish, RBF, and CPFP flows
- keep setup explicit instead of hiding chain transitions

Scenarios should avoid:
- relying on external services
- depending on test execution order
- assuming automatic input selection is stable unless the test explicitly constrains it
- duplicating wallet-core logic in assertions

## Summary

The regtest scenario suite validates real wallet behavior across funding, sync, fixed sends, coin control, send-max, sweep, consolidation, RBF, and CPFP.

`test_support` keeps the setup and inspection code reusable while production crates remain responsible for wallet behavior.
