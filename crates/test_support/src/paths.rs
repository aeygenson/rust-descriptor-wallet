use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const REPO_MARKERS: &[&str] = &["Cargo.toml", "crates", "apps", "infra"];

/// Resolve the repository root by walking upward from the current working
/// directory until the expected project markers are found.
pub fn repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().context("failed to read current directory")?;

    loop {
        if looks_like_repo_root(&current) {
            return Ok(current);
        }

        if !current.pop() {
            break;
        }
    }

    Err(anyhow::anyhow!(
        "could not locate repository root from current working directory"
    ))
}

fn looks_like_repo_root(path: &Path) -> bool {
    REPO_MARKERS.iter().all(|marker| path.join(marker).exists())
}

/// `<repo>/infra/regtest`
pub fn regtest_root() -> Result<PathBuf> {
    Ok(repo_root()?.join("infra").join("regtest"))
}

/// `<repo>/infra/regtest/scripts`
pub fn regtest_scripts_dir() -> Result<PathBuf> {
    Ok(regtest_root()?.join("scripts"))
}

/// `<repo>/infra/regtest/scripts/start.sh`
pub fn regtest_start_script() -> Result<PathBuf> {
    Ok(regtest_scripts_dir()?.join("start.sh"))
}

/// `<repo>/infra/regtest/scripts/stop.sh`
pub fn regtest_stop_script() -> Result<PathBuf> {
    Ok(regtest_scripts_dir()?.join("stop.sh"))
}

/// `<repo>/infra/regtest/scripts/reset.sh`
pub fn regtest_reset_script() -> Result<PathBuf> {
    Ok(regtest_scripts_dir()?.join("reset.sh"))
}

/// `<repo>/infra/regtest/scripts/mine.sh`
pub fn regtest_mine_script() -> Result<PathBuf> {
    Ok(regtest_scripts_dir()?.join("mine.sh"))
}

/// `<repo>/infra/regtest/scripts/fund.sh`
pub fn regtest_fund_script() -> Result<PathBuf> {
    Ok(regtest_scripts_dir()?.join("fund.sh"))
}

/// `<repo>/infra/regtest/bitcoin/data`
pub fn regtest_bitcoind_data_dir() -> Result<PathBuf> {
    Ok(regtest_root()?.join("bitcoin").join("data"))
}

/// `<repo>/infra/regtest/electrs/db`
pub fn regtest_electrs_db_dir() -> Result<PathBuf> {
    Ok(regtest_root()?.join("electrs").join("db"))
}

/// `<repo>/wallet-regtest-local.json`
pub fn regtest_wallet_json() -> Result<PathBuf> {
    Ok(repo_root()?.join("wallet-regtest-local.json"))
}

/// `<repo>/wallet-mutiny-soft.json`
pub fn mutiny_wallet_json() -> Result<PathBuf> {
    Ok(repo_root()?.join("wallet-mutiny-soft.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_root_contains_expected_entries() {
        let root = repo_root().expect("repo root should resolve");
        assert!(root.join("Cargo.toml").exists());
        assert!(root.join("crates").exists());
        assert!(root.join("apps").exists());
        assert!(root.join("infra").exists());
    }

    #[test]
    fn regtest_script_paths_are_under_regtest_scripts_dir() {
        let scripts = regtest_scripts_dir().expect("scripts dir should resolve");

        assert_eq!(
            regtest_start_script().expect("start script"),
            scripts.join("start.sh")
        );
        assert_eq!(
            regtest_stop_script().expect("stop script"),
            scripts.join("stop.sh")
        );
        assert_eq!(
            regtest_reset_script().expect("reset script"),
            scripts.join("reset.sh")
        );
        assert_eq!(
            regtest_mine_script().expect("mine script"),
            scripts.join("mine.sh")
        );
        assert_eq!(
            regtest_fund_script().expect("fund script"),
            scripts.join("fund.sh")
        );
    }

    #[test]
    fn wallet_json_paths_resolve_under_repo_root() {
        let root = repo_root().expect("repo root should resolve");
        assert_eq!(
            regtest_wallet_json().expect("regtest wallet json"),
            root.join("wallet-regtest-local.json")
        );
        assert_eq!(
            mutiny_wallet_json().expect("mutiny wallet json"),
            root.join("wallet-mutiny-soft.json")
        );
    }
}
