use console::style;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::ui::{self, emoji};

const NARGO_TOML: &str = "Nargo.toml";
const TARGET_DIR: &str = "target";

/// Structure to parse Nargo.toml
#[derive(Debug, Deserialize)]
struct NargoToml {
    package: NargoPackage,
}

#[derive(Debug, Deserialize)]
struct NargoPackage {
    name: String,
    #[serde(rename = "type")]
    _package_type: Option<String>,
    #[serde(default)]
    _authors: Vec<String>,
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Read and parse Nargo.toml to get the circuit name
fn read_circuit_name(base_path: &Path) -> io::Result<String> {
    let nargo_path = base_path.join(NARGO_TOML);

    if !nargo_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Nargo.toml not found at: {}\nMake sure you are in a Noir project directory.",
                nargo_path.display()
            ),
        ));
    }

    let contents = fs::read_to_string(&nargo_path)?;
    let nargo_toml: NargoToml = toml::from_str(&contents).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse Nargo.toml: {}", e),
        )
    })?;

    Ok(nargo_toml.package.name)
}

/// Run a command and stream output to stdout (with spinner)
fn run_command_with_spinner(
    cmd: &str,
    args: &[&str],
    working_dir: &Path,
    message: &str,
) -> io::Result<u128> {
    let spinner = ui::spinner(message);
    let start = Instant::now();

    let output = Command::new(cmd)
        .args(args)
        .current_dir(working_dir)
        .output()?;

    let duration = start.elapsed().as_millis();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ui::spinner_error(&spinner, &format!("Failed: {} {}", cmd, args.join(" ")));

        // Print stderr if available
        if !stderr.is_empty() {
            ui::blank();
            for line in stderr.lines().take(10) {
                println!("    {}", style(line).red().dim());
            }
        }

        return Err(io::Error::other(format!(
            "Command '{}' failed with exit code: {:?}",
            cmd,
            output.status.code()
        )));
    }

    ui::spinner_success_with_duration(&spinner, &message.replace("...", ""), duration);
    Ok(duration)
}

/// Run a command and capture its output
fn run_command_capture(cmd: &str, args: &[&str], working_dir: &Path) -> io::Result<String> {
    let spinner = ui::spinner(&format!("Running {} {}...", cmd, args.join(" ")));

    let output = Command::new(cmd)
        .args(args)
        .current_dir(working_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ui::spinner_error(
            &spinner,
            &format!("Command failed: {} {}", cmd, args.join(" ")),
        );
        return Err(io::Error::other(format!(
            "Command '{}' failed with exit code: {:?}\n{}",
            cmd,
            output.status.code(),
            stderr
        )));
    }

    ui::spinner_success(&spinner, &format!("{} {}", cmd, args.join(" ")));
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Pipeline step definition
struct PipelineStep {
    name: &'static str,
    description: &'static str,
    command: &'static str,
    args_fn: fn(&str) -> Vec<String>,
    working_dir_is_target: bool,
}

/// Get all pipeline steps
fn get_pipeline_steps() -> Vec<PipelineStep> {
    vec![
        PipelineStep {
            name: "Execute",
            description: "Running nargo execute",
            command: "nargo",
            args_fn: |_| vec!["execute".to_string()],
            working_dir_is_target: false,
        },
        PipelineStep {
            name: "Compile",
            description: "Compiling ACIR to CCS",
            command: "sunspot",
            args_fn: |circuit| vec!["compile".to_string(), format!("{}.json", circuit)],
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "Setup",
            description: "Generating proving and verifying keys",
            command: "sunspot",
            args_fn: |circuit| vec!["setup".to_string(), format!("{}.ccs", circuit)],
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "Prove",
            description: "Creating Groth16 proof",
            command: "sunspot",
            args_fn: |circuit| {
                vec![
                    "prove".to_string(),
                    format!("{}.json", circuit),
                    format!("{}.gz", circuit),
                    format!("{}.ccs", circuit),
                    format!("{}.pk", circuit),
                ]
            },
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "Verify",
            description: "Verifying proof",
            command: "sunspot",
            args_fn: |circuit| {
                vec![
                    "verify".to_string(),
                    format!("{}.vk", circuit),
                    format!("{}.proof", circuit),
                    format!("{}.pw", circuit),
                ]
            },
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "Deploy",
            description: "Creating Solana verification program",
            command: "sunspot",
            args_fn: |circuit| vec!["deploy".to_string(), format!("{}.vk", circuit)],
            working_dir_is_target: true,
        },
    ]
}

/// Check prerequisites before running the pipeline
fn check_prerequisites() -> io::Result<()> {
    ui::section(emoji::SEARCH, "Checking Prerequisites");

    let mut missing = Vec::new();

    // Check nargo
    if command_exists("nargo") {
        println!("  {} {} found", emoji::SUCCESS, style("nargo").green());
    } else {
        println!("  {} {} not found", emoji::ERROR, style("nargo").red());
        missing.push("nargo");
    }

    // Check sunspot
    if command_exists("sunspot") {
        println!("  {} {} found", emoji::SUCCESS, style("sunspot").green());
    } else {
        println!("  {} {} not found", emoji::ERROR, style("sunspot").red());
        missing.push("sunspot");
    }

    ui::blank();

    if !missing.is_empty() {
        // If only sunspot is missing, offer to install it
        if missing.len() == 1 && missing.contains(&"sunspot") {
            return handle_missing_sunspot();
        }

        let mut suggestions = Vec::new();

        if missing.contains(&"nargo") {
            suggestions
                .push("Install nargo: https://noir-lang.org/docs/getting_started/installation");
        }
        if missing.contains(&"sunspot") {
            suggestions.push("Install sunspot: https://github.com/reilabs/sunspot");
        }

        ui::panel_error(
            "MISSING PREREQUISITES",
            &format!("Missing required commands: {}", missing.join(", ")),
            None,
            Some(&suggestions.iter().map(|s| s.as_ref()).collect::<Vec<_>>()),
        );

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Missing required commands: {}", missing.join(", ")),
        ));
    }

    Ok(())
}

