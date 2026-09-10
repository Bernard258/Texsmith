#!/usr/bin/env sh
set -eu

install_dir="${TEXSMITH_INSTALL_DIR:-$HOME/.local/bin}"
binary="$install_dir/texsmith"

if [ -e "$binary" ]; then
    rm "$binary"
    printf 'Removed %s\n' "$binary"
else
    printf 'Texsmith is not installed at %s\n' "$binary"
fi