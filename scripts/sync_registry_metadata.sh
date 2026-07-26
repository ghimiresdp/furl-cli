#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(sed -nE 's/^version = "([^"]+)"/\1/p' "$repo_root/Cargo.toml" | head -n 1)"
fi

if [[ -z "$version" ]]; then
  echo "Could not determine version from Cargo.toml" >&2
  exit 1
fi

tag="v${version}"

winget_file="$repo_root/registries/winget/ghimiresdp.furl.yml"
flatpak_file="$repo_root/registries/flatpak/io.github.ghimiresdp.furl.yml"
apt_changelog="$repo_root/registries/apt/debian/changelog"

if [[ -f "$winget_file" ]]; then
  sed -Ei "s|^(PackageVersion: ).*$|\\1${version}|" "$winget_file"
  sed -Ei "s|(InstallerUrl: .*?/download/)v[^/]+(/furl-)v[^-]+(-windows-x86_64\\.zip)|\\1${tag}\\2${tag}\\3|" "$winget_file"
fi

if [[ -f "$flatpak_file" ]]; then
  sed -Ei "s|(url: .*?/download/)v[^/]+(/furl-)v[^-]+(-linux-x86_64\\.tar\\.gz)|\\1${tag}\\2${tag}\\3|" "$flatpak_file"
fi

if [[ -f "$apt_changelog" ]]; then
  sed -Ei "1 s/^furl-cli \([^)-]+-([0-9]+)\) /furl-cli (${version}-\\1) /" "$apt_changelog"
fi

echo "Synchronized registry metadata to version ${version}."
