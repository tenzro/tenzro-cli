//! Ceremony command for participating in ZK MPC trusted setup ceremonies.
//!
//! Allows participants to initialize, contribute to, verify, and finalize
//! Phase 1 (Powers of Tau) and Phase 2 (circuit-specific) trusted setup ceremonies.

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::fs;
use std::path::PathBuf;
use crate::output;

use tenzro_zk::{
    Phase1Accumulator, Phase1ContributionProof,
    contribute_phase1, apply_random_beacon,
};

/// ZK trusted setup ceremony commands
#[derive(Debug, Subcommand)]
pub enum CeremonyCommand {
    /// Initialize a new Phase 1 ceremony
    Init(InitCmd),
    /// Contribute to an existing ceremony
    Contribute(ContributeCmd),
    /// Verify a ceremony contribution
    Verify(VerifyCmd),
    /// Apply random beacon and finalize Phase 1
    Finalize(FinalizeCmd),
    /// Show ceremony status
    Status(StatusCmd),
}

/// Initialize a new Phase 1 ceremony
#[derive(Debug, Parser)]
pub struct InitCmd {
    /// Output file for the initial accumulator
    #[arg(short, long, default_value = "ceremony_init.bin")]
    output: PathBuf,

    /// Number of G1 powers (must be >= 2)
    #[arg(long, default_value = "32")]
    num_g1: usize,

    /// Number of G2 powers (must be >= 2)
    #[arg(long, default_value = "2")]
    num_g2: usize,

    /// Ceremony description
    #[arg(long, default_value = "Tenzro Network trusted setup ceremony")]
    description: String,
}

/// Contribute to an existing ceremony
#[derive(Debug, Parser)]
pub struct ContributeCmd {
    /// Input accumulator file
    #[arg(short, long)]
    input: PathBuf,

    /// Output accumulator file
    #[arg(short, long)]
    output: PathBuf,

    /// Contributor identifier (name or public key)
    #[arg(long)]
    id: String,

    /// Output file for contribution proof (optional)
    #[arg(long)]
    proof_output: Option<PathBuf>,
}

/// Verify a ceremony contribution
#[derive(Debug, Parser)]
pub struct VerifyCmd {
    /// Previous accumulator file
    #[arg(long)]
    prev: PathBuf,

    /// New accumulator file
    #[arg(long)]
    new: PathBuf,

    /// Proof file (optional, will verify hash chain if not provided)
    #[arg(long)]
    proof: Option<PathBuf>,
}

/// Apply random beacon and finalize Phase 1
#[derive(Debug, Parser)]
pub struct FinalizeCmd {
    /// Input accumulator file
    #[arg(short, long)]
    input: PathBuf,

    /// Output accumulator file
    #[arg(short, long)]
    output: PathBuf,

    /// Beacon value (hex string or file path)
    #[arg(long)]
    beacon: Option<String>,

    /// Number of hash iterations for beacon
    #[arg(long, default_value = "1024")]
    iterations: u32,
}

/// Show ceremony status
#[derive(Debug, Parser)]
pub struct StatusCmd {
    /// Accumulator file or transcript file
    #[arg(short, long)]
    file: PathBuf,
}

impl CeremonyCommand {
    pub async fn execute(&self) -> Result<()> {
        match self {
            CeremonyCommand::Init(cmd) => execute_init(cmd).await,
            CeremonyCommand::Contribute(cmd) => execute_contribute(cmd).await,
            CeremonyCommand::Verify(cmd) => execute_verify(cmd).await,
            CeremonyCommand::Finalize(cmd) => execute_finalize(cmd).await,
            CeremonyCommand::Status(cmd) => execute_status(cmd).await,
        }
    }
}

/// Execute init command
async fn execute_init(cmd: &InitCmd) -> Result<()> {
    output::print_header("Initialize Ceremony");
    println!();

    // Validate parameters
    if cmd.num_g1 < 2 {
        anyhow::bail!("num_g1 must be at least 2");
    }
    if cmd.num_g2 < 2 {
        anyhow::bail!("num_g2 must be at least 2");
    }

    output::print_field("Description", &cmd.description);
    output::print_field("G1 Powers", &cmd.num_g1.to_string());
    output::print_field("G2 Powers", &cmd.num_g2.to_string());
    output::print_field("Output", &cmd.output.display().to_string());
    println!();

    let spinner = output::create_spinner("Creating initial accumulator...");

    // Create initial accumulator (all generators)
    let accumulator = Phase1Accumulator::new(cmd.num_g1, cmd.num_g2);

    // Serialize using arkworks canonical serialization
    let bytes = accumulator.to_bytes()
        .context("Failed to serialize accumulator")?;

    spinner.finish_and_clear();

    // Write to file
    let spinner = output::create_spinner("Writing to file...");
    fs::write(&cmd.output, &bytes)
        .with_context(|| format!("Failed to write to {}", cmd.output.display()))?;
    spinner.finish_and_clear();

    println!();
    output::print_success(&format!(
        "Initial accumulator created ({} bytes)",
        bytes.len()
    ));

    let hash = accumulator.hash();
    output::print_field("Hash", &hex::encode(hash));

    println!();
    output::print_info("Next step: Participants can contribute using 'tenzro ceremony contribute'");

    Ok(())
}

