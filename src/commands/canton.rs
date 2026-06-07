//! Canton/DAML integration commands for the Tenzro CLI
//!
//! Interact with the shared Canton synchronizer domain and DAML contracts
//! through the local Tenzro node. The node proxies calls to its configured
//! Canton participant; callers never see the Auth0 secret.

use clap::{Parser, Subcommand};
use anyhow::{anyhow, Result};
use crate::output;

/// Canton/DAML integration commands (Canton 3.5+ JSON Ledger API).
///
/// All reads + writes route through the local Tenzro node, which proxies
/// to its configured Canton participant. Callers never see the Auth0
/// secret. Every method requires an API key with scope `canton`
/// (passed via `--api-key` or `TENZRO_API_KEY` env var).
#[derive(Debug, Subcommand)]
pub enum CantonCommand {
    /// List configured Canton synchronizer domains
    Domains(CantonDomainsCmd),
    /// Query active DAML contracts (requires at least one template id)
    Contracts(CantonContractsCmd),
    /// Submit a DAML create or exercise command
    Submit(CantonSubmitCmd),
    // ── Canton 3.5+ JSON Ledger API extension surface ──
    /// Combined health probe (`/livez` + `/readyz` + `/v2/version`)
    Health(CantonSimpleCmd),
    /// Participant version + CIP feature flags (`GET /v2/version`)
    Version(CantonSimpleCmd),
    /// OAuth principal's Canton user record (`GET /v2/users/<client_id>@clients`, CIP-26)
    MyUser(CantonSimpleCmd),
    /// List every party known to the participant (`GET /v2/parties/known`)
    Parties(CantonSimpleCmd),
    /// List every DAML package installed on the participant (`GET /v2/packages`)
    Packages(CantonSimpleCmd),
    /// CIP-56 Canton Coin balance (sums every `Splice.Amulet:Amulet` contract)
    CoinBalance(CantonSimpleCmd),
    /// Canton fee schedule from the latest `Splice.AmuletRules:AmuletRules` contract
    FeeSchedule(CantonSimpleCmd),
    /// Connected synchronizers for the participant's party
    ConnectedSynchronizers(CantonSimpleCmd),
    /// Fetch a Canton transaction tree by hex update id
    GetTransaction(CantonGetTransactionCmd),
    /// Upload a DAR (DAML Archive) to the participant via `POST /v2/packages`
    UploadDar(CantonUploadDarCmd),
}

impl CantonCommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            Self::Domains(cmd) => cmd.execute().await,
            Self::Contracts(cmd) => cmd.execute().await,
            Self::Submit(cmd) => cmd.execute().await,
            Self::Health(cmd) => cmd.execute("Canton Health", "tenzro_canton_health").await,
            Self::Version(cmd) => cmd.execute("Canton Version", "tenzro_canton_version").await,
            Self::MyUser(cmd) => cmd.execute("Canton My User", "tenzro_canton_getMyUser").await,
            Self::Parties(cmd) => cmd.execute("Canton Parties", "tenzro_canton_listParties").await,
            Self::Packages(cmd) => cmd.execute("Canton Packages", "tenzro_canton_listPackages").await,
            Self::CoinBalance(cmd) => {
                cmd.execute("Canton Coin Balance", "tenzro_canton_coinBalance").await
            }
            Self::FeeSchedule(cmd) => {
                cmd.execute("Canton Fee Schedule", "tenzro_canton_feeSchedule").await
            }
            Self::ConnectedSynchronizers(cmd) => {
                cmd.execute(
                    "Canton Connected Synchronizers",
                    "tenzro_canton_connectedSynchronizers",
                )
                .await
            }
            Self::GetTransaction(cmd) => cmd.execute().await,
            Self::UploadDar(cmd) => cmd.execute().await,
        }
    }
}

/// List Canton domains
#[derive(Debug, Parser)]
pub struct CantonDomainsCmd {
    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    /// Operator-issued Tenzro API key (tnz_...) with scope `canton`.
    /// Falls back to the TENZRO_API_KEY env var when omitted.
    #[arg(long, env = "TENZRO_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

impl CantonDomainsCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Canton Domains");

        let spinner = output::create_spinner("Loading domains...");

        let mut rpc = RpcClient::new(&self.rpc);
        if let Some(key) = &self.api_key {
            rpc = rpc.with_api_key(key);
        }

        let result: serde_json::Value = rpc
            .call("tenzro_listCantonDomains", serde_json::json!({}))
            .await?;

        spinner.finish_and_clear();

        let enabled = result.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        output::print_field("Canton Enabled", if enabled { "Yes" } else { "No" });

        if !enabled {
            if let Some(msg) = result.get("message").and_then(|v| v.as_str()) {
                output::print_info(msg);
            }
            return Ok(());
        }

