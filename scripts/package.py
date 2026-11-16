#!/usr/bin/env python3
"""Package a release with stable metadata and complete dependency license texts."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile


def dependency_notices(crate):
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--manifest-path", str(crate / "Cargo.toml")],
        text=True))
    output = ["Third-party dependency notices\n\nResolved from Cargo.lock, including build dependencies.\n"]
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if not package.get("source"):
            continue
        output.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nLicense: {package.get('license', 'see source')}\n")
        directory = Path(package["manifest_path"]).parent
        files = sorted(p for p in directory.iterdir() if p.is_file()
                       and (p.name.upper().startswith("LICENSE") or p.name.upper().startswith("COPYRIGHT")))
        if not files:
            raise RuntimeError(f"Missing dependency license text for {package['name']}")
        for path in files:
            output.append(f"\n--- {path.name} ---\n{path.read_text()}\n")
    return "".join(output).encode()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--version", required=True)
    parser.add_argument("--arch", required=True, choices=["amd64", "arm64"])
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    crate = Path(__file__).resolve().parent.parent
    payloads = {"ciaicd-credential-provider": args.binary.read_bytes(),
                "credential-provider.example.yaml": (crate / "config.service-account.example.yaml").read_bytes(),
                "LICENSE": (crate / "LICENSE").read_bytes(),
                "README.md": (crate / "README.md").read_bytes(),
                "THIRD_PARTY_NOTICES.txt": dependency_notices(crate)}
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / f"ciaicd-credential-provider-v{args.version}-linux-{args.arch}.tar.gz"
    with archive.open("wb") as raw:
        with gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w") as tar:
                for name, content in sorted(payloads.items()):
                    info = tarfile.TarInfo(name)
                    info.size = len(content)
                    info.mode = 0o755 if name == "ciaicd-credential-provider" else 0o644
                    info.uid = info.gid = info.mtime = 0
                    tar.addfile(info, io.BytesIO(content))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
    print(f"Packaged {archive}")


if __name__ == "__main__":
    main()
