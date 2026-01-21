use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    transaction::Transaction,
};
use serde_json::json;
use std::str::FromStr;
use std::fs;
use std::path::PathBuf;

struct ProofResult {
    proof: Vec<u8>,
    public_witness: Vec<u8>,
}

fn find_file_by_extension(extension: &str) -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    
    // Search recursively in current directory and subdirectories
    fn search_recursive(dir: &std::path::Path, extension: &str) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == extension {
                                return Some(path);
                            }
                        }
                    } else if path.is_dir() {
                        // Skip hidden directories and common build directories that are unlikely to contain proof files
                        let dir_name = path.file_name().and_then(|n| n.to_str());
                        if let Some(name) = dir_name {
                            if !name.starts_with('.') && name != "node_modules" && name != ".git" {
                                if let Some(found) = search_recursive(&path, extension) {
                                    return Some(found);
                                }
                            }
                        }
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
        if dir_path.exists() && dir_path.is_dir() {
            if let Some(found) = search_recursive(&dir_path, extension) {
                return Ok(found);
            }
        }
    }
    
    Err(anyhow::anyhow!("Could not find file with extension .{}", extension))
}

fn read_proof_files() -> Result<ProofResult> {
    let proof_path = find_file_by_extension("proof")?;
    let witness_path = find_file_by_extension("pw")?;
    
    println!("Found proof file: {}", proof_path.display());
    println!("Found witness file: {}", witness_path.display());
    
    let proof = fs::read(&proof_path)
        .with_context(|| format!("Failed to read proof file: {}", proof_path.display()))?;
    let public_witness = fs::read(&witness_path)
        .with_context(|| format!("Failed to read witness file: {}", witness_path.display()))?;
    
    Ok(ProofResult {
        proof,
        public_witness,
    })
}

fn create_instruction_data(proof_result: &ProofResult) -> Vec<u8> {
    let mut instruction_data = proof_result.proof.clone();
    instruction_data.extend_from_slice(&proof_result.public_witness);
    instruction_data
}