/// Handle missing sunspot - offer to install it automatically
fn handle_missing_sunspot() -> io::Result<()> {
    ui::panel_warning(
        "SUNSPOT NOT FOUND",
        "Sunspot is required to compile and prove Noir circuits for Solana.\n\nSunspot repository: https://github.com/reilabs/sunspot",
    );

    // Ask if user wants to install sunspot
    let should_install = ui::confirm_custom(
        "Would you like to install Sunspot now?",
        &format!("{} Yes, install Sunspot", emoji::CHECKMARK),
        &format!("{} No, I'll install it manually", emoji::CROSSMARK),
    )?;

    if !should_install {
        ui::info("You can install Sunspot manually from: https://github.com/reilabs/sunspot");
        ui::blank();
        println!("  {} Installation steps:", emoji::BULB);
        println!("     1. git clone https://github.com/reilabs/sunspot.git ~/sunspot");
        println!("     2. cd ~/sunspot/go && go build -o sunspot .");
        println!("     3. sudo mv sunspot /usr/local/bin/");
        println!("     4. export GNARK_VERIFIER_BIN=\"$HOME/sunspot/gnark-solana/crates/verifier-bin\"");
        ui::blank();
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Sunspot is required. Install it and try again.",
        ));
    }

    ui::blank();

    // Check prerequisites for sunspot installation
    check_sunspot_prerequisites()?;

    // Install sunspot
    install_sunspot()?;

    // Verify installation
    if command_exists("sunspot") {
        ui::panel_success(
            "SUNSPOT INSTALLED",
            "Sunspot has been installed successfully!\n\nYou may need to restart your terminal or run 'source ~/.zshrc' (or ~/.bashrc) to use it.",
        );
        Ok(())
    } else {
        ui::panel_warning(
            "INSTALLATION COMPLETE",
            "Sunspot has been built. Please restart your terminal or run:\n\n  source ~/.zshrc  (or ~/.bashrc)\n\nThen run 'zklense run' again.",
        );
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Please restart your terminal and try again.",
        ))
    }
}

