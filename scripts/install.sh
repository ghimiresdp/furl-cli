#!/bin/sh
# Installer for furl (https://github.com/ghimiresdp/furl-cli).
#
#   curl -LsSf https://raw.githubusercontent.com/ghimiresdp/furl-cli/main/scripts/install.sh | sh
#
# Env vars:
#   FURL_VERSION      Install a specific tag (e.g. "v0.9.1") instead of latest.
#   FURL_INSTALL_DIR  Where to put the binary (default: "$HOME/.local/bin").
set -eu

REPO="ghimiresdp/furl-cli"
INSTALL_DIR="${FURL_INSTALL_DIR:-$HOME/.local/bin}"

info() { printf '\033[1;34minfo\033[0m: %s\n' "$1"; }
warn() { printf '\033[1;33mwarn\033[0m: %s\n' "$1" >&2; }
die() { printf '\033[1;31merror\033[0m: %s\n' "$1" >&2; exit 1; }

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "'$1' is required but not installed"
}

detect_platform() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux) os="linux" ;;
        Darwin) os="macos" ;;
        *) die "unsupported OS: $os (see https://github.com/${REPO}#installation to build from source)" ;;
    esac

    case "$arch" in
        x86_64 | amd64) arch="x86_64" ;;
        arm64 | aarch64) arch="aarch64" ;;
        *) die "unsupported architecture: $arch" ;;
    esac

    if [ "$os" = "linux" ] && [ "$arch" = "aarch64" ]; then
        die "no prebuilt binary for linux-aarch64 yet; install with 'cargo install furl-cli' instead"
    fi

    platform="${os}-${arch}"
}

# Finds the newest release whose tag looks like a furl-cli binary release
# (a bare "vX.Y.Z" tag, as opposed to this repo's separate, binary-less
# "furl-cli@X.Y.Z" / "furl-core@X.Y.Z" version tags).
latest_tag() {
    # Not `grep -m1`: that closes the pipe as soon as it matches, which
    # makes curl print a spurious "Failure writing output" warning when it
    # still has bytes left to send.
    curl -fsSL "https://api.github.com/repos/${REPO}/releases" \
        | grep '"tag_name": *"v[0-9][^"]*"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/' \
        | head -n1
}

main() {
    need_cmd curl
    need_cmd tar

    detect_platform

    tag="${FURL_VERSION:-}"
    if [ -z "$tag" ]; then
        info "looking up the latest release..."
        tag="$(latest_tag)"
        [ -n "$tag" ] || die "couldn't find a release to install; see https://github.com/${REPO}/releases"
    fi

    asset="furl-${tag}-${platform}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

    tmp_dir="$(mktemp -d)"
    trap 'rm -rf "$tmp_dir"' EXIT

    info "downloading furl ${tag} for ${platform}..."
    curl -fsSL -o "${tmp_dir}/${asset}" "$url" \
        || die "download failed: $url (does a release for ${platform} exist for ${tag}?)"

    tar -xzf "${tmp_dir}/${asset}" -C "$tmp_dir"

    mkdir -p "$INSTALL_DIR"
    install -m 755 "${tmp_dir}/furl" "${INSTALL_DIR}/furl"

    info "installed furl to ${INSTALL_DIR}/furl"

    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *)
            warn "${INSTALL_DIR} is not on your PATH"
            shell_rc=""
            case "${SHELL:-}" in
                */zsh) shell_rc="$HOME/.zshrc" ;;
                */bash) shell_rc="$HOME/.bashrc" ;;
                *) shell_rc="$HOME/.profile" ;;
            esac
            line="export PATH=\"${INSTALL_DIR}:\$PATH\""
            if [ -f "$shell_rc" ] && grep -qF "$line" "$shell_rc" 2>/dev/null; then
                : # already there
            else
                printf '\n# added by furl-cli installer\n%s\n' "$line" >> "$shell_rc"
                info "added ${INSTALL_DIR} to PATH in ${shell_rc} (restart your shell, or run: ${line})"
            fi
            ;;
    esac

    info "run 'furl --help' to get started"
}

main "$@"
