# Tenzro Network CLI

The official command-line interface for operating Tenzro Network nodes, managing wallets, models, staking, and governance. Interact with Tenzro Ledger (the L1 settlement layer) and earn TNZO tokens.

## Features

- **Node Management**: Start, stop, and monitor Tenzro Network nodes
- **Wallet Operations**: Create MPC wallets, check balances, send transactions
- **Model Management**: List, download, and serve AI models
- **Staking**: Stake TNZO tokens as validator or provider
- **Governance**: Participate in on-chain governance and voting
- **Provider Tools**: Register and manage inference/TEE providers
- **Inference Requests**: Submit AI inference requests to the network

## Installation

```bash
# From source
cargo install --path crates/tenzro-cli

# Or build and run directly
cargo run -p tenzro-cli -- --help
```

## Quick Start

```bash
# Start a full node
tenzro node start --role user

# Create a new wallet
tenzro wallet create

# Check your balance
tenzro wallet balance

# List available models
tenzro model list

# Submit an inference request
tenzro inference request gemma4-9b "What is Tenzro Network?"
```

## Commands

### Node Management

```bash
# Start a node with specific role
tenzro node start --role validator
tenzro node start --role model-provider
tenzro node start --role tee-provider
tenzro node start --role user

# Check node status
tenzro node status

# Stop the node
tenzro node stop
```

### Wallet Operations

```bash
# Create a new MPC wallet (2-of-3 threshold by default)
tenzro wallet create

# Create with custom threshold
tenzro wallet create --threshold 3 --total-shares 5

# Import existing wallet
tenzro wallet import <seed-phrase|private-key>

# Check balance
tenzro wallet balance --address <address>

# Send tokens
tenzro wallet send <to-address> <amount> --asset TNZO

# Send stablecoins
tenzro wallet send <to-address> 100 --asset USDC

# List all wallets
tenzro wallet list
```

### Model Management

```bash
# List all available models
tenzro model list

# Filter by modality
tenzro model list --modality text
tenzro model list --modality image

# Show model details
tenzro model info gemma4-9b --providers

# Download a model
tenzro model download gemma4-9b

# Start serving a model
tenzro model serve gemma4-9b --gpus 0,1 --port 8080

# Stop serving
tenzro model stop gemma4-9b
```

### Staking

```bash
# Stake TNZO tokens
tenzro stake deposit 10000

# Stake as specific provider type
tenzro stake deposit 10000 --provider-type validator

# Stake with lock period for higher APY
tenzro stake deposit 10000 --lock-days 180

# Withdraw staked tokens
tenzro stake withdraw 5000

# View staking information
tenzro stake info --detailed
```

### Governance

```bash
# List active proposals
tenzro governance list --active

# View detailed proposal info
tenzro governance list --detailed

# Create a new proposal
tenzro governance propose \
  "Increase validator rewards" \
  "This proposal increases validator rewards by 10%" \
  --type parameter \
  --duration-days 14

# Vote on a proposal
tenzro governance vote prop_001 yes
tenzro governance vote prop_002 no --reason "Insufficient justification"
```

### Provider Management

```bash
# Register as inference provider
tenzro provider register --type inference --stake 10000

# Register as TEE provider
tenzro provider register --type tee --stake 15000

# Check provider status
tenzro provider status --detailed

# List models you're serving
tenzro provider models
```

### Inference Requests

```bash
# Submit text inference
tenzro inference request gemma4-9b "Explain quantum computing"

# Image generation
tenzro inference request stable-diffusion-xl "A sunset over mountains"

# With parameters (price in TNZO)
tenzro inference request gemma4-9b "Write a poem" \
  --temperature 0.8 \
  --max-tokens 500 \
  --max-price 1.0

# Require TEE attestation
tenzro inference request gpt-4o "Sensitive query" --require-tee

# Save output to file
tenzro inference request gemma4-9b "Generate code" --output-file result.txt
```

### Network Information

```bash
# Show network stats
tenzro info

# Show version
tenzro version --detailed
```

## Global Options

```bash
# Enable verbose logging
tenzro --verbose <command>

# JSON output format
tenzro --format json <command>
```

## Configuration

The CLI stores configuration and wallet data in:
- Linux: `~/.tenzro/`
- macOS: `~/.tenzro/`
- Windows: `%USERPROFILE%\.tenzro\`

### Directory Structure

```
~/.tenzro/
├── config.toml          # CLI configuration
├── wallets/             # Wallet keystores
│   ├── wallet_1.json
│   └── wallet_2.json
├── data/                # Node data (if running a node)
│   ├── db/
│   └── keystore/
└── models/              # Downloaded models
    └── gemma4-9b/
```

## Examples

### Running a Validator Node

```bash
# 1. Create wallet for validator
tenzro wallet create --name validator

# 2. Stake tokens
tenzro stake deposit 100000 --provider-type validator

# 3. Start validator node
tenzro node start --role validator --data-dir ~/.tenzro/validator
```

### Becoming an Inference Provider

```bash
# 1. Register as provider
tenzro provider register --type inference --stake 10000

# 2. Download models
tenzro model download gemma4-9b
tenzro model download stable-diffusion-xl

# 3. Start serving models
tenzro model serve gemma4-9b --gpus 0
tenzro model serve stable-diffusion-xl --gpus 1

# 4. Monitor provider status
tenzro provider status --detailed
```

### Participating in Governance

```bash
# 1. Check your voting power
tenzro stake info

# 2. List active proposals
tenzro governance list --active --detailed

# 3. Vote on proposals
tenzro governance vote prop_001 yes --reason "Good for the network"

# 4. Create your own proposal
tenzro governance propose \
  "Add new stablecoin support" \
  "Proposal to add DAI as supported stablecoin" \
  --type parameter
```

## Development

### Building from Source

```bash
# Build debug version
cargo build -p tenzro-cli

# Build release version
cargo build -p tenzro-cli --release

# Run tests
cargo test -p tenzro-cli
```

### Architecture

The CLI is organized into several modules:

- `main.rs` - Entry point and command routing
- `output.rs` - Output formatting utilities (tables, progress bars, colors)
- `commands/` - Command implementations
  - `node.rs` - Node management
  - `wallet.rs` - Wallet operations
  - `model.rs` - Model management
  - `stake.rs` - Staking operations
  - `governance.rs` - Governance and voting
  - `provider.rs` - Provider management
  - `inference.rs` - Inference requests

## Note on Current Implementation

This is an initial implementation with stub RPC client functionality. The commands demonstrate the intended UX and output formatting, but make simulated calls rather than actual network requests. In production:

- All commands will connect to actual node RPC endpoints
- Real transaction signing and broadcasting will occur
- Actual model downloads and inference will be performed
- TEE attestation will be verified
- All on-chain state will be queried from the blockchain

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