/// Check prerequisites for sunspot installation (Go, Rust)
fn check_sunspot_prerequisites() -> io::Result<()> {
    ui::section(emoji::SEARCH, "Checking Sunspot Prerequisites");

    let mut missing_prereqs = Vec::new();

    // Check Go (required for sunspot)
    if command_exists("go") {
        println!("  {} {} found", emoji::SUCCESS, style("go").green());
    } else {
        println!("  {} {} not found", emoji::ERROR, style("go").red());
        missing_prereqs.push("go");
    }

    // Check Rust/Cargo (required for gnark-solana verifier)
    if command_exists("cargo") {
        println!("  {} {} found", emoji::SUCCESS, style("cargo/rust").green());
    } else {
        println!("  {} {} not found", emoji::ERROR, style("cargo/rust").red());
        missing_prereqs.push("rust");
    }

    // Check git (required for cloning)
    if command_exists("git") {
        println!("  {} {} found", emoji::SUCCESS, style("git").green());
    } else {
        println!("  {} {} not found", emoji::ERROR, style("git").red());
        missing_prereqs.push("git");
    }

    ui::blank();

    if !missing_prereqs.is_empty() {
        // Offer to install missing prerequisites
        return install_sunspot_prerequisites(&missing_prereqs);
    }

    Ok(())
}

/// Install missing prerequisites for sunspot
fn install_sunspot_prerequisites(missing: &[&str]) -> io::Result<()> {
    ui::panel_warning(
        "MISSING SUNSPOT PREREQUISITES",
        &format!(
            "The following tools are required to install Sunspot:\n\n  • {}\n\nThese must be installed before Sunspot can be built.",
            missing.join("\n  • ")
        ),
    );

    let should_install = ui::confirm_custom(
        "Would you like to install the missing prerequisites?",
        &format!("{} Yes, install prerequisites", emoji::CHECKMARK),
        &format!("{} No, I'll install them manually", emoji::CROSSMARK),
    )?;

    if !should_install {
        ui::info("Please install the following manually:");
        ui::blank();
        for prereq in missing {
            match *prereq {
                "go" => {
                    println!("  {} Go (1.24+): https://go.dev/doc/install", emoji::ARROW_RIGHT);
                    println!("     Or on macOS: brew install go");
                }
                "rust" => {
                    println!("  {} Rust: https://rustup.rs/", emoji::ARROW_RIGHT);
                    println!("     Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
                }
                "git" => {
                    println!("  {} Git: https://git-scm.com/downloads", emoji::ARROW_RIGHT);
                    println!("     Or on macOS: xcode-select --install");
                }
                _ => {}
            }
        }
        ui::blank();
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Missing prerequisites. Install them and try again.",
        ));
    }

    ui::blank();

    for prereq in missing {
        match *prereq {
            "go" => install_go()?,
            "rust" => install_rust()?,
            "git" => {
                ui::panel_error(
                    "GIT REQUIRED",
                    "Git must be installed manually.",
                    None,
                    Some(&[
                        "macOS: xcode-select --install",
                        "Linux: sudo apt install git (Ubuntu) or sudo dnf install git (Fedora)",
                        "Download: https://git-scm.com/downloads",
                    ]),
                );
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Git is required. Please install it manually.",
                ));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Install Go using the appropriate method
fn install_go() -> io::Result<()> {
    ui::section(emoji::PACKAGE, "Installing Go");

    // Check if brew is available (macOS/Linux)
    if command_exists("brew") {
        let spinner = ui::spinner("Installing Go via Homebrew...");

        let output = Command::new("brew")
            .args(["install", "go"])
            .output()?;

        if output.status.success() {
            ui::spinner_success(&spinner, "Go installed via Homebrew");
            return Ok(());
        } else {
            ui::spinner_error(&spinner, "Failed to install Go via Homebrew");
        }
    }

    // Fallback: show manual instructions
    ui::panel_info(
        "INSTALL GO MANUALLY",
        "Please install Go (1.24+) from:\n\nhttps://go.dev/doc/install\n\nOr use your package manager:\n  • macOS: brew install go\n  • Ubuntu: sudo apt install golang-go\n  • Fedora: sudo dnf install golang",
    );

    // Wait for user to confirm they've installed Go
    ui::info("Press Enter after installing Go to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if command_exists("go") {
        ui::success("Go is now available");
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Go is still not found. Please install it and try again.",
        ))
    }
}

