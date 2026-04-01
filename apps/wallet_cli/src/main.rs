pub mod cli;
pub mod commands;

use anyhow::Result;
use clap::Parser;
use wallet_api::WalletApi;
use crate::cli::Cli;
use crate::commands::handle_command;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let api = WalletApi::new().await?;
    
    handle_command(&api, cli.command).await?;
    
    Ok(())
}