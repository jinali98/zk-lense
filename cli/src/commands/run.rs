use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

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

/// Run a command and stream output to stdout
fn run_command(cmd: &str, args: &[&str], working_dir: &Path) -> io::Result<()> {
    println!("  Running: {} {}", cmd, args.join(" "));
    
    let status = Command::new(cmd)
        .args(args)
        .current_dir(working_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command '{}' failed with exit code: {:?}",
                cmd,
                status.code()
            ),
        ));
    }

    Ok(())
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
            name: "execute",
            description: "Running nargo execute",
            command: "nargo",
            args_fn: |_| vec!["execute".to_string()],
            working_dir_is_target: false,
        },
        PipelineStep {
            name: "compile",
            description: "Compiling ACIR to CCS",
            command: "sunspot",
            args_fn: |circuit| vec!["compile".to_string(), format!("{}.json", circuit)],
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "setup",
            description: "Generating proving and verifying keys",
            command: "sunspot",
            args_fn: |circuit| vec!["setup".to_string(), format!("{}.ccs", circuit)],
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "prove",
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
            name: "verify",
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
            name: "deploy",
            description: "Deploying verification program",
            command: "sunspot",
            args_fn: |circuit| {
                vec![
                    "deploy".to_string(),
                    format!("{}.vk", circuit),
                ]
            },
            working_dir_is_target: true,
        },
        PipelineStep {
            name: "deploy",
            description: "Creating Solana verification program",
            command: "sunspot",
            args_fn: |circuit| vec!["deploy".to_string(), format!("{}.vk", circuit)],
            working_dir_is_target: true,
        },
    ]
}

/// Check prerequisites before running the pipeline
fn check_prerequisites() -> io::Result<()> {
    println!("🔍 Checking prerequisites...\n");

    let mut missing = Vec::new();

    // Check nargo
    if command_exists("nargo") {
        println!("  ✅ nargo found");
    } else {
        println!("  ❌ nargo not found");
        missing.push("nargo");
    }

    // Check sunspot
    if command_exists("sunspot") {
        println!("  ✅ sunspot found");
    } else {
        println!("  ❌ sunspot not found");
        missing.push("sunspot");
    }

    println!();

    if !missing.is_empty() {
        let mut error_msg = format!("Missing required commands: {}\n\n", missing.join(", "));
        
        if missing.contains(&"nargo") {
            error_msg.push_str("Install nargo: https://noir-lang.org/docs/getting_started/installation\n");
        }
        if missing.contains(&"sunspot") {
            error_msg.push_str("Install sunspot: https://github.com/solana-foundation/noir-examples\n");
        }

        return Err(io::Error::new(io::ErrorKind::NotFound, error_msg));
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
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path does not exist: {}", base_path.display()),
        ));
    }

    println!("🚀 zkprof run - Noir Circuit Build Pipeline\n");
    println!("📁 Project directory: {}\n", base_path.display());

    // Check for Nargo.toml and read circuit name
    let circuit_name = read_circuit_name(&base_path)?;
    println!("📦 Circuit name: {}\n", circuit_name);

    // Check prerequisites
    check_prerequisites()?;

    // Ensure target directory exists (will be created by nargo execute)
    let target_dir = base_path.join(TARGET_DIR);

    // Get pipeline steps
    let steps = get_pipeline_steps();
    let total_steps = steps.len();

    println!("═══════════════════════════════════════════════════════════════");
    println!("  Starting build pipeline ({} steps)", total_steps);
    println!("═══════════════════════════════════════════════════════════════\n");

    for (i, step) in steps.iter().enumerate() {
        let step_num = i + 1;
        println!(
            "📌 Step {}/{}: {} ({})",
            step_num, total_steps, step.name, step.description
        );
        println!("───────────────────────────────────────────────────────────────");

        let working_dir = if step.working_dir_is_target {
            // For sunspot commands, check that target dir exists
            if !target_dir.exists() {
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

        run_command(step.command, &args, &working_dir)?;

        println!("  ✅ {} completed\n", step.name);
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("  🎉 Build pipeline completed successfully!");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("📂 Generated files in {}:", target_dir.display());
    println!("   • {}.ccs       - Compiled circuit", circuit_name);
    println!("   • {}.pk        - Proving key", circuit_name);
    println!("   • {}.vk        - Verifying key", circuit_name);
    println!("   • {}.proof     - Groth16 proof", circuit_name);
    println!("   • {}.pw        - Public witness", circuit_name);
    println!("   • {}.so        - Solana program", circuit_name);

    // Prompt user to deploy the Solana program
    let program_path = target_dir.join(format!("{}.so", circuit_name));
    
    if program_path.exists() {
        println!("\n🚀 Solana Program Deployment");
        println!("───────────────────────────────────────────────────────────────");
        println!("   Program file: {}", program_path.display());
        print!("\n   Do you want to deploy the Solana program? (y/n): ");
        io::Write::flush(&mut io::stdout())?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let consent = input.trim().to_lowercase();
        if consent == "y" || consent == "yes" {
            println!("\n📤 Deploying Solana program...\n");
            
            // Check if solana CLI exists
            if !command_exists("solana") {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Solana CLI not found. Install it from: https://docs.solana.com/cli/install-solana-cli-tools",
                ));
            }
            
            run_command(
                "solana",
                &["program", "deploy", program_path.to_str().unwrap()],
                &target_dir,
            )?;
            
            println!("\n═══════════════════════════════════════════════════════════════");
            println!("  🎉 Solana program deployed successfully!");
            println!("═══════════════════════════════════════════════════════════════\n");
        } else {
            println!("\n   Deployment skipped.");
        }
    } else {
        println!("\n⚠️  No .so file found at: {}", program_path.display());
        println!("   Skipping deployment prompt.");
    }

    Ok(())
}