/// Install Rust using rustup
fn install_rust() -> io::Result<()> {
    ui::section(emoji::PACKAGE, "Installing Rust");

    let spinner = ui::spinner("Installing Rust via rustup...");

    // Use rustup installer
    let output = Command::new("sh")
        .args(["-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"])
        .output()?;

    if output.status.success() {
        ui::spinner_success(&spinner, "Rust installed via rustup");

        // Source cargo env
        let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
        let cargo_env = format!("{}/.cargo/env", home);
        if std::path::Path::new(&cargo_env).exists() {
            ui::info(&format!("Run 'source {}' or restart your terminal to use Rust", cargo_env));
        }

        Ok(())
    } else {
        ui::spinner_error(&spinner, "Failed to install Rust");

        ui::panel_info(
            "INSTALL RUST MANUALLY",
            "Please install Rust from:\n\nhttps://rustup.rs/\n\nRun:\n  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        );

        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to install Rust. Please install it manually.",
        ))
    }
}

/// Install sunspot from GitHub
fn install_sunspot() -> io::Result<()> {
    ui::section(emoji::ROCKET, "Installing Sunspot");

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let sunspot_dir = format!("{}/sunspot", home);

    // Step 1: Clone the repository
    if !std::path::Path::new(&sunspot_dir).exists() {
        let spinner = ui::spinner("Cloning Sunspot repository...");

        let output = Command::new("git")
            .args(["clone", "https://github.com/reilabs/sunspot.git", &sunspot_dir])
            .output()?;

        if !output.status.success() {
            ui::spinner_error(&spinner, "Failed to clone Sunspot repository");
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to clone repository: {}", stderr),
            ));
        }

        ui::spinner_success(&spinner, "Cloned Sunspot repository");
    } else {
        ui::info(&format!("Sunspot directory already exists at: {}", sunspot_dir));

        // Pull latest changes
        let spinner = ui::spinner("Updating Sunspot repository...");
        let _ = Command::new("git")
            .args(["-C", &sunspot_dir, "pull"])
            .output();
        ui::spinner_success(&spinner, "Updated Sunspot repository");
    }

    // Step 2: Build sunspot
    let spinner = ui::spinner("Building Sunspot (this may take a few minutes)...");

    let go_dir = format!("{}/go", sunspot_dir);
    let output = Command::new("go")
        .args(["build", "-o", "sunspot", "."])
        .current_dir(&go_dir)
        .output()?;

    if !output.status.success() {
        ui::spinner_error(&spinner, "Failed to build Sunspot");
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to build Sunspot: {}", stderr),
        ));
    }

    ui::spinner_success(&spinner, "Built Sunspot");

    // Step 3: Install to PATH
    let spinner = ui::spinner("Installing Sunspot to PATH...");

    let sunspot_binary = format!("{}/go/sunspot", sunspot_dir);

    // Try to move to /usr/local/bin (may require sudo)
    // Use stdin(null) to prevent hanging on password prompt
    let install_result = Command::new("sudo")
        .args(["-n", "mv", &sunspot_binary, "/usr/local/bin/sunspot"])
        .stdin(std::process::Stdio::null())
        .output();

    match install_result {
        Ok(output) if output.status.success() => {
            ui::spinner_success(&spinner, "Installed Sunspot to /usr/local/bin");
        }
        _ => {
            // Fallback: add to ~/bin
            let user_bin = format!("{}/bin", home);
            fs::create_dir_all(&user_bin)?;
            let user_sunspot = format!("{}/sunspot", user_bin);

            fs::copy(&sunspot_binary, &user_sunspot)?;

            // Make executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&user_sunspot)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&user_sunspot, perms)?;
            }

            ui::spinner_success(&spinner, &format!("Installed Sunspot to {}", user_sunspot));

            // Add to PATH in shell config
            add_to_shell_path(&user_bin)?;
        }
    }

    // Step 4: Set GNARK_VERIFIER_BIN environment variable
    let verifier_bin = format!("{}/gnark-solana/crates/verifier-bin", sunspot_dir);
    set_gnark_verifier_bin_env(&verifier_bin)?;

    ui::blank();
    ui::panel_success(
        "SUNSPOT INSTALLATION COMPLETE",
        &format!(
            "Sunspot has been installed!\n\nRepository: {}\nVerifier: {}\n\nPlease restart your terminal or run:\n  source ~/.zshrc  (or ~/.bashrc)",
            sunspot_dir,
            verifier_bin
        ),
    );

    Ok(())
}