/// Execute contribute command
async fn execute_contribute(cmd: &ContributeCmd) -> Result<()> {
    output::print_header("Contribute to Ceremony");
    println!();

    output::print_field("Contributor", &cmd.id);
    output::print_field("Input", &cmd.input.display().to_string());
    output::print_field("Output", &cmd.output.display().to_string());
    println!();

    // Load previous accumulator
    let spinner = output::create_spinner("Loading accumulator...");
    let input_bytes = fs::read(&cmd.input)
        .with_context(|| format!("Failed to read {}", cmd.input.display()))?;
    let prev_acc = Phase1Accumulator::from_bytes(&input_bytes)
        .context("Failed to deserialize accumulator")?;
    spinner.finish_and_clear();

    output::print_field("Previous Hash", &hex::encode(prev_acc.hash()));

    // Generate contribution using system entropy (OsRng)
    let spinner = output::create_spinner("Generating contribution (this may take a moment)...");
    let (new_acc, proof) = contribute_phase1(&prev_acc, &cmd.id)
        .context("Failed to generate contribution")?;
    spinner.finish_and_clear();

    println!();
    output::print_success("Contribution generated!");
    output::print_field("New Hash", &hex::encode(new_acc.hash()));

    // Write new accumulator
    let spinner = output::create_spinner("Writing new accumulator...");
    let new_bytes = new_acc.to_bytes()
        .context("Failed to serialize new accumulator")?;
    fs::write(&cmd.output, &new_bytes)
        .with_context(|| format!("Failed to write to {}", cmd.output.display()))?;
    spinner.finish_and_clear();

    // Write proof if requested
    if let Some(ref proof_path) = cmd.proof_output {
        let spinner = output::create_spinner("Writing proof...");
        // Serialize proof manually using custom format
        let proof_json = serde_json::json!({
            "prev_hash": hex::encode(proof.prev_accumulator_hash),
            "new_hash": hex::encode(proof.new_accumulator_hash),
            "tau_g1": serialize_g1_affine(&proof.tau_g1),
            "tau_g2": serialize_g2_affine(&proof.tau_g2),
            "alpha_g1": serialize_g1_affine(&proof.alpha_g1),
            "beta_g1": serialize_g1_affine(&proof.beta_g1),
            "beta_g2": serialize_g2_affine(&proof.beta_g2),
        });
        fs::write(proof_path, serde_json::to_string_pretty(&proof_json)?)
            .with_context(|| format!("Failed to write proof to {}", proof_path.display()))?;
        spinner.finish_and_clear();
        output::print_field("Proof saved", &proof_path.display().to_string());
    }

    println!();
    output::print_success(&format!("Contribution saved to {}", cmd.output.display()));
    println!();
    output::print_info("IMPORTANT: The contribution was generated using your system's");
    output::print_info("cryptographic random number generator. Your toxic waste has been");
    output::print_info("destroyed automatically.");

    Ok(())
}

