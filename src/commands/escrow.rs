//! Escrow and payment channel commands for the Tenzro CLI
//!
//! Manage escrow accounts and micropayment channels for settlement.

use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::output;

/// Escrow and payment channel commands
#[derive(Debug, Subcommand)]
pub enum EscrowCommand {
    /// Create a new escrow
    Create(EscrowCreateCmd),
    /// Release an escrow
    Release(EscrowReleaseCmd),
    /// Open a micropayment channel
    OpenChannel(ChannelOpenCmd),
    /// Close a micropayment channel
    CloseChannel(ChannelCloseCmd),
    /// Delegate voting power to a validator
    Delegate(DelegateCmd),
    /// Settle a payment immediately
    Settle(SettleCmd),
    /// Get settlement details
    GetSettlement(GetSettlementCmd),
}

impl EscrowCommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute().await,
            Self::Release(cmd) => cmd.execute().await,
            Self::OpenChannel(cmd) => cmd.execute().await,
            Self::CloseChannel(cmd) => cmd.execute().await,
            Self::Delegate(cmd) => cmd.execute().await,
            Self::Settle(cmd) => cmd.execute().await,
            Self::GetSettlement(cmd) => cmd.execute().await,
        }
    }
}

/// Create a new escrow
#[derive(Debug, Parser)]
pub struct EscrowCreateCmd {
    /// Payer address (hex, e.g. 0x...)
    #[arg(long)]
    payer: String,

    /// Payee address (hex, e.g. 0x...)
    #[arg(long)]
    payee: String,

    /// Amount in TNZO
    #[arg(long)]
    amount: u64,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl EscrowCreateCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Create Escrow");

        let spinner = output::create_spinner("Creating escrow...");

        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_createEscrow", serde_json::json!({
            "payer": self.payer,
            "payee": self.payee,
            "amount": self.amount,
        })).await?;

        spinner.finish_and_clear();

        output::print_success("Escrow created!");
        println!();

        if let Some(v) = result.get("escrow_id").and_then(|v| v.as_str()) {
            output::print_field("Escrow ID", v);
        }
        output::print_field("Payer", &self.payer);
        output::print_field("Payee", &self.payee);
        output::print_field("Amount", &format!("{} TNZO", self.amount));
        if let Some(v) = result.get("status").and_then(|v| v.as_str()) {
            output::print_field("Status", v);
        }

        Ok(())
    }
}

/// Release an escrow
#[derive(Debug, Parser)]
pub struct EscrowReleaseCmd {
    /// Escrow ID (UUID)
    escrow_id: String,

    /// Proof data (hex, optional)
    #[arg(long)]
    proof: Option<String>,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl EscrowReleaseCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Release Escrow");

        let spinner = output::create_spinner("Releasing escrow...");

        let rpc = RpcClient::new(&self.rpc);

        let mut params = serde_json::json!({
            "escrow_id": self.escrow_id,
        });
        if let Some(proof) = &self.proof {
            params["proof"] = serde_json::Value::String(proof.clone());
        }

        let result: serde_json::Value = rpc.call("tenzro_releaseEscrow", params).await?;

        spinner.finish_and_clear();

        output::print_success("Escrow released!");
        println!();

        output::print_field("Escrow ID", &self.escrow_id);
        if let Some(v) = result.get("status").and_then(|v| v.as_str()) {
            output::print_field("Status", v);
        }
        if let Some(receipt) = result.get("receipt") {
            if let Some(v) = receipt.get("receipt_id").and_then(|v| v.as_str()) {
                output::print_field("Receipt ID", v);
            }
            if let Some(v) = receipt.get("transaction_hash").and_then(|v| v.as_str()) {
                output::print_field("Transaction Hash", v);
            }
        }

        Ok(())
    }
}

/// Open a micropayment channel
#[derive(Debug, Parser)]
pub struct ChannelOpenCmd {
    /// Counterparty address (hex, e.g. 0x...)
    #[arg(long)]
    counterparty: String,

    /// Deposit amount in TNZO
    #[arg(long)]
    deposit: u64,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl ChannelOpenCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Open Payment Channel");

        let spinner = output::create_spinner("Opening channel...");

        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_openPaymentChannel", serde_json::json!({
            "counterparty": self.counterparty,
            "deposit": self.deposit,
        })).await?;

        spinner.finish_and_clear();

        output::print_success("Payment channel opened!");
        println!();

        if let Some(v) = result.get("channel_id").and_then(|v| v.as_str()) {
            output::print_field("Channel ID", v);
        }
        output::print_field("Counterparty", &self.counterparty);
        if let Some(v) = result.get("deposit").and_then(|v| v.as_str()) {
            output::print_field("Deposit", &format!("{} TNZO", v));
        }
        if let Some(v) = result.get("status").and_then(|v| v.as_str()) {
            output::print_field("Status", v);
        }

        Ok(())
    }
}