fn parse_compute_budget_instructions(transaction: &Transaction) -> (u32, u64) {
    let mut cu_limit = 200_000u32; // Default CU limit
    let mut cu_price = 0u64; // Default CU price (microlamports per CU)
    
    let compute_budget_program_id = Pubkey::from_str("ComputeBudget111111111111111111111111111111")
        .unwrap();
    
    for instruction in &transaction.message.instructions {
        // Get program_id from account_keys using program_id_index
        let program_id = transaction.message.account_keys.get(instruction.program_id_index as usize);
        if let Some(&pid) = program_id {
            if pid == compute_budget_program_id {
                let data = &instruction.data;
                if data.len() >= 4 {
                    let instruction_type = data[0];
                    if instruction_type == 2 && data.len() >= 8 {
                        // setComputeUnitLimit
                        cu_limit = u32::from_le_bytes([
                            data[4], data[5], data[6], data[7]
                        ]);
                    } else if instruction_type == 3 && data.len() >= 12 {
                        // setComputeUnitPrice
                        cu_price = u64::from_le_bytes([
                            data[4], data[5], data[6], data[7],
                            data[8], data[9], data[10], data[11],
                        ]);
                    }
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

    // Extract transaction details
    let transaction_size = bincode::serialize(transaction).unwrap_or_default().len();
    let message_size = bincode::serialize(&transaction.message).unwrap_or_default().len();
    let max_message_size = 1232; // Solana max transaction size
    let message_within_size = message_size <= max_message_size;
    
    // Extract logs
    let logs = sim_result.logs.as_ref().map(|l| {
        l.iter().map(|s| s.as_str()).collect::<Vec<_>>()
    }).unwrap_or_default();

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

    // Fee calculations
    let base_fee = 5000u64; // Base fee in lamports
    
    // Calculate prioritization fee (convert from microlamports to lamports)
    let prioritization_fee_lamports = (cu_limit as u64 * cu_price_microlamports) / 1_000_000;
    let total_fee = base_fee + prioritization_fee_lamports;
    let cost_in_sol = total_fee as f64 / LAMPORTS_PER_SOL as f64;
    
    // Calculate write lock CUs
    let header = &transaction.message.header;
    let total_accounts = transaction.message.account_keys.len();
    let writable_signed = (header.num_required_signatures as usize)
        .saturating_sub(header.num_readonly_signed_accounts as usize);
    let writable_unsigned = total_accounts
        .saturating_sub(header.num_required_signatures as usize)
        .saturating_sub(header.num_readonly_unsigned_accounts as usize);
    let write_lock_cus = writable_signed + writable_unsigned;
    let signature_cus = 0u64;
    
    // Calculate priority
    let priority_numerator = prioritization_fee_lamports + base_fee;
    let priority_denominator = 1 + compute_budget + signature_cus + write_lock_cus as u64;
    let priority = if priority_denominator > 0 {
        priority_numerator as f64 / priority_denominator as f64
    } else {
        0.0
    };

    // Generate suggestions
    let compute_suggestion = if compute_budget_percentage > 90.0 {
        "Consider optimizing compute usage - near budget limit"
    } else if compute_budget_percentage > 70.0 {
        "Monitor compute usage - approaching budget limit"
    } else {
        "Compute usage is within acceptable range"
    };

    let size_suggestion = if !message_within_size {
        format!("Transaction size ({}) exceeds maximum ({})", message_size, max_message_size)
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
            "percentage_of_compute_budget_used": format!("{:.2}%", compute_budget_percentage),
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
            "gas_fee": base_fee,
            "base_fee": base_fee,
            "cu_limit": cu_limit,
            "cu_price_microlamports": cu_price_microlamports,
            "prioritization_fee": prioritization_fee_lamports,
            "priority_fee": prioritization_fee_lamports,
            "total_fee": total_fee,
            "priority": format!("{:.9}", priority),
            "signature_cus": signature_cus,
            "write_lock_cus": write_lock_cus,
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
            "max_message_size": max_message_size,
            "message_within_size": message_within_size,
            "message": if message_within_size {
                format!("Success: Message size ({}) is within limits ({})", message_size, max_message_size)
            } else {
                format!("Fail: Message size ({}) exceeds maximum ({})", message_size, max_message_size)
            },
            "suggestion": size_suggestion
        },
        "transaction_logs": {
            "logs": logs,
            "log_count": logs.len()
        },
        "fee_recommendation": {
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
        "recent_prioritization_fees": recent_prioritization_fees.unwrap_or(json!(null))
    })
}

pub async fn run_simulate() -> Result<()> {
    let program_id_str = "68V29RzWpHhYS5qdu9ovcAfDCLeddhYMH7vafy53PvdB";
    
    // Read proof and witness files (automatically found by extension)
    let proof_result = read_proof_files()?;
    let proof_size = proof_result.proof.len();
    let witness_size = proof_result.public_witness.len();
    
    println!("Proof size: {} bytes", proof_size);
    println!("Witness size: {} bytes", witness_size);
    
    // Create instruction data by concatenating proof + witness
    let instruction_data = create_instruction_data(&proof_result);
    println!("Total instruction data: {} bytes\n", instruction_data.len());

    // Create a connection to cluster
    let connection = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Parse program ID
    let program_id = Pubkey::from_str(program_id_str)?;
    
    // Create a keypair for the fee payer (can be loaded from file or generated)
    // For simulation, we can use a dummy keypair
    let fee_payer = Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM")?;

    // Create the verify instruction with proof + witness data
    let verify_instruction = Instruction {
        program_id,
        accounts: vec![], // No accounts needed for this instruction
        data: instruction_data,
    };

    // Create compute budget instruction manually
    // Compute Budget Program ID: ComputeBudget111111111111111111111111111111
    let compute_budget_program_id = Pubkey::from_str("ComputeBudget111111111111111111111111111111")?;
    let compute_units = 500_000u32;
    
    // setComputeUnitLimit instruction: [2, 0, 0, 0] + u32_le_bytes
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
    
    let blockhash = connection.get_latest_blockhash().await?;
    transaction.message.recent_blockhash = blockhash;

    // Simulate the transaction
    let sim_response = connection
        .simulate_transaction(&transaction)
        .await?;

    let recent_prioritization_fees = match connection
        .get_recent_prioritization_fees(&[])
        .await
    {
        Ok(fees_vec) => {
            // Take only the last 50 entries
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
            Some(json!(fees))
        }
        Err(e) => {
            eprintln!("Warning: Could not fetch recent prioritization fees: {}", e);
            None
        }
    };

    // Create JSON output with proof and witness sizes
    let simulation_json = create_simulation_json(
        &sim_response.value,
        &transaction,
        proof_size,
        witness_size,
        recent_prioritization_fees,
    );

    // Print formatted JSON
    let json_output = serde_json::to_string_pretty(&simulation_json)?;
    println!("{}", json_output);

    // Save to .zkproof/report.json
    let zkproof_dir = std::env::current_dir()?.join(".zkproof");
    fs::create_dir_all(&zkproof_dir)
        .with_context(|| format!("Failed to create .zkproof directory: {}", zkproof_dir.display()))?;
    
    let report_path = zkproof_dir.join("report.json");
    fs::write(&report_path, &json_output)
        .with_context(|| format!("Failed to write report to: {}", report_path.display()))?;
    
    println!("\nReport saved to: {}", report_path.display());

    Ok(())
}

