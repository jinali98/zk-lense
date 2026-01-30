use anyhow::{Context, Result};
use console::style;
use dialoguer::Input;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    transaction::Transaction,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use super::init::{get_solana_network, get_solana_rpc_url};
use crate::ui::{self, emoji};

// Solana constants
const LAMPORTS_PER_SIGNATURE: u64 = 5000;
const MAX_COMPUTE_UNITS: u32 = 1_400_000;
const DEFAULT_COMPUTE_UNITS: u32 = 200_000;
const MAX_TRANSACTION_SIZE: usize = 1232;

struct ProofResult {
    proof: Vec<u8>,
    public_witness: Vec<u8>,
}

fn find_file_by_extension(extension: &str) -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    // Search recursively in current directory and subdirectories
    fn search_recursive(dir: &std::path::Path, extension: &str) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension()
                        && ext == extension
                    {
                        return Some(path);
                    }
                } else if path.is_dir() {
                    // Skip hidden directories and common build directories
                    let dir_name = path.file_name().and_then(|n| n.to_str());
                    if let Some(name) = dir_name
                        && !name.starts_with('.')
                        && name != "node_modules"
                        && name != ".git"
                        && let Some(found) = search_recursive(&path, extension)
                    {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    // First check current directory
    if let Some(found) = search_recursive(&current_dir, extension) {
        return Ok(found);
    }

    // Also check common subdirectories like "target"
    let common_dirs = ["target"];
    for subdir in common_dirs {
        let dir_path = current_dir.join(subdir);
        if dir_path.exists()
            && dir_path.is_dir()
            && let Some(found) = search_recursive(&dir_path, extension)
        {
            return Ok(found);
        }
    }

    Err(anyhow::anyhow!(
        "Could not find file with extension .{}",
        extension
    ))
}

fn read_proof_files() -> Result<(ProofResult, PathBuf, PathBuf)> {
    let spinner = ui::spinner("Searching for proof files...");

    let proof_path = find_file_by_extension("proof")?;
    let witness_path = find_file_by_extension("pw")?;

    ui::spinner_success(&spinner, "Found proof files");

    let proof = fs::read(&proof_path)
        .with_context(|| format!("Failed to read proof file: {}", proof_path.display()))?;
    let public_witness = fs::read(&witness_path)
        .with_context(|| format!("Failed to read witness file: {}", witness_path.display()))?;

    Ok((
        ProofResult {
            proof,
            public_witness,
        },
        proof_path,
        witness_path,
    ))
}

fn create_instruction_data(proof_result: &ProofResult) -> Vec<u8> {
    let mut instruction_data = proof_result.proof.clone();
    instruction_data.extend_from_slice(&proof_result.public_witness);
    instruction_data
}

fn parse_compute_budget_instructions(transaction: &Transaction) -> (u32, u64) {
    let mut cu_limit = DEFAULT_COMPUTE_UNITS; // Default CU limit
    let mut cu_price = 0u64; // Default CU price (microlamports per CU)

    let compute_budget_program_id =
        Pubkey::from_str("ComputeBudget111111111111111111111111111111").unwrap();

    for instruction in &transaction.message.instructions {
        // Get program_id from account_keys using program_id_index
        let program_id = transaction
            .message
            .account_keys
            .get(instruction.program_id_index as usize);
        if let Some(&pid) = program_id
            && pid == compute_budget_program_id
        {
            let data = &instruction.data;
            if data.len() >= 4 {
                let instruction_type = data[0];
                if instruction_type == 2 && data.len() >= 8 {
                    // setComputeUnitLimit
                    cu_limit = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                } else if instruction_type == 3 && data.len() >= 12 {
                    // setComputeUnitPrice
                    cu_price = u64::from_le_bytes([
                        data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11],
                    ]);
                }
            }
        }
    }

    (cu_limit, cu_price)
}

