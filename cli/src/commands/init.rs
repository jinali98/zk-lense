use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::fmt;
use std::str::FromStr;
use console::style;

use crate::ui::{self, emoji};

const ZKLENSE_DIR: &str = ".zklense";
const CONFIG_FILE: &str = "config.toml";
pub const DEFAULT_WEB_APP_URL: &str = "https://zklense.netlify.app/";

/// Solana network environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SolanaNetwork {
    #[default]
    Devnet,
    Testnet,
    Mainnet,
}

impl SolanaNetwork {
    /// Get all available networks
    pub fn all() -> &'static [SolanaNetwork] {
        &[SolanaNetwork::Devnet, SolanaNetwork::Testnet, SolanaNetwork::Mainnet]
    }

    /// Get the RPC URL for this network
    pub fn rpc_url(&self) -> &'static str {
        match self {
            SolanaNetwork::Devnet => "https://api.devnet.solana.com",
            SolanaNetwork::Testnet => "https://api.testnet.solana.com",
            SolanaNetwork::Mainnet => "https://api.mainnet-beta.solana.com",
        }
    }

    /// Get the network name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            SolanaNetwork::Devnet => "devnet",
            SolanaNetwork::Testnet => "testnet",
            SolanaNetwork::Mainnet => "mainnet",
        }
    }
}

impl fmt::Display for SolanaNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SolanaNetwork {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "devnet" => Ok(SolanaNetwork::Devnet),
            "testnet" => Ok(SolanaNetwork::Testnet),
            "mainnet" | "mainnet-beta" => Ok(SolanaNetwork::Mainnet),
            _ => Err(format!(
                "Invalid network '{}'. Valid options: devnet, testnet, mainnet",
                s
            )),
        }
    }
}

/// Configuration structure for zklense
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ZkLenseConfig {
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl ZkLenseConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        let default_network = SolanaNetwork::default();
        let mut settings = HashMap::new();
        settings.insert("version".to_string(), "0.1.0".to_string());
        settings.insert("initialized_at".to_string(), chrono_timestamp());
        settings.insert("web_app_url".to_string(), DEFAULT_WEB_APP_URL.to_string());
        settings.insert("solana_network".to_string(), default_network.as_str().to_string());
        settings.insert("solana_rpc_url".to_string(), default_network.rpc_url().to_string());
        Self { settings }
    }

    /// Get the current Solana network
    pub fn get_solana_network(&self) -> SolanaNetwork {
        self.get("solana_network")
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    /// Set the Solana network (also updates RPC URL to the default for that network)
    pub fn set_solana_network(&mut self, network: SolanaNetwork) {
        self.set("solana_network", network.as_str());
        self.set("solana_rpc_url", network.rpc_url());
    }

    /// Get the current Solana RPC URL
    pub fn get_solana_rpc_url(&self) -> String {
        self.get("solana_rpc_url")
            .cloned()
            .unwrap_or_else(|| self.get_solana_network().rpc_url().to_string())
    }

    /// Set a custom Solana RPC URL
    pub fn set_solana_rpc_url(&mut self, rpc_url: &str) {
        self.set("solana_rpc_url", rpc_url);
    }

    /// Get a value from the configuration
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Set a value in the configuration
    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut file = fs::File::create(path)?;
        file.write_all(toml_string.as_bytes())?;
        Ok(())
    }

    /// Load configuration from file
    pub fn load(path: &Path) -> io::Result<Self> {
        let contents = fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Get the path to the .zklense directory
pub fn get_zklense_dir(base_path: &Path) -> PathBuf {
    base_path.join(ZKLENSE_DIR)
}

/// Get the path to the config file
pub fn get_config_path(base_path: &Path) -> PathBuf {
    get_zklense_dir(base_path).join(CONFIG_FILE)
}

/// Check if the .zklense directory exists at the given path
pub fn is_initialized(base_path: &Path) -> bool {
    get_zklense_dir(base_path).exists()
}

/// Check if the config file exists at the given path
pub fn config_exists(base_path: &Path) -> bool {
    get_config_path(base_path).is_file()
}

/// Read a specific value from the config file
pub fn read_config_value(base_path: &Path, key: &str) -> io::Result<Option<String>> {
    let config_path = get_config_path(base_path);
    if !config_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Config file not found. Run 'zklense init' first.",
        ));
    }
    let config = ZkLenseConfig::load(&config_path)?;
    Ok(config.get(key).cloned())
}

