#!/usr/bin/env sh
set -eu

repository="${TEXSMITH_REPOSITORY:-Bernard258/Texsmith}"
install_dir="${TEXSMITH_INSTALL_DIR:-$HOME/.local/bin}"
release="${1:-latest}"

if ! command -v wget >/dev/null 2>&1; then
    printf '%s\n' 'error: wget is required to install Texsmith.' >&2
    exit 1
fi

os=$(uname -s)
arch=$(uname -m)
case "$os/$arch" in
    Linux/x86_64) asset='texsmith-linux-x86_64' ;;
    Darwin/x86_64) asset='texsmith-macos-x86_64' ;;
    Darwin/arm64) asset='texsmith-macos-aarch64' ;;
    *)
        printf 'error: unsupported platform: %s/%s\n' "$os" "$arch" >&2
        exit 1
        ;;
esac

if [ "$release" = latest ]; then
    url="https://github.com/$repository/releases/latest/download/$asset"
else
    url="https://github.com/$repository/releases/download/$release/$asset"
fi

mkdir -p "$install_dir"
temporary_file=$(mktemp "${TMPDIR:-/tmp}/texsmith.XXXXXX")
trap 'rm -f "$temporary_file"' EXIT HUP INT TERM

printf 'Downloading Texsmith (%s)...\n' "$asset"
wget --quiet --show-progress -O "$temporary_file" "$url"
chmod 755 "$temporary_file"
mv "$temporary_file" "$install_dir/texsmith"
trap - EXIT HUP INT TERM

printf 'Installed Texsmith at %s\n' "$install_dir/texsmith"
case ":${PATH}:" in
    *:"$install_dir":*) ;;
    *) printf 'Add %s to PATH to run texsmith from any directory.\n' "$install_dir" ;;
esac