#!/usr/bin/env bash
# ISO helpers for runix: validate the kernel image (`test`) or pack the
# `iso/` tree into `runix.iso` (`build`). Kept separate from the Makefile
# so both `make iso <sub>` and direct invocation share one implementation.
set -euo pipefail
cd "$(dirname "$0")/.."

kernel_binary="target/i686-runix-target/debug/runix"

usage() {
  echo "usage: iso.sh test|build" >&2
  exit 1
}

# Validate the built kernel: real ELF + GRUB-recognized Multiboot2 header.
iso_test() {
  bash scripts/kernel_image.sh
}

# Copy the kernel to iso/boot/rkrnl and build the bootable ISO with GRUB.
iso_build() {
  if [[ ! -f "$kernel_binary" ]]; then
    echo "iso.sh: no kernel binary at $kernel_binary, run 'make build' first" >&2
    exit 1
  fi
  cp -f "$kernel_binary" iso/boot/rkrnl
  grub-mkrescue -o runix.iso iso
}

case "${1:-}" in
  test) iso_test ;;
  build) iso_build ;;
  *) usage ;;
esac
