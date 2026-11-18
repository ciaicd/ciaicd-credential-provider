# CIAICD credential provider

[![CI](https://github.com/ciaicd/ciaicd-credential-provider/actions/workflows/ci.yml/badge.svg)](https://github.com/ciaicd/ciaicd-credential-provider/actions/workflows/ci.yml)
[![Security checks](https://github.com/ciaicd/ciaicd-credential-provider/actions/workflows/security.yml/badge.svg)](https://github.com/ciaicd/ciaicd-credential-provider/actions/workflows/security.yml)
[![Release](https://github.com/ciaicd/ciaicd-credential-provider/actions/workflows/release.yml/badge.svg)](https://github.com/ciaicd/ciaicd-credential-provider/actions/workflows/release.yml)
[![Latest release](https://img.shields.io/github/v/release/ciaicd/ciaicd-credential-provider)](https://github.com/ciaicd/ciaicd-credential-provider/releases)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

A Rust-based Kubernetes kubelet credential provider for secure image pulls from
**CIAICD**, using short-lived, pod-bound ServiceAccount tokens.

The same protocol works independently of AWS, Google Cloud and Azure identities.
The executable performs no network calls and requires no Python interpreter,
Rust toolchain or stored registry password on the node.

> This is an early project. CI badges link to real checks; they are not a security
> certification. A compatible CIAICD build and a node offering that supports custom
> kubelet exec credential providers are required.

## How it works

1. Kubelet requests a pod-bound ServiceAccount token for audience `ciaicd-image-pull`.
2. It invokes `ciaicd-credential-provider`, passing a `CredentialProviderRequest`
   through stdin using `credentialprovider.kubelet.k8s.io/v1`.
3. The provider returns the original token as the Docker password for
   `ciaicd_oidc_access_token`, only for the exact configured registry hostname.
4. The container runtime follows the registry's normal Distribution Bearer challenge.
5. CIAICD verifies the JWT signature, trusted cluster issuer, dedicated audience,
   expiry, pod binding and exact ServiceAccount-to-role mapping, then issues a
   short-lived, image-scoped, **pull-only** registry token.

The plugin parses JWT claims only to bound credential caching. **It does not verify
signatures or authorize access; the registry must do that.** Missing, malformed,
expired and wrong-audience tokens fail without a cloud/node identity fallback.

## Download

Use the [versioned GitHub Releases](https://github.com/ciaicd/ciaicd-credential-provider/releases),
not temporary Actions artifacts or an unpinned `latest` download URL.

| Node architecture | Release archive |
| --- | --- |
| Linux x86-64 / amd64 | `ciaicd-credential-provider-v<VERSION>-linux-amd64.tar.gz` |
| Linux ARM64 / aarch64 | `ciaicd-credential-provider-v<VERSION>-linux-arm64.tar.gz` |

Each release includes SHA-256 checksum files, third-party license notices and GitHub
build-provenance attestations. Archives contain the executable, example kubelet
configuration, README, license and notices. The Linux binaries are statically
linked with musl. See [release verification](docs/releases.md#verify-a-download).

## Install on Kubernetes 1.36

Verify and extract the archive for your node architecture. From the extracted directory:

```sh
sudo install -d -m 0755 /usr/local/libexec/ciaicd /etc/kubernetes
sudo install -m 0755 ciaicd-credential-provider /usr/local/libexec/ciaicd/ciaicd-credential-provider
sudo install -m 0600 credential-provider.example.yaml /etc/kubernetes/ciaicd-credential-provider.yaml
```

Edit the example registry hostname and audience, then configure kubelet with:

```text
--image-credential-provider-bin-dir=/usr/local/libexec/ciaicd
--image-credential-provider-config=/etc/kubernetes/ciaicd-credential-provider.yaml
```

The executable filename and provider `name` must both be
`ciaicd-credential-provider`. If the node already has cloud credential providers,
merge the new entry into its existing configuration and retain existing binaries
and flags. Do not replace a working cloud-managed provider setup.

```yaml
apiVersion: kubelet.config.k8s.io/v1
kind: CredentialProviderConfig
providers:
  - name: ciaicd-credential-provider
    apiVersion: credentialprovider.kubelet.k8s.io/v1
    matchImages: ["registry.example.com"]
    defaultCacheDuration: "0s"
    tokenAttributes:
      serviceAccountTokenAudience: "ciaicd-image-pull"
      cacheType: Token
      requireServiceAccount: true
    env:
      - name: CIAICD_REGISTRY
        value: "registry.example.com"
      - name: CIAICD_AUDIENCE
        value: "ciaicd-image-pull"
```

Keep `cacheType: Token`: a pod-bound credential must not be reused for other tokens
or pods. The plugin caps caching at five minutes and ends it at least 60 seconds
before token expiry. `matchImages` and `CIAICD_REGISTRY` must include the registry
port if it is non-default. The two audience fields must match the registry's audience.

Configure one Kubernetes trust provider per cluster in CIAICD, mapping an exact
subject such as `system:serviceaccount:production:image-reader` to a local read role.
The [registry configuration example](identity.service-account.example.json) is for
a new installation; merge its provider into existing settings rather than replacing
unrelated providers or remote bindings.

The [ServiceAccount/RBAC/Pod example](service-account.example.yaml) demonstrates the
node audience permission needed with Kubernetes audience restrictions. Create the
`production` namespace first. The application pod selects its ServiceAccount;
no stored `imagePullSecrets` are required for this flow.

### Managed clusters and Terraform

Kubernetes 1.36 supplies the protocol, but **not every managed node offering lets
you install custom executables or change kubelet flags**. Validate the supported
provisioning path for each EKS, GKE or AKS node pool. Static pods without a
ServiceAccount cannot use this provider. There is no built-in integration with
ECR, ACR or Artifact Registry authentication: the registry-side contract is CIAICD's.

The [Terraform example](examples/terraform) produces a node bootstrap script that
downloads a pinned version from this repository, verifies a trusted expected
SHA-256 and installs the binary/config. It does not create cloud resources or
reconfigure/restart kubelet. Integrate it into the supported node provisioning
method and preserve existing credential providers. New and replacement nodes
must receive the same installation automatically.

## Build and test

The Rust toolchain is pinned in `rust-toolchain.toml`; dependencies are pinned in
`Cargo.lock`, which is intentionally committed for this executable.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
./target/release/ciaicd-credential-provider --version
```

Build both static Linux archives (Python 3 is used only by the build-time packager):

```sh
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
./package.sh
```

Outputs are written to `artifacts/` and are excluded from Git. CI runs tests on
Linux amd64 and arm64 and executes each packaged binary on its native architecture.
Local macOS executables are for development; use Linux archives on worker nodes.

## Security and maintenance

- [Security policy and reporting](SECURITY.md)
- [Versioning, release process and verification](docs/releases.md)
- [Contributing](CONTRIBUTING.md) and [code of conduct](CODE_OF_CONDUCT.md)
- [Changelog](CHANGELOG.md)

Security checks run Cargo Deny against RustSec advisories, license and source
policies, plus CodeQL for Rust and workflow code. Dependabot proposes updates.
Release workflows use pinned action SHAs and job-scoped permissions. The project
has not undergone an independent security audit.

Offline JWT verification does not immediately revoke credentials when a pod or
ServiceAccount is deleted. Tokens expire naturally; remove the CIAICD subject
mapping or disable the provider/user to revoke registry access earlier.

## License

[Apache License 2.0](LICENSE). Release archives include third-party dependency notices.
