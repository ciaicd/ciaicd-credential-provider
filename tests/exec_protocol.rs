use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

fn execute(request: &[u8], registry: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ciaicd-credential-provider"))
        .env_clear()
        .env("CIAICD_REGISTRY", registry)
        .env("CIAICD_AUDIENCE", "ciaicd-image-pull")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Large/invalid input may cause the plugin to close stdin before the complete write.
    let _ = child.stdin.take().unwrap().write_all(request);
    child.wait_with_output().unwrap()
}
fn request(token: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"apiVersion":"credentialprovider.kubelet.k8s.io/v1",
        "kind":"CredentialProviderRequest","image":"registry.example/images/app:v1","serviceAccountToken":token})).unwrap()
}
#[test]
fn real_executable_returns_the_kubelet_protocol_without_diagnostics_on_stdout() {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 600;
    let token = format!(
        "header.{}.signature",
        URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&json!({"aud":["ciaicd-image-pull"],"exp":exp})).unwrap())
    );
    let output = execute(&request(&token), "registry.example");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["kind"], "CredentialProviderResponse");
    assert_eq!(response["auth"]["registry.example"]["password"], token);
    assert_eq!(
        response["auth"]["registry.example"]["username"],
        "ciaicd_oidc_access_token"
    );
    assert_eq!(response["cacheDuration"], "300s");
}
#[test]
fn executable_fails_without_emitting_or_logging_the_supplied_secret() {
    let token = "confidential-token-that-must-not-be-logged";
    for (input, registry) in [
        (request(token), "registry.example"),
        (request(token), "https://invalid"),
        (vec![b' '; 1024 * 1024 + 1], "registry.example"),
    ] {
        let output = execute(&input, registry);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(!diagnostic.contains(token));
        assert_eq!(diagnostic.lines().count(), 1);
    }
}
#[test]
fn version_does_not_require_credentials_or_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_ciaicd-credential-provider"))
        .env_clear()
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .starts_with("ciaicd-credential-provider ")
    );
    assert!(output.stderr.is_empty());
}
