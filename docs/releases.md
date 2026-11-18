# Versioning and releases

## Policy

This project follows [Semantic Versioning](https://semver.org/).

- `Cargo.toml` contains the executable version; `--version` reports it.
- The matching Git tag is `vMAJOR.MINOR.PATCH` (for example, `v0.2.0`).
- Update `Cargo.lock` and `CHANGELOG.md` with the version change.
- Until 1.0, changes to the configuration/protocol may occur in a minor release.
  Document them before rollout.
- Release tags and published assets are immutable by project policy. Never move
  a released tag, overwrite an archive, or reuse a version for different bytes.
  Publish a new patch/minor version instead. GitHub release immutability is enabled
  for this repository to lock a release's assets and tag after publication.
- Pin both version and expected archive checksum in Terraform/node provisioning.
  Do not use `/releases/latest/download/` for production provisioning.

## Publish a version

1. Prepare the change on a branch. Update the version, lockfile and changelog.
2. Open a pull request; require CI and Security checks to pass before merging.
3. From the merged `main` commit, create an annotated version tag and push it:

   ```sh
   git checkout main
   git pull --ff-only
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

The Release workflow validates the tag/version and `main` ancestry, runs tests,
formatting, Clippy and the dependency audit, then builds and tests the static Linux
amd64 and arm64 packages. Its publish job verifies checksums, generates GitHub
build-provenance attestations, creates a draft release, uploads both architectures
and publishes the release after upload succeeds. It refuses to replace an existing
release. The workflow's `workflow_dispatch` can publish the current version from
`main` without creating a tag locally; GitHub creates that version tag on publication.

CI Actions artifacts are retained temporarily for review. Terraform should download
from **GitHub Release assets**, whose URLs have the following shape:

```text
https://github.com/ciaicd/ciaicd-credential-provider/releases/download/v0.2.0/ciaicd-credential-provider-v0.2.0-linux-amd64.tar.gz
https://github.com/ciaicd/ciaicd-credential-provider/releases/download/v0.2.0/ciaicd-credential-provider-v0.2.0-linux-arm64.tar.gz
```

A failed publish may leave a draft release. Inspect it before retrying. Do not
silently overwrite published assets; delete only an incomplete, unpublished draft
when appropriate, or publish a new version.

## Verify a download

Select your version and architecture. The `.sha256` file is adjacent to its archive.

```sh
version=0.2.0
arch=amd64
archive="ciaicd-credential-provider-v${version}-linux-${arch}.tar.gz"
base="https://github.com/ciaicd/ciaicd-credential-provider/releases/download/v${version}"
curl --fail --location --proto '=https' --tlsv1.2 --output "$archive" "$base/$archive"
curl --fail --location --proto '=https' --tlsv1.2 --output "$archive.sha256" "$base/$archive.sha256"
sha256sum --check "$archive.sha256"
```

For stronger origin verification, use a recent GitHub CLI with attestation support:

```sh
gh attestation verify "$archive" \
  --repo ciaicd/ciaicd-credential-provider \
  --signer-workflow ciaicd/ciaicd-credential-provider/.github/workflows/release.yml
```

The attestation binds the archive digest to its GitHub build provenance. Check the
reported source commit and workflow. A checksum fetched from the same location
alone is an integrity check, not an independent origin guarantee. For unattended
bootstrap, provision a trusted expected checksum through Terraform after verifying
the release, as in `examples/terraform`.

## Local packaging

`package.sh` builds the two supported musl targets and writes deterministic archive
metadata plus per-archive SHA-256 files to `artifacts/`. It includes third-party
license text resolved from the locked dependency graph. Linux executables need no
interpreter or separately installed application libraries on the node. Identical
archive metadata does not imply bit-for-bit reproducible builds across different
Rust toolchains or source revisions.

Build-time requirements are Rust/rustup, the musl targets and Python 3. No token or
cloud account is required to build. Publishing requires the GitHub workflow token
and its explicitly declared permissions, not a stored personal access token.
