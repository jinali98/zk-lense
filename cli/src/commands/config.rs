use std::io;
use console::style;

use super::init::{
    get_solana_network, get_solana_rpc_url, read_config, reset_solana_rpc_url,
    resolve_project_path, set_solana_network, set_solana_rpc_url, SolanaNetwork,
    DEFAULT_WEB_APP_URL,
};
use crate::ui::{self, emoji};

/// Display current configuration
pub fn run_config_show(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let config = read_config(&base_path)?;
    let network = config.get_solana_network();
    let rpc_url = config.get_solana_rpc_url();

    // Header panel
    ui::panel_header(emoji::GEAR, "ZKLENSE CONFIGURATION", None);

    // Create a formatted table
    let mut table = ui::create_kv_table();
    
    ui::add_kv_row(&mut table, emoji::GLOBE, "Network", network.as_str());
    ui::add_kv_row(&mut table, emoji::LINK, "RPC URL", &rpc_url);
    
    // Show if RPC is custom
    if rpc_url != network.rpc_url() {
        ui::add_kv_row(&mut table, "", "(Custom)", &format!("default is {}", network.rpc_url()));
    }
    
    ui::add_kv_row(
        &mut table,
        emoji::GLOBE,
        "Web App",
        config.get("web_app_url").map(|s| s.as_str()).unwrap_or(DEFAULT_WEB_APP_URL),
    );
    ui::add_kv_row(
        &mut table,
        emoji::PACKAGE,
        "Version",
        config.get("version").map(|s| s.as_str()).unwrap_or("0.1.0"),
    );

    println!("{table}");
    ui::blank();

    Ok(())
}

/// Get the current Solana network
pub fn run_config_get_network(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let network = get_solana_network(&base_path)?;
    let rpc_url = get_solana_rpc_url(&base_path)?;

    ui::section(emoji::GLOBE, "Current Solana Network");
    
    let items = vec![
        ("Network", network.as_str()),
        ("RPC URL", &rpc_url),
    ];
    ui::print_tree(&items);
    ui::blank();

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
        ui::info(&format!("Solana network is already set to: {}", style(network).bold()));
        return Ok(());
    }

    let spinner = ui::spinner(&format!("Switching to {}...", network));
    set_solana_network(&base_path, network)?;
    ui::spinner_success(&spinner, &format!(
        "Network changed: {} {} {}",
        style(old_network).dim(),
        emoji::ARROW_RIGHT,
        style(network).green().bold()
    ));

    ui::blank();
    ui::print_value_with_emoji(emoji::LINK, "RPC URL", network.rpc_url());
    ui::blank();

    Ok(())
}

/// List available Solana networks
pub fn run_config_list_networks(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let current = get_solana_network(&base_path)?;
    let current_rpc = get_solana_rpc_url(&base_path)?;

    ui::panel_header(emoji::GLOBE, "AVAILABLE NETWORKS", None);

    for network in SolanaNetwork::all() {
        let is_current = *network == current;
        let marker = if is_current { emoji::ACTIVE } else { emoji::PENDING };
        let name = if is_current {
            style(network.as_str()).green().bold().to_string()
        } else {
            style(network.as_str()).dim().to_string()
        };
        let url = style(network.rpc_url()).dim();
        
        println!("  {} {:<12} {}", marker, name, url);
    }

    ui::blank();
    
    // Show current RPC URL
    let is_custom = current_rpc != current.rpc_url();
    if is_custom {
        println!(
            "  {} {} {}",
            emoji::LINK,
            style("Current RPC (custom):").dim(),
            style(&current_rpc).cyan()
        );
    } else {
        println!(
            "  {} {} {}",
            emoji::LINK,
            style("Current RPC:").dim(),
            style(&current_rpc).cyan()
        );
    }

    ui::blank();
    println!("  {} {}", emoji::BULB, style("Commands:").dim());
    println!("     {} Switch network", style("zklense config set-network <network>").cyan());
    println!("     {} Custom RPC", style("zklense config set-rpc <url>").cyan());
    ui::blank();

    Ok(())
}

/// Get the current Solana RPC URL
pub fn run_config_get_rpc(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let rpc_url = get_solana_rpc_url(&base_path)?;
    let network = get_solana_network(&base_path)?;

    ui::section(emoji::LINK, "Current Solana RPC");

    let is_custom = rpc_url != network.rpc_url();
    
    let items: Vec<(&str, &str)> = if is_custom {
        vec![
            ("RPC URL", &rpc_url),
            ("Network", network.as_str()),
            ("Status", "Custom RPC"),
        ]
    } else {
        vec![
            ("RPC URL", &rpc_url),
            ("Network", network.as_str()),
        ]
    };
    
    ui::print_tree(&items);
    
    if is_custom {
        ui::blank();
        println!(
            "  {} Default for {} is: {}",
            emoji::INFO,
            network,
            style(network.rpc_url()).dim()
        );
    }
    
    ui::blank();

    Ok(())
}

/// Set a custom Solana RPC URL
pub fn run_config_set_rpc(rpc_url: &str, path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;

    // Basic validation
    if !rpc_url.starts_with("http://") && !rpc_url.starts_with("https://") {
        ui::panel_error(
            "INVALID URL",
            "RPC URL must start with http:// or https://",
            None,
            Some(&["Example: https://api.mainnet-beta.solana.com"]),
        );
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RPC URL must start with http:// or https://",
        ));
    }

    let old_rpc = get_solana_rpc_url(&base_path)?;

    if old_rpc == rpc_url {
        ui::info(&format!("RPC URL is already set to: {}", style(rpc_url).bold()));
        return Ok(());
    }

    let spinner = ui::spinner("Updating RPC URL...");
    set_solana_rpc_url(&base_path, rpc_url)?;
    ui::spinner_success(&spinner, "RPC URL updated");

    ui::blank();
    println!(
        "  {} {} {}",
        emoji::TREE_BRANCH,
        style("Old:").dim(),
        style(&old_rpc).dim().strikethrough()
    );
    println!(
        "  {} {} {}",
        emoji::TREE_END,
        style("New:").dim(),
        style(rpc_url).green().bold()
    );
    ui::blank();

    Ok(())
}

/// Reset the Solana RPC URL to the default for the current network
pub fn run_config_reset_rpc(path: Option<String>) -> io::Result<()> {
    let base_path = resolve_project_path(path.as_deref())?;
    let network = get_solana_network(&base_path)?;
    let old_rpc = get_solana_rpc_url(&base_path)?;

    let new_rpc = reset_solana_rpc_url(&base_path)?;

    if old_rpc == new_rpc {
        ui::info(&format!(
            "RPC URL is already set to the default: {}",
            style(&new_rpc).bold()
        ));
    } else {
        ui::success(&format!(
            "RPC URL reset to default for {}",
            style(network).bold()
        ));
        ui::blank();
        println!(
            "  {} {} {}",
            emoji::TREE_BRANCH,
            style("Old:").dim(),
            style(&old_rpc).dim().strikethrough()
        );
        println!(
            "  {} {} {}",
            emoji::TREE_END,
            style("New:").dim(),
            style(&new_rpc).green().bold()
        );
        ui::blank();
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
