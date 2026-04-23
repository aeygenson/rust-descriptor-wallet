# Invariants

This document records the core correctness rules enforced by the current `wallet_core` implementation.

## Typed Values

Domain code should use typed wrappers instead of raw scalar values for wallet-critical data:

- `AmountSat`
- `FeeRateSatPerVb`
- `WalletTxid`
- `WalletOutPoint`
- `PsbtBase64`
- `TxHex`
- `VSize`
- `BlockHeight`
- `Percent`
- `WalletKeychain`
- `TxDirection`

The wrappers reduce accidental unit and identifier mixups.

## Amount And Fee Rate

New validated amounts must be positive.

New validated fee rates must be positive.

PSBT builders also reject zero fee rates defensively because some internal/test paths can construct raw wrapper values with `From<u64>`.

## Destination Network

PSBT creation parses the destination address and requires it to match the wallet network.

Invalid address strings return `InvalidDestinationAddress`.

Wrong-network addresses return `DestinationNetworkMismatch`.

## Value Conservation

Built transactions must satisfy:

```text
sum(inputs) = sum(outputs) + fee
```

The core computes final fee from PSBT inputs and outputs and fails with `FeeCalculationFailed` if outputs exceed inputs.

## Coin Control

Include/exclude overlap must fail.

Included outpoints must exist in the wallet.

Included outpoints must be confirmed when `confirmed_only` is set.

Strict manual mode must not silently add wallet inputs. Non-selected wallet UTXOs are marked unspendable before calling the BDK builder.

## Send-Max And Sweep

Send-max and sweep must produce a non-zero recipient amount after fees.

Sweep is represented as send-max plus coin control. With the default include-set behavior, it uses strict manual selection and drains only the selected inputs.

## Consolidation

Consolidation must:

- use a positive fee rate
- select at least two inputs
- respect explicit include/exclude settings
- respect confirmed-only filtering
- respect min/max input count
- respect min/max UTXO value filters
- reject selected input value that cannot cover estimated fee
- reject fees above `max_fee_pct_of_input_value`
- send the resulting output to an internal wallet address

## RBF

RBF must:

- locate the original transaction in wallet state
- reject confirmed transactions
- reject non-RBF transactions
- require requested fee rate to be strictly greater than original fee rate
- preserve `original_txid` in the returned PSBT info

## CPFP

CPFP must:

- receive a non-empty parent txid
- use a positive fee rate
- reject watch-only wallets in the current implementation
- spend an unconfirmed wallet UTXO matching the selected parent outpoint
- keep child fee below selected input value
- build a one-input child PSBT to an internal wallet address

## Signing And Finalization

Signing must reject watch-only wallets.

Finalization for broadcast must reject unfinalized PSBTs.

`wallet_core` only extracts the finalized transaction. It does not broadcast it.

## No Silent Success

When the requested policy cannot be honored, core should return a specific `WalletCoreError` instead of silently changing intent.
