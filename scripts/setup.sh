#!/usr/bin/env bash
# Dev environment setup for runix: `setup.sh android` (Termux) or
# `setup.sh arch` (Arch Linux). Installs the toolchain, then verifies each
# required command and reports what still cannot work on that platform.
set -euo pipefail
cd "$(dirname "$0")/.."

usage() {
  echo "usage: setup.sh android|arch" >&2
  exit 1
}

# One line per required tool: [+] present / [!] missing.
verify() {
  local cmd
  for cmd in "$@"; do
    if command -v "$cmd" >/dev/null 2>&1; then
      echo "  [+] $cmd"
    else
      echo "  [!] $cmd missing"
    fi
  done
}

setup_android() {
  if ! command -v pkg >/dev/null 2>&1; then
    echo "setup.sh: pkg not found — run this inside Termux" >&2
    exit 1
  fi

  # Install only what's missing: `pkg install` on a long-lived install would
  # also pull in every pending upgrade.
  local pkgs=()
  command -v make >/dev/null 2>&1 || pkgs+=(make)
  command -v git >/dev/null 2>&1 || pkgs+=(git)
  command -v cargo >/dev/null 2>&1 || pkgs+=(rust)
  command -v python3 >/dev/null 2>&1 || pkgs+=(python)
  command -v readelf >/dev/null 2>&1 || pkgs+=(binutils)
  command -v qemu-system-i386 >/dev/null 2>&1 || pkgs+=(qemu-system-i386-headless)
  # -Z build-std compiles `core` from source; on Termux those sources come
  # from the rust-src package (no rustup component here).
  [[ -d "${PREFIX:-}/lib/rustlib/src/rust/library" ]] || pkgs+=(rust-src)

  if [[ ${#pkgs[@]} -gt 0 ]]; then
    echo "[*] installing: ${pkgs[*]}"
    pkg install -y "${pkgs[@]}"
  else
    echo "[*] all packages already installed"
  fi

  echo "[*] verifying tools..."
  verify make git cargo rustc python3 readelf qemu-system-i386

  cat <<'EOF'

[*] Termux notes:
  - no rustup package here: `make build` falls back to the distro stable
    rustc with RUSTC_BOOTSTRAP=1 (rust-src compiles `core`) — same -Z flags
    as everywhere else
  - no grub package either: `make iso test|build` and anything that boots
    runix.iso (`make run`, `make vm ...`) need grub-file/grub-mkrescue from
    another machine; on-device you get `make build` and `make tests`
EOF
}

setup_arch() {
  if ! command -v pacman >/dev/null 2>&1; then
    echo "setup.sh: pacman not found — run this on Arch Linux" >&2
    exit 1
  fi

  local sudo=()
  if [[ $EUID -ne 0 ]]; then
    if ! command -v sudo >/dev/null 2>&1; then
      echo "setup.sh: not root and no sudo — cannot run pacman" >&2
      exit 1
    fi
    sudo=(sudo)
  fi

  echo "[*] installing packages (pacman)..."
  # grub brings grub-mkrescue, grub-file, and the i386-pc modules; qemu-system-x86
  # brings qemu-system-i386 (headless and gtk displays).
  "${sudo[@]}" pacman -S --needed --noconfirm \
    base-devel git rustup grub qemu-system-x86 xorriso mtools python

  echo "[*] installing rust nightly (the kernel build is cargo +nightly)..."
  rustup toolchain install nightly
  rustup component add rust-src --toolchain nightly

  echo "[*] verifying tools..."
  verify make git cargo rustc python3 readelf grub-mkrescue grub-file qemu-system-i386
}

case "${1:-}" in
  android) setup_android ;;
  arch) setup_arch ;;
  *) usage ;;
esac