/// Read all config values
pub fn read_config(base_path: &Path) -> io::Result<ZkLenseConfig> {
    let config_path = get_config_path(base_path);
    if !config_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Config file not found. Run 'zklense init' first.",
        ));
    }
    ZkLenseConfig::load(&config_path)
}

/// Write a value to the config file
pub fn write_config_value(base_path: &Path, key: &str, value: &str) -> io::Result<()> {
    let config_path = get_config_path(base_path);
    let mut config = if config_path.exists() {
        ZkLenseConfig::load(&config_path)?
    } else {
        ZkLenseConfig::default()
    };
    config.set(key, value);
    config.save(&config_path)
}

/// Get the current Solana network from config
pub fn get_solana_network(base_path: &Path) -> io::Result<SolanaNetwork> {
    let config = read_config(base_path)?;
    Ok(config.get_solana_network())
}

/// Set the Solana network in config (also updates RPC URL to the default for that network)
pub fn set_solana_network(base_path: &Path, network: SolanaNetwork) -> io::Result<()> {
    let config_path = get_config_path(base_path);
    let mut config = read_config(base_path)?;
    config.set_solana_network(network);
    config.save(&config_path)
}

/// Get the current Solana RPC URL from config
pub fn get_solana_rpc_url(base_path: &Path) -> io::Result<String> {
    let config = read_config(base_path)?;
    Ok(config.get_solana_rpc_url())
}

/// Set a custom Solana RPC URL in config
pub fn set_solana_rpc_url(base_path: &Path, rpc_url: &str) -> io::Result<()> {
    let config_path = get_config_path(base_path);
    let mut config = read_config(base_path)?;
    config.set_solana_rpc_url(rpc_url);
    config.save(&config_path)
}

/// Reset the Solana RPC URL to the default for the current network
pub fn reset_solana_rpc_url(base_path: &Path) -> io::Result<String> {
    let config_path = get_config_path(base_path);
    let mut config = read_config(base_path)?;
    let default_url = config.get_solana_network().rpc_url().to_string();
    config.set_solana_rpc_url(&default_url);
    config.save(&config_path)?;
    Ok(default_url)
}

/// Generate a simple timestamp string (without external chrono dependency)
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Resolve and validate a path from an optional string, defaulting to current directory
pub fn resolve_project_path(path: Option<&str>) -> io::Result<PathBuf> {
    match path {
        Some(p) => resolve_path(p),
        None => std::env::current_dir(),
    }
}

/// Check if the project is initialized, and if not, prompt the user to initialize.
/// Returns Ok(true) if initialized (or just initialized), Ok(false) if user declined,
/// or an error if something went wrong.
pub fn ensure_initialized(path: Option<&str>) -> io::Result<bool> {
    let base_path = resolve_project_path(path)?;

    if !base_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path does not exist: {}", base_path.display()),
        ));
    }

    if is_initialized(&base_path) {
        return Ok(true);
    }

    // Show warning panel
    ui::panel_warning(
        "NOT INITIALIZED",
        &format!("zklense is not initialized in:\n{}", base_path.display()),
    );

    // Interactive selection instead of Y/N prompt
    let should_init = ui::confirm_custom(
        "Would you like to initialize it now?",
        &format!("{} Yes, initialize now", emoji::CHECKMARK),
        &format!("{} No, cancel", emoji::CROSSMARK),
    )?;

    if should_init {
        run_init(Some(base_path.to_string_lossy().to_string()));
        // Check if initialization succeeded
        if is_initialized(&base_path) {
            Ok(true)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Initialization failed",
            ))
        }
    } else {
        ui::info("Run 'zklense init' to initialize the project first.");
        Ok(false)
    }
}

/// Resolve a path (handles both relative and absolute paths)
fn resolve_path(path_str: &str) -> io::Result<PathBuf> {
    let path = Path::new(path_str);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join(path))
    }
}

