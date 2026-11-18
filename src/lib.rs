#![forbid(unsafe_code)]
//! Kubelet credential-provider v1 transport. CIAICD, not this plugin, verifies JWT signatures.
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io::Read};

pub const API_VERSION: &str = "credentialprovider.kubelet.k8s.io/v1";
pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_TOKEN_BYTES: usize = 32768;
const EXPIRY_MARGIN_SECONDS: u64 = 60;
const MAX_CACHE_SECONDS: u64 = 300;

/// Deliberately contains no token, request data or parser diagnostic.
#[derive(Debug, PartialEq, Eq)]
pub struct CredentialError;

type Result<T> = std::result::Result<T, CredentialError>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    api_version: String,
    kind: String,
    image: String,
    service_account_token: Option<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub api_version: &'static str,
    pub kind: &'static str,
    pub cache_key_type: &'static str,
    pub cache_duration: String,
    // Do not derive Debug on AuthConfig: responses can carry the original credential.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub auth: BTreeMap<String, AuthConfig>,
}

#[derive(Serialize, PartialEq, Eq)]
pub struct AuthConfig {
    pub username: &'static str,
    pub password: String,
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthConfig")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Deserialize)]
struct Claims {
    aud: Vec<String>,
    exp: u64,
}

pub struct Config {
    registry: String,
    audience: String,
}

impl Config {
    pub fn new(registry: String, audience: String) -> Result<Self> {
        if registry.is_empty()
            || !registry
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-_:[]".contains(&b))
            || audience.trim().is_empty()
        {
            return Err(CredentialError);
        }
        Ok(Self { registry, audience })
    }
}

/// Read a bounded request; do not buffer arbitrary data from stdin.
pub fn read_request(reader: impl Read) -> Result<Request> {
    let mut raw = Vec::new();
    reader
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| CredentialError)?;
    if raw.len() > MAX_REQUEST_BYTES {
        return Err(CredentialError);
    }
    serde_json::from_slice(&raw).map_err(|_| CredentialError)
}

