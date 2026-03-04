use lws_core::EncryptedWallet;

use crate::CliError;

// Delegate vault operations to lws-lib, using the default vault path.

pub fn save_encrypted_wallet(wallet: &EncryptedWallet) -> Result<(), CliError> {
    Ok(lws_lib::vault::save_encrypted_wallet(wallet, None)?)
}

pub fn list_encrypted_wallets() -> Result<Vec<EncryptedWallet>, CliError> {
    Ok(lws_lib::vault::list_encrypted_wallets(None)?)
}

pub fn load_wallet_by_name_or_id(name_or_id: &str) -> Result<EncryptedWallet, CliError> {
    Ok(lws_lib::vault::load_wallet_by_name_or_id(name_or_id, None)?)
}

pub fn delete_wallet(id: &str) -> Result<(), CliError> {
    Ok(lws_lib::vault::delete_wallet_file(id, None)?)
}

pub fn wallet_name_exists(name: &str) -> Result<bool, CliError> {
    Ok(lws_lib::vault::wallet_name_exists(name, None)?)
}

/// Prompt the user for a passphrase (with confirmation for new wallets).
pub fn prompt_passphrase(confirm: bool) -> Result<String, CliError> {
    let pass = rpassword::prompt_password("Enter vault passphrase: ")
        .map_err(|e| CliError::InvalidArgs(format!("failed to read passphrase: {e}")))?;

    if pass.len() < 12 {
        return Err(CliError::InvalidArgs(
            "passphrase must be at least 12 characters".into(),
        ));
    }

    if confirm {
        let pass2 = rpassword::prompt_password("Confirm vault passphrase: ")
            .map_err(|e| CliError::InvalidArgs(format!("failed to read passphrase: {e}")))?;
        if pass != pass2 {
            return Err(CliError::InvalidArgs("passphrases do not match".into()));
        }
    }

    Ok(pass)
}

/// Read passphrase from LWS_PASSPHRASE env var, falling back to interactive prompt.
pub fn get_passphrase(confirm: bool) -> Result<String, CliError> {
    if let Ok(pass) = std::env::var("LWS_PASSPHRASE") {
        if pass.len() < 12 {
            return Err(CliError::InvalidArgs(
                "LWS_PASSPHRASE must be at least 12 characters".into(),
            ));
        }
        return Ok(pass);
    }
    prompt_passphrase(confirm)
}
