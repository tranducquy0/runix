#!/usr/bin/env bash
# Vendor GRUB's ISO tooling for Termux. There is no grub package in Termux,
# but the pieces grub-mkrescue actually needs are portable:
#
#   - grub-common's grub-mkrescue / grub-file / grub-mkimage (Debian arm64, so
#     they run natively on aarch64 Termux through its glibc loader)
#   - grub-pc-bin's i386-pc images and 303 modules (architecture-independent
#     files; grub-pc-bin has no arm64 build, so the amd64 package is used for
#     its payload alone)
#   - the few shared libs grub-mkrescue links against, all plain arm64
#
# Everything lands under .tools/grub (gitignored). scripts/iso.sh runs the
# binaries through .tools/grub/ld-linux-aarch64.so.1 from there. Re-run this
# with --force after bumping the versions below.
set -euo pipefail
cd "$(dirname "$0")/.."

DEST=".tools/grub"
DEB_BASE="https://deb.debian.org/debian"
DEB_ARCH="arm64" # grub-common/grub-file/grub-mkrescue are native arm64
FORCE=false
[[ "${1:-}" == "--force" ]] && FORCE=true

if [[ -d "$DEST/bin" ]] && [[ "$FORCE" != true ]]; then
  echo "[*] $DEST already populated (use --force to refresh)"
  exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# package -> pool path. grub-pc-bin is amd64 (payload is arch-independent).
packages=(
  "grub-common:pool/main/g/grub2/grub-common_2.06-13+deb12u2_arm64.deb"
  "grub-pc-bin:pool/main/g/grub2/grub-pc-bin_2.06-13+deb12u2_amd64.deb"
  "libdevmapper1.02.1:pool/main/l/lvm2/libdevmapper1.02.1_1.02.185-2_arm64.deb"
  "liblzma5:pool/main/x/xz-utils/liblzma5_5.4.1-1+deb12u1_arm64.deb"
  "libefivar1:pool/main/e/efivar/libefivar1_37-6_arm64.deb"
  "libefiboot1:pool/main/e/efivar/libefiboot1_37-6_arm64.deb"
  "libselinux1:pool/main/libs/libselinux/libselinux1_3.4-1+b6_arm64.deb"
  "libpcre2-8-0:pool/main/p/pcre2/libpcre2-8-0_10.42-1_arm64.deb"
  "libfuse3-3:pool/main/f/fuse3/libfuse3-3_3.14.0-4_arm64.deb"
  "libudev1:pool/main/s/systemd/libudev1_252.39-1~deb12u2_arm64.deb"
  "libcap2:pool/main/libc/libcap2/libcap2_2.66-4+deb12u3+b1_arm64.deb"
)

echo "[*] downloading ${#packages[@]} Debian packages..."
for kv in "${packages[@]}"; do
  name="${kv%%:*}"
  path="${kv#*:}"
  echo "    $name"
  python3 -c "import urllib.request,sys; open(sys.argv[1],'wb').write(urllib.request.urlopen(sys.argv[2],timeout=120).read())" \
    "$tmp/$name.deb" "$DEB_BASE/$path"
done

echo "[*] extracting into $DEST..."
rm -rf "$DEST"
mkdir -p "$DEST/bin" "$DEST/lib" "$DEST/boot/grub" "$tmp/x"
for kv in "${packages[@]}"; do
  name="${kv%%:*}"
  dpkg-deb -x "$tmp/$name.deb" "$tmp/x/$name"
done

# Binaries we invoke directly.
for b in grub-mkrescue grub-file grub-mkimage; do
  cp "$tmp/x/grub-common/usr/bin/$b" "$DEST/bin/$b"
done

# i386-pc boot images + modules (from grub-pc-bin, arch-independent payload).
cp -r "$tmp/x/grub-pc-bin/usr/lib/grub/i386-pc" "$DEST/boot/grub/i386-pc"

# Shared libs grub-mkrescue/grub-file link against. Collected flat so one
# --library-path covers them.
find "$tmp/x" -name '*.so*' -exec cp -P {} "$DEST/lib/" \;

# The Termux glibc loader, used to run the Debian binaries in-place. The libs
# above are resolved first via --library-path; glibc itself comes from the
# Termux glibc package at run time.
prefix="${PREFIX:-/data/data/com.termux/files/usr}"
if [[ -e "$prefix/glibc/lib/ld-linux-aarch64.so.1" ]]; then
  cp "$prefix/glibc/lib/ld-linux-aarch64.so.1" "$DEST/ld-linux-aarch64.so.1"
else
  echo "[!] no glibc loader at $prefix/glibc/lib — install the glibc repoverlay" >&2
  exit 1
fi

echo "[+] vendored GRUB into $DEST"
echo "    binaries: $(ls "$DEST/bin" | tr '\n' ' ')"
echo "    modules:  $(ls "$DEST/boot/grub/i386-pc" | wc -l) i386-pc files"