/// Add a directory to the shell PATH
fn add_to_shell_path(dir: &str) -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());

    // Determine shell config file
    let shell = std::env::var("SHELL").unwrap_or_default();
    let config_file = if shell.contains("zsh") {
        format!("{}/.zshrc", home)
    } else {
        format!("{}/.bashrc", home)
    };

    let export_line = format!("\n# Added by zklense for Sunspot\nexport PATH=\"{}:$PATH\"\n", dir);

    // Check if already in config
    if let Ok(contents) = fs::read_to_string(&config_file) {
        if contents.contains(dir) {
            return Ok(());
        }
    }

    // Append to config file
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)?;
    file.write_all(export_line.as_bytes())?;

    ui::info(&format!("Added {} to PATH in {}", dir, config_file));

    Ok(())
}

/// Set GNARK_VERIFIER_BIN environment variable in shell config
fn set_gnark_verifier_bin_env(verifier_path: &str) -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());

    // Determine shell config file
    let shell = std::env::var("SHELL").unwrap_or_default();
    let config_file = if shell.contains("zsh") {
        format!("{}/.zshrc", home)
    } else {
        format!("{}/.bashrc", home)
    };

    let export_line = format!("\n# Sunspot GNARK verifier path (added by zklense)\nexport GNARK_VERIFIER_BIN=\"{}\"\n", verifier_path);

    // Check if already in config
    if let Ok(contents) = fs::read_to_string(&config_file) {
        if contents.contains("GNARK_VERIFIER_BIN") {
            return Ok(());
        }
    }

    // Append to config file
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)?;
    file.write_all(export_line.as_bytes())?;

    ui::info(&format!("Set GNARK_VERIFIER_BIN in {}", config_file));

    Ok(())
}