/// Close a micropayment channel
#[derive(Debug, Parser)]
pub struct ChannelCloseCmd {
    /// Channel ID (UUID)
    channel_id: String,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl ChannelCloseCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Close Payment Channel");

        let spinner = output::create_spinner("Closing channel...");

        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_closePaymentChannel", serde_json::json!({
            "channel_id": self.channel_id,
        })).await?;

        spinner.finish_and_clear();

        output::print_success("Payment channel closing!");
        println!();

        output::print_field("Channel ID", &self.channel_id);
        if let Some(v) = result.get("status").and_then(|v| v.as_str()) {
            output::print_field("Status", v);
        }
        if let Some(balances) = result.get("final_balances") {
            if let Some(v) = balances.get("deposit").and_then(|v| v.as_str()) {
                output::print_field("Total Deposit", &format!("{} wei", v));
            }
            if let Some(v) = balances.get("spent").and_then(|v| v.as_str()) {
                output::print_field("Total Spent", &format!("{} wei", v));
            }
            if let Some(v) = balances.get("remaining").and_then(|v| v.as_str()) {
                output::print_field("Remaining", &format!("{} wei", v));
            }
        }

        Ok(())
    }
}

/// Settle a payment immediately
#[derive(Debug, Parser)]
pub struct SettleCmd {
    /// Payer address (hex)
    #[arg(long)]
    payer: String,

    /// Payee address (hex)
    #[arg(long)]
    payee: String,

    /// Amount in TNZO
    #[arg(long)]
    amount: u64,

    /// Settlement type (immediate, escrow, channel)
    #[arg(long, default_value = "immediate")]
    settlement_type: String,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl SettleCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Settle Payment");

        let spinner = output::create_spinner("Settling payment...");

        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_settle", serde_json::json!([{
            "payer": self.payer,
            "payee": self.payee,
            "amount": self.amount,
            "settlement_type": self.settlement_type,
        }])).await?;

        spinner.finish_and_clear();

        output::print_success("Settlement completed!");
        println!();

        if let Some(v) = result.get("settlement_id").and_then(|v| v.as_str()) {
            output::print_field("Settlement ID", v);
        }
        output::print_field("Payer", &self.payer);
        output::print_field("Payee", &self.payee);
        output::print_field("Amount", &format!("{} TNZO", self.amount));
        if let Some(v) = result.get("status").and_then(|v| v.as_str()) {
            output::print_field("Status", v);
        }
        if let Some(v) = result.get("transaction_hash").and_then(|v| v.as_str()) {
            output::print_field("Transaction Hash", v);
        }

        Ok(())
    }
}

/// Get settlement details
#[derive(Debug, Parser)]
pub struct GetSettlementCmd {
    /// Settlement ID
    settlement_id: String,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl GetSettlementCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Settlement Details");

        let spinner = output::create_spinner("Fetching settlement...");

        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_getSettlement", serde_json::json!([self.settlement_id])).await?;

        spinner.finish_and_clear();

        println!();
        for (key, val) in result.as_object().unwrap_or(&serde_json::Map::new()) {
            output::print_field(key, val.to_string().trim_matches('"'));
        }

        Ok(())
    }
}

/// Delegate voting power to a validator
#[derive(Debug, Parser)]
pub struct DelegateCmd {
    /// Delegator address (your address, hex)
    #[arg(long)]
    from: String,

    /// Delegatee/validator address (hex)
    #[arg(long)]
    to: String,

    /// Amount in TNZO to delegate
    #[arg(long)]
    amount: u64,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl DelegateCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Delegate Voting Power");

        let spinner = output::create_spinner("Delegating...");

        let rpc = RpcClient::new(&self.rpc);

        let result: serde_json::Value = rpc.call("tenzro_delegateVotingPower", serde_json::json!({
            "delegator": self.from,
            "delegatee": self.to,
            "amount": self.amount,
        })).await?;

        spinner.finish_and_clear();

        output::print_success("Voting power delegated!");
        println!();

        output::print_field("Delegator", &self.from);
        output::print_field("Delegatee", &self.to);
        output::print_field("Amount", &format!("{} TNZO", self.amount));
        if let Some(v) = result.get("status").and_then(|v| v.as_str()) {
            output::print_field("Status", v);
        }

        Ok(())
    }
}
