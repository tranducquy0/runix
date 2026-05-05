#!/usr/bin/env bash
set -euo pipefail

kernel_path="${1:-target/i686-runix-target/debug/runix}"

if [[ ! -f "${kernel_path}" ]]; then
    echo "[!] kernel image not found: ${kernel_path}"
    echo "[!] build it first with: make build"
    exit 1
fi

if ! command -v readelf >/dev/null 2>&1; then
    echo "[!] readelf is required but not installed"
    exit 1
fi

if ! command -v grub-file >/dev/null 2>&1; then
    echo "[!] grub-file is required but not installed"
    exit 1
fi

echo "[*] checking ELF header: ${kernel_path}"

# This only succeeds for real ELF binaries, so it is a cheap sanity check
# before asking GRUB-specific tooling to inspect the image.
if ! readelf -h "${kernel_path}" >/dev/null 2>&1; then
    echo "[!] kernel image is not a valid ELF binary"
    exit 1
fi

echo "[*] checking Multiboot2 header with grub-file"

# grub-file returns success when it can find and validate the Multiboot2
# header in the image the same way GRUB would during boot discovery.
if ! grub-file --is-x86-multiboot2 "${kernel_path}"; then
    echo "[!] kernel image is not recognized as Multiboot2"
    exit 1
fi

echo "[+] kernel image passed ELF and Multiboot2 checks"
