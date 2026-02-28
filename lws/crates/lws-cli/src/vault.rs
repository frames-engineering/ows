use lws_core::{Config, WalletDescriptor};
use std::fs;
use std::path::PathBuf;

use crate::CliError;

/// Returns the wallets directory, creating it if necessary.
pub fn wallets_dir() -> Result<PathBuf, CliError> {
    let config = Config::default();
    let dir = config.vault_path.join("wallets");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Save a wallet descriptor as pretty JSON.
pub fn save_wallet(wallet: &WalletDescriptor) -> Result<(), CliError> {
    let dir = wallets_dir()?;
    let path = dir.join(format!("{}.json", wallet.id));
    let json = serde_json::to_string_pretty(wallet)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load all wallet descriptors from the wallets directory.
/// Skips malformed files with a warning to stderr.
/// Returns wallets sorted by created_at descending (newest first).
pub fn list_wallets() -> Result<Vec<WalletDescriptor>, CliError> {
    let dir = wallets_dir()?;
    let mut wallets = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(wallets),
        Err(e) => return Err(e.into()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<WalletDescriptor>(&contents) {
                Ok(w) => wallets.push(w),
                Err(e) => {
                    eprintln!("warning: skipping {}: {e}", path.display());
                }
            },
            Err(e) => {
                eprintln!("warning: skipping {}: {e}", path.display());
            }
        }
    }

    wallets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(wallets)
}
