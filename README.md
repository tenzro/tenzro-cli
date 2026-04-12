# Tenzro CLI

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

The official command-line interface for the **Tenzro Network** — an AI-native, agentic, tokenized settlement layer. Operate nodes, manage wallets, serve AI models, bridge tokens cross-chain, interact with TEEs and ZK proofs, and more.

## Installation

### Quick install (macOS / Linux)

```bash
curl -sSL https://get.tenzro.network | sh
```

### Homebrew (macOS / Linux)

```bash
brew install tenzro/tap/tenzro
```

### Cargo (Rust)

```bash
cargo install tenzro-cli
```

### Download binary

Pre-built binaries for all platforms on the [Releases](https://github.com/tenzro/tenzro-cli/releases) page:

| Platform | Binary |
|----------|--------|
| macOS (Apple Silicon) | `tenzro-aarch64-apple-darwin` |
| macOS (Intel) | `tenzro-x86_64-apple-darwin` |
| Linux (x86_64) | `tenzro-x86_64-unknown-linux-gnu` |
| Linux (ARM64) | `tenzro-aarch64-unknown-linux-gnu` |
| Windows | `tenzro-x86_64-pc-windows-msvc.exe` |

### Docker

```bash
docker run -it tenzro/cli
```

### Other package managers

| Manager | Command |
|---------|---------|
| Snap | `snap install tenzro` |
| Scoop (Windows) | `scoop install tenzro` |
| AUR (Arch) | `yay -S tenzro-cli` |
| Nix | `nix-env -i tenzro-cli` |

### Build from source

The CLI is part of the [tenzro-network](https://github.com/tenzro/tenzro-network) monorepo. Clone the full workspace and build:

```bash
git clone https://github.com/tenzro/tenzro-network
cd tenzro-network
cargo build --release -p tenzro-cli
# Binary at: target/release/tenzro
```

GPU acceleration features (optional):

```bash
cargo build --release -p tenzro-cli --features metal   # macOS Metal
cargo build --release -p tenzro-cli --features cuda     # NVIDIA CUDA
cargo build --release -p tenzro-cli --features rocm     # AMD ROCm
cargo build --release -p tenzro-cli --features vulkan   # Vulkan
```

## Interactive Mode

Run `tenzro` with no arguments to enter interactive mode — a navigable menu for discovering and executing commands without memorizing flags:

```
$ tenzro

  ████████╗███████╗███╗   ██╗███████╗██████╗  ██████╗
  ╚══██╔══╝██╔════╝████╗  ██║╚══███╔╝██╔══██╗██╔═══██╗
     ██║   █████╗  ██╔██╗ ██║  ███╔╝ ██████╔╝██║   ██║
     ██║   ██╔══╝  ██║╚██╗██║ ███╔╝  ██╔══██╗██║   ██║
     ██║   ███████╗██║ ╚████║███████╗██║  ██║╚██████╔╝
     ╚═╝   ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝

  Connected: rpc.tenzro.network | Block: 5,105,290 | Peers: 3

  What would you like to do?

  > Wallet & Tokens
    AI Models & Inference
    Agents & Swarms
    Bridge & Cross-Chain
    Identity & Credentials
    Payments & Settlement
    Security (Crypto, TEE, ZK)
    Developer (Apps, Contracts)
    Network & Governance
```

Use arrow keys to navigate, Enter to select. Each category opens a sub-menu with the relevant commands and guided inputs.

## Quick Start

```bash
# Join the network (provisions identity, wallet, hardware profile)
tenzro join --rpc https://rpc.tenzro.network

# Check your wallet balance
tenzro wallet balance --rpc https://rpc.tenzro.network

# Request testnet TNZO tokens
tenzro faucet 0xYOUR_ADDRESS --rpc https://rpc.tenzro.network

# Send TNZO to another address
tenzro wallet send --to 0xRECIPIENT --amount 10 --rpc https://rpc.tenzro.network

# Download and serve an AI model
tenzro model download gemma-3-4b
tenzro model serve gemma-3-4b

# Interactive chat with a served model
tenzro chat gemma-3-4b

# Register an AI agent
tenzro agent register --name my-agent --capabilities nlp,chain

# Bridge tokens cross-chain
tenzro bridge quote --from ethereum --to tenzro --amount 100 --token USDC
```

## Commands

36 command groups covering the full Tenzro Network feature set:

| Command | Description |
|---------|-------------|
| `tenzro join` | Join the network as a MicroNode (one-click participate) |
| `tenzro node` | Node management (start, stop, status, config) |
| `tenzro wallet` | Create wallets, check balances, send TNZO, list accounts |
| `tenzro model` | Download, serve, stop, delete, and manage AI models |
| `tenzro chat` | Interactive AI chat with served models (local + network) |
| `tenzro agent` | Register agents, spawn, swarms, messaging, templates |
| `tenzro crypto` | Sign, verify, encrypt, decrypt, hash, keygen |
| `tenzro tee` | TEE detection, attestation, verification, seal/unseal |
| `tenzro zk` | ZK proof creation, verification, keygen, circuits |
| `tenzro custody` | MPC wallets, keystores, key rotation, session keys, limits |
| `tenzro app` | Register apps, manage user wallets, sponsor gas, stats |
| `tenzro bridge` | Cross-chain bridge (quote, execute, status, routes) |
| `tenzro debridge` | deBridge DLN cross-chain swaps |
| `tenzro lifi` | LI.FI bridge aggregation (chains, tokens, routes, quotes) |
| `tenzro nft` | Create collections, mint, transfer, burn NFTs (ERC-721/1155) |
| `tenzro compliance` | ERC-3643 compliance (KYC, accreditation, freeze, agents) |
| `tenzro crosschain` | ERC-7802 cross-chain token operations (mint/burn bridges) |
| `tenzro events` | Event subscriptions, history, and webhooks |
| `tenzro token` | Create tokens, info, list, balance, wrap, cross-VM transfer |
| `tenzro contract` | Deploy smart contracts (EVM, SVM, DAML) |
| `tenzro identity` | TDIP identity management (register, resolve, credentials) |
| `tenzro governance` | Proposals, voting, and voting power |
| `tenzro stake` | Stake/unstake TNZO, view staking info |
| `tenzro task` | Task marketplace (post, list, get, cancel, quote, assign) |
| `tenzro marketplace` | Agent template marketplace (list, register, get) |
| `tenzro skill` | Skills registry (list, register, search, use) |
| `tenzro tool` | Tools registry / MCP server management |
| `tenzro canton` | Canton/DAML operations (domains, contracts, commands) |
| `tenzro escrow` | Escrow, payment channels, delegation, settlement |
| `tenzro payment` | Payment protocols (MPP, x402) — challenges, pay, sessions |
| `tenzro provider` | Provider management, pricing, status, model listing |
| `tenzro schedule` | Provider availability scheduling (set, show, enable) |
| `tenzro ceremony` | ZK trusted setup ceremony (init, contribute, verify) |
| `tenzro inference` | Direct inference requests |
| `tenzro hardware` | Hardware profile detection |
| `tenzro faucet` | Request testnet TNZO tokens (100 TNZO, 24h cooldown) |
| `tenzro set-username` | Set your Tenzro username |
| `tenzro info` | Network information (chain ID, block height, peers, supply) |
| `tenzro version` | Version info (with `--detailed` for full build info) |

Run `tenzro <command> --help` for detailed usage of any command.

## Configuration

### RPC endpoint

Every command that communicates with the network accepts `--rpc`:

```bash
tenzro wallet balance --rpc https://rpc.tenzro.network
```

The default RPC endpoint is `http://127.0.0.1:8545` (local node).

### Persistent config

The CLI stores configuration at `~/.tenzro/config.json`, shared with the Tenzro Desktop app. Fields include: endpoint, wallet address, DID, display name, role, provider schedule, and pricing.

### Environment variables

| Variable | Description |
|----------|-------------|
| `TENZRO_NO_BANNER` | Set to `1` to suppress the startup banner |
| `TENZRO_SIMULATE_TDX` | Simulate Intel TDX TEE for testing |
| `TENZRO_SIMULATE_SEV` | Simulate AMD SEV-SNP TEE for testing |
| `TENZRO_SIMULATE_NITRO` | Simulate AWS Nitro TEE for testing |

### Output format

All commands support `--format json` for machine-readable output:

```bash
tenzro wallet balance --format json --rpc https://rpc.tenzro.network
```

## Live Testnet

| Service | URL |
|---------|-----|
| JSON-RPC | `https://rpc.tenzro.network` |
| Web API | `https://api.tenzro.network` |
| Faucet | `https://api.tenzro.network/api/faucet` |
| MCP Server | `https://mcp.tenzro.network/mcp` |
| A2A Server | `https://a2a.tenzro.network` |

Genesis supply: 1,000,000,000 TNZO. Faucet: 100 TNZO per request, 24h cooldown.

## Architecture

The CLI communicates with Tenzro nodes via JSON-RPC 2.0 over HTTP. For AI model inference, it supports both local execution (via llama.cpp through `tenzro-model`) and network inference via RPC fallback.

```
tenzro CLI  --->  JSON-RPC 2.0  --->  tenzro-node (port 8545)
                  HTTP REST     --->  Web API     (port 8080)
```

Chat sessions are persisted at `~/.tenzro/chat_history/` with session management (`/history`, `/load`).

## Building

This crate is part of the [tenzro-network](https://github.com/tenzro/tenzro-network) Cargo workspace. It depends on internal crates (`tenzro-types`, `tenzro-crypto`, `tenzro-wallet`, `tenzro-node`, `tenzro-model`, `tenzro-zk`) and cannot be built standalone or published to crates.io independently.

```bash
# From the tenzro-network workspace root:
cargo build --release -p tenzro-cli
cargo test -p tenzro-cli
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Links

- Website: [tenzro.com](https://tenzro.com)
- GitHub: [github.com/tenzro](https://github.com/tenzro)
- Email: eng@tenzro.com
