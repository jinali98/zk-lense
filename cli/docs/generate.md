# zkprof generate

Create a new Noir project with optional templates for common ZK use cases.

## Synopsis

```bash
zkprof generate [OPTIONS]
```

## Description

The `generate` command scaffolds a new Noir project using `nargo new` and optionally applies a pre-built template to get you started quickly with common zero-knowledge proof patterns.

After creating the project, you'll be prompted to initialize zkprof in the new project directory.

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--name <NAME>` | `-n` | Name of the new Noir project. If not provided, you'll be prompted to enter it. |
| `--template <TEMPLATE>` | `-t` | Template to use. Options: `age_verifier`, `merkle_inclusion`, or `none`. If not provided, you'll select from an interactive list. |
| `--help` | `-h` | Print help information |

## Prerequisites

- **Nargo** must be installed and available in your PATH. Install it from [Noir's official documentation](https://noir-lang.org/docs/getting_started/installation/).

## Usage Examples

### Interactive Mode

Run without arguments for a fully guided experience:

```bash
zkprof generate
```

You'll be prompted for:
1. Project name
2. Template selection (from a list)
3. Whether to initialize zkprof

### With Arguments

Create a project with a specific template:

```bash
zkprof generate --name my_age_proof --template age_verifier
```

Create a project without a template:

```bash
zkprof generate --name my_circuit --template none
```

### Partial Arguments

Specify only the name (template will be selected interactively):

```bash
zkprof generate --name my_project
```

## Available Templates

### None (Default Noir Template)

Start with the default Noir project structure. Use this if you want to write your circuit from scratch.

---

### Age Verifier

**Description:** Verify if a person meets an age threshold based on their year of birth.

**Use Cases:**
- Age-gated access without revealing exact birth date
- KYC compliance proofs
- Anonymous age verification

**Inputs:**
| Parameter | Type | Visibility | Description |
|-----------|------|------------|-------------|
| `year_of_birth` | `u64` | Private | The person's birth year (kept secret) |
| `current_year` | `u64` | Public | The current year |
| `age_threshold` | `u64` | Public | Minimum age required |

**Example Usage:**
```noir
// Prove someone is at least 18 years old
// Private input: year_of_birth = 2000
// Public inputs: current_year = 2024, age_threshold = 18
```

**Template Code:**
```noir
fn main(year_of_birth: u64, current_year: pub u64, age_threshold: pub u64) -> bool {
    let age = current_year - year_of_birth;
    assert(age >= age_threshold);
}
```

---

### Merkle Inclusion Proof

**Description:** Prove membership in a Merkle tree without revealing which leaf you own.

**Use Cases:**
- Anonymous set membership (e.g., allowlists, voting eligibility)
- Proving ownership without revealing identity
- Privacy-preserving authentication

**Inputs:**
| Parameter | Type | Visibility | Description |
|-----------|------|------------|-------------|
| `value` | `Field` | Private | The leaf value to prove inclusion of |
| `path_elements` | `[Field; DEPTH]` | Public | Sibling hashes along the Merkle path |
| `path_indices` | `[Field; DEPTH]` | Public | Direction indicators (0 = left, 1 = right) |
| `root` | `Field` | Public | The Merkle root to verify against |

**Configuration:**
- `DEPTH`: Tree depth (default: 3, supporting up to 8 leaves)

**Example Usage:**
```noir
// Prove you're in a Merkle tree of 8 members
// Private input: your secret value
// Public inputs: path, indices, and the known root
```

**Template Code:**
```noir
global DEPTH: u32 = 3;

fn main(
    value: Field,
    path_elements: pub [Field; DEPTH],
    path_indices: pub [Field; DEPTH],
    root: pub Field,
) -> pub Field {
    let hashed_value = calculate_hash(value);
    let merkle_tree_root = construct_merkle_tree(hashed_value, path_indices, path_elements);
    assert(merkle_tree_root == root, "Merkle tree root does not match");
}
```

## Output

After successful execution, the command will:

1. ✅ Create a new Noir project directory with the specified name
2. 📝 Apply the selected template to `src/main.nr` (if a template was chosen)
3. 🔧 Optionally initialize zkprof in the project (if you accept the prompt)

### Example Output

```
📦 Creating new Noir project: my_age_proof
✅ Created Noir project: my_age_proof
📝 Applying template: Age Verifier
✅ Applied template to src/main.nr

🎉 Project 'my_age_proof' created successfully!
Would you like to initialize zkprof in this project? [Y/n]: y
✅ Created directory: /path/to/my_age_proof/.zkproof
✅ Created config file: /path/to/my_age_proof/.zkproof/config.toml

🎉 zkproof initialized successfully!

Next steps:
  cd my_age_proof
  nargo check    # Verify the project compiles
  nargo prove    # Generate a proof
```

## Project Structure

After running `zkprof generate --name my_project --template age_verifier`:

```
my_project/
├── Nargo.toml          # Noir project configuration
├── src/
│   └── main.nr         # Your circuit (with template applied)
└── .zkproof/           # zkprof configuration (if initialized)
    └── config.toml
```

## Next Steps

After generating your project:

1. **Navigate to your project:**
   ```bash
   cd my_project
   ```

2. **Verify it compiles:**
   ```bash
   nargo check
   ```

3. **Create a Prover.toml** with your inputs:
   ```toml
   # For age_verifier template
   year_of_birth = "2000"
   current_year = "2024"
   age_threshold = "18"
   ```

4. **Generate a proof:**
   ```bash
   nargo prove
   ```

5. **Run zkprof to analyze your proof:**
   ```bash
   zkprof run
   ```

## See Also

- [`zkprof init`](./init.md) - Initialize zkprof in an existing project
- [`zkprof run`](./run.md) - Run the profiling pipeline
- [`zkprof simulate`](./simulate.md) - Simulate proof verification on Solana
