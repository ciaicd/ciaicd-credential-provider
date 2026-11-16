terraform {
  required_version = ">= 1.5.0"
}

variable "plugin_version" {
  type        = string
  description = "Pinned released version without the v prefix."
  validation {
    condition     = can(regex("^[0-9]+\\.[0-9]+\\.[0-9]+$", var.plugin_version))
    error_message = "Use a pinned MAJOR.MINOR.PATCH version, not latest."
  }
}

variable "node_architecture" {
  type        = string
  default     = "amd64"
  description = "Architecture of this node pool: amd64 or arm64."
  validation {
    condition     = contains(["amd64", "arm64"], var.node_architecture)
    error_message = "Supported architectures are amd64 and arm64."
  }
}

variable "archive_sha256" {
  type        = string
  description = "Trusted expected SHA-256 of the exact version/architecture release archive."
  validation {
    condition     = can(regex("^[a-fA-F0-9]{64}$", var.archive_sha256))
    error_message = "Provide the 64-character SHA-256 verified from the release."
  }
}

variable "registry_host" {
  type        = string
  description = "Exact CIAICD registry hostname with optional port; no URL scheme or path."
  validation {
    condition     = can(regex("^[A-Za-z0-9][A-Za-z0-9.:-]*$", var.registry_host))
    error_message = "Use an exact registry hostname and optional port."
  }
}

variable "registry_audience" {
  type        = string
  default     = "ciaicd-image-pull"
  description = "Dedicated ServiceAccount token audience trusted by CIAICD."
  validation {
    condition     = can(regex("^[A-Za-z0-9][A-Za-z0-9._:/-]*$", var.registry_audience))
    error_message = "Use an audience containing only letters, digits, dots, underscores, colons, slashes and hyphens."
  }
}

locals {
  archive_name = "ciaicd-credential-provider-v${var.plugin_version}-linux-${var.node_architecture}.tar.gz"
  download_url = "https://github.com/ciaicd/ciaicd-credential-provider/releases/download/v${var.plugin_version}/${local.archive_name}"
  node_bootstrap_script = templatefile("${path.module}/node-bootstrap.sh.tftpl", {
    download_url      = local.download_url
    archive_sha256    = lower(var.archive_sha256)
    registry_host     = var.registry_host
    registry_audience = var.registry_audience
  })
}

output "release_download_url" {
  value = local.download_url
}

output "node_bootstrap_script" {
  value       = local.node_bootstrap_script
  description = "Merge into your supported node provisioning/user-data mechanism; this does not configure kubelet flags."
}