        if let Some(domains) = result.get("domains").and_then(|v| v.as_array()) {
            if domains.is_empty() {
                output::print_info("No Canton domains configured.");
            } else {
                let headers = vec!["ID", "Name", "Native Token", "Finality (s)"];
                let mut rows = Vec::new();
                for domain in domains {
                    rows.push(vec![
                        domain.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                        domain.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                        domain.get("native_token").and_then(|v| v.as_str()).unwrap_or("-").to_string(),
                        domain
                            .get("finality_time_secs")
                            .and_then(|v| v.as_u64())
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ]);
                }
                output::print_table(&headers, &rows);
            }
        }

        Ok(())
    }
}

/// Query active DAML contracts
#[derive(Debug, Parser)]
pub struct CantonContractsCmd {
    /// Template ID to query (repeat to include multiple). At least one
    /// template id is required by the Canton v2 active-contracts endpoint.
    #[arg(long = "template", required = true)]
    templates: Vec<String>,

    /// Optional JSON object applied as a structural filter against
    /// `createArguments`. Example: '{"owner":"alice"}'
    #[arg(long)]
    query: Option<String>,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    /// Operator-issued Tenzro API key (tnz_...) with scope `canton`.
    /// Falls back to the TENZRO_API_KEY env var when omitted.
    #[arg(long, env = "TENZRO_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

impl CantonContractsCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("DAML Contracts");

        let spinner = output::create_spinner("Loading contracts...");

        let mut rpc = RpcClient::new(&self.rpc);
        if let Some(key) = &self.api_key {
            rpc = rpc.with_api_key(key);
        }

        let mut params = serde_json::Map::new();
        params.insert(
            "template_ids".to_string(),
            serde_json::Value::Array(
                self.templates
                    .iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            ),
        );
        if let Some(q) = &self.query {
            let parsed: serde_json::Value = serde_json::from_str(q)
                .map_err(|e| anyhow!("--query is not valid JSON: {}", e))?;
            params.insert("query".to_string(), parsed);
        }

        let result: serde_json::Value = rpc
            .call(
                "tenzro_listDamlContracts",
                serde_json::Value::Object(params),
            )
            .await?;

        spinner.finish_and_clear();

        if let Some(contracts) = result.get("contracts").and_then(|v| v.as_array()) {
            if contracts.is_empty() {
                output::print_info("No DAML contracts found.");
            } else {
                let headers = vec!["Contract ID", "Template", "Payload"];
                let mut rows = Vec::new();
                for contract in contracts {
                    let payload_str = match contract.get("payload") {
                        Some(p) => serde_json::to_string(p).unwrap_or_else(|_| "?".to_string()),
                        None => String::new(),
                    };
                    rows.push(vec![
                        contract
                            .get("contract_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        contract
                            .get("template_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        payload_str,
                    ]);
                }
                output::print_table(&headers, &rows);
                println!("Total: {} contracts", contracts.len());
            }
        }

        Ok(())
    }
}

/// Submit a DAML command
#[derive(Debug, Parser)]
pub struct CantonSubmitCmd {
    /// Command type: `create` or `exercise`
    #[arg(long)]
    command_type: String,

    /// DAML template ID (required for both create and exercise)
    #[arg(long)]
    template: String,

    /// JSON object holding the create arguments (required when
    /// `--command-type create`).
    #[arg(long)]
    create_arguments: Option<String>,

    /// Existing contract id (required when `--command-type exercise`).
    #[arg(long)]
    contract_id: Option<String>,

    /// Choice name (required when `--command-type exercise`).
    #[arg(long)]
    choice: Option<String>,

    /// JSON object holding the choice argument (required when
    /// `--command-type exercise`).
    #[arg(long)]
    choice_argument: Option<String>,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    /// Operator-issued Tenzro API key (tnz_...) with scope `canton`.
    /// Falls back to the TENZRO_API_KEY env var when omitted.
    #[arg(long, env = "TENZRO_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

impl CantonSubmitCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Submit DAML Command");

        let spinner = output::create_spinner("Submitting command...");

        let mut rpc = RpcClient::new(&self.rpc);
        if let Some(key) = &self.api_key {
            rpc = rpc.with_api_key(key);
        }

        let params = match self.command_type.as_str() {
            "create" => {
                let raw = self
                    .create_arguments
                    .as_deref()
                    .ok_or_else(|| anyhow!("--create-arguments is required for create commands"))?;
                let create_arguments: serde_json::Value = serde_json::from_str(raw)
                    .map_err(|e| anyhow!("--create-arguments is not valid JSON: {}", e))?;
                serde_json::json!({
                    "command_type": "create",
                    "template_id": self.template,
                    "create_arguments": create_arguments,
                })
            }
            "exercise" => {
                let contract_id = self
                    .contract_id
                    .as_deref()
                    .ok_or_else(|| anyhow!("--contract-id is required for exercise commands"))?;
                let choice = self
                    .choice
                    .as_deref()
                    .ok_or_else(|| anyhow!("--choice is required for exercise commands"))?;
                let raw = self
                    .choice_argument
                    .as_deref()
                    .ok_or_else(|| anyhow!("--choice-argument is required for exercise commands"))?;
                let choice_argument: serde_json::Value = serde_json::from_str(raw)
                    .map_err(|e| anyhow!("--choice-argument is not valid JSON: {}", e))?;
                serde_json::json!({
                    "command_type": "exercise",
                    "template_id": self.template,
                    "contract_id": contract_id,
                    "choice": choice,
                    "choice_argument": choice_argument,
                })
            }
            other => {
                return Err(anyhow!(
                    "Unsupported command type '{}' (supported: create, exercise)",
                    other
                ))
            }
        };

