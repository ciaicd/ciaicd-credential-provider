# Changelog

This project follows Semantic Versioning. Release notes are published on GitHub Releases.

## 0.2.0

- Use `ciaicd_oidc_access_token` as the Docker username for Kubernetes ServiceAccount tokens.
  Deploy a CIAICD Registry version that accepts this username before updating the provider on nodes.

## 0.1.0

Initial development release:

- Rust kubelet credential-provider v1 executable for CIAICD.
- Short-lived pod-bound ServiceAccount token transport across cloud environments.
- Exact registry matching, bounded input, expiry-aware caching and redacted errors.
- Static Linux amd64 and arm64 archives with SHA-256 checksums.
- Versioned release workflow with build-provenance attestations.

Kubernetes 1.36 integration requires a compatible registry build and a node offering
that supports custom kubelet exec credential providers. No managed-cloud installation
or external security audit is implied by this release.
