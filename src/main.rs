#![forbid(unsafe_code)]
use ciaicd_credential_provider::{Config, CredentialError, credentials, read_request};
use std::{
    env,
    io::{self, Write},
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

fn run() -> Result<(), CredentialError> {
    let config = Config::new(
        env::var("CIAICD_REGISTRY").map_err(|_| CredentialError)?,
        env::var("CIAICD_AUDIENCE").map_err(|_| CredentialError)?,
    )?;
    let request = read_request(io::stdin().lock())?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CredentialError)?
        .as_secs();
    let response = credentials(request, &config, now)?;
    // Serialize completely before writing so parse/configuration failures leave stdout empty.
    let mut output = serde_json::to_vec(&response).map_err(|_| CredentialError)?;
    output.push(b'\n');
    io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|_| CredentialError)
}

fn main() -> ExitCode {
    if env::args().len() == 2 && env::args().nth(1).as_deref() == Some("--version") {
        println!("ciaicd-credential-provider {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if env::args().len() != 1 || run().is_err() {
        eprintln!(
            "ciaicd-credential-provider: could not obtain ServiceAccount registry credentials"
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
