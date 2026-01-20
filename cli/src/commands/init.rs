use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const ZKPROOF_DIR: &str = ".zkproof";
const CONFIG_FILE: &str = "config.toml";

/// Configuration structure for zkproof
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ZkProofConfig {
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

impl ZkProofConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        let mut settings = HashMap::new();
        settings.insert("version".to_string(), "0.1.0".to_string());
        settings.insert("initialized_at".to_string(), chrono_timestamp());
        Self { settings }
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

/// Get the path to the .zkproof directory
pub fn get_zkproof_dir(base_path: &Path) -> PathBuf {
    base_path.join(ZKPROOF_DIR)
}

/// Get the path to the config file
pub fn get_config_path(base_path: &Path) -> PathBuf {
    get_zkproof_dir(base_path).join(CONFIG_FILE)
}

/// Check if the .zkproof directory exists at the given path
pub fn is_initialized(base_path: &Path) -> bool {
    get_zkproof_dir(base_path).exists()
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
            "Config file not found. Run 'zkprof init' first.",
        ));
    }
    let config = ZkProofConfig::load(&config_path)?;
    Ok(config.get(key).cloned())
}

/// Read all config values
pub fn read_config(base_path: &Path) -> io::Result<ZkProofConfig> {
    let config_path = get_config_path(base_path);
    if !config_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Config file not found. Run 'zkprof init' first.",
        ));
    }
    ZkProofConfig::load(&config_path)
}

/// Write a value to the config file
pub fn write_config_value(base_path: &Path, key: &str, value: &str) -> io::Result<()> {
    let config_path = get_config_path(base_path);
    let mut config = if config_path.exists() {
        ZkProofConfig::load(&config_path)?
    } else {
        ZkProofConfig::default()
    };
    config.set(key, value);
    config.save(&config_path)
}

/// Generate a simple timestamp string (without external chrono dependency)
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
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
                eprintln!("❌ Error resolving path: {}", e);
                return;
            }
        },
        None => match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(e) => {
                eprintln!("❌ Error getting current directory: {}", e);
                return;
            }
        },
    };

    // Check if base path exists
    if !base_path.exists() {
        eprintln!("❌ Path does not exist: {}", base_path.display());
        return;
    }

    let zkproof_dir = get_zkproof_dir(&base_path);
    let config_path = get_config_path(&base_path);

    // Check if already initialized
    if zkproof_dir.exists() {
        println!("⚠️  zkproof is already initialized at: {}", zkproof_dir.display());
        if config_path.exists() {
            println!("   Config file exists at: {}", config_path.display());
        }
        return;
    }

    // Create .zkproof directory
    match fs::create_dir_all(&zkproof_dir) {
        Ok(_) => {
            println!("✅ Created directory: {}", zkproof_dir.display());
        }
        Err(e) => {
            eprintln!("❌ Failed to create directory: {}", e);
            return;
        }
    }

    // Create default config
    let config = ZkProofConfig::new();
    match config.save(&config_path) {
        Ok(_) => {
            println!("✅ Created config file: {}", config_path.display());
            println!();
            println!("🎉 zkproof initialized successfully!");
            println!();
            println!("Configuration:");
            for (key, value) in &config.settings {
                println!("   {} = {}", key, value);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to create config file: {}", e);
            // Clean up the created directory
            let _ = fs::remove_dir(&zkproof_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_creation_and_loading() {
        let temp_dir = std::env::temp_dir().join("zkproof_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let config = ZkProofConfig::new();
        let config_path = temp_dir.join("test_config.toml");
        config.save(&config_path).unwrap();

        let loaded = ZkProofConfig::load(&config_path).unwrap();
        assert_eq!(loaded.get("version"), Some(&"0.1.0".to_string()));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_is_initialized() {
        let temp_dir = std::env::temp_dir().join("zkproof_test_init");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        assert!(!is_initialized(&temp_dir));

        fs::create_dir_all(get_zkproof_dir(&temp_dir)).unwrap();
        assert!(is_initialized(&temp_dir));

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