/// Run the full proof generation pipeline
pub fn run_pipeline(path: Option<String>) -> io::Result<()> {
    // Resolve base path
    let base_path = match path {
        Some(p) => {
            let path = Path::new(&p);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir()?.join(path)
            }
        }
        None => std::env::current_dir()?,
    };

    if !base_path.exists() {
        ui::panel_error(
            "PATH NOT FOUND",
            &format!("Path does not exist: {}", base_path.display()),
            None,
            None,
        );
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path does not exist: {}", base_path.display()),
        ));
    }

    // Check for Nargo.toml and read circuit name
    let circuit_name = read_circuit_name(&base_path)?;

    // Header panel
    ui::panel_header(
        emoji::ROCKET,
        "NOIR BUILD PIPELINE",
        Some(&format!(
            "Circuit: {} | Path: {}",
            circuit_name,
            base_path.display()
        )),
    );

    // Check prerequisites
    check_prerequisites()?;

    // Ensure target directory exists (will be created by nargo execute)
    let target_dir = base_path.join(TARGET_DIR);

    // Get pipeline steps
    let steps = get_pipeline_steps();
    let total_steps = steps.len();

    // Print pipeline overview
    ui::section(
        emoji::PIN,
        &format!("Build Pipeline ({} steps)", total_steps),
    );

    for (i, step) in steps.iter().enumerate() {
        println!(
            "  {} [{}] {}",
            emoji::PENDING,
            style(format!("{}/{}", i + 1, total_steps)).dim(),
            style(step.name).dim()
        );
    }
    ui::blank();

    // Execute pipeline
    ui::divider();
    let mut step_durations: Vec<(&str, u128)> = Vec::new();

    for (i, step) in steps.iter().enumerate() {
        let step_num = i + 1;

        let working_dir = if step.working_dir_is_target {
            // For sunspot commands, check that target dir exists
            if !target_dir.exists() {
                ui::panel_error(
                    "TARGET DIRECTORY NOT FOUND",
                    &format!("Target directory not found: {}", target_dir.display()),
                    None,
                    Some(&["Run 'nargo execute' first"]),
                );
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "Target directory not found: {}\nRun 'nargo execute' first.",
                        target_dir.display()
                    ),
                ));
            }
            target_dir.clone()
        } else {
            base_path.clone()
        };

        let args_vec = (step.args_fn)(&circuit_name);
        let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        let step_message = format!("[{}/{}] {}...", step_num, total_steps, step.description);

        let duration = run_command_with_spinner(step.command, &args, &working_dir, &step_message)?;
        step_durations.push((step.name, duration));
    }

    ui::divider();
    ui::blank();

    // Success panel
    let total_duration: u128 = step_durations.iter().map(|(_, d)| d).sum();
    ui::panel_success(
        "BUILD COMPLETE",
        &format!(
            "Pipeline completed successfully in {:.2}s",
            total_duration as f64 / 1000.0
        ),
    );

    // Generated files section
    ui::section(emoji::FOLDER, "Generated Files");

    let file_ccs = format!("{}.ccs", circuit_name);
    let file_pk = format!("{}.pk", circuit_name);
    let file_vk = format!("{}.vk", circuit_name);
    let file_proof = format!("{}.proof", circuit_name);
    let file_pw = format!("{}.pw", circuit_name);
    let file_so = format!("{}.so", circuit_name);

    let files: Vec<(&str, &str)> = vec![
        (&file_ccs, "Compiled circuit"),
        (&file_pk, "Proving key"),
        (&file_vk, "Verifying key"),
        (&file_proof, "Groth16 proof"),
        (&file_pw, "Public witness"),
        (&file_so, "Solana program"),
    ];

    for (file, desc) in &files {
        let file_path = target_dir.join(file);
        let exists = file_path.exists();
        let icon = if exists {
            emoji::SUCCESS
        } else {
            emoji::PENDING
        };
        let file_style = if exists {
            style(*file).green().to_string()
        } else {
            style(*file).dim().to_string()
        };
        println!("  {} {:<20} {}", icon, file_style, style(*desc).dim());
    }
    ui::blank();

    // Prompt user to deploy the Solana program
    let program_path = target_dir.join(format!("{}.so", circuit_name));

    if program_path.exists() {
        ui::section(emoji::ROCKET, "Solana Program Deployment");
        println!(
            "  {} Program file: {}",
            emoji::FILE,
            style(program_path.display()).dim()
        );
        ui::blank();

        // Interactive selection for deployment
        let should_deploy = ui::confirm_custom(
            "Deploy the Solana program?",
            &format!("{} Yes, deploy now", emoji::CHECKMARK),
            &format!("{} No, skip deployment", emoji::CROSSMARK),
        )?;

        if should_deploy {
            ui::blank();

            // Check if solana CLI exists
            if !command_exists("solana") {
                ui::panel_error(
                    "SOLANA CLI NOT FOUND",
                    "The Solana CLI is required to deploy programs.",
                    None,
                    Some(&["Install from: https://docs.solana.com/cli/install-solana-cli-tools"]),
                );
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Solana CLI not found",
                ));
            }

            let output = run_command_capture(
                "solana",
                &["program", "deploy", program_path.to_str().unwrap()],
                &target_dir,
            )?;

            // Parse Program ID from output (format: "Program Id: <address>")
            let program_id = output
                .lines()
                .find(|line| line.contains("Program Id:"))
                .and_then(|line| line.split(':').nth(1))
                .map(|id| id.trim())
                .unwrap_or("Unknown");

            ui::blank();
            ui::panel_success(
                "DEPLOYED",
                &format!(
                    "Solana program deployed successfully!\n\nProgram ID:\n{}",
                    program_id
                ),
            );
        } else {
            ui::info("Deployment skipped. You can deploy later with:");
            println!(
                "  {} solana program deploy {}",
                emoji::ARROW_RIGHT,
                style(program_path.display()).cyan()
            );
            ui::blank();
        }
    } else {
        ui::warn(&format!(
            "No .so file found at: {}",
            style(program_path.display()).dim()
        ));
    }

    Ok(())
}
