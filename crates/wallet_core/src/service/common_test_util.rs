// crates/wallet_core/src/service/common_test

#![allow(dead_code)]

#[cfg(test)]
pub(crate) mod test_support {
    use crate::config::{
        BroadcastBackendConfig, SyncBackendConfig, WalletBackendConfig, WalletDescriptors,
    };
    use bdk_wallet::LocalOutput;
    use bitcoin::{Amount, BlockHash, Network, OutPoint, ScriptBuf, TxOut};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::model::{
        WalletConsolidationInfo, WalletInputSelectionConfig, WalletInputSelectionMode,
    };
    use crate::types::WalletOutPoint;
    use crate::{WalletConfig, WalletService};

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Create a fully initialized test wallet + config.
    pub fn load_test_wallet() -> (WalletConfig, WalletService) {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");
        (config, wallet)
    }

    /// Load a wallet from an arbitrary test config.
    pub fn load_wallet(config: &WalletConfig) -> WalletService {
        WalletService::load_or_create(config).expect("wallet should load or create successfully")
    }

    /// Valid Signet address used in tests.
    pub fn valid_signet_address() -> &'static str {
        "tb1pckmj4jv3z4399h0se8stn0f5c39eq6266hv296w00ysds0gkc79srg7udu"
    }

    /// Shared unsigned PSBT fixture used across signing tests.
    pub const UNSIGNED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGIRZVNVyoPJc/HZfODjhDyF14kFrxa03FMbxIjlchLSMBFhkAc8XaClYAAIABAACAAAAAgAAAAAAAAAAAARcgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYAAQUgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYhB1U1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWGQBzxdoKVgAAgAEAAIAAAACAAAAAAAAAAAAAAQUgsQrJf2ds8fPM2ssLeBcSgrvpSpTfFDIBcA3Fm8wV82ghB7EKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoGQBzxdoKVgAAgAEAAIAAAACAAQAAAAAAAAAA";

    /// Shared finalized PSBT fixture used across publish/broadcast tests.
    pub const FINALIZED_TEST_PSBT: &str = "cHNidP8BAHECAAAAAZ7///////////////////////////////8AAAAAAAAA/////wIAAAAAAAAAIgAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAEBH3///////////////////////////////wAAAAAAAAAA/////wEAAAAAAAAAABYAFJQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEHAgAAAAAAACIAIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAQEf///////////////////////////////8AAAAAAAAAAP////8BAAAAAAAAAAAWABSUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    /// Build a sample local wallet output for selector/unit tests.
    pub fn sample_local_output(value_sat: u64, vout: u32, confirmed: bool) -> LocalOutput {
        LocalOutput {
            outpoint: OutPoint::new(
                "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
                    .parse()
                    .unwrap(),
                vout,
            ),
            txout: TxOut {
                value: Amount::from_sat(value_sat),
                script_pubkey: ScriptBuf::new(),
            },
            keychain: bdk_wallet::KeychainKind::External,
            is_spent: false,
            derivation_index: 0,
            chain_position: if confirmed {
                bdk_chain::ChainPosition::Confirmed {
                    anchor: bdk_chain::ConfirmationBlockTime {
                        block_id: bdk_chain::BlockId {
                            height: 100,
                            hash:
                                "0000000000000000000000000000000000000000000000000000000000000000"
                                    .parse::<BlockHash>()
                                    .unwrap(),
                        },
                        confirmation_time: 0,
                    },
                    transitively: None,
                }
            } else {
                bdk_chain::ChainPosition::Unconfirmed {
                    last_seen: None,
                    first_seen: None,
                }
            },
        }
    }

    /// Build a default selector config for unit tests.
    pub fn default_selection_config() -> WalletInputSelectionConfig {
        WalletInputSelectionConfig {
            include_outpoints: vec![],
            exclude_outpoints: vec![],
            confirmed_only: false,
            selection_mode: Some(WalletInputSelectionMode::AutomaticOnly),
            max_input_count: None,
            min_input_count: None,
            min_utxo_value_sat: None,
            max_utxo_value_sat: None,
            strategy: None,
        }
    }

    /// Build a standard watch-only Signet test wallet config.
    pub fn test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            descriptors: WalletDescriptors {
                external: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
                internal: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            },
            backend: WalletBackendConfig {
                sync: SyncBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                },
                broadcast: Some(BroadcastBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                }),
            },
            db_path: unique_test_db_path("wallet_core_psbt"),
            is_watch_only: true,
        }
    }

    /// Build the standard watch-only Signet test config but with a caller-supplied
    /// unique DB prefix.
    pub fn test_config_with_db_prefix(prefix: &str) -> WalletConfig {
        let mut config = test_config();
        config.db_path = unique_test_db_path(prefix);
        config
    }

    /// Load the standard watch-only test wallet using a caller-supplied unique DB prefix.
    pub fn load_test_wallet_with_db_prefix(prefix: &str) -> (WalletConfig, WalletService) {
        let config = test_config_with_db_prefix(prefix);
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");
        (config, wallet)
    }

    /// Build a standard signing Signet test wallet config with private descriptors.
    pub fn signing_test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            descriptors: WalletDescriptors {
                external: "tr([73c5da0a/86'/1'/0']tprv8gytrHbFLhE7zLJ6BvZWEDDGJe8aS8VrmFnvqpMv8CEZtUbn2NY5KoRKQNpkcL1yniyCBRi7dAPy4kUxHkcSvd9jzLmLMEG96TPwant2jbX/0/*)#ps8nx7gn".to_string(),
                internal: "tr([73c5da0a/86'/1'/0']tprv8gytrHbFLhE7zLJ6BvZWEDDGJe8aS8VrmFnvqpMv8CEZtUbn2NY5KoRKQNpkcL1yniyCBRi7dAPy4kUxHkcSvd9jzLmLMEG96TPwant2jbX/1/*)#syzjmtct".to_string(),
            },
            backend: WalletBackendConfig {
                sync: SyncBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                },
                broadcast: Some(BroadcastBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                }),
            },
            db_path: unique_test_db_path("wallet_core_lifecycle_signing"),
            is_watch_only: false,
        }
    }

    /// Build the standard signing Signet test config but with a caller-supplied
    /// unique DB prefix.
    pub fn signing_test_config_with_db_prefix(prefix: &str) -> WalletConfig {
        let mut config = signing_test_config();
        config.db_path = unique_test_db_path(prefix);
        config
    }

    /// Load the standard signing test wallet using a caller-supplied unique DB prefix.
    pub fn load_signing_test_wallet_with_db_prefix(prefix: &str) -> (WalletConfig, WalletService) {
        let config = signing_test_config_with_db_prefix(prefix);
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");
        (config, wallet)
    }

    /// Build a strict-manual consolidation config for unit tests.
    pub fn strict_manual_consolidation_cfg() -> WalletConsolidationInfo {
        WalletConsolidationInfo {
            selection: WalletInputSelectionConfig {
                include_outpoints: vec![WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000001:0",
                )
                .expect("valid test outpoint")],
                exclude_outpoints: Vec::new(),
                confirmed_only: false,
                selection_mode: Some(WalletInputSelectionMode::StrictManual),
                max_input_count: None,
                min_input_count: None,
                min_utxo_value_sat: None,
                max_utxo_value_sat: None,
                strategy: None,
            },
            max_fee_pct_of_input_value: None,
        }
    }

    /// Build a consolidation config with the provided selection mode.
    pub fn consolidation_cfg_with_mode(
        selection_mode: WalletInputSelectionMode,
    ) -> WalletConsolidationInfo {
        WalletConsolidationInfo {
            selection: WalletInputSelectionConfig {
                selection_mode: Some(selection_mode),
                ..WalletInputSelectionConfig::default()
            },
            ..WalletConsolidationInfo::default()
        }
    }

    /// Generate unique temporary DB path per test.
    pub fn unique_test_db_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX_EPOCH")
            .as_nanos();

        let seq = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);

        std::env::temp_dir().join(format!(
            "{}_{}_{}_{}.db",
            prefix,
            std::process::id(),
            nanos,
            seq
        ))
    }

    #[test]
    fn shared_test_config_is_watch_only() {
        let cfg = test_config();
        assert!(cfg.is_watch_only);
        assert_eq!(cfg.network, Network::Signet);
    }

    #[test]
    fn test_config_with_db_prefix_overrides_db_path_prefix() {
        let cfg = test_config_with_db_prefix("wallet_core_txs");
        let path = cfg.db_path.to_string_lossy();

        assert!(cfg.is_watch_only);
        assert!(path.contains("wallet_core_txs"));
    }

    #[test]
    fn load_test_wallet_with_db_prefix_loads_watch_only_wallet() {
        let (_cfg, wallet) = load_test_wallet_with_db_prefix("wallet_core_txs_load");
        assert!(wallet.is_watch_only());
    }

    #[test]
    fn shared_signing_test_config_is_not_watch_only() {
        let cfg = signing_test_config();
        assert!(!cfg.is_watch_only);
        assert_eq!(cfg.network, Network::Signet);
    }

    #[test]
    fn signing_test_config_with_db_prefix_overrides_db_path_prefix() {
        let cfg = signing_test_config_with_db_prefix("wallet_core_psbt_sign");
        let path = cfg.db_path.to_string_lossy();

        assert!(!cfg.is_watch_only);
        assert!(path.contains("wallet_core_psbt_sign"));
    }

    #[test]
    fn load_signing_test_wallet_with_db_prefix_loads_signing_wallet() {
        let (_cfg, wallet) = load_signing_test_wallet_with_db_prefix("wallet_core_psbt_sign_load");
        assert!(!wallet.is_watch_only());
    }

    #[test]
    fn load_wallet_helper_loads_signing_config() {
        let cfg = signing_test_config();
        let wallet = load_wallet(&cfg);
        assert!(!wallet.is_watch_only());
    }

    #[test]
    fn unique_test_db_path_returns_distinct_paths() {
        let first = unique_test_db_path("wallet_core_test");
        let second = unique_test_db_path("wallet_core_test");
        assert_ne!(first, second);
    }

    #[test]
    fn consolidation_cfg_with_mode_sets_requested_mode() {
        let cfg = consolidation_cfg_with_mode(WalletInputSelectionMode::AutomaticOnly);
        assert_eq!(
            cfg.selection.selection_mode,
            Some(WalletInputSelectionMode::AutomaticOnly)
        );
    }

    #[test]
    fn strict_manual_consolidation_cfg_sets_strict_manual_mode() {
        let cfg = strict_manual_consolidation_cfg();
        assert_eq!(
            cfg.selection.selection_mode,
            Some(WalletInputSelectionMode::StrictManual)
        );
        assert_eq!(cfg.selection.include_outpoints.len(), 1);
    }

    #[test]
    fn sample_local_output_builds_confirmed_output() {
        let out = sample_local_output(1_000, 1, true);
        assert_eq!(out.txout.value.to_sat(), 1_000);
        assert_eq!(out.outpoint.vout, 1);
        assert!(out.chain_position.is_confirmed());
    }

    #[test]
    fn sample_local_output_builds_unconfirmed_output() {
        let out = sample_local_output(2_000, 2, false);
        assert_eq!(out.txout.value.to_sat(), 2_000);
        assert_eq!(out.outpoint.vout, 2);
        assert!(!out.chain_position.is_confirmed());
    }

    #[test]
    fn default_selection_config_starts_empty_and_automatic() {
        let cfg = default_selection_config();
        assert!(cfg.include_outpoints.is_empty());
        assert!(cfg.exclude_outpoints.is_empty());
        assert_eq!(
            cfg.selection_mode,
            Some(WalletInputSelectionMode::AutomaticOnly)
        );
    }
}