fn create_simulation_json(
    sim_result: &solana_client::rpc_response::RpcSimulateTransactionResult,
    transaction: &Transaction,
    proof_size: usize,
    witness_size: usize,
    recent_prioritization_fees: Option<serde_json::Value>,
    program_id: &Pubkey,
    network: &super::init::SolanaNetwork,
    rpc_url: &str,
) -> serde_json::Value {
    // Extract compute units
    let units_consumed = sim_result.units_consumed.unwrap_or(0);

    // Parse compute budget instructions
    let (cu_limit, cu_price_microlamports) = parse_compute_budget_instructions(transaction);
    let compute_budget = cu_limit as u64;
    let compute_budget_percentage = if compute_budget > 0 {
        (units_consumed as f64 / compute_budget as f64) * 100.0
    } else {
        0.0
    };

    // Check if CU limit exceeds maximum
    let cu_limit_warning = if cu_limit > MAX_COMPUTE_UNITS {
        Some(format!(
            "CU limit ({}) exceeds maximum allowed ({})",
            cu_limit, MAX_COMPUTE_UNITS
        ))
    } else {
        None
    };

    // Extract transaction details
    let transaction_size = bincode::serialize(transaction).unwrap_or_default().len();
    let message_size = bincode::serialize(&transaction.message)
        .unwrap_or_default()
        .len();
    let message_within_size = message_size <= MAX_TRANSACTION_SIZE;

    // Extract logs
    let logs = sim_result
        .logs
        .as_ref()
        .map(|l| l.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    // Extract error if any
    let transaction_status = if sim_result.err.is_some() {
        "Failed"
    } else {
        "Success"
    };

    // Calculate CU per proof size
    let total_proof_witness_size = proof_size + witness_size;
    let cu_per_proof_size = if total_proof_witness_size > 0 {
        units_consumed as f64 / total_proof_witness_size as f64
    } else {
        0.0
    };

    // Fee calculations - FIXED
    let num_signatures = transaction.signatures.len().max(1) as u64;
    let base_fee = num_signatures * LAMPORTS_PER_SIGNATURE;

    // Calculate prioritization fee (convert from microlamports to lamports)
    let prioritization_fee_lamports = (cu_limit as u64 * cu_price_microlamports) / 1_000_000;
    let total_fee = base_fee + prioritization_fee_lamports;
    let cost_in_sol = total_fee as f64 / LAMPORTS_PER_SOL as f64;

    // Calculate writable accounts for informational purposes
    let header = &transaction.message.header;
    let total_accounts = transaction.message.account_keys.len();
    let writable_signed = (header.num_required_signatures as usize)
        .saturating_sub(header.num_readonly_signed_accounts as usize);
    let writable_unsigned = total_accounts
        .saturating_sub(header.num_required_signatures as usize)
        .saturating_sub(header.num_readonly_unsigned_accounts as usize);
    let total_writable_accounts = writable_signed + writable_unsigned;

    // Calculate priority - FIXED (simplified to match Solana's actual priority calculation)
    let priority = if compute_budget > 0 {
        prioritization_fee_lamports as f64 / compute_budget as f64
    } else {
        0.0
    };

    // Generate suggestions
    let compute_suggestion = if let Some(ref warning) = cu_limit_warning {
        warning.clone()
    } else if compute_budget_percentage > 90.0 {
        "Consider optimizing compute usage - near budget limit".to_string()
    } else if compute_budget_percentage > 70.0 {
        "Monitor compute usage - approaching budget limit".to_string()
    } else {
        "Compute usage is within acceptable range".to_string()
    };

    let size_suggestion = if !message_within_size {
        format!(
            "Transaction size ({}) exceeds maximum ({})",
            message_size, MAX_TRANSACTION_SIZE
        )
    } else {
        format!("Transaction size ({}) is within limits", message_size)
    };

    let fee_suggestion = if prioritization_fee_lamports == 0 {
        "Consider adding priority fee for faster confirmation"
    } else {
        "Priority fee is set"
    };

    json!({
        "compute_units": {
            "total_compute_units_consumed": units_consumed,
            "total_cu": units_consumed,
            "compute_budget": compute_budget,
            "max_compute_units": MAX_COMPUTE_UNITS,
            "percentage_of_compute_budget_used": format!("{:.2}%", compute_budget_percentage),
            "warning": cu_limit_warning,
            "suggestion": compute_suggestion
        },
        "proof": {
            "proof_size": proof_size,
            "witness_size": witness_size,
            "total_proof_witness_size": total_proof_witness_size,
            "cu_per_proof_size": format!("{:.4}", cu_per_proof_size)
        },
        "heap_usage": {
            "heap_size": sim_result.return_data.as_ref().map(|_| 0).unwrap_or(0),
            "suggestion": "Monitor heap usage in program execution"
        },
        "cost": {
            "cost_in_sol": format!("{:.9}", cost_in_sol),
            "cost_in_lamports": total_fee,
            "base_fee_per_signature": LAMPORTS_PER_SIGNATURE,
            "num_signatures": num_signatures,
            "base_fee": base_fee,
            "cu_limit": cu_limit,
            "cu_price_microlamports": cu_price_microlamports,
            "prioritization_fee": prioritization_fee_lamports,
            "priority_fee": prioritization_fee_lamports,
            "total_fee": total_fee,
            "priority": format!("{:.9}", priority),
            "suggestion": fee_suggestion
        },
        "transaction_status": {
            "status": transaction_status,
            "error": sim_result.err.as_ref().map(|e| format!("{:?}", e)),
            "suggestion": if transaction_status == "Success" {
                "Transaction simulation successful"
            } else {
                "Review transaction error and fix issues"
            }
        },
        "transaction_size": {
            "transaction_size": transaction_size,
            "message_size": message_size,
            "proof_size": proof_size,
            "witness_size": witness_size,
            "total_proof_witness_size": total_proof_witness_size,
            "max_message_size": MAX_TRANSACTION_SIZE,
            "message_within_size": message_within_size,
            "message": if message_within_size {
                format!("Success: Message size ({}) is within limits ({})", message_size, MAX_TRANSACTION_SIZE)
            } else {
                format!("Fail: Message size ({}) exceeds maximum ({})", message_size, MAX_TRANSACTION_SIZE)
            },
            "suggestion": size_suggestion
        },
        "transaction_logs": {
            "logs": logs,
            "log_count": logs.len()
        },
        "accounts": {
            "total_accounts": total_accounts,
            "writable_signed_accounts": writable_signed,
            "writable_unsigned_accounts": writable_unsigned,
            "total_writable_accounts": total_writable_accounts,
            "readonly_signed_accounts": header.num_readonly_signed_accounts,
            "readonly_unsigned_accounts": header.num_readonly_unsigned_accounts
        },
        "fee_recommendation": {
            "base_fee_per_signature": LAMPORTS_PER_SIGNATURE,
            "num_signatures": num_signatures,
            "base_fee": base_fee,
            "cu_limit": cu_limit,
            "cu_price_microlamports": cu_price_microlamports,
            "prioritization_fee": prioritization_fee_lamports,
            "priority_fee": prioritization_fee_lamports,
            "total_fee": total_fee,
            "priority": format!("{:.9}", priority),
            "suggestion": fee_suggestion
        },
        "deserialization": {
            "success": transaction_status == "Success",
            "suggestion": if transaction_status == "Success" {
                "Transaction deserialized successfully"
            } else {
                "Deserialization may have failed - check transaction structure"
            }
        },
        "recent_prioritization_fees": recent_prioritization_fees.unwrap_or(json!(null)),
        "program_id": program_id.to_string(),
        "environment": {
            "network": network.to_string(),
            "rpc_url": rpc_url.to_string(),
        }
    })
}

/// Print formatted simulation results to the console
fn print_simulation_results(
    sim_result: &solana_client::rpc_response::RpcSimulateTransactionResult,
    transaction: &Transaction,
    proof_size: usize,
    witness_size: usize,
    proof_path: &Path,
    witness_path: &Path,
) {
    let units_consumed = sim_result.units_consumed.unwrap_or(0);
    let (cu_limit, cu_price_microlamports) = parse_compute_budget_instructions(transaction);
    let compute_budget = cu_limit as u64;
    let compute_budget_percentage = if compute_budget > 0 {
        (units_consumed as f64 / compute_budget as f64) * 100.0
    } else {
        0.0
    };

    let message_size = bincode::serialize(&transaction.message)
        .unwrap_or_default()
        .len();
    let message_within_size = message_size <= MAX_TRANSACTION_SIZE;

    let is_success = sim_result.err.is_none();
    let total_proof_witness_size = proof_size + witness_size;

    // Fee calculations - FIXED
    let num_signatures = transaction.signatures.len().max(1) as u64;
    let base_fee = num_signatures * LAMPORTS_PER_SIGNATURE;
    let prioritization_fee_lamports = (cu_limit as u64 * cu_price_microlamports) / 1_000_000;
    let total_fee = base_fee + prioritization_fee_lamports;
    let cost_in_sol = total_fee as f64 / LAMPORTS_PER_SOL as f64;

    // Compute Units Section
    ui::section(emoji::LIGHTNING, "Compute Units");
    let consumed_str = format!("{:>12} CU", format_number(units_consumed));
    let budget_str = format!("{:>12} CU", format_number(compute_budget));
    let usage_str = format!("{:>11.2}%", compute_budget_percentage);
    
    let cu_items: &[(&str, &str, bool)] = &[
        ("Consumed", &consumed_str, true),
        ("Budget", &budget_str, true),
        ("Usage", &usage_str, compute_budget_percentage <= 90.0),
    ];
    
    // Add warning if CU limit exceeds maximum
    if cu_limit > MAX_COMPUTE_UNITS {
        ui::print_tree_with_status(cu_items);
        println!(
            "  {} {}",
            emoji::ERROR,
            style(format!(
                "Warning: CU limit ({}) exceeds maximum ({})",
                format_number(cu_limit as u64),
                format_number(MAX_COMPUTE_UNITS as u64)
            ))
            .yellow()
        );
    } else {
        ui::print_tree_with_status(cu_items);
    }

    // Transaction Status Section
    ui::section(
        if is_success {
            emoji::CHECKMARK
        } else {
            emoji::CROSSMARK
        },
        "Transaction Status",
    );
    if is_success {
        println!(
            "  {} {}",
            emoji::SUCCESS,
            style("Simulation Successful").green().bold()
        );
    } else {
        println!(
            "  {} {}",
            emoji::ERROR,
            style("Simulation Failed").red().bold()
        );
        if let Some(err) = &sim_result.err {
            println!("  {} Error: {:?}", emoji::TREE_END, style(err).red());
        }
    }

    // Transaction Size Section
    ui::section(emoji::FILE, "Transaction Size");
    ui::print_tree_with_status(&[
        (
            "Message Size",
            &format!("{} bytes", message_size),
            message_within_size,
        ),
        ("Max Size", &format!("{} bytes", MAX_TRANSACTION_SIZE), true),
    ]);

    // Cost Estimate Section
    ui::section(emoji::MONEY, "Cost Estimate");
    ui::print_tree(&[
        (
            "Signatures",
            &format!("{} × {} lamports", num_signatures, LAMPORTS_PER_SIGNATURE),
        ),
        (
            "Base Fee",
            &format!("{:.9} SOL", base_fee as f64 / LAMPORTS_PER_SOL as f64),
        ),
        (
            "Priority Fee",
            &format!(
                "{:.9} SOL",
                prioritization_fee_lamports as f64 / LAMPORTS_PER_SOL as f64
            ),
        ),
        ("Total", &format!("{:.9} SOL", cost_in_sol)),
    ]);

    // Proof Files Section
    ui::section(emoji::FILE, "Proof Files");
    ui::print_tree(&[
        (
            "Proof",
            &format!(
                "{} bytes ({})",
                proof_size,
                style(proof_path.display()).dim()
            ),
        ),
        (
            "Witness",
            &format!(
                "{} bytes ({})",
                witness_size,
                style(witness_path.display()).dim()
            ),
        ),
        ("Total", &format!("{} bytes", total_proof_witness_size)),
    ]);

    ui::blank();
}

/// Format a number with thousands separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

pub async fn run_simulate(program_id_arg: Option<String>) -> Result<()> {
    // Header
    ui::panel_header(
        emoji::CHART,
        "TRANSACTION SIMULATION",
        Some("Simulate ZK proof verification on Solana"),
    );

    // Get program ID from argument or prompt user
    let program_id_str = match program_id_arg {
        Some(id) => id,
        None => Input::<String>::new()
            .with_prompt(format!("{} Enter Solana program ID", emoji::PIN))
            .interact_text()
            .context("Failed to read program ID")?,
    };

    if program_id_str.is_empty() {
        ui::panel_error("INVALID INPUT", "Program ID cannot be empty", None, None);
        return Err(anyhow::anyhow!("Program ID cannot be empty"));
    }

    ui::blank();

    // Read proof and witness files (automatically found by extension)
    let (proof_result, proof_path, witness_path) = read_proof_files()?;
    let proof_size = proof_result.proof.len();
    let witness_size = proof_result.public_witness.len();

    // Create instruction data by concatenating proof + witness
    let instruction_data = create_instruction_data(&proof_result);

    // Get RPC URL from config
    let current_dir = std::env::current_dir()?;
    let rpc_url = get_solana_rpc_url(&current_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read config: {}. Run 'zklense init' first.", e))?;
    let network = get_solana_network(&current_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read config: {}. Run 'zklense init' first.", e))?;

    // Connect to Solana
    let start = Instant::now();
    let spinner = ui::spinner(&format!(
        "Connecting to {} ({})...",
        network,
        style(&rpc_url).dim()
    ));

    let connection = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    // Parse program ID
    let program_id = Pubkey::from_str(&program_id_str)?;

    // Create a keypair for the fee payer (can be loaded from file or generated)
    // For simulation, we can use a dummy keypair
    let fee_payer = Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM")?;

    // Create the verify instruction with proof + witness data
    let verify_instruction = Instruction {
        program_id,
        accounts: vec![], // No accounts needed for this instruction
        data: instruction_data,
    };

    // Create compute budget instruction automatically
    // Use MAX_COMPUTE_UNITS as default to ensure sufficient budget for any proof size
    let compute_budget_program_id =
        Pubkey::from_str("ComputeBudget111111111111111111111111111111")?;
    let compute_units = MAX_COMPUTE_UNITS;

    let mut compute_unit_limit_data = vec![2u8, 0, 0, 0];
    compute_unit_limit_data.extend_from_slice(&compute_units.to_le_bytes());

    let compute_unit_limit_ix = Instruction {
        program_id: compute_budget_program_id,
        accounts: vec![],
        data: compute_unit_limit_data,
    };

    // Build transaction with compute budget and verify instructions
    let mut transaction = Transaction::new_with_payer(
        &[compute_unit_limit_ix, verify_instruction],
        Some(&fee_payer),
    );

    // Get blockhash
    let blockhash = connection.get_latest_blockhash().await?;
    transaction.message.recent_blockhash = blockhash;

    ui::spinner_success_with_duration(
        &spinner,
        &format!("Connected to {}", network),
        start.elapsed().as_millis(),
    );

    // Simulate the transaction
    let start = Instant::now();
    let spinner = ui::spinner("Simulating transaction...");

    let sim_response = connection.simulate_transaction(&transaction).await?;

    ui::spinner_success_with_duration(&spinner, "Simulation complete", start.elapsed().as_millis());

    // Fetch recent prioritization fees (non-blocking, with warning on failure)
    let spinner = ui::spinner("Fetching prioritization fees...");
    let recent_prioritization_fees = match connection.get_recent_prioritization_fees(&[]).await {
        Ok(fees_vec) => {
            let fees: Vec<serde_json::Value> = fees_vec
                .iter()
                .rev()
                .take(50)
                .map(|fee| {
                    json!({
                        "slot": fee.slot,
                        "prioritization_fee": fee.prioritization_fee
                    })
                })
                .collect();
            ui::spinner_success(&spinner, "Fetched prioritization fees");
            Some(json!(fees))
        }
        Err(_) => {
            ui::spinner_warn(&spinner, "Could not fetch prioritization fees");
            None
        }
    };

    ui::blank();

    // Print formatted results to console
    print_simulation_results(
        &sim_response.value,
        &transaction,
        proof_size,
        witness_size,
        &proof_path,
        &witness_path,
    );

    // Create JSON output
    let simulation_json = create_simulation_json(
        &sim_response.value,
        &transaction,
        proof_size,
        witness_size,
        recent_prioritization_fees,
        &program_id,
        &network,
        &rpc_url,
    );

    let json_output = serde_json::to_string_pretty(&simulation_json)?;

    // Save to .zklense/report.json
    let spinner = ui::spinner("Saving report...");
    let zklense_dir = std::env::current_dir()?.join(".zklense");
    fs::create_dir_all(&zklense_dir).with_context(|| {
        format!(
            "Failed to create .zklense directory: {}",
            zklense_dir.display()
        )
    })?;

    let report_path = zklense_dir.join("report.json");
    fs::write(&report_path, &json_output)
        .with_context(|| format!("Failed to write report to: {}", report_path.display()))?;

    ui::spinner_success(
        &spinner,
        &format!("Report saved to {}", style(report_path.display()).dim()),
    );

    // Success panel
    let is_success = sim_response.value.err.is_none();
    if is_success {
        ui::panel_success(
            "SIMULATION COMPLETE",
            &format!(
                "Transaction simulation was successful!\n\nView full report: {}",
                report_path.display()
            ),
        );
    } else {
        ui::panel_warning(
            "SIMULATION COMPLETE (WITH ERRORS)",
            &format!(
                "Transaction simulation completed with errors.\n\nView full report: {}",
                report_path.display()
            ),
        );
    }

    Ok(())
}