/// Execute verify command
async fn execute_verify(cmd: &VerifyCmd) -> Result<()> {
    output::print_header("Verify Contribution");
    println!();

    output::print_field("Previous", &cmd.prev.display().to_string());
    output::print_field("New", &cmd.new.display().to_string());
    println!();

    // Load accumulators
    let spinner = output::create_spinner("Loading accumulators...");
    let prev_bytes = fs::read(&cmd.prev)
        .with_context(|| format!("Failed to read {}", cmd.prev.display()))?;
    let prev_acc = Phase1Accumulator::from_bytes(&prev_bytes)
        .context("Failed to deserialize previous accumulator")?;

    let new_bytes = fs::read(&cmd.new)
        .with_context(|| format!("Failed to read {}", cmd.new.display()))?;
    let new_acc = Phase1Accumulator::from_bytes(&new_bytes)
        .context("Failed to deserialize new accumulator")?;
    spinner.finish_and_clear();

    output::print_field("Previous Hash", &hex::encode(prev_acc.hash()));
    output::print_field("New Hash", &hex::encode(new_acc.hash()));

    // Load proof if provided and run full pairing verification
    if let Some(ref proof_path) = cmd.proof {
        let spinner = output::create_spinner("Loading proof...");
        let proof_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(proof_path)?
        )?;
        spinner.finish_and_clear();

        // Reconstruct the Phase1ContributionProof from serialized fields
        let proof = deserialize_contribution_proof(&proof_json)
            .context("Failed to deserialize contribution proof")?;

        let spinner = output::create_spinner("Running pairing-based verification...");
        let valid = tenzro_zk::ceremony::verify_phase1_contribution(&prev_acc, &new_acc, &proof)
            .context("Verification failed")?;
        spinner.finish_and_clear();

        if valid {
            println!();
            output::print_success("Full pairing verification passed!");
            output::print_info("Hash chain, cross-group consistency, and structural checks all valid");
        } else {
            anyhow::bail!("Contribution proof verification FAILED — pairing checks did not pass");
        }
    } else {
        // Basic hash chain check without full pairing verification
        println!();
        output::print_info("Note: Full pairing verification requires the proof file");
        output::print_info("Performing basic hash chain check...");

        if prev_acc.hash() == new_acc.hash() {
            output::print_warning("Accumulators have identical hashes (no change)");
        } else {
            output::print_success("Accumulators differ (contribution present)");
        }
    }

    println!();
    output::print_field("Previous G1", &prev_acc.num_g1().to_string());
    output::print_field("Previous G2", &prev_acc.num_g2().to_string());
    output::print_field("New G1", &new_acc.num_g1().to_string());
    output::print_field("New G2", &new_acc.num_g2().to_string());

    if prev_acc.num_g1() != new_acc.num_g1() || prev_acc.num_g2() != new_acc.num_g2() {
        anyhow::bail!("Accumulator dimensions changed (invalid)");
    }

    println!();
    output::print_success("Verification complete");

    Ok(())
}

/// Deserializes a `Phase1ContributionProof` from a JSON value.
///
/// Expected format: each G1/G2 element is hex-encoded compressed bytes,
/// and the accumulator hashes are hex strings.
fn deserialize_contribution_proof(json: &serde_json::Value) -> Result<Phase1ContributionProof> {
    use ark_serialize::CanonicalDeserialize;

    fn decode_g1(json: &serde_json::Value, field: &str) -> Result<ark_bn254::G1Affine> {
        let hex_str = json.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing field: {}", field))?;
        let bytes = hex::decode(hex_str).context("invalid hex")?;
        ark_bn254::G1Affine::deserialize_compressed(&bytes[..])
            .map_err(|e| anyhow::anyhow!("failed to deserialize {}: {}", field, e))
    }

    fn decode_g2(json: &serde_json::Value, field: &str) -> Result<ark_bn254::G2Affine> {
        let hex_str = json.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing field: {}", field))?;
        let bytes = hex::decode(hex_str).context("invalid hex")?;
        ark_bn254::G2Affine::deserialize_compressed(&bytes[..])
            .map_err(|e| anyhow::anyhow!("failed to deserialize {}: {}", field, e))
    }

    fn decode_hash(json: &serde_json::Value, field: &str) -> Result<[u8; 32]> {
        let hex_str = json.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing field: {}", field))?;
        let bytes = hex::decode(hex_str).context("invalid hex")?;
        let arr: [u8; 32] = bytes.try_into()
            .map_err(|_| anyhow::anyhow!("{} must be exactly 32 bytes", field))?;
        Ok(arr)
    }

    Ok(Phase1ContributionProof {
        tau_g1: decode_g1(json, "tau_g1")?,
        tau_g2: decode_g2(json, "tau_g2")?,
        alpha_g1: decode_g1(json, "alpha_g1")?,
        beta_g1: decode_g1(json, "beta_g1")?,
        beta_g2: decode_g2(json, "beta_g2")?,
        prev_accumulator_hash: decode_hash(json, "prev_hash")?,
        new_accumulator_hash: decode_hash(json, "new_hash")?,
    })
}

