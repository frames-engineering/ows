use lws_core::{AccountDescriptor, AccountId, ChainId, ChainType, WalletDescriptor, WalletId};
use lws_signer::{signer_for_chain, HdDeriver, Mnemonic, MnemonicStrength};

use crate::vault;
use crate::{parse_chain, CliError};

/// Returns a default CAIP-2 chain reference for a given chain type.
fn default_chain_reference(chain: ChainType) -> &'static str {
    match chain {
        ChainType::Evm => "1",
        ChainType::Solana => "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        ChainType::Bitcoin => "000000000019d6689c085ae165831e93",
        ChainType::Cosmos => "cosmoshub-4",
        ChainType::Tron => "mainnet",
    }
}

pub fn create(name: &str, chain_str: &str, words: u32) -> Result<(), CliError> {
    let chain = parse_chain(chain_str)?;
    let strength = match words {
        12 => MnemonicStrength::Words12,
        24 => MnemonicStrength::Words24,
        _ => return Err(CliError::InvalidArgs("--words must be 12 or 24".into())),
    };

    let mnemonic = Mnemonic::generate(strength)?;
    let signer = signer_for_chain(chain);
    let path = signer.default_derivation_path(0);
    let curve = signer.curve();

    let key = HdDeriver::derive_from_mnemonic(&mnemonic, "", &path, curve)?;
    let address = signer.derive_address(key.expose())?;

    let chain_id_str = format!("{}:{}", chain.namespace(), default_chain_reference(chain));
    let chain_id: ChainId = chain_id_str
        .parse()
        .map_err(|e: lws_core::LwsError| CliError::InvalidArgs(e.to_string()))?;

    let account_id_str = format!("{chain_id}:{address}");
    let account_id: AccountId = account_id_str
        .parse()
        .map_err(|e: lws_core::LwsError| CliError::InvalidArgs(e.to_string()))?;

    let wallet = WalletDescriptor {
        id: WalletId::new(),
        name: name.to_string(),
        chains: vec![chain],
        accounts: vec![AccountDescriptor {
            chain: chain_id,
            address: address.clone(),
            derivation_path: path.clone(),
            account_id,
        }],
        created_at: chrono::Utc::now(),
        updated_at: None,
    };

    vault::save_wallet(&wallet)?;

    let phrase = mnemonic.phrase();
    let phrase_str = String::from_utf8(phrase.expose().to_vec())
        .map_err(|e| CliError::InvalidArgs(format!("invalid UTF-8 in mnemonic: {e}")))?;

    println!("Wallet created: {}", wallet.id);
    println!("Name:           {}", wallet.name);
    println!("Chain:          {chain}");
    println!("Address:        {address}");
    println!("Path:           {path}");
    println!();
    println!("Mnemonic (save this — it will NOT be stored):");
    println!("{phrase_str}");

    Ok(())
}

pub fn list() -> Result<(), CliError> {
    let wallets = vault::list_wallets()?;

    if wallets.is_empty() {
        println!("No wallets found.");
        return Ok(());
    }

    for w in &wallets {
        println!("ID:      {}", w.id);
        println!("Name:    {}", w.name);
        println!(
            "Chains:  {}",
            w.chains
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        for acct in &w.accounts {
            println!("  {} → {}", acct.chain, acct.address);
        }
        println!("Created: {}", w.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!();
    }

    Ok(())
}
