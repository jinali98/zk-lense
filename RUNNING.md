# Running Instructions

Complete guide for running zklense CLI with all available commands and parameters.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Command Overview](#command-overview)
- [Command Reference](#command-reference)
  - [version](#version)
  - [init](#init)
  - [generate](#generate)
  - [run](#run)
  - [simulate](#simulate)
  - [view](#view)
  - [config](#config)
- [Complete Workflow Examples](#complete-workflow-examples)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

Before running zklense commands, ensure you have:

- **zklense CLI** installed (see [INSTALL.md](INSTALL.md))
- **Nargo** (Noir compiler) installed and in PATH
- **Sunspot** (optional, for Solana deployment) installed and in PATH
- **Solana CLI** (optional, for program deployment) installed and in PATH

Verify installations:

```bash
# Check zklense
zklense --version

# Check nargo
nargo --version

# Check sunspot (if needed)
sunspot --version

# Check solana CLI (if needed)
solana --version
```

---

## Command Overview

| Command | Description | Required Initialization |
|---------|-------------|------------------------|
| `version` | Display version information | No |
| `init` | Initialize zklense in a project | No |
| `generate` | Create a new Noir project | No |
| `run` | Execute full build pipeline | Yes |
| `simulate` | Simulate proof on Solana | Yes |
| `view` | View profiling report | Yes |
| `config` | Manage configuration | Yes |

---

## Command Reference

### version

Display the current version of zklense.

**Usage:**
```bash
zklense version
```

**Options:**
- `-h, --help` - Print help information

**Examples:**
```bash
# Display version
zklense version

# Alternative using global flag
zklense --version
```

**Output:**
```
zklense 0.1.0
```

---

### init

Initialize zklense in an existing Noir project. Creates a `.zklense/` directory with configuration files.

**Usage:**
```bash
zklense init [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory if not specified.
  - Can be relative: `./my_project` or `../parent/project`
  - Can be absolute: `/home/user/projects/my_project`

**Options:**
- `-h, --help` - Print help information

**Examples:**
```bash
# Initialize in current directory
zklense init

# Initialize in specific directory (relative path)
zklense init ./my_noir_project

# Initialize in specific directory (absolute path)
zklense init /path/to/my_noir_project

# Initialize in parent directory
zklense init ..
```

**What it does:**
- Creates `.zklense/` directory
- Creates `.zklense/config.toml` with default configuration:
  - Network: `devnet`
  - RPC URL: `https://api.devnet.solana.com`
  - Web App URL: `https://zklense.netlify.app/`
  - Version: `0.1.0`

**Output:**
```
✓ Created directory: .zklense
✓ Created config file: .zklense/config.toml

INITIALIZED
zklense initialized successfully!
Project: /path/to/project

Configuration:
  Network: devnet
  RPC URL: https://api.devnet.solana.com
  Web App: https://zklense.netlify.app/
  Version: 0.1.0
```

**Notes:**
- If `.zklense/` already exists, it will check for `config.toml` and recreate it if missing
- The command will not overwrite existing configuration

---

### generate

Create a new Noir project with optional templates. This command uses `nargo new` under the hood and optionally applies a template to the generated project.

**Usage:**
```bash
zklense generate [OPTIONS]
```

**Options:**
- `-n, --name <NAME>` - Name of the new Noir project
  - If not provided, prompts interactively
  - Must be a valid directory name
- `-t, --template <TEMPLATE>` - Template to use
  - Valid values: `age_verifier`, `merkle_inclusion`, or `none`
  - If not provided, prompts interactively
  - `none` - Start with default Noir template
  - `age_verifier` - Verify age threshold based on year of birth
  - `merkle_inclusion` - Prove membership in a Merkle tree
- `-h, --help` - Print help information

**Examples:**
```bash
# Interactive mode (prompts for name and template)
zklense generate

# Specify name only (prompts for template)
zklense generate --name my_circuit

# Specify template only (prompts for name)
zklense generate --template age_verifier

# Specify both name and template
zklense generate --name age_proof --template age_verifier

# Use short flags
zklense generate -n merkle_proof -t merkle_inclusion

# Create project with default Noir template
zklense generate --name my_project --template none
```

**What it does:**
1. Creates a new Noir project using `nargo new <name>`
2. If a template is selected, replaces `src/main.nr` with template content
3. Optionally prompts to initialize zklense in the new project

**Output:**
```
✨ CREATE NEW NOIR PROJECT
Generate a new Noir circuit with optional templates

📦 Project name: my_circuit
📄 Select a template:
  ⏳ None - Start with default Noir template
  📄 Age Verifier - Verify age threshold based on year of birth
  📄 Merkle Inclusion Proof - Prove membership in a Merkle tree
  📄 Hash Preimage Proof - Prove that a value is the preimage of a hash
  📄 Range Proof - Prove that a value is within a range

✓ Created Noir project: my_circuit
✓ Applied template: Age Verifier

PROJECT CREATED
Noir project 'my_circuit' created successfully!

Initialize zklense in this project?
  ✓ Yes, initialize zklense
  ✗ No, skip for now

💡 Next Steps
  1. cd my_circuit
  2. nargo check    # Verify the project compiles
  3. nargo prove     # Generate a proof
```

**Notes:**
- Requires `nargo` to be installed and in PATH
- The project is created in the current directory
- After creation, you can optionally run `zklense init` if not done automatically

---

### run

Run the full proof generation pipeline. This command executes all steps needed to build, prove, verify, and prepare a Solana program from a Noir circuit.

**Usage:**
```bash
zklense run [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory if not specified.
  - Can be relative: `./my_project` or `../parent/project`
  - Can be absolute: `/home/user/projects/my_project`

**Options:**
- `-h, --help` - Print help information

**Prerequisites:**
- Project must be initialized (run `zklense init` first)
- `Nargo.toml` must exist in the project directory
- `nargo` must be installed and in PATH
- `sunspot` must be installed and in PATH (for steps 2-6)

**Pipeline Steps:**

1. **Execute** - Run `nargo execute` to generate witness
   - Working directory: Project root
   - Generates: `target/<circuit_name>.json` (ACIR format)

2. **Compile** - Convert ACIR to CCS format
   - Working directory: `target/`
   - Command: `sunspot compile <circuit>.json`
   - Generates: `target/<circuit_name>.ccs`

3. **Setup** - Generate proving and verifying keys
   - Working directory: `target/`
   - Command: `sunspot setup <circuit>.ccs`
   - Generates: `target/<circuit_name>.pk` (proving key)
   - Generates: `target/<circuit_name>.vk` (verifying key)

4. **Prove** - Create Groth16 proof
   - Working directory: `target/`
   - Command: `sunspot prove <circuit>.json <circuit>.gz <circuit>.ccs <circuit>.pk`
   - Generates: `target/<circuit_name>.proof`
   - Generates: `target/<circuit_name>.pw` (public witness)

5. **Verify** - Verify the proof locally
   - Working directory: `target/`
   - Command: `sunspot verify <circuit>.vk <circuit>.proof <circuit>.pw`
   - Validates: Proof correctness

6. **Deploy** - Generate Solana verification program
   - Working directory: `target/`
   - Command: `sunspot deploy <circuit>.vk`
   - Generates: `target/<circuit_name>.so` (Solana program binary)

**Examples:**
```bash
# Run pipeline in current directory
zklense run

# Run pipeline in specific directory (relative)
zklense run ./my_project

# Run pipeline in specific directory (absolute)
zklense run /path/to/my_project

# Run pipeline in parent directory
zklense run ..
```

**Generated Files:**

All files are created in the `target/` directory:

| File Extension | Description | Generated By |
|----------------|-------------|--------------|
| `.json` | Compiled ACIR (Abstract Circuit Intermediate Representation) | Step 1: Execute |
| `.ccs` | Compiled CCS (Constraint System) | Step 2: Compile |
| `.pk` | Proving key | Step 3: Setup |
| `.vk` | Verifying key | Step 3: Setup |
| `.proof` | Groth16 proof | Step 4: Prove |
| `.pw` | Public witness | Step 4: Prove |
| `.so` | Solana program binary | Step 6: Deploy |

**Output:**
```
🚀 NOIR BUILD PIPELINE
Circuit: my_circuit | Path: /path/to/project

🔍 Checking Prerequisites
  ✓ nargo found
  ✓ sunspot found

📍 Build Pipeline (6 steps)
  ⏳ [1/6] Execute
  ⏳ [2/6] Compile
  ⏳ [3/6] Setup
  ⏳ [4/6] Prove
  ⏳ [5/6] Verify
  ⏳ [6/6] Deploy

─────────────────────────────────────
[1/6] Running nargo execute... ✓ (1234ms)
[2/6] Compiling ACIR to CCS... ✓ (567ms)
[3/6] Generating proving and verifying keys... ✓ (2341ms)
[4/6] Creating Groth16 proof... ✓ (3456ms)
[5/6] Verifying proof... ✓ (123ms)
[6/6] Creating Solana verification program... ✓ (890ms)
─────────────────────────────────────

BUILD COMPLETE
Pipeline completed successfully in 8.61s

📁 Generated Files
  ✓ my_circuit.ccs        Compiled circuit
  ✓ my_circuit.pk         Proving key
  ✓ my_circuit.vk         Verifying key
  ✓ my_circuit.proof      Groth16 proof
  ✓ my_circuit.pw         Public witness
  ✓ my_circuit.so         Solana program

🚀 Solana Program Deployment
  📄 Program file: target/my_circuit.so

Deploy the Solana program?
  ✓ Yes, deploy now
  ✗ No, skip deployment
```

**Optional Deployment:**

After the pipeline completes, you can optionally deploy the `.so` file to Solana:

- Requires `solana` CLI to be installed
- Prompts interactively: "Deploy the Solana program?"
- If yes, runs: `solana program deploy <circuit>.so`
- Returns Program ID on successful deployment

**Error Handling:**

- If project is not initialized, prompts to initialize
- If `Nargo.toml` is missing, shows error with path
- If `nargo` or `sunspot` is missing, lists missing commands with installation links
- If any step fails, shows error message and exits

---

### simulate

Simulate proof verification on Solana network and generate a comprehensive cost analysis report. This command reads proof and witness files, creates a transaction, and simulates it on the configured Solana network.

**Usage:**
```bash
zklense simulate [OPTIONS]
```

**Options:**
- `-p, --program-id <PROGRAM_ID>` - Solana program ID to simulate against
  - Must be a valid Solana public key (base58 encoded)
  - If not provided, prompts interactively
  - Example: `9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM`
- `-h, --help` - Print help information

**Prerequisites:**
- Project must be initialized (run `zklense init` first)
- Proof file (`.proof`) must exist in the project
- Witness file (`.pw`) must exist in the project
- Files are automatically searched recursively in the project directory

**Examples:**
```bash
# Interactive mode (prompts for program ID)
zklense simulate

# Specify program ID
zklense simulate --program-id 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM

# Use short flag
zklense simulate -p 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
```

**What it does:**
1. Searches for `.proof` and `.pw` files in the project directory
2. Reads proof and witness data
3. Creates a Solana transaction with:
   - Compute budget instruction (500,000 CU limit)
   - Verify instruction with proof + witness data
4. Connects to configured Solana network (from `.zklense/config.toml`)
5. Simulates the transaction
6. Fetches recent prioritization fees
7. Generates comprehensive report
8. Saves report to `.zklense/report.json`

**Report Contents:**

The report includes detailed metrics:

**Compute Units:**
- `total_compute_units_consumed` - Actual CUs used
- `total_cu` - Same as above
- `compute_budget` - Maximum CU limit (500,000)
- `percentage_of_compute_budget_used` - Usage percentage
- `suggestion` - Optimization recommendations

**Proof Metrics:**
- `proof_size` - Proof size in bytes
- `witness_size` - Witness size in bytes
- `total_proof_witness_size` - Combined size
- `cu_per_proof_size` - Efficiency ratio

**Cost Analysis:**
- `cost_in_sol` - Total cost in SOL
- `cost_in_lamports` - Total cost in lamports
- `gas_fee` / `base_fee` - Base transaction fee (5,000 lamports)
- `cu_limit` - Compute unit limit
- `cu_price_microlamports` - Price per CU
- `prioritization_fee` / `priority_fee` - Priority fee in lamports
- `total_fee` - Sum of all fees
- `priority` - Priority score
- `signature_cus` - CUs for signatures
- `write_lock_cus` - CUs for write locks
- `suggestion` - Fee recommendations

**Transaction Status:**
- `status` - "Success" or "Failed"
- `error` - Error details if failed
- `suggestion` - Action recommendations

**Transaction Size:**
- `transaction_size` - Total transaction size
- `message_size` - Message size
- `proof_size` - Proof size
- `witness_size` - Witness size
- `total_proof_witness_size` - Combined size
- `max_message_size` - Solana limit (1232 bytes)
- `message_within_size` - Boolean check
- `message` - Status message
- `suggestion` - Size optimization tips

**Transaction Logs:**
- `logs` - Array of program execution logs
- `log_count` - Number of log entries

**Environment:**
- `network` - Solana network (devnet/testnet/mainnet)
- `rpc_url` - RPC endpoint used

**Recent Prioritization Fees:**
- Array of recent fees from the network (up to 50 entries)
- Each entry contains `slot` and `prioritization_fee`

**Output:**
```
📊 TRANSACTION SIMULATION
Simulate ZK proof verification on Solana

📍 Enter Solana program ID: 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM

✓ Found proof files

✓ Connected to devnet (https://api.devnet.solana.com) (234ms)
✓ Simulation complete (567ms)
✓ Fetched prioritization fees
✓ Report saved to .zklense/report.json

⚡ Compute Units
  Consumed:      123,456 CU
  Budget:       500,000 CU
  Usage:           24.69%

✓ Transaction Status
  ✓ Simulation Successful

📄 Transaction Size
  Message Size:  856 bytes
  Max Size:      1,232 bytes

💰 Cost Estimate
  Base Fee:      0.000005000 SOL
  Priority Fee:  0.000000000 SOL
  Total:         0.000005000 SOL

📄 Proof Files
  Proof:         256 bytes (target/my_circuit.proof)
  Witness:      600 bytes (target/my_circuit.pw)
  Total:         856 bytes

SIMULATION COMPLETE
Transaction simulation was successful!

View full report: .zklense/report.json
```

**Report File Location:**
- Saved to: `.zklense/report.json`
- Format: Pretty-printed JSON
- Can be viewed with: `zklense view`

**Error Handling:**
- If project not initialized, prompts to initialize
- If proof/witness files not found, shows error with search details
- If program ID invalid, shows parsing error
- If network connection fails, shows RPC error
- If simulation fails, shows error details in report

**Notes:**
- Uses the network and RPC URL from `.zklense/config.toml`
- Automatically searches for proof files recursively
- Uses a dummy fee payer for simulation (no real SOL required)
- Compute budget is set to 500,000 CU by default

---

### view

Open an interactive web viewer for the profiling report. Starts a local HTTP server and opens the report in your default browser.

**Usage:**
```bash
zklense view [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory if not specified.
  - Can be relative: `./my_project` or `../parent/project`
  - Can be absolute: `/home/user/projects/my_project`

**Options:**
- `-h, --help` - Print help information

**Prerequisites:**
- Project must be initialized (run `zklense init` first)
- Report file must exist at `.zklense/report.json`
- Report must be valid JSON (generated by `zklense simulate`)

**Examples:**
```bash
# View report in current directory
zklense view

# View report in specific directory (relative)
zklense view ./my_project

# View report in specific directory (absolute)
zklense view /path/to/my_project
```

**What it does:**
1. Checks for `.zklense/report.json` in the project directory
2. Validates the report is valid JSON
3. Reads web app URL from `.zklense/config.toml` (default: `https://zklense.netlify.app/`)
4. Finds an available port on localhost
5. Starts a local HTTP server on that port
6. Opens browser to: `<web_app_url>?port=<port>`
7. Serves the report JSON at the root endpoint
8. Handles CORS for cross-origin requests

**Output:**
```
◉ Starting local server on port 54321
◉ Opening viewer at https://zklense.netlify.app/?port=54321
◉ Serving report... Press Ctrl+C to stop.
```

**Server Details:**
- Binds to: `127.0.0.1:0` (automatically finds available port)
- Serves: JSON report data
- Endpoints:
  - `GET /` - Returns the report JSON
  - `OPTIONS /` - CORS preflight
- Headers:
  - `Content-Type: application/json`
  - `Access-Control-Allow-Origin: *`
  - `Access-Control-Allow-Methods: GET, OPTIONS`
  - `Access-Control-Allow-Headers: Content-Type`

**Error Handling:**
- If project not initialized, shows error
- If report file not found, shows error with path
- If report is invalid JSON, shows validation error
- If port binding fails, shows error
- If browser open fails, shows warning with manual URL

**Stopping the Server:**
- Press `Ctrl+C` to stop the server
- The server will continue running until interrupted

**Notes:**
- The web viewer URL is configurable in `.zklense/config.toml` (`web_app_url`)
- The server handles multiple concurrent connections
- Each request spawns a new thread
- The report content is loaded once at startup

---

### config

Manage zklense configuration. View and modify settings stored in `.zklense/config.toml`.

**Usage:**
```bash
zklense config <COMMAND>
```

**Subcommands:**

#### `config show`

Show all configuration values in a formatted table.

**Usage:**
```bash
zklense config show [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# Show config in current directory
zklense config show

# Show config in specific directory
zklense config show ./my_project
```

**Output:**
```
⚙️  ZKLENSE CONFIGURATION

🌐 Network:     devnet
🔗 RPC URL:    https://api.devnet.solana.com
🌐 Web App:     https://zklense.netlify.app/
📦 Version:     0.1.0
```

---

#### `config get-network`

Get the current Solana network and RPC URL.

**Usage:**
```bash
zklense config get-network [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# Get network in current directory
zklense config get-network

# Get network in specific directory
zklense config get-network ./my_project
```

**Output:**
```
🌐 Current Solana Network
  Network:  devnet
  RPC URL:  https://api.devnet.solana.com
```

---

#### `config set-network`

Set the Solana network (devnet, testnet, or mainnet). Also updates the RPC URL to the default for that network.

**Usage:**
```bash
zklense config set-network <NETWORK> [PATH]
```

**Arguments:**
- `<NETWORK>` - (Required) Network to use. Valid values:
  - `devnet` - Solana devnet
  - `testnet` - Solana testnet
  - `mainnet` or `mainnet-beta` - Solana mainnet
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# Switch to devnet
zklense config set-network devnet

# Switch to testnet
zklense config set-network testnet

# Switch to mainnet
zklense config set-network mainnet

# Switch in specific directory
zklense config set-network devnet ./my_project
```

**Output:**
```
✓ Network changed: devnet → testnet
🔗 RPC URL: https://api.testnet.solana.com
```

**Default RPC URLs:**
- `devnet`: `https://api.devnet.solana.com`
- `testnet`: `https://api.testnet.solana.com`
- `mainnet`: `https://api.mainnet-beta.solana.com`

**Error Handling:**
- If network is invalid, shows error with valid options
- If network is already set, shows info message

---

#### `config list-networks`

List all available Solana networks with their RPC URLs. Highlights the currently selected network.

**Usage:**
```bash
zklense config list-networks [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# List networks in current directory
zklense config list-networks

# List networks in specific directory
zklense config list-networks ./my_project
```

**Output:**
```
🌐 AVAILABLE NETWORKS
  ✓ devnet        https://api.devnet.solana.com
  ⏳ testnet      https://api.testnet.solana.com
  ⏳ mainnet      https://api.mainnet-beta.solana.com

  🔗 Current RPC: https://api.devnet.solana.com

  💡 Commands:
     zklense config set-network <network>
     zklense config set-rpc <url>
```

---

#### `config get-rpc`

Get the current Solana RPC URL and network. Shows if a custom RPC is being used.

**Usage:**
```bash
zklense config get-rpc [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# Get RPC in current directory
zklense config get-rpc

# Get RPC in specific directory
zklense config get-rpc ./my_project
```

**Output:**
```
🔗 Current Solana RPC
  RPC URL:  https://api.devnet.solana.com
  Network:  devnet
```

**If custom RPC is set:**
```
🔗 Current Solana RPC
  RPC URL:  https://my-custom-rpc.example.com
  Network:  devnet
  Status:   Custom RPC

  ℹ️  Default for devnet is: https://api.devnet.solana.com
```

---

#### `config set-rpc`

Set a custom Solana RPC URL. This allows you to use a custom RPC provider instead of the default public endpoints.

**Usage:**
```bash
zklense config set-rpc <RPC_URL> [PATH]
```

**Arguments:**
- `<RPC_URL>` - (Required) Custom RPC URL. Must start with `http://` or `https://`
  - Example: `https://my-rpc.example.com`
  - Example: `https://api.mainnet-beta.solana.com` (public mainnet)
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# Set custom RPC
zklense config set-rpc https://my-rpc.example.com

# Set custom RPC in specific directory
zklense config set-rpc https://my-rpc.example.com ./my_project

# Use a different public endpoint
zklense config set-rpc https://api.mainnet-beta.solana.com
```

**Output:**
```
✓ RPC URL updated
  ├─ Old: https://api.devnet.solana.com
  └─ New: https://my-rpc.example.com
```

**Error Handling:**
- If URL doesn't start with `http://` or `https://`, shows error
- If URL is already set, shows info message

**Notes:**
- Setting a custom RPC does not change the network setting
- The network setting is independent of the RPC URL
- Useful for using private RPC providers or different endpoints

---

#### `config reset-rpc`

Reset the Solana RPC URL to the default for the current network. Removes any custom RPC setting.

**Usage:**
```bash
zklense config reset-rpc [PATH]
```

**Arguments:**
- `[PATH]` - (Optional) Project path. Defaults to current directory.

**Examples:**
```bash
# Reset RPC in current directory
zklense config reset-rpc

# Reset RPC in specific directory
zklense config reset-rpc ./my_project
```

**Output:**
```
✓ RPC URL reset to default for devnet
  ├─ Old: https://my-custom-rpc.example.com
  └─ New: https://api.devnet.solana.com
```

**If already at default:**
```
ℹ️  RPC URL is already set to the default: https://api.devnet.solana.com
```

**Notes:**
- Resets to the default RPC for the currently selected network
- Does not change the network setting

---

## Complete Workflow Examples

### Example 1: Complete Project from Scratch

```bash
# 1. Create a new project with template
zklense generate --name age_verifier --template age_verifier

# 2. Navigate to project
cd age_verifier

# 3. Initialize zklense (or it may prompt during generate)
zklense init

# 4. Configure inputs (edit Prover.toml)
cat > Prover.toml << EOF
year_of_birth = "1990"
current_year = "2024"
age_threshold = "21"
EOF

# 5. Run the full pipeline
zklense run

# 6. Deploy to Solana (when prompted, or manually)
# Save the Program ID from the output

# 7. Simulate on Solana
zklense simulate --program-id <YOUR_PROGRAM_ID>

# 8. View the report
zklense view
```

### Example 2: Using Existing Noir Project

```bash
# 1. Navigate to existing project
cd /path/to/existing/noir/project

# 2. Initialize zklense
zklense init

# 3. Configure network (optional)
zklense config set-network devnet

# 4. Run pipeline
zklense run

# 5. Simulate (after deploying program)
zklense simulate --program-id <PROGRAM_ID>

# 6. View results
zklense view
```

### Example 3: Switching Networks

```bash
# 1. Check current network
zklense config get-network

# 2. List available networks
zklense config list-networks

# 3. Switch to testnet
zklense config set-network testnet

# 4. Verify change
zklense config show

# 5. Use custom RPC
zklense config set-rpc https://my-custom-rpc.example.com

# 6. Reset to default
zklense config reset-rpc
```

### Example 4: Multiple Projects

```bash
# Project 1: Age verifier
cd ~/projects/age_verifier
zklense run
zklense simulate --program-id <PROGRAM_ID_1>

# Project 2: Merkle proof
cd ~/projects/merkle_proof
zklense run
zklense simulate --program-id <PROGRAM_ID_2>

# View each project's report
zklense view ~/projects/age_verifier
zklense view ~/projects/merkle_proof
```

---

## Troubleshooting

### Command Not Found

**Problem:** `zklense: command not found`

**Solutions:**
1. Verify installation: Check [INSTALL.md](INSTALL.md)
2. Check PATH: `echo $PATH` (Linux/macOS) or `echo %PATH%` (Windows)
3. Reinstall or add to PATH manually

### Project Not Initialized

**Problem:** Commands fail with "not initialized" error

**Solution:**
```bash
# Initialize the project
zklense init

# Or specify path
zklense init /path/to/project
```

### Missing Proof Files

**Problem:** `simulate` command fails: "Could not find file with extension .proof"

**Solutions:**
1. Run the pipeline first: `zklense run`
2. Check that files exist in `target/` directory
3. Verify file extensions: `.proof` and `.pw`

### Missing Dependencies

**Problem:** `run` command fails: "Missing required commands: nargo, sunspot"

**Solutions:**
1. Install nargo: https://noir-lang.org/docs/getting_started/installation
2. Install sunspot: https://github.com/reilabs/sunspot
3. Verify in PATH: `which nargo` and `which sunspot`

### Network Connection Errors

**Problem:** `simulate` command fails: "Failed to connect to RPC"

**Solutions:**
1. Check network configuration: `zklense config show`
2. Test RPC URL manually
3. Switch network: `zklense config set-network devnet`
4. Use custom RPC: `zklense config set-rpc <your-rpc-url>`

### Invalid Program ID

**Problem:** `simulate` command fails: "Invalid program ID"

**Solutions:**
1. Verify program ID is a valid Solana public key (base58)
2. Check that program is deployed on the configured network
3. Ensure program ID matches the deployed program

### Report Not Found

**Problem:** `view` command fails: "No report found"

**Solutions:**
1. Run simulation first: `zklense simulate --program-id <ID>`
2. Check that `.zklense/report.json` exists
3. Verify report is valid JSON

### Port Already in Use

**Problem:** `view` command fails: "Failed to bind to a port"

**Solutions:**
1. Close other instances of `zklense view`
2. Wait a few seconds and try again
3. Check for other processes using ports: `lsof -i :<port>` (macOS/Linux)

---

## Additional Resources

- **Installation Guide:** [INSTALL.md](INSTALL.md)
- **Main Documentation:** [README.md](README.md)
- **Noir Documentation:** https://noir-lang.org/docs/
- **Solana Documentation:** https://docs.solana.com/
- **Sunspot Documentation:** https://github.com/reilabs/sunspot

---

## Command Quick Reference

```bash
# Version
zklense version
zklense --version

# Initialize
zklense init [PATH]

# Generate Project
zklense generate [-n NAME] [-t TEMPLATE]

# Run Pipeline
zklense run [PATH]

# Simulate
zklense simulate [-p PROGRAM_ID]

# View Report
zklense view [PATH]

# Configuration
zklense config show [PATH]
zklense config get-network [PATH]
zklense config set-network <NETWORK> [PATH]
zklense config list-networks [PATH]
zklense config get-rpc [PATH]
zklense config set-rpc <RPC_URL> [PATH]
zklense config reset-rpc [PATH]

# Help
zklense --help
zklense <COMMAND> --help
```
