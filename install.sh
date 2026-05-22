#!/usr/bin/env bash
#
# cuaca installer
# Supports Linux and macOS. Downloads and verifies release artifacts.
#
# Usage: ./install.sh [OPTIONS]
#   --bin-dir DIR       Install binary to DIR (default: ~/.local/bin)
#   --dry-run           Show what would be done without making changes
#   --force             Overwrite existing installation even if up-to-date
#   --no-verify         Skip checksum verification
#   --version TAG       Install specific version (default: latest)
#   --yes               Assume yes to prompts
#   --help              Show this help
#
# Example:
#   ./install.sh --bin-dir /usr/local/bin --yes

set -euo pipefail

# Defaults
BIN_DIR="${HOME}/.local/bin"
DRY_RUN=0
FORCE=0
VERIFY=1
ASSUME_YES=0
VERSION=""
GITHUB_REPO="rafiyq/cuaca"
TEMP_DIR=""
USER_AGENT="cuaca-installer/1.0"

# Help
show_help() {
    grep '^#' "$0" | cut -c4-
    exit 0
}

# Parse args
while (( $# )); do
    case "$1" in
        --bin-dir)
            BIN_DIR="${2:?missing dir}"; shift 2 ;;
        --dry-run)
            DRY_RUN=1; shift ;;
        --force)
            FORCE=1; shift ;;
        --no-verify)
            VERIFY=0; shift ;;
        --version)
            VERSION="${2:?missing version}"; shift 2 ;;
        --yes)
            ASSUME_YES=1; shift ;;
        --help|-\?)
            show_help ;;
        *)
            echo "Unknown option: $1" >&2; show_help ;;
    esac
done

# OS/Arch detection
OS_TYPE="$(uname -s)"
ARCH="$(uname -m)"

case "$OS_TYPE" in
    Linux)     OS="linux" ;;
    Darwin)    OS="macos" ;;
    *)         echo "Unsupported OS: $OS_TYPE" >&2; exit 1 ;;
esac

case "$ARCH" in
    x86_64)    ARCH_FULL="x86_64" ;;
    aarch64)   ARCH_FULL="aarch64" ;;
    *)         echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# Determine target name as used in CI artifacts
TARGET="${ARCH_FULL}-${OS}"
# Archive extension
case "$OS" in
    linux|macos) EXT="tar.gz" ;;
esac

# Determine version to install
if [ -z "$VERSION" ]; then
    # Fetch latest release info from GitHub API
    echo "Fetching latest release information..."
    API_URL="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    if ! RELEASE_JSON=$(curl -s -f -A "$USER_AGENT" "$API_URL"); then
        echo "Failed to query GitHub API." >&2
        exit 1
    fi
    TAG="$(printf "%s" "$RELEASE_JSON" | grep -oP '"tag_name"\s*:\s*"\K[^"]+')"
    if [ -z "$TAG" ]; then
        echo "Could not determine latest tag." >&2
        exit 1
    fi
    VERSION="${TAG#v}"  # strip leading 'v'
else
    # Ensure version does not have leading 'v'
    VERSION="${VERSION#v}"
fi

echo "Installing cuaca version v$VERSION for $TARGET"

# Prepare paths
ASSET_NAME="cuaca-v${VERSION}-${TARGET}.tar.gz"
ASSET_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${ASSET_NAME}"
CHECKSUM_NAME="${ASSET_NAME}.sha256"
CHECKSUM_URL="https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${CHECKSUM_NAME}"

# Create temp dir
TEMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Download
download() {
    local url="$1" out="$2"
    echo "Downloading: $url"
    if ! curl -s -f -L -A "$USER_AGENT" -o "$out" "$url"; then
        echo "Download failed: $url" >&2
        return 1
    fi
}

# Main
ARCHIVE_PATH="$TEMP_DIR/$ASSET_NAME"
CHECKSUM_PATH="$TEMP_DIR/$CHECKSUM_NAME"

if [ $DRY_RUN -eq 1 ]; then
    echo "[DRY RUN] Would download: $ASSET_URL"
    echo "[DRY RUN] Would verify checksum: $CHECKSUM_URL"
    echo "[DRY RUN] Would extract and install to: $BIN_DIR"
    exit 0
fi

# Ensure bin directory exists
mkdir -p "$BIN_DIR"

# Check existing version
if command -v cuaca >/dev/null 2>&1; then
    EXISTING_VERSION="$(cuaca --version 2>&1 || true)"
    if [ -n "$EXISTING_VERSION" ]; then
        echo "Existing installation: $EXISTING_VERSION"
        if [ "$FORCE" -eq 0 ] && [ "$EXISTING_VERSION" = "v$VERSION" ]; then
            echo "Same version already installed. Use --force to reinstall."
            exit 0
        fi
    fi
fi

# Prompt for installation if not --yes
if [ $ASSUME_YES -eq 0 ] && [ -t 0 ]; then
    read -r -p "Install cuaca v$VERSION to $BIN_DIR? [y/N] " answer
    case "$answer" in
        [Yy]* ) ;;
        * ) echo "Aborted."; exit 1 ;;
    esac
fi

# Download archive and checksum
download "$ASSET_URL" "$ARCHIVE_PATH"
if [ $VERIFY -eq 1 ]; then
    if ! download "$CHECKSUM_URL" "$CHECKSUM_PATH"; then
        echo "Warning: checksum file not found, skipping verification."
        VERIFY=0
    fi
fi

# Verify checksum if present
if [ $VERIFY -eq 1 ] && [ -f "$CHECKSUM_PATH" ]; then
    echo "Verifying checksum..."
    # Expected format: <hash>  <filename>
    EXPECTED="$(awk '{print $1}' "$CHECKSUM_PATH")"
    # Compute actual
    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL="$(sha256sum "$ARCHIVE_PATH" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL="$(shasum -a 256 "$ARCHIVE_PATH" | awk '{print $1}')"
    else
        echo "Error: no tool to compute SHA256 (need sha256sum or shasum)" >&2
        exit 1
    fi
    if [ "$EXPECTED" != "$ACTUAL" ]; then
        echo "Checksum mismatch!" >&2
        echo "Expected: $EXPECTED" >&2
        echo "Actual:   $ACTUAL" >&2
        exit 1
    fi
    echo "Checksum verified."
fi

# Extract
EXTRACT_DIR="$TEMP_DIR/extract"
mkdir -p "$EXTRACT_DIR"
case "$EXT" in
    tar.gz)
        tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"
        ;;
esac

# The binary is at the root of the archive
BIN_SRC="$EXTRACT_DIR/cuaca"
if [ ! -f "$BIN_SRC" ]; then
    echo "Binary not found in archive at $BIN_SRC" >&2
    ls -R "$EXTRACT_DIR"
    exit 1
fi

# Determine destination path
DEST_PATH="$BIN_DIR/cuaca"
echo "Installing to: $DEST_PATH"

# If we need sudo? Check if BIN_DIR is writable.
if [ -w "$BIN_DIR" ]; then
    cp "$BIN_SRC" "$DEST_PATH"
else
    echo "Requires root privileges to write to $BIN_DIR. Trying sudo..."
    if ! command -v sudo >/dev/null 2>&1; then
        echo "sudo not available. Please run installer as root or choose a different --bin-dir." >&2
        exit 1
    fi
    sudo cp "$BIN_SRC" "$DEST_PATH"
fi

chmod +x "$DEST_PATH"

echo "cuaca v$VERSION installed successfully."
echo "Run 'cuaca --help' for usage."
