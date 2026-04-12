//! Zero-knowledge proof commands for the Tenzro CLI
//!
//! Create proofs, generate proving keys, verify proofs, and list circuits.

use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::output;

/// ZK proof operations
#[derive(Debug, Subcommand)]
pub enum ZkCommand {
    /// Create a zero-knowledge proof
    Prove(ZkProveCmd),
    /// Verify a zero-knowledge proof
    Verify(ZkVerifyCmd),
    /// Generate a proving key for a circuit
    Keygen(ZkKeygenCmd),
    /// List available ZK circuits
    Circuits(ZkCircuitsCmd),
}

impl ZkCommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            Self::Prove(cmd) => cmd.execute().await,
            Self::Verify(cmd) => cmd.execute().await,
            Self::Keygen(cmd) => cmd.execute().await,
            Self::Circuits(cmd) => cmd.execute().await,
        }
    }
}

/// Create a ZK proof
#[derive(Debug, Parser)]
pub struct ZkProveCmd {
    /// Circuit name (InferenceVerification, SettlementProof, IdentityProof)
    #[arg(long)]
    circuit: String,
    /// Public inputs (JSON array)
    #[arg(long)]
    inputs: String,
    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl ZkProveCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Create ZK Proof");
        let spinner = output::create_spinner("Generating proof...");
        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_createZkProof", serde_json::json!({
            "circuit": self.circuit,
            "inputs": self.inputs,
        })).await?;

        spinner.finish_and_clear();

        output::print_success("Proof generated!");
        output::print_field("Proof Type", result.get("proof_type").and_then(|v| v.as_str()).unwrap_or("groth16"));
        output::print_field("Proof", &output::format_hash(result.get("proof").and_then(|v| v.as_str()).unwrap_or("")));

        Ok(())
    }
}

/// Verify a ZK proof
#[derive(Debug, Parser)]
pub struct ZkVerifyCmd {
    /// Proof data (hex)
    #[arg(long)]
    proof: String,
    /// Proof type: groth16, plonk, or stark
    #[arg(long, default_value = "groth16")]
    proof_type: String,
    /// Public inputs (JSON array)
    #[arg(long)]
    inputs: String,
    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl ZkVerifyCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Verify ZK Proof");
        let spinner = output::create_spinner("Verifying...");
        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_verifyZkProof", serde_json::json!({
            "proof": self.proof,
            "proof_type": self.proof_type,
            "public_inputs": self.inputs,
        })).await?;

        spinner.finish_and_clear();

        let valid = result.get("valid").and_then(|v| v.as_bool()).unwrap_or(false);
        if valid {
            output::print_success("Proof is valid!");
        } else {
            output::print_error("Proof verification failed.");
        }

        Ok(())
    }
}

/// Generate a proving key
#[derive(Debug, Parser)]
pub struct ZkKeygenCmd {
    /// Circuit name
    #[arg(long)]
    circuit: String,
    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl ZkKeygenCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Generate Proving Key");
        let spinner = output::create_spinner("Generating proving key...");
        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_generateProvingKey", serde_json::json!({
            "circuit": self.circuit,
        })).await?;

        spinner.finish_and_clear();

        output::print_success("Proving key generated!");
        output::print_field("Circuit", &self.circuit);
        output::print_field("Key ID", result.get("key_id").and_then(|v| v.as_str()).unwrap_or(""));

        Ok(())
    }
}

/// List ZK circuits
#[derive(Debug, Parser)]
pub struct ZkCircuitsCmd {
    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl ZkCircuitsCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Available ZK Circuits");
        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_listZkCircuits", serde_json::json!([])).await?;

        if let Some(circuits) = result.as_array() {
            if circuits.is_empty() {
                output::print_info("No circuits available.");
            } else {
                for c in circuits {
                    let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                    let desc = c.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    output::print_field(name, desc);
                }
            }
        } else {
            output::print_json(&result)?;
        }

        Ok(())
    }
}
