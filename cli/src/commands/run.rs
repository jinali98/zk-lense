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