        let result: serde_json::Value = rpc
            .call("tenzro_submitDamlCommand", params)
            .await?;

        spinner.finish_and_clear();

        output::print_success("DAML command submitted.");
        println!();

        output::print_field("Command Type", &self.command_type);
        output::print_field("Template ID", &self.template);

        if let Some(cid) = result.get("contract_id").and_then(|v| v.as_str()) {
            output::print_field("Contract ID", cid);
        }
        if let Some(choice) = result.get("choice").and_then(|v| v.as_str()) {
            output::print_field("Choice", choice);
        }
        if let Some(payload) = result.get("payload") {
            if !payload.is_null() {
                output::print_field(
                    "Payload",
                    &serde_json::to_string_pretty(payload).unwrap_or_default(),
                );
            }
        }
        if let Some(er) = result.get("exercise_result") {
            if !er.is_null() {
                output::print_field(
                    "Exercise Result",
                    &serde_json::to_string_pretty(er).unwrap_or_default(),
                );
            }
        }

        Ok(())
    }
}

// ── Canton 3.5+ extension subcommands ──

/// Shared shape for parameter-less Canton 3.5+ JSON Ledger API reads.
#[derive(Debug, Parser)]
pub struct CantonSimpleCmd {
    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    /// Operator-issued Tenzro API key (tnz_...) with scope `canton`.
    /// Falls back to the TENZRO_API_KEY env var when omitted.
    #[arg(long, env = "TENZRO_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

impl CantonSimpleCmd {
    pub async fn execute(&self, header: &str, method: &str) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header(header);

        let mut rpc = RpcClient::new(&self.rpc);
        if let Some(key) = &self.api_key {
            rpc = rpc.with_api_key(key);
        }
        let spinner = output::create_spinner("Querying Canton…");

        let result: serde_json::Value = rpc.call(method, serde_json::json!({})).await?;
        spinner.finish_and_clear();

        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "?".into())
        );
        Ok(())
    }
}

/// Fetch a Canton transaction tree by hex update id
#[derive(Debug, Parser)]
pub struct CantonGetTransactionCmd {
    /// Hex update id (Canton 3.5+ rejects bare labels)
    #[arg(long)]
    update_id: String,

    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    #[arg(long, env = "TENZRO_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

impl CantonGetTransactionCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;

        output::print_header("Canton Get Transaction");

        let mut rpc = RpcClient::new(&self.rpc);
        if let Some(key) = &self.api_key {
            rpc = rpc.with_api_key(key);
        }
        let params = serde_json::json!({ "update_id": self.update_id });
        let spinner = output::create_spinner("Fetching transaction…");

        let result: serde_json::Value =
            rpc.call("tenzro_canton_getTransaction", params).await?;
        spinner.finish_and_clear();

        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "?".into())
        );
        Ok(())
    }
}

/// Upload a DAR (DAML Archive) to the participant
#[derive(Debug, Parser)]
pub struct CantonUploadDarCmd {
    /// Path to the .dar file on disk
    #[arg(long)]
    file: std::path::PathBuf,

    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,

    #[arg(long, env = "TENZRO_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

impl CantonUploadDarCmd {
    pub async fn execute(&self) -> Result<()> {
        use crate::rpc::RpcClient;
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

        output::print_header("Canton Upload DAR");

        let dar_bytes = std::fs::read(&self.file)
            .map_err(|e| anyhow!("failed to read DAR file {}: {}", self.file.display(), e))?;
        let size = dar_bytes.len();
        let b64 = B64.encode(&dar_bytes);

        output::print_field("File", &self.file.display().to_string());
        output::print_field("Size (bytes)", &size.to_string());

        let mut rpc = RpcClient::new(&self.rpc);
        if let Some(key) = &self.api_key {
            rpc = rpc.with_api_key(key);
        }
        let params = serde_json::json!({ "dar_content_base64": b64 });
        let spinner = output::create_spinner("Uploading DAR…");

        let result: serde_json::Value =
            rpc.call("tenzro_canton_uploadDar", params).await?;
        spinner.finish_and_clear();

        output::print_success("DAR uploaded.");
        println!();
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "?".into())
        );
        Ok(())
    }
}
