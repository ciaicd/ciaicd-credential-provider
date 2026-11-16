#!/usr/bin/env sh
set -eu
crate_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$crate_dir/Cargo.toml")
output_dir=${ARTIFACT_OUTPUT_DIR:-"$crate_dir/artifacts"}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$crate_dir/target"}
if [ "$#" -eq 0 ]; then
  set -- x86_64-unknown-linux-musl aarch64-unknown-linux-musl
fi
for target in "$@"; do
  case "$target" in
    x86_64-unknown-linux-musl) arch=amd64 ;;
    aarch64-unknown-linux-musl) arch=arm64 ;;
    *) printf '%s\n' 'Supported targets: x86_64-unknown-linux-musl, aarch64-unknown-linux-musl' >&2; exit 1 ;;
  esac
  RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$crate_dir=. -C linker=rust-lld" cargo build \
    --locked --release --manifest-path "$crate_dir/Cargo.toml" --target "$target"
  python3 "$crate_dir/scripts/package.py" \
    --binary "$CARGO_TARGET_DIR/$target/release/ciaicd-credential-provider" \
    --version "$version" --arch "$arch" --output "$output_dir"
done
