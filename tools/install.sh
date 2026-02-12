#!/bin/sh
# CAVE CLI installer
# Usage:  sh -c "$(curl -fsSL https://raw.githubusercontent.com/simvia-tech/cave/main/tools/install.sh)"
#
# This script auto-detects your Linux distribution and installs the
# appropriate package (.deb / .rpm) or falls back to a binary tarball.

set -e

REPO="simvia-tech/cave"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"
GITHUB_DL="https://github.com/${REPO}/releases/download"
INSTALL_DIR="/usr/local/bin"
MAN_DIR="/usr/share/man/man1"
TMP_DIR=""

# ── Output functions ─────────────────────────────────────────────────

info()    { printf "[info]  %s\n" "$*"; }
success() { printf "[ok]    %s\n" "$*"; }
warn()    { printf "[warn]  %s\n" "$*"; }
error()   { printf "[error] %s\n" "$*" >&2; }

cleanup() {
    if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}
trap cleanup EXIT

# ── Pre-flight checks ───────────────────────────────────────────────

check_os() {
    OS="$(uname -s)"
    case "$OS" in
        Linux) ;;
        *)
            error "Unsupported operating system: $OS"
            error "CAVE currently only supports Linux."
            exit 1
            ;;
    esac
}

check_arch() {
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64) ;;
        *)
            error "Unsupported architecture: $ARCH"
            error "CAVE currently only provides x86_64 binaries."
            exit 1
            ;;
    esac
}

check_command() {
    command -v "$1" >/dev/null 2>&1
}

check_dependencies() {
    if ! check_command curl && ! check_command wget; then
        error "Either 'curl' or 'wget' is required but neither was found."
        exit 1
    fi

    if ! check_command sudo; then
        error "'sudo' is required but was not found."
        exit 1
    fi
}

check_docker() {
    if ! check_command docker; then
        warn "Docker is not installed or not in PATH."
        warn "CAVE requires Docker to run code_aster containers."
        warn "Install Docker first: https://docs.docker.com/engine/install/"
        printf "\n"
    fi
}

# ── HTTP helper (curl preferred, wget fallback) ─────────────────────

fetch() {
    url="$1"
    output="$2"
    max_retries=3
    retry_delay=2
    
    for attempt in $(seq 1 $max_retries); do
        if check_command curl; then
            if [ -n "$output" ]; then
                if curl -fsSL -o "$output" "$url"; then
                    return 0
                fi
            else
                if curl -fsSL "$url"; then
                    return 0
                fi
            fi
        elif check_command wget; then
            if [ -n "$output" ]; then
                if wget -qO "$output" "$url"; then
                    return 0
                fi
            else
                if wget -qO- "$url"; then
                    return 0
                fi
            fi
        fi
        
        # If we get here, the download failed
        if [ $attempt -lt $max_retries ]; then
            warn "Download failed (attempt $attempt/$max_retries). Retrying in ${retry_delay}s..."
            sleep $retry_delay
        else
            error "Download failed after $max_retries attempts."
            return 1
        fi
    done
}

# ── Detect latest version ───────────────────────────────────────────

get_latest_version() {
    info "Fetching latest release version..."
    RELEASE_JSON="$(fetch "$GITHUB_API" "")"
    # Extract tag_name value without jq
    VERSION="$(printf '%s' "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')"

    if [ -z "$VERSION" ]; then
        error "Could not determine the latest version."
        error "Check your internet connection or try again later."
        exit 1
    fi

    # Strip leading 'v' if present (e.g. v0.1.5 -> 0.1.5)
    VERSION_NUM="$(printf '%s' "$VERSION" | sed 's/^v//')"

    info "Latest version: ${VERSION_NUM}"
}

# ── Detect distro / package format ──────────────────────────────────

detect_package_format() {
    if check_command dpkg && check_command apt-get; then
        PKG_FORMAT="deb"
    elif check_command rpm; then
        PKG_FORMAT="rpm"
    else
        PKG_FORMAT="tar"
    fi
    info "Detected package format: ${PKG_FORMAT}"
}

# ── Download & install ──────────────────────────────────────────────

install_deb() {
    DEB_FILE="${TMP_DIR}/cave_${VERSION_NUM}.deb"
    DEB_URL="${GITHUB_DL}/${VERSION}/cave_${VERSION_NUM}.deb"

    info "Downloading .deb package..."
    fetch "$DEB_URL" "$DEB_FILE"

    info "Installing .deb package..."
    sudo dpkg -i "$DEB_FILE"
}

install_rpm() {
    RPM_FILE="${TMP_DIR}/cave_${VERSION_NUM}.rpm"
    RPM_URL="${GITHUB_DL}/${VERSION}/cave_${VERSION_NUM}.rpm"

    info "Downloading .rpm package..."
    fetch "$RPM_URL" "$RPM_FILE"

    info "Installing .rpm package..."
    if check_command dnf; then
        sudo dnf install -y "$RPM_FILE"
    elif check_command yum; then
        sudo yum install -y "$RPM_FILE"
    else
        sudo rpm -i "$RPM_FILE"
    fi
}

install_tarball() {
    TAR_FILE="${TMP_DIR}/cave_${VERSION_NUM}.tar.gz"
    TAR_URL="${GITHUB_DL}/${VERSION}/cave_${VERSION_NUM}.tar.gz"

    info "Downloading binary tarball..."
    fetch "$TAR_URL" "$TAR_FILE"

    info "Extracting binary..."
    tar -xzf "$TAR_FILE" -C "$TMP_DIR"

    info "Installing binary to ${INSTALL_DIR}..."
    sudo install -Dm755 "${TMP_DIR}/cave" "${INSTALL_DIR}/cave"

    # Install man page
    MAN_URL="${GITHUB_DL}/${VERSION}/cave.1"
    MAN_FILE="${TMP_DIR}/cave.1"

    info "Downloading man page..."
    if fetch "$MAN_URL" "$MAN_FILE" 2>/dev/null; then
        sudo install -Dm644 "$MAN_FILE" "${MAN_DIR}/cave.1"
        success "Man page installed to ${MAN_DIR}/cave.1"
    else
        warn "Could not download man page (non-critical)."
        warn "Man page is included in .deb and .rpm packages."
    fi
}

# ── Post-install verification ───────────────────────────────────────

verify_install() {
    printf "\n"
    if check_command cave; then
        INSTALLED_VERSION="$(cave --version 2>/dev/null || true)"
        success "CAVE installed successfully! (${INSTALLED_VERSION})"
    else
        warn "Installation completed but 'cave' was not found in PATH."
        warn "You may need to add ${INSTALL_DIR} to your PATH."
    fi
}

print_next_steps() {
    printf "\n"
    printf "Next steps:\n"
    printf "  cave --help          Show available commands\n"
    printf "  cave list            List installed code_aster images\n"
    printf "  cave available       List images available on DockerHub\n"
    printf "  man cave             Read the manual\n"
    printf "\n"
    printf "Documentation: https://github.com/%s\n" "$REPO"
}

# ── Main ─────────────────────────────────────────────────────────────

main() {
    printf "\n"
    printf "  CAVE CLI Installer\n"
    printf "  ─────────────────────\n"
    printf "\n"

    check_os
    check_arch
    check_dependencies
    check_docker

    get_latest_version
    detect_package_format

    TMP_DIR="$(mktemp -d)"

    case "$PKG_FORMAT" in
        deb)     install_deb     ;;
        rpm)     install_rpm     ;;
        tar)     install_tarball ;;
    esac

    verify_install
    print_next_steps
}

main "$@"
