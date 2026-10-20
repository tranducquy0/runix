#!/usr/bin/env bash
# ISO helpers for runix: validate the kernel image (`test`) or pack the
# `iso/` tree into `runix.iso` (`build`). Kept separate from the Makefile
# so both `make iso <sub>` and direct invocation share one implementation.
#
# GRUB tooling normally comes from the distro (grub-common + grub-pc-bin).
# Termux has no grub package but does have glibc, so `make setup android`
# vendors Debian's arm64 grub-mkrescue/grub-file plus the arch-independent
# i386-pc images under .tools/grub and runs them through the Termux glibc
# loader. When grub-mkrescue/grub-file are on PATH (Arch), that copy is used.
set -euo pipefail
cd "$(dirname "$0")/.."

kernel_binary="target/i686-runix-target/debug/runix"
grub_tools="$PWD/.tools/grub"

usage() {
  echo "usage: iso.sh test|build" >&2
  exit 1
}

have_system_grub() {
  command -v grub-mkrescue >/dev/null 2>&1 && command -v grub-file >/dev/null 2>&1
}

have_vendored_grub() {
  [[ -x "$grub_tools/bin/grub-mkrescue" && -x "$grub_tools/bin/grub-file" &&
     -x "$grub_tools/ld-linux-aarch64.so.1" ]]
}

# Run a vendored grub binary through the glibc loader with the vendored libs.
# Termux's glibc loader and its libc are also required; include them in the path.
grub_run() {
  local glibc_lib="${PREFIX:-/data/data/com.termux/files/usr}/glibc/lib"
  local lp="$grub_tools/lib"
  [[ -d "$glibc_lib" ]] && lp="$lp:$glibc_lib"
  "$grub_tools/ld-linux-aarch64.so.1" --library-path "$lp" "$grub_tools/bin/$1" "${@:2}"
}

# Validate the built kernel: real ELF + GRUB-recognized Multiboot2 header.
iso_test() {
  if have_system_grub; then
    bash scripts/kernel_image.sh
  elif have_vendored_grub; then
    # grub-file needs the loader too; wrap it for kernel_image.sh.
    local glibc_lib="${PREFIX:-/data/data/com.termux/files/usr}/glibc/lib"
    local lp="$grub_tools/lib"
    [[ -d "$glibc_lib" ]] && lp="$lp:$glibc_lib"
    GRUB_FILE="$grub_tools/ld-linux-aarch64.so.1 --library-path $lp $grub_tools/bin/grub-file" \
      bash scripts/kernel_image.sh
  else
    echo "iso.sh: grub tooling missing, run 'make setup android'" >&2
    exit 1
  fi
}

# Copy the kernel to iso/boot/rkrnl and build the bootable ISO with GRUB.
iso_build() {
  if [[ ! -f "$kernel_binary" ]]; then
    echo "iso.sh: no kernel binary at $kernel_binary, run 'make build' first" >&2
    exit 1
  fi
  cp -f "$kernel_binary" iso/boot/rkrnl
  if have_system_grub; then
    grub-mkrescue -o runix.iso iso
  elif have_vendored_grub; then
    grub_run grub-mkrescue --directory "$grub_tools/boot/grub/i386-pc" -o runix.iso iso
  else
    echo "iso.sh: grub tooling missing, run 'make setup android'" >&2
    exit 1
  fi
}

case "${1:-}" in
  test) iso_test ;;
  build) iso_build ;;
  *) usage ;;
esac
