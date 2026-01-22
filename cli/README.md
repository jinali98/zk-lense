# zkprof CLI

A command-line tool for profiling, building, and deploying zero-knowledge proofs built with [Noir](https://noir-lang.org/) for Solana Blockchain.

## Overview

zkprof is developed using Noir and Sunspot to build and deploy zero-knowledge proofs for Solana Blockchain.

zkprof streamlines the ZK development workflow by providing:

- 🚀 **Project scaffolding** with pre-built templates for common ZK patterns
  - Age Verifier
  - Merkle Inclusion
- 🔧 **Build pipeline** automation (compile, proof generation, verify, solana deployment)
- 📊 **Proof simulation** and cost analysis for Solana deployment
- 🌐 **Interactive viewer** for profiling reports

## Installation

### Prerequisites

- **Rust** (1.70+): [Install Rust](https://rustup.rs/)
- **Nargo** (Noir compiler): [Install Noir](https://noir-lang.org/docs/getting_started/installation/)
- **Sunspot** (optional, for Solana deployment): [Sunspot](https://github.com/solana-foundation/noir-examples)
- **Solana CLI** (optional, for deployment): [Install Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)

### Build from Source

```bash
cd cli
cargo build --release
```

The binary will be available at `target/release/zkprof`.

### Add to PATH

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="$PATH:/path/to/zkprof/cli/target/release"
```

## Quick Start

```bash
# Create a new Noir project with a template
zkprof generate --name my_circuit --template age_verifier

# Navigate to the project
cd my_circuit

# Run the full build pipeline
zkprof run

# Simulate on Solana devnet
zkprof simulate --program-id <PROGRAM_ID>

# View profiling results
zkprof view
```

## Commands

### `zkprof generate`

Create a new Noir project with optional templates.

```bash
zkprof generate [OPTIONS]

Options:
  -n, --name <NAME>          Project name (prompts if not provided)
  -t, --template <TEMPLATE>  Template: age_verifier, merkle_inclusion, or none
```

**Available Templates:**

| Template | Description |
|----------|-------------|
| `none` | Start with default Noir template |
| `age_verifier` | Verify age threshold based on year of birth |
| `merkle_inclusion` | Prove membership in a Merkle tree |

**Examples:**

```bash
# Interactive mode
zkprof generate

# With arguments
zkprof generate --name my_proof --template merkle_inclusion
```

---

### `zkprof init`

Initialize zkprof in an existing Noir project.

```bash
zkprof init [PATH]

Arguments:
  [PATH]  Project path (defaults to current directory)
```

Creates a `.zkproof/` directory with configuration:

```
.zkproof/
└── config.toml
```

---

### `zkprof run`

Run the full proof generation pipeline.

```bash
zkprof run [PATH]

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

### `zkprof simulate`

Simulate proof verification on Solana devnet and generate a cost analysis report.

```bash
zkprof simulate [OPTIONS]

Options:
  -p, --program-id <PROGRAM_ID>  Solana program ID (prompts if not provided)
```

**Report includes:**

- Compute units consumed
- Transaction costs (SOL/lamports)
- Proof and witness sizes
- Priority fee recommendations
- Transaction status and logs

The report is saved to `.zkproof/report.json`.

---

### `zkprof view`

Open an interactive web viewer for the profiling report.

```bash
zkprof view [PATH]

Arguments:
  [PATH]  Project path (defaults to current directory)
```

Starts a local server and opens the report in your browser at [zkprofile.netlify.app](https://zkprofile.netlify.app/).

---

### `zkprof version`

Display the current version.

```bash
zkprof version
```

## Project Structure

After running `zkprof generate` and `zkprof run`:

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
└── .zkproof/
    ├── config.toml         # zkprof configuration
    └── report.json         # Simulation report
```

## Workflow Example

### 1. Create a New Project

```bash
zkprof generate --name age_proof --template age_verifier
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
zkprof run

# When prompted, deploy to Solana devnet
# Save the Program ID that's returned
```

### 4. Analyze Performance

```bash
# Simulate with your deployed program
zkprof simulate --program-id <YOUR_PROGRAM_ID>

# View the report
zkprof view
```

## Configuration

zkprof stores configuration in `.zkproof/config.toml`:

```toml
[settings]
version = "0.1.0"
initialized_at = "1234567890"
web_app_url = "https://zkprofile.netlify.app/"
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

zkprof provides helpful error messages:

```bash
# Missing nargo
❌ Error: Failed to execute 'nargo new'. Is Nargo installed and in PATH?

# Not initialized
⚠️  zkprof is not initialized in: /path/to/project
Would you like to initialize it now? [y/N]:

# Missing proof files
❌ Error: Could not find file with extension .proof
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

[Add your license here]

## Related Links

- [Noir Documentation](https://noir-lang.org/docs/)
- [Sunspot Documentation](https://github.com/reilabs/sunspot)
- [Solana Documentation](https://docs.solana.com/)
- [zkprof Web Viewer](https://zkprofile.netlify.app/)

