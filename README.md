# zklense CLI

A command-line tool for profiling, building, and deploying zero-knowledge proofs built with [Noir](https://noir-lang.org/) for Solana Blockchain.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Project Structure](#project-structure)
- [Workflow Example](#workflow-example)
- [Metrics Guide](#metrics-guide)
- [Configuration](#configuration)
- [Dependencies](#dependencies)
- [Error Handling](#error-handling)
- [Contributing](#contributing)
- [License](#license)
- [Related Links](#related-links)

## Overview

zklense is developed using Noir and Sunspot to build and deploy zero-knowledge proofs for Solana Blockchain.

zklense streamlines the ZK development workflow by providing:

- 🚀 **Project scaffolding** with pre-built templates for common ZK patterns
  - Age Verifier
  - Merkle Inclusion
  - Hash Preimage
  - Range Proof
- 🔧 **Build pipeline** automation (compile, proof generation, verify, solana deployment)
- 📊 **Proof simulation** and cost analysis for Solana deployment
- 🌐 **Interactive viewer** for profiling reports

## Installation

📦 **For detailed installation instructions, see [INSTALL.md](INSTALL.md)**

### Quick Install Options

- **GitHub Releases** (Recommended): Download pre-built binaries from [Releases](https://github.com/jinali98/zk-profiling-solana/releases)
- **crates.io**: `cargo install zklense` (requires Rust)
- **Homebrew**: `brew tap gihanrcg/zklense && brew install zklense` (macOS/Linux)
- **Scoop**: `scoop install zklense` (Windows)
- **Build from Source**: See [INSTALL.md](INSTALL.md#build-from-source)

### Prerequisites

- **Rust** (1.70+): [Install Rust](https://rustup.rs/) - Only needed for building from source
- **Nargo** (Noir compiler): [Install Noir](https://noir-lang.org/docs/getting_started/installation/)
- **Sunspot** (optional, for Solana deployment): [Sunspot](https://github.com/reilabs/sunspot)
- **Solana CLI** (optional, for deployment): [Install Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)

## Quick Start

```bash
# Create a new Noir project with a template
zklense generate --name my_circuit --template age_verifier

# Navigate to the project
cd my_circuit

# Run the full build pipeline
zklense run

# Simulate on Solana devnet
zklense simulate --program-id <PROGRAM_ID>

# View profiling results
zklense view
```

## Commands

### `zklense generate`

Create a new Noir project with optional templates.

```bash
zklense generate [OPTIONS]

Options:
  -n, --name <NAME>          Project name (prompts if not provided)
  -t, --template <TEMPLATE>  Template: age_verifier, merkle_inclusion, hash_preimage, range_proof or none
```

**Available Templates:**

| Template | Description |
|----------|-------------|
| `none` | Start with default Noir template |
| `age_verifier` | Verify age threshold based on year of birth |
| `merkle_inclusion` | Prove membership in a Merkle tree |
| `hash_preimage` | Prove that a value is the preimage of a hash |
| `range_proof` | Prove that a value is within a range |

**Examples:**

```bash
# Interactive mode
zklense generate

# With arguments
zklense generate --name my_proof --template merkle_inclusion
```

---

### `zklense init`

Initialize zklense in an existing Noir project.

```bash
zklense init [PATH]

Arguments:
  [PATH]  Project path (defaults to current directory)
```

Creates a `.zklense/` directory with configuration:

```
.zklense/
└── config.toml
```

---

### `zklense run`

Run the full proof generation pipeline.

```bash
zklense run [PATH]

Arguments:
  [PATH]  Project path (defaults to current directory)
```

**Pipeline Steps:**

1. **Execute** - Run `nargo execute` to generate witness
2. **Compile** - Convert ACIR to CCS format
3. **Setup** - Generate proving and verifying keys
4. **Prove** - Create Groth16 proof
5. **Verify** - Verify the proof locally
6. **Deploy** - Generate Solana verification program

**Generated Files:**

| File | Description |
|------|-------------|
| `*.ccs` | Compiled circuit |
| `*.pk` | Proving key |
| `*.vk` | Verifying key |
| `*.proof` | Groth16 proof |
| `*.pw` | Public witness |
| `*.so` | Solana program |

---

### `zklense simulate`

Simulate proof verification on Solana devnet and generate a cost analysis report.

```bash
zklense simulate [OPTIONS]

Options:
  -p, --program-id <PROGRAM_ID>  Solana program ID (prompts if not provided)
```

**Report includes:**

- Compute units consumed
- Transaction costs (SOL/lamports)
- Proof and witness sizes
- Priority fee recommendations
- Transaction status and logs

The report is saved to `.zklense/report.json`.

---

### `zklense view`

Open an interactive web viewer for the profiling report.

```bash
zklense view [PATH]

Arguments:
  [PATH]  Project path (defaults to current directory)
```

Starts a local server and opens the report in your browser at [zklenseile.netlify.app](https://zklenseile.netlify.app/).

---

### `zklense version`

Display the current version.

```bash
zklense version
```

## Project Structure

After running `zklense generate` and `zklense run`:

```
my_project/
├── Nargo.toml              # Noir project configuration
├── Prover.toml             # Proof inputs
├── src/
│   └── main.nr             # Circuit code
├── target/
│   ├── my_project.json     # Compiled ACIR
│   ├── my_project.ccs      # Compiled CCS
│   ├── my_project.pk       # Proving key
│   ├── my_project.vk       # Verifying key
│   ├── my_project.proof    # Groth16 proof
│   ├── my_project.pw       # Public witness
│   └── my_project.so       # Solana program
└── .zklense/
    ├── config.toml         # zklense configuration
    └── report.json         # Simulation report
```

## Workflow Example

### 1. Create a New Project

```bash
zklense generate --name age_proof --template age_verifier
cd age_proof
```

### 2. Configure Inputs

Create or edit `Prover.toml`:

```toml
year_of_birth = "1990"
current_year = "2024"
age_threshold = "21"
```

### 3. Build and Deploy

```bash
# Run the full pipeline
zklense run

# When prompted, deploy to Solana devnet
# Save the Program ID that's returned
```

### 4. Analyze Performance

```bash
# Simulate with your deployed program
zklense simulate --program-id <YOUR_PROGRAM_ID>

# View the report
zklense view
```

## Metrics Guide

zklense tracks several metrics to help optimize zero-knowledge proofs for Solana deployment. Understanding these metrics and how to interpret them is important for building efficient and cost-effective ZK applications.

### Compute Units
**What it measures:**
- total_compute_units_consumed: The actual compute units (CU) used during proof verification
- compute_budget: The maximum CU limit set for the transaction (default: 500,000)
- percentage_of_compute_budget_used: Percentage of budget consumed

**What it means:**

Compute units represent the computational resources consumed by your Solana program. Each operation in your circuit consumes a certain amount of CUs.

**How to use it:**

- below 70% usage: ✅ Optimal - You have headroom for future optimizations or additional features
- 70 to 90% usage: ⚠️ Monitor - Consider optimizing if you plan to add more functionality
- above 90% usage: 🔴 Critical - Optimize immediately to avoid transaction failures

**What to change:**
- Simplify circuit logic (reduce constraints, optimize arithmetic operations)
- Reduce the number of public inputs/outputs
- Use more efficient data structures in your Noir code
- Consider splitting complex proofs into multiple smaller proofs


### Proof Metrics
**What it measures:**
- proof_size: Size of the Groth16 proof in bytes
- witness_size: Size of the public witness in bytes
- total_proof_witness_size: Combined size of proof + witness
- cu_per_proof_size: Compute units consumed per byte of proof+witness data

**What it means:**

Smaller proofs reduce transaction size and costs. The CU per proof size ratio indicates how efficiently your proof is processed.

**How to use it:**
- small proof size (< 500 bytes): ✅ Good for simple circuits
- medium proof size (500-1000 bytes): ⚠️ Acceptable, but monitor transaction size limits
- large proof size (> 1000 bytes): 🔴 May approach Solana's transaction size limits (1232 bytes)

**What to change:**
- Optimize your circuit to reduce constraint count (fewer constraints = smaller proofs)
- Minimize public inputs (move data to private inputs when possible)
- Use more efficient hash functions or cryptographic primitives
- Consider proof aggregation techniques for multiple proofs

### Cost Metrics
**What it measures:**
- base_fee: Fixed transaction fee (5,000 lamports = 0.000005 SOL)
- prioritization_fee: Optional fee paid for faster transaction confirmation
- total_fee: Sum of base fee + prioritization fee
- cost_in_sol: Total cost in SOL
- cu_price_microlamports: Price per compute unit (in microlamports)

**What it means:**

Transaction costs determine how expensive it is to verify proofs on-chain. Lower costs make your application more accessible.

**How to use it:**
- no prioritization fee: ⚠️ Transactions may be slower during network congestion
- low prioritization fee: ✅ Good for most use cases
- high prioritization fee: 💰 Consider if faster confirmation is critical

**What to change:**
- Reduce compute units consumed (see Compute Units section)
- Optimize proof size to reduce transaction size
- Set appropriate prioritization fees based on network conditions
- Monitor recent_prioritization_fees in the report to set competitive fees


### Transaction Size
**What it measures:**
- transaction_size: Total serialized transaction size in bytes
- message_size: Size of the transaction message (proof + witness + instructions)
- max_message_size: Solana's maximum message size (1232 bytes)
- message_within_size: Boolean indicating if transaction fits within limits

**What it means:**

Solana has strict transaction size limits. If your transaction exceeds these limits, it will be rejected.

**How to use it:**
- below 1000 bytes: ✅ Safe - Plenty of room
- 1000 to 1200 bytes: ⚠️ Warning - Approaching limit
- above 1232 bytes: 🔴 Critical - Transaction will fail

**What to change:**
- Reduce proof size (see Proof Metrics section)
- Minimize witness size by reducing public inputs
- Optimize instruction data encoding
- Consider using instruction data compression techniques

### Transaction Status
**What it measures:**
- status: "Success" or "Failed"
- error: Error message if transaction failed
- logs: Program execution logs

**What it means:**

Indicates whether your proof verification transaction would succeed on-chain.

**How to use it:**
- Success: ✅ Your proof verification works correctly
- Failed: 🔴 Debug the error message and logs to identify issues

**What to change:**
- Fix circuit logic errors if verification fails
- Check that proof and witness files are correctly generated
- Ensure program ID matches your deployed program
- Review transaction logs for detailed error information

## Configuration

zklense stores configuration in `.zklense/config.toml`:

```toml
[settings]
version = "0.1.0"
initialized_at = "1234567890"
web_app_url = "https://zklenseile.netlify.app/"
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | Command-line argument parsing |
| `dialoguer` | Interactive prompts |
| `console` | Terminal styling |
| `solana-client` | Solana RPC interactions |
| `solana-sdk` | Transaction building |
| `serde` / `serde_json` | Serialization |
| `toml` | Configuration files |
| `webbrowser` | Opening browser for viewer |
| `anyhow` | Error handling |

## Error Handling

zklense provides helpful error messages:

```bash
# Missing nargo
❌ Error: Failed to execute 'nargo new'. Is Nargo installed and in PATH?

# Not initialized
⚠️  zklense is not initialized in: /path/to/project
Would you like to initialize it now? [y/N]:

# Missing proof files
❌ Error: Could not find file with extension .proof
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License


## Related Links

- [Noir Documentation](https://noir-lang.org/docs/)
- [Sunspot Documentation](https://github.com/reilabs/sunspot)
- [Solana Documentation](https://docs.solana.com/)
- [zklense Web Viewer](https://zklenseile.netlify.app/)

