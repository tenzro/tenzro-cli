//! Inference request commands for the Tenzro CLI

use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::output;

/// Inference commands
#[derive(Debug, Subcommand)]
pub enum InferenceCommand {
    /// Submit an inference request
    Request(InferenceRequestCmd),
}

impl InferenceCommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            Self::Request(cmd) => cmd.execute().await,
        }
    }
}

/// Submit an inference request
#[derive(Debug, Parser)]
pub struct InferenceRequestCmd {
    /// Model ID to use
    model_id: String,

    /// Input text, file path, or URL
    input: String,

    /// Maximum price willing to pay (TNZO)
    #[arg(long, default_value = "1.0")]
    max_price: String,

    /// Require TEE attestation
    #[arg(long)]
    require_tee: bool,

    /// Temperature (0.0-2.0, for text models)
    #[arg(long)]
    temperature: Option<f32>,

    /// Maximum output tokens (for text models)
    #[arg(long)]
    max_tokens: Option<u32>,

    /// Save output to file
    #[arg(long)]
    output_file: Option<String>,

    /// RPC endpoint
    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc: String,
}

impl InferenceRequestCmd {
    pub async fn execute(&self) -> Result<()> {
        output::print_header("Inference Request");

        // Parse max price
        let _max_price_float: f64 = self.max_price.parse()?;

        // Determine if input is a file or direct text
        let input_preview = if std::path::Path::new(&self.input).exists() {
            format!("File: {}", self.input)
        } else if self.input.starts_with("http://") || self.input.starts_with("https://") {
            format!("URL: {}", self.input)
        } else {
            // Direct text input
            if self.input.len() > 100 {
                format!("{}...", &self.input[..97])
            } else {
                self.input.clone()
            }
        };

        // Show request details
        println!();
        output::print_field("Model", &self.model_id);
        output::print_field("Input", &input_preview);
        output::print_field("Max Price", &format!("{} TNZO", self.max_price));

        if self.require_tee {
            output::print_field("TEE Required", "Yes");
        }

        if let Some(temp) = self.temperature {
            output::print_field("Temperature", &format!("{:.2}", temp));
        }

        if let Some(max_tokens) = self.max_tokens {
            output::print_field("Max Tokens", &max_tokens.to_string());
        }

        println!();

        // Confirm with user
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt("Submit inference request?")
            .default(true)
            .interact()?;

        if !confirmed {
            output::print_warning("Request cancelled");
            return Ok(());
        }

        let spinner = output::create_spinner("Creating inference request...");

        use crate::rpc::RpcClient;
        let rpc = RpcClient::new(&self.rpc);

        spinner.set_message("Finding available provider...");

        // Submit inference request
        let result: serde_json::Value = rpc.call("tenzro_inferenceRequest", serde_json::json!([{
            "model_id": self.model_id,
            "input": self.input,
            "max_price": self.max_price,
            "require_tee": self.require_tee,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens
        }])).await?;

        spinner.set_message("Receiving response...");

        spinner.finish_and_clear();

        output::print_success("Inference completed successfully!");

        // Display response
        println!();
        output::print_header("Response");
        println!();

        if let Some(response_text) = result.get("response").and_then(|v| v.as_str()) {
            println!("{}", response_text);
        } else if let Some(output) = result.get("output") {
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("Model inference completed. Response data received.");
        }

        println!();

        // Show metadata
        output::print_header("Metadata");
        println!();

        if let Some(request_id) = result.get("request_id").and_then(|v| v.as_str()) {
            output::print_field("Request ID", request_id);
        }
        if let Some(provider) = result.get("provider").and_then(|v| v.as_str()) {
            output::print_field("Provider", &output::format_address(provider));
        }
        if let Some(latency) = result.get("latency").and_then(|v| v.as_f64()) {
            output::print_field("Latency", &format!("{:.2} seconds", latency));
        }
        if let Some(input_tokens) = result.get("input_tokens").and_then(|v| v.as_u64()) {
            output::print_field("Input Tokens", &input_tokens.to_string());
        }
        if let Some(output_tokens) = result.get("output_tokens").and_then(|v| v.as_u64()) {
            output::print_field("Output Tokens", &output_tokens.to_string());
        }
        if let Some(price) = result.get("actual_price").and_then(|v| v.as_str()) {
            output::print_field("Actual Price", price);
        }

        if self.require_tee {
            if let Some(attestation) = result.get("attestation") {
                println!();
                output::print_success("TEE Attestation verified");
                if let Some(vendor) = attestation.get("vendor").and_then(|v| v.as_str()) {
                    output::print_field("Vendor", vendor);
                }
                if let Some(enclave_id) = attestation.get("enclave_id").and_then(|v| v.as_str()) {
                    output::print_field("Enclave ID", enclave_id);
                }
            }
        }

        // Save to file if requested
        if let Some(output_file) = &self.output_file {
            if let Some(response_text) = result.get("response").and_then(|v| v.as_str()) {
                std::fs::write(output_file, response_text)?;
                println!();
                output::print_success(&format!("Response saved to: {}", output_file));
            }
        }

        Ok(())
    }
}
