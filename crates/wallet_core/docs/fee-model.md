# Fee Model

`wallet_core` uses `FeeRateSatPerVb` as the domain fee-rate type and converts it to BDK's `FeeRate` at transaction-builder boundaries.

## Fee Rate Validation

`FeeRateSatPerVb::new` rejects zero.

Some tests and internal paths can still construct `FeeRateSatPerVb::from(0)`, so PSBT builders defensively reject zero fee rates before building.

## Fixed Sends

Fixed sends add a recipient output for the requested amount and pass the fee rate to BDK.

For strict manual coin control, the core performs a conservative pre-check:

```text
estimated_vsize = 11 + input_count * 58 + output_count * 43
fee_estimate = estimated_vsize * fee_rate
required = amount + fee_estimate
```

If explicitly selected inputs cannot cover the amount, the error is `CoinControlInsufficientSelectedFunds`.

If they cover the amount but not the conservative amount-plus-fee estimate, the error is `CoinControlStrictModeViolation`.

Final fee is computed from the finished PSBT.

## Send-Max And Sweep

Send-max and sweep use `WalletSendAmountMode::Max`.

The builder drains selected or eligible wallet value to the recipient. The final recipient amount is derived from PSBT outputs matching the recipient script.

If that derived amount is zero, the core returns `SendMaxAmountTooSmall`.

For strict manual send-max or sweep, the conservative pre-check requires:

```text
selected_total > fee_estimate
```

## Consolidation

Consolidation estimates fee before building:

```text
estimated_vsize = 11 + input_count * 58 + 1 * 43
fee_estimate = estimated_vsize * fee_rate
```

It rejects:

- zero fee rate
- fewer than two selected inputs
- selected total less than or equal to estimated fee
- fee estimate above `max_fee_pct_of_input_value`

After BDK builds the PSBT, the final fee is recomputed from actual PSBT inputs and outputs.

## RBF

RBF uses the original transaction's effective fee rate when available:

- compute original fee rate from wallet transaction data
- require requested fee rate to be strictly higher
- pass requested fee rate to BDK's fee-bump builder

Equal or lower fee rates return `FeeRateTooLowForBump`.

## CPFP

CPFP uses a fixed child-size estimate:

```text
vsize = 58 + 43 + 11
fee = fee_rate * vsize
child_output = selected_input_value - fee
```

If the fee consumes the selected input value, the error is `CpfpInsufficientValue`.

## Final Fee Reporting

For normal PSBT builds, fee is reported as:

```text
fee = sum(psbt input witness_utxo values) - sum(unsigned tx output values)
```

If that subtraction fails, the core returns `FeeCalculationFailed`.
