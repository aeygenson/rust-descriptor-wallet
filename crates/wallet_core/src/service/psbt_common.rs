use std::str::FromStr;

use bitcoin::psbt::Psbt;
use bitcoin::Transaction;

use bitcoin::{Sequence, Txid};

use crate::{WalletCoreError, WalletCoreResult};

/// Parse a PSBT from its encoded string representation.
pub fn parse_psbt(psbt_base64: &str) -> WalletCoreResult<Psbt> {
    Psbt::from_str(psbt_base64).map_err(|e| WalletCoreError::InvalidPsbt(e.to_string()))
}

/// Check if PSBT is finalized (ready for broadcast)
pub fn is_psbt_finalized(psbt: &Psbt) -> bool {
    psbt.inputs
        .iter()
        .all(|input| input.final_script_sig.is_some() || input.final_script_witness.is_some())
}

/// Extract finalized transaction from PSBT
pub fn extract_finalized_tx(psbt: &Psbt) -> WalletCoreResult<Transaction> {
    if !is_psbt_finalized(psbt) {
        return Err(WalletCoreError::PsbtNotFinalized);
    }

    psbt.clone().extract_tx().map_err(|e| {
        WalletCoreError::ExtractTxFailed(format!("failed to extract tx from PSBT: {}", e))
    })
}

/// Check if a transaction is explicitly RBF-enabled.
pub fn is_rbf_enabled(tx: &Transaction) -> bool {
    tx.input
        .iter()
        .any(|txin| txin.sequence.0 < Sequence::ENABLE_LOCKTIME_NO_RBF.0)
}

/// Parse a txid from string.
pub fn parse_txid(txid: &str) -> WalletCoreResult<Txid> {
    Txid::from_str(txid).map_err(|_| WalletCoreError::InvalidTxid(txid.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{absolute, transaction, Amount, OutPoint, ScriptBuf, TxIn, TxOut, Witness};

    const FINALIZED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGAQhCAUBQOwjdd/7aYgEH2ZHtHfwqt01+CB3A29cdWLeeXj+EejPrC6Y6pnpcto0TJA8BwCK1uMICqlUyEsXb+xY0dkYBAAEFIFU1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWAAEFILEKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoAA==";

    const UNSIGNED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGIRZVNVyoPJc/HZfODjhDyF14kFrxa03FMbxIjlchLSMBFhkAc8XaClYAAIABAACAAAAAgAAAAAAAAAAAARcgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYAAQUgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYhB1U1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWGQBzxdoKVgAAgAEAAIAAAACAAAAAAAAAAAAAAQUgsQrJf2ds8fPM2ssLeBcSgrvpSpTfFDIBcA3Fm8wV82ghB7EKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoGQBzxdoKVgAAgAEAAIAAAACAAQAAAAAAAAAA";

    fn build_tx_with_sequence(sequence: Sequence) -> Transaction {
        Transaction {
            version: transaction::Version(2),
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(1_000),
                script_pubkey: ScriptBuf::new(),
            }],
        }
    }

    #[test]
    fn parse_psbt_works() {
        let psbt = parse_psbt(FINALIZED_TEST_PSBT).unwrap();
        assert_eq!(psbt.inputs.len(), 1);
    }

    #[test]
    fn parse_psbt_fails_for_invalid_string() {
        let result = parse_psbt("not-a-psbt");

        assert!(matches!(result, Err(WalletCoreError::InvalidPsbt(_))));
    }

    #[test]
    fn finalized_detection() {
        let psbt = parse_psbt(FINALIZED_TEST_PSBT).unwrap();
        assert!(is_psbt_finalized(&psbt));

        let psbt2 = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        assert!(!is_psbt_finalized(&psbt2));
    }

    #[test]
    fn extract_finalized_tx_works() {
        let psbt = parse_psbt(FINALIZED_TEST_PSBT).unwrap();
        let tx = extract_finalized_tx(&psbt).unwrap();

        assert_eq!(
            tx.compute_txid().to_string(),
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
    }

    #[test]
    fn extract_fails_for_unsigned() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let result = extract_finalized_tx(&psbt);

        assert!(matches!(result, Err(WalletCoreError::PsbtNotFinalized)));
    }

    #[test]
    fn detects_rbf_enabled_transaction() {
        let tx = build_tx_with_sequence(Sequence(0xFFFFFFFD));
        assert!(is_rbf_enabled(&tx));
    }

    #[test]
    fn detects_non_rbf_transaction() {
        let tx = build_tx_with_sequence(Sequence::MAX);
        assert!(!is_rbf_enabled(&tx));
    }

    #[test]
    fn parse_txid_works_for_valid_txid() {
        let txid =
            parse_txid("d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d").unwrap();

        assert_eq!(
            txid.to_string(),
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
    }

    #[test]
    fn parse_txid_fails_for_invalid_string() {
        let result = parse_txid("not-a-txid");

        assert!(matches!(result, Err(WalletCoreError::InvalidTxid(_))));
    }
}
/// Parse an outpoint string in the form "txid:vout".
pub fn parse_outpoint(outpoint: &str) -> WalletCoreResult<(&str, u32)> {
    let (txid, vout) = outpoint
        .split_once(':')
        .ok_or_else(|| WalletCoreError::InvalidTxid(outpoint.to_string()))?;

    let vout = vout
        .parse::<u32>()
        .map_err(|_| WalletCoreError::InvalidTxid(outpoint.to_string()))?;

    Ok((txid, vout))
}

/// Parse and deduplicate a list of outpoint strings into `bitcoin::OutPoint`s.
pub fn parse_unique_outpoints(outpoints: &[String]) -> WalletCoreResult<Vec<bitcoin::OutPoint>> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut result = Vec::with_capacity(outpoints.len());

    for item in outpoints {
        let (txid_str, vout) = parse_outpoint(item)?;
        let txid = parse_txid(txid_str)?;
        let outpoint = bitcoin::OutPoint { txid, vout };

        if !seen.insert(outpoint) {
            return Err(WalletCoreError::CoinControlConflict(format!(
                "duplicate outpoint {} in input set",
                item
            )));
        }

        result.push(outpoint);
    }

    Ok(result)
}