/// Run the init command
pub fn run_init(path: Option<String>) {
    let base_path = match path {
        Some(p) => match resolve_path(&p) {
            Ok(resolved) => resolved,
            Err(e) => {
                ui::error(&format!("Error resolving path: {}", e));
                return;
            }
        },
        None => match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(e) => {
                ui::error(&format!("Error getting current directory: {}", e));
                return;
            }
        },
    };

    // Check if base path exists
    if !base_path.exists() {
        ui::error(&format!("Path does not exist: {}", base_path.display()));
        return;
    }

    let zklense_dir = get_zklense_dir(&base_path);
    let config_path = get_config_path(&base_path);

    // Check if already initialized
    if zklense_dir.exists() {
        ui::info(&format!(
            "zklense directory already exists at: {}",
            style(zklense_dir.display()).dim()
        ));

        // Check if config.toml exists, recreate if missing
        if !config_path.exists() {
            ui::warn("Config file missing, recreating...");
            let config = ZkLenseConfig::new();
            match config.save(&config_path) {
                Ok(_) => {
                    ui::success(&format!(
                        "Recreated config file: {}",
                        style(config_path.display()).dim()
                    ));
                    print_config_summary(&config);
                }
                Err(e) => {
                    ui::error(&format!("Failed to recreate config file: {}", e));
                }
            }
        } else {
            ui::success(&format!(
                "Config file exists at: {}",
                style(config_path.display()).dim()
            ));
        }
        return;
    }

    // Create .zklense directory
    let spinner = ui::spinner("Creating .zklense directory...");
    match fs::create_dir_all(&zklense_dir) {
        Ok(_) => {
            ui::spinner_success(&spinner, &format!(
                "Created directory: {}",
                style(zklense_dir.display()).dim()
            ));
        }
        Err(e) => {
            ui::spinner_error(&spinner, &format!("Failed to create directory: {}", e));
            return;
        }
    }

    // Create default config
    let spinner = ui::spinner("Creating configuration...");
    let config = ZkLenseConfig::new();
    match config.save(&config_path) {
        Ok(_) => {
            ui::spinner_success(&spinner, &format!(
                "Created config file: {}",
                style(config_path.display()).dim()
            ));

            // Show success panel
            ui::panel_success(
                "INITIALIZED",
                &format!("zklense initialized successfully!\nProject: {}", base_path.display()),
            );

            // Show configuration summary
            print_config_summary(&config);
        }
        Err(e) => {
            ui::spinner_error(&spinner, &format!("Failed to create config file: {}", e));
            // Clean up the created directory
            let _ = fs::remove_dir(&zklense_dir);
        }
    }
}

/// Print a formatted configuration summary
fn print_config_summary(config: &ZkLenseConfig) {
    ui::section(emoji::GEAR, "Configuration");
    
    let network = config.get_solana_network();
    let items = vec![
        ("Network", config.get("solana_network").map(|s| s.as_str()).unwrap_or("devnet")),
        ("RPC URL", config.get("solana_rpc_url").map(|s| s.as_str()).unwrap_or(network.rpc_url())),
        ("Web App", config.get("web_app_url").map(|s| s.as_str()).unwrap_or(DEFAULT_WEB_APP_URL)),
        ("Version", config.get("version").map(|s| s.as_str()).unwrap_or("0.1.0")),
    ];
    
    ui::print_tree(&items);
    ui::blank();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_creation_and_loading() {
        let temp_dir = std::env::temp_dir().join("zklense_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let config = ZkLenseConfig::new();
        let config_path = temp_dir.join("test_config.toml");
        config.save(&config_path).unwrap();

        let loaded = ZkLenseConfig::load(&config_path).unwrap();
        assert_eq!(loaded.get("version"), Some(&"0.1.0".to_string()));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_is_initialized() {
        let temp_dir = std::env::temp_dir().join("zklense_test_init");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert!(!is_initialized(&temp_dir));

        fs::create_dir_all(get_zklense_dir(&temp_dir)).unwrap();
        assert!(is_initialized(&temp_dir));

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
