use bitcoin::{OutPoint, Txid};

use crate::types::WalletOutPoint;
use crate::{WalletCoreError, WalletCoreResult};

/// Parse a txid from string.
pub fn parse_txid(txid: &str) -> WalletCoreResult<Txid> {
    txid.parse::<Txid>()
        .map_err(|_| WalletCoreError::InvalidTxid(txid.to_string()))
}

/// Parse an outpoint string in the form `txid:vout`.
pub fn parse_outpoint(outpoint: &str) -> WalletCoreResult<(&str, u32)> {
    let (txid, vout) = outpoint
        .split_once(':')
        .ok_or_else(|| WalletCoreError::InvalidOutpoint(outpoint.to_string()))?;

    let vout = vout
        .parse::<u32>()
        .map_err(|_| WalletCoreError::InvalidOutpoint(outpoint.to_string()))?;

    Ok((txid, vout))
}

/// Parse an outpoint string in the form `txid:vout` directly into `OutPoint`.
pub fn parse_bitcoin_outpoint(outpoint: &str) -> WalletCoreResult<OutPoint> {
    let (txid_str, vout) = parse_outpoint(outpoint)?;
    let txid = parse_txid(txid_str)?;
    Ok(OutPoint { txid, vout })
}

/// Parse an outpoint string in the form `txid:vout` directly into `WalletOutPoint`.
pub fn parse_wallet_outpoint(outpoint: &str) -> WalletCoreResult<WalletOutPoint> {
    parse_bitcoin_outpoint(outpoint).map(WalletOutPoint::from)
}

/// Parse an optional list of unique outpoints. Returns an empty vector when the
/// input list is empty.
pub fn parse_optional_unique_outpoints(
    outpoints: &[String],
) -> WalletCoreResult<Vec<WalletOutPoint>> {
    if outpoints.is_empty() {
        Ok(Vec::new())
    } else {
        parse_unique_outpoints(outpoints)
    }
}

/// Parse and deduplicate a list of outpoint strings into `WalletOutPoint`s.
pub fn parse_unique_outpoints(outpoints: &[String]) -> WalletCoreResult<Vec<WalletOutPoint>> {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    let mut result = Vec::with_capacity(outpoints.len());

    for item in outpoints {
        let outpoint = parse_wallet_outpoint(item)?;

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

/// Ensure there is no overlap between included and excluded outpoints.
/// Returns an error if the same outpoint appears in both sets.
pub fn ensure_no_outpoint_overlap(
    included: &[WalletOutPoint],
    excluded: &[WalletOutPoint],
) -> WalletCoreResult<()> {
    use std::collections::HashSet;

    let excluded_set: HashSet<_> = excluded.iter().collect();

    for outpoint in included {
        if excluded_set.contains(outpoint) {
            return Err(WalletCoreError::CoinControlConflict(format!(
                "outpoint {} present in both include and exclude sets",
                outpoint
            )));
        }
    }

    Ok(())
}

/// Return the transaction id portion of a strongly-typed wallet outpoint.
pub fn outpoint_txid(outpoint: &WalletOutPoint) -> crate::types::WalletTxid {
    crate::types::WalletTxid::from(outpoint.as_ref().txid)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_outpoint_works_for_valid_input() {
        let (txid, vout) =
            parse_outpoint("d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:2")
                .unwrap();

        assert_eq!(
            txid,
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
        assert_eq!(vout, 2);
    }

    #[test]
    fn parse_outpoint_fails_for_missing_separator() {
        let result = parse_outpoint("not-an-outpoint");
        assert!(matches!(result, Err(WalletCoreError::InvalidOutpoint(_))));
    }

    #[test]
    fn parse_bitcoin_outpoint_works_for_valid_input() {
        let outpoint = parse_bitcoin_outpoint(
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:2",
        )
        .unwrap();

        assert_eq!(
            outpoint.txid.to_string(),
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
        assert_eq!(outpoint.vout, 2);
    }

    #[test]
    fn parse_wallet_outpoint_works_for_valid_input() {
        let outpoint = parse_wallet_outpoint(
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:2",
        )
        .unwrap();

        assert_eq!(
            outpoint.as_ref().txid.to_string(),
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
        assert_eq!(outpoint.as_ref().vout, 2);
    }

    #[test]
    fn parse_bitcoin_outpoint_fails_for_invalid_txid() {
        let result = parse_bitcoin_outpoint(
            "not-a-real-txid000000000000000000000000000000000000000000000000000000:0",
        );
        assert!(matches!(result, Err(WalletCoreError::InvalidTxid(_))));
    }

    #[test]
    fn parse_optional_unique_outpoints_returns_empty_for_empty_input() {
        let parsed = parse_optional_unique_outpoints(&[]).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_unique_outpoints_deduplicates_and_fails_on_duplicates() {
        let result = parse_unique_outpoints(&[
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:0".to_string(),
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:0".to_string(),
        ]);

        assert!(matches!(
            result,
            Err(WalletCoreError::CoinControlConflict(_))
        ));
    }

    #[test]
    fn ensure_no_outpoint_overlap_detects_conflict() {
        let parsed = parse_unique_outpoints(&[
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:0".to_string(),
        ])
        .unwrap();

        let result = ensure_no_outpoint_overlap(&parsed, &parsed);

        assert!(matches!(
            result,
            Err(WalletCoreError::CoinControlConflict(_))
        ));
    }

    #[test]
    fn ensure_no_outpoint_overlap_allows_disjoint_sets() {
        let included = parse_unique_outpoints(&[
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d:0".to_string(),
        ])
        .unwrap();

        let excluded = parse_unique_outpoints(&[
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:1".to_string(),
        ])
        .unwrap();

        let result = ensure_no_outpoint_overlap(&included, &excluded);

        assert!(result.is_ok());
    }

    #[test]
    fn outpoint_txid_extracts_txid_prefix() {
        let outpoint = WalletOutPoint::parse(
            "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee:1",
        )
        .unwrap();
        let txid = outpoint_txid(&outpoint);
        assert_eq!(
            txid,
            crate::types::WalletTxid::parse(
                "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee"
            )
            .unwrap()
        );
    }
}
