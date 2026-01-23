use std::io;

use super::init::{
    get_solana_network, get_solana_rpc_url, read_config, reset_solana_rpc_url,
    resolve_project_path, set_solana_network, set_solana_rpc_url, SolanaNetwork,
};

/// Display current configuration
pub fn run_config_show(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;

    let config = read_config(&base_path)?;
    let network = config.get_solana_network();

    println!("📋 zklense Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    for (key, value) in &config.settings {
        if key == "solana_network" {
            println!("   {} = {} (RPC: {})", key, value, network.rpc_url());
        } else {
            println!("   {} = {}", key, value);
        }
    }

    println!();
    Ok(())
}

/// Get the current Solana network
pub fn run_config_get_network(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let network = get_solana_network(&base_path)?;

    println!("🌐 Current Solana Network: {}", network);
    println!("   RPC URL: {}", network.rpc_url());

    Ok(())
}

/// Set the Solana network
pub fn run_config_set_network(network_str: &str, path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;

    let network: SolanaNetwork = network_str.parse().map_err(|e: String| {
        io::Error::new(io::ErrorKind::InvalidInput, e)
    })?;

    let old_network = get_solana_network(&base_path)?;

    if old_network == network {
        println!("ℹ️  Solana network is already set to: {}", network);
        return Ok(());
    }

    set_solana_network(&base_path, network)?;

    println!("✅ Solana network changed: {} → {}", old_network, network);
    println!("   RPC URL: {}", network.rpc_url());

    Ok(())
}

/// List available Solana networks
pub fn run_config_list_networks(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let current = get_solana_network(&base_path)?;
    let current_rpc = get_solana_rpc_url(&base_path)?;

    println!("🌐 Available Solana Networks:");
    println!();

    for network in SolanaNetwork::all() {
        let marker = if *network == current { "●" } else { "○" };
        println!("   {} {} - {}", marker, network, network.rpc_url());
    }

    println!();
    println!("Current RPC URL: {}", current_rpc);
    println!();
    println!("Use 'zklense config set-network <network>' to change the network.");
    println!("Use 'zklense config set-rpc <url>' to set a custom RPC URL.");

    Ok(())
}

/// Get the current Solana RPC URL
pub fn run_config_get_rpc(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let rpc_url = get_solana_rpc_url(&base_path)?;
    let network = get_solana_network(&base_path)?;

    println!("🔗 Current Solana RPC URL: {}", rpc_url);
    println!("   Network: {}", network);

    // Check if it's a custom RPC
    if rpc_url != network.rpc_url() {
        println!("   (Custom RPC - default for {} is {})", network, network.rpc_url());
    }

    Ok(())
}

/// Set a custom Solana RPC URL
pub fn run_config_set_rpc(rpc_url: &str, path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;

    // Basic validation
    if !rpc_url.starts_with("http://") && !rpc_url.starts_with("https://") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RPC URL must start with http:// or https://",
        ));
    }

    let old_rpc = get_solana_rpc_url(&base_path)?;

    if old_rpc == rpc_url {
        println!("ℹ️  RPC URL is already set to: {}", rpc_url);
        return Ok(());
    }

    set_solana_rpc_url(&base_path, rpc_url)?;

    println!("✅ Solana RPC URL changed:");
    println!("   Old: {}", old_rpc);
    println!("   New: {}", rpc_url);

    Ok(())
}

/// Reset the Solana RPC URL to the default for the current network
pub fn run_config_reset_rpc(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let network = get_solana_network(&base_path)?;
    let old_rpc = get_solana_rpc_url(&base_path)?;

    let new_rpc = reset_solana_rpc_url(&base_path)?;

    if old_rpc == new_rpc {
        println!("ℹ️  RPC URL is already set to the default: {}", new_rpc);
    } else {
        println!("✅ Solana RPC URL reset to default for {}:", network);
        println!("   Old: {}", old_rpc);
        println!("   New: {}", new_rpc);
    }

    Ok(())
}

/// Main config command runner
pub fn run_config(action: ConfigAction, path: Option<String>) -> io::Result<()> {
    match action {
        ConfigAction::Show => run_config_show(path),
        ConfigAction::GetNetwork => run_config_get_network(path),
        ConfigAction::SetNetwork(network) => run_config_set_network(&network, path),
        ConfigAction::ListNetworks => run_config_list_networks(path),
        ConfigAction::GetRpc => run_config_get_rpc(path),
        ConfigAction::SetRpc(rpc_url) => run_config_set_rpc(&rpc_url, path),
        ConfigAction::ResetRpc => run_config_reset_rpc(path),
    }
}

/// Config subcommand actions
pub enum ConfigAction {
    Show,
    GetNetwork,
    SetNetwork(String),
    ListNetworks,
    GetRpc,
    SetRpc(String),
    ResetRpc,
}
