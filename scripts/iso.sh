#!/usr/bin/env bash
# ISO helpers and QEMU control for runix.
set -euo pipefail
cd "$(dirname "$0")/.."

kernel_binary="target/i686-runix-target/debug/runix"

usage() {
  echo "usage: iso.sh test|build" >&2
  exit 1
}

iso_test() {
  bash scripts/kernel_image.sh
}

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