pub fn credentials(request: Request, config: &Config, now: u64) -> Result<Response> {
    if request.api_version != API_VERSION || request.kind != "CredentialProviderRequest" {
        return Err(CredentialError);
    }
    let mut response = Response {
        api_version: API_VERSION,
        kind: "CredentialProviderResponse",
        cache_key_type: "Registry",
        cache_duration: "0s".into(),
        auth: BTreeMap::new(),
    };
    let (registry, _) = request.image.split_once('/').ok_or(CredentialError)?;
    if registry != config.registry {
        return Ok(response);
    }
    let token = request.service_account_token.ok_or(CredentialError)?;
    if token.len() > MAX_TOKEN_BYTES {
        return Err(CredentialError);
    }
    let mut segments = token.split('.');
    let header = segments.next().ok_or(CredentialError)?;
    let payload = segments.next().ok_or(CredentialError)?;
    let signature = segments.next().ok_or(CredentialError)?;
    if header.is_empty() || signature.is_empty() || segments.next().is_some() {
        return Err(CredentialError);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| CredentialError)?;
    let claims: Claims = serde_json::from_slice(&payload).map_err(|_| CredentialError)?;
    if claims.aud != [config.audience.as_str()] {
        return Err(CredentialError);
    }
    let ttl = claims
        .exp
        .checked_sub(now)
        .and_then(|remaining| remaining.checked_sub(EXPIRY_MARGIN_SECONDS))
        .filter(|remaining| *remaining > 0)
        .ok_or(CredentialError)?;
    // Unverified JWT claims only bound caching; they never authorize access. CIAICD verifies
    // issuer, signature, pod binding and exact ServiceAccount mappings at the token exchange.
    // Kubelet MUST be configured with tokenAttributes.cacheType: Token for pod isolation.
    response.cache_duration = format!("{}s", ttl.min(MAX_CACHE_SECONDS));
    response.auth.insert(
        config.registry.clone(),
        AuthConfig {
            username: "ciaicd_oidc_access_token",
            password: token,
        },
    );
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn config() -> Config {
        Config::new("registry.example:5443".into(), "ciaicd-image-pull".into()).unwrap()
    }
    fn token(claims: Value) -> String {
        format!(
            "header.{}.signature",
            URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap())
        )
    }
    fn request(token: Option<&str>, image: &str) -> Request {
        serde_json::from_value(
            json!({"apiVersion":API_VERSION,"kind":"CredentialProviderRequest",
            "image":image,"serviceAccountToken":token}),
        )
        .unwrap()
    }
    fn normal_token(exp: u64) -> String {
        token(
            json!({"sub":"system:serviceaccount:production:reader","exp":exp,"aud":["ciaicd-image-pull"]}),
        )
    }
    #[test]
    fn passes_the_original_token_and_expires_cache_before_it() {
        let jwt = normal_token(1300);
        let response = credentials(
            request(Some(&jwt), "registry.example:5443/images/app:v1"),
            &config(),
            1000,
        )
        .unwrap();
        assert_eq!(response.cache_duration, "240s");
        assert_eq!(
            response.auth["registry.example:5443"].username,
            "ciaicd_oidc_access_token"
        );
        assert_eq!(response.auth["registry.example:5443"].password, jwt);
        assert!(!format!("{response:?}").contains(&jwt));
    }
    #[test]
    fn caps_caching_at_five_minutes() {
        let jwt = normal_token(9000);
        assert_eq!(
            credentials(
                request(Some(&jwt), "registry.example:5443/images/app"),
                &config(),
                1000
            )
            .unwrap()
            .cache_duration,
            "300s"
        );
    }
    #[test]
    fn never_returns_credentials_to_a_different_registry() {
        for image in [
            "registry.example:5443.attacker/app",
            "other.example/app",
            "registry.example:5444/app",
        ] {
            assert!(
                credentials(request(None, image), &config(), 1000)
                    .unwrap()
                    .auth
                    .is_empty()
            );
        }
    }
    #[test]
    fn rejects_missing_malformed_expired_or_oversized_tokens() {
        for jwt in [
            String::new(),
            "not-a-jwt".into(),
            "h.!!!.s".into(),
            ".e30.s".into(),
            "h.e30.".into(),
            "h.e30.s.extra".into(),
            "x".repeat(MAX_TOKEN_BYTES + 1),
            normal_token(999),
            normal_token(1060),
        ] {
            assert!(
                credentials(
                    request(Some(&jwt), "registry.example:5443/app"),
                    &config(),
                    1000
                )
                .is_err()
            );
        }
        assert!(credentials(request(None, "registry.example:5443/app"), &config(), 1000).is_err());
    }
    #[test]
    fn rejects_wrong_or_multiple_audiences_and_noninteger_expiry() {
        for claims in [
            json!({"aud":["other"],"exp":1300}),
            json!({"aud":["ciaicd-image-pull","other"],"exp":1300}),
            json!({"aud":"ciaicd-image-pull","exp":1300}),
            json!({"aud":["ciaicd-image-pull"],"exp":true}),
            json!({"aud":["ciaicd-image-pull"],"exp":-1}),
            json!({"aud":["ciaicd-image-pull"],"exp":1300.5}),
            json!({"aud":["ciaicd-image-pull"]}),
        ] {
            assert!(
                credentials(
                    request(Some(&token(claims)), "registry.example:5443/app"),
                    &config(),
                    1000
                )
                .is_err()
            );
        }
    }
    #[test]
    fn preserves_separate_pod_tokens() {
        let first = normal_token(1300);
        let second = token(
            json!({"aud":["ciaicd-image-pull"],"exp":1300,"sub":"system:serviceaccount:private:reader"}),
        );
        assert_ne!(
            credentials(
                request(Some(&first), "registry.example:5443/app"),
                &config(),
                1000
            )
            .unwrap()
            .auth,
            credentials(
                request(Some(&second), "registry.example:5443/app"),
                &config(),
                1000
            )
            .unwrap()
            .auth
        );
    }
    #[test]
    fn validates_envelope_and_bounded_input() {
        for input in [b"not-json".to_vec(), vec![b' '; MAX_REQUEST_BYTES + 1],
            br#"{"apiVersion":"v1","apiVersion":"duplicate","kind":"CredentialProviderRequest","image":"registry.example/app"}"#.to_vec()] {
            assert!(read_request(input.as_slice()).is_err());
        }
        let req = serde_json::from_value(json!({"apiVersion":"wrong","kind":"CredentialProviderRequest","image":"registry.example/app"})).unwrap();
        assert!(credentials(req, &config(), 1000).is_err());
    }
    #[test]
    fn rejects_url_wildcard_and_credential_registry_configuration() {
        for host in [
            "",
            "https://registry.example",
            "user@registry.example",
            "*.example",
            "registry.example/path",
            "registry.example\n",
        ] {
            assert!(Config::new(host.into(), "audience".into()).is_err());
        }
        assert!(Config::new("registry.example".into(), " ".into()).is_err());
    }
}
