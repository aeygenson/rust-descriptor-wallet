use std::str::FromStr;

use bitcoin::psbt::Psbt;
use bitcoin::Transaction;

use crate::{WalletCoreError, WalletCoreResult};

/// Parse PSBT from base64 string
pub fn parse_psbt(psbt_base64: &str) -> WalletCoreResult<Psbt> {
    Psbt::from_str(psbt_base64)
        .map_err(|e| WalletCoreError::InvalidPsbt(e.to_string()))
}

/// Check if PSBT is finalized (ready for broadcast)
pub fn is_psbt_finalized(psbt: &Psbt) -> bool {
    psbt.inputs.iter().all(|input| {
        input.final_script_sig.is_some() || input.final_script_witness.is_some()
    })
}

/// Extract finalized transaction from PSBT
pub fn extract_finalized_tx(psbt: &Psbt) -> WalletCoreResult<Transaction> {
    if !is_psbt_finalized(psbt) {
        return Err(WalletCoreError::PsbtNotFinalized);
    }

    psbt.clone().extract_tx().map_err(|e| {
        WalletCoreError::ExtractTxFailed(e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FINALIZED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGAQhCAUBQOwjdd/7aYgEH2ZHtHfwqt01+CB3A29cdWLeeXj+EejPrC6Y6pnpcto0TJA8BwCK1uMICqlUyEsXb+xY0dkYBAAEFIFU1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWAAEFILEKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoAA==";

    const UNSIGNED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGIRZVNVyoPJc/HZfODjhDyF14kFrxa03FMbxIjlchLSMBFhkAc8XaClYAAIABAACAAAAAgAAAAAAAAAAAARcgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYAAQUgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYhB1U1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWGQBzxdoKVgAAgAEAAIAAAACAAAAAAAAAAAAAAQUgsQrJf2ds8fPM2ssLeBcSgrvpSpTfFDIBcA3Fm8wV82ghB7EKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoGQBzxdoKVgAAgAEAAIAAAACAAQAAAAAAAAAA";

    #[test]
    fn parse_psbt_works() {
        let psbt = parse_psbt(FINALIZED_TEST_PSBT).unwrap();
        assert_eq!(psbt.inputs.len(), 1);
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

        assert!(matches!(
            result,
            Err(WalletCoreError::PsbtNotFinalized)
        ));
    }
}