/// Execute finalize command
async fn execute_finalize(cmd: &FinalizeCmd) -> Result<()> {
    output::print_header("Finalize Ceremony");
    println!();

    output::print_field("Input", &cmd.input.display().to_string());
    output::print_field("Output", &cmd.output.display().to_string());
    println!();

    // Load accumulator
    let spinner = output::create_spinner("Loading accumulator...");
    let input_bytes = fs::read(&cmd.input)
        .with_context(|| format!("Failed to read {}", cmd.input.display()))?;
    let acc = Phase1Accumulator::from_bytes(&input_bytes)
        .context("Failed to deserialize accumulator")?;
    spinner.finish_and_clear();

    output::print_field("Current Hash", &hex::encode(acc.hash()));

    // Get beacon value
    let beacon_bytes = if let Some(ref beacon_str) = cmd.beacon {
        // Try to parse as hex first
        if let Ok(bytes) = hex::decode(beacon_str) {
            output::print_field("Beacon Source", "hex string");
            output::print_field("Beacon Value", beacon_str);
            bytes
        } else {
            // Try to read as file
            let path = PathBuf::from(beacon_str);
            if path.exists() {
                let bytes = fs::read(&path)
                    .with_context(|| format!("Failed to read beacon file {}", path.display()))?;
                output::print_field("Beacon Source", "file");
                output::print_field("Beacon File", &path.display().to_string());
                output::print_field("Beacon Size", &format!("{} bytes", bytes.len()));
                bytes
            } else {
                // Use as raw string bytes
                output::print_field("Beacon Source", "string");
                beacon_str.as_bytes().to_vec()
            }
        }
    } else {
        // Use default beacon (current timestamp)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        output::print_field("Beacon Source", "timestamp");
        output::print_field("Beacon Value", &timestamp.to_string());
        timestamp.to_le_bytes().to_vec()
    };

    output::print_field("Hash Iterations", &cmd.iterations.to_string());
    println!();

    // Apply random beacon
    let spinner = output::create_spinner("Applying random beacon...");
    let (final_acc, _proof) = apply_random_beacon(&acc, &beacon_bytes, cmd.iterations)
        .context("Failed to apply random beacon")?;
    spinner.finish_and_clear();

    println!();
    output::print_success("Random beacon applied!");
    output::print_field("Final Hash", &hex::encode(final_acc.hash()));

    // Write final accumulator
    let spinner = output::create_spinner("Writing final accumulator...");
    let final_bytes = final_acc.to_bytes()
        .context("Failed to serialize final accumulator")?;
    fs::write(&cmd.output, &final_bytes)
        .with_context(|| format!("Failed to write to {}", cmd.output.display()))?;
    spinner.finish_and_clear();

    println!();
    output::print_success(&format!("Finalized ceremony saved to {}", cmd.output.display()));
    println!();
    output::print_info("This accumulator can now be used for Phase 2 (circuit-specific setup)");

    Ok(())
}

/// Execute status command
async fn execute_status(cmd: &StatusCmd) -> Result<()> {
    output::print_header("Ceremony Status");
    println!();

    output::print_field("File", &cmd.file.display().to_string());
    println!();

    // Try to load as accumulator
    let spinner = output::create_spinner("Loading file...");
    let file_bytes = fs::read(&cmd.file)
        .with_context(|| format!("Failed to read {}", cmd.file.display()))?;

    match Phase1Accumulator::from_bytes(&file_bytes) {
        Ok(acc) => {
            spinner.finish_and_clear();
            output::print_success("Valid Phase 1 accumulator");
            println!();

            output::print_field("G1 Powers", &acc.num_g1().to_string());
            output::print_field("G2 Powers", &acc.num_g2().to_string());
            output::print_field("Hash", &hex::encode(acc.hash()));
            output::print_field("Size", &format!("{} bytes", file_bytes.len()));

            // Check if it's the identity accumulator (all generators)
            use ark_ec::AffineRepr;
            use ark_bn254::{G1Affine, G2Affine};
            let g1 = G1Affine::generator();
            let g2 = G2Affine::generator();

            let is_identity = acc.tau_powers_g1.iter().all(|p| *p == g1)
                && acc.tau_powers_g2.iter().all(|p| *p == g2)
                && acc.alpha_tau_powers_g1.iter().all(|p| *p == g1)
                && acc.beta_tau_powers_g1.iter().all(|p| *p == g1)
                && acc.beta_g2 == g2;

            println!();
            if is_identity {
                output::print_info("State: Initial (identity SRS, no contributions yet)");
            } else {
                output::print_info("State: Modified (contains contributions)");
            }
        }
        Err(e) => {
            spinner.finish_and_clear();
            output::print_warning(&format!("Not a valid accumulator: {}", e));
        }
    }

    Ok(())
}

// Helper functions for serializing curve points to JSON

use ark_bn254::{G1Affine, G2Affine};
use ark_serialize::CanonicalSerialize;

fn serialize_g1_affine(point: &G1Affine) -> String {
    let mut bytes = Vec::new();
    point.serialize_compressed(&mut bytes).unwrap();
    hex::encode(bytes)
}

fn serialize_g2_affine(point: &G2Affine) -> String {
    let mut bytes = Vec::new();
    point.serialize_compressed(&mut bytes).unwrap();
    hex::encode(bytes)
}
