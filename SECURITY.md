# Security policy

## Supported versions

Security fixes are released for the latest published version. Upgrade to the latest
release and pin its version and SHA-256 in your node provisioning configuration.
Until the first release is published, the project is a development preview.

## Reporting a vulnerability

Use **Security → Advisories → Report a vulnerability** on this GitHub repository
when private vulnerability reporting is enabled. If that option is unavailable,
contact a repository maintainer privately through GitHub before sharing technical
exploit details. Do not put tokens, credentials, private cluster endpoints or
unpatched vulnerability details in a public issue.

There is no guaranteed response time or paid support SLA.

## Trust boundaries

- The provider runs on the node, invoked by kubelet. Treat its executable,
  configuration, installation source and node administrator as trusted.
- It passes the pod-bound ServiceAccount token to a configured CIAICD registry.
  It does not verify JWT signatures or independently authorize image access.
- CIAICD must verify the signature, exact trusted issuer, dedicated audience,
  expiry, pod-binding claims and explicit ServiceAccount mappings.
- Keep `tokenAttributes.cacheType: Token`; do not reuse a pod-bound credential
  across tokens, pods or ServiceAccounts.
- The configured registry hostname is an exact match. Missing credentials never
  fall back to node identity or cloud metadata credentials.
- The provider performs no network calls and writes no credential files. Credentials
  are emitted on stdout only as required by kubelet's protocol. Diagnostics and
  debug formatting redact credential values.
- Offline registry verification does not immediately invalidate a token when its
  pod or ServiceAccount is deleted. Use short lifetimes and registry-side revocation.

## Release verification

Release archives include SHA-256 files and GitHub build-provenance attestations.
Verify the expected checksum and signer repository/workflow before installation.
A checksum downloaded from the same location detects corruption; it is not by
itself an independent proof of who produced the file. See `docs/releases.md`.

## Automated checks

CI runs tests, formatting and Clippy on Linux amd64 and arm64. Security workflows
run Cargo Deny (RustSec advisories, licenses, source policy) and CodeQL for Rust and
GitHub Actions. Dependabot proposes dependency/action updates. Workflows pin action
commit SHAs and use job-scoped permissions. Passing checks do not certify the
absence of vulnerabilities or replace a review of your node and registry setup.
