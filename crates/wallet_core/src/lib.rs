pub mod error;
pub  use error::WalletCoreError;
pub  type WalletCoreResult<T> = Result<T, WalletCoreError>;
#[derive(Debug)]
pub  struct WalletCore;
impl WalletCore {
    pub fn new() -> Self {
        Self
    }
    pub  fn health_check(&self)->WalletCoreResult<&'static str> {
        Ok("wallet_core OK")
    }
}
