# Terraform node bootstrap example

This example renders a bootstrap script for a node pool. It creates no cloud
resources and contains no credentials. It downloads a **pinned GitHub Release**,
checks a trusted SHA-256 before extraction, and installs the static executable.

```hcl
module "credential_provider" {
  source            = "./examples/terraform"
  plugin_version    = "0.2.0"
  node_architecture = "amd64"
  archive_sha256    = "<trusted 64-character archive checksum>"
  registry_host     = "registry.example.com"
  registry_audience = "ciaicd-image-pull"
}
```

Verify the release checksum and provenance before setting `archive_sha256`. There
is deliberately no default checksum. Use a separate checksum for each architecture.
The source path above assumes the module is consumed from the repository root;
adjust it for your Terraform checkout or vendor/pin the module source.

Integrate `module.credential_provider.node_bootstrap_script` into your cloud's
**supported** node initialization mechanism. EKS AL2023 managed node groups can
use launch-template user data/nodeadm; preserve existing MIME parts and kubelet
flags. GKE and AKS expose limited kubelet settings: do not assume arbitrary exec
provider installation is supported for a particular managed node type.

The script requires curl, sha256sum, tar and install on the node. It needs no
Python interpreter or Rust toolchain. It installs a separate configuration fragment
and refuses to overwrite an existing one. It does **not** merge existing cloud
provider configurations, alter kubelet flags, restart kubelet, grant audience RBAC,
or configure CIAICD trust. Complete those steps using the supported cluster/node
provisioning process. Do not run it manually on production nodes as a substitute
for repeatable node-pool configuration.

Kubelet flags and ServiceAccount audience RBAC are documented in the root README.
Pin and test the installation before rolling it out to new/replacement nodes.
