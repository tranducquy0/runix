#!/usr/bin/env bash
# QEMU helpers for runix. `launch` opens a normal GTK window;
# `agent` boots headless and exposes a QEMU monitor on a unix socket so
# agents (or scripts) can inspect the kernel while it runs.
#
# The kernel prints to VGA text memory (0xb8000) only, so `screen`
# decodes that buffer through the monitor: it is the way to "read" the
# kernel's output without a display.
set -euo pipefail
cd "$(dirname "$0")/.."

monitor_sock="$PWD/.run/vm-monitor.sock"
run_dir=".run"
image="runix.iso"

cleanup_existing() {
  if [[ -f "$run_dir/vm.pid" ]]; then
    local pid
    pid="$(cat "$run_dir/vm.pid" 2>/dev/null || true)"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      echo "[*] Stopping existing VM (pid $pid)..."
      if [[ -S "$monitor_sock" ]]; then
        monitor_talk quit >/dev/null 2>&1 || true
      fi
      if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        local count=0
        while kill -0 "$pid" 2>/dev/null && (( count < 20 )); do
          sleep 0.1
          (( count++ )) || true
        done
      fi
    fi
  elif [[ -S "$monitor_sock" ]]; then
    monitor_talk quit >/dev/null 2>&1 || true
  fi
  rm -rf "$run_dir"
}

# Boot runix.iso headless in the background with a monitor on a unix socket.
start() {
  if [[ ! -f "$image" ]]; then
    echo "vm.sh: missing $image, run 'make build && make iso build' first" >&2
    exit 1
  fi
  cleanup_existing
  mkdir -p "$run_dir"
  qemu-system-i386 \
    -cdrom "$image" \
    -m 128M \
    -display none \
    -monitor "unix:$monitor_sock,server,nowait" \
    -daemonize \
    -pidfile "$run_dir/vm.pid"
  echo "[*] VM booting headless (pid $(cat "$run_dir/vm.pid"))"
  echo "[*] monitor: unix:$monitor_sock"
  echo "[*] read the kernel's screen with: make vm screen"
}

serve() {
  # Same window-less launch as start, but in the foreground so a caller
  # can attach the monitor itself (e.g. an agent owning its own tooling).
  if [[ ! -f "$image" ]]; then
    echo "vm.sh: missing $image, run 'make build && make iso build' first" >&2
    exit 1
  fi
  cleanup_existing
  mkdir -p "$run_dir"
  exec qemu-system-i386 \
    -cdrom "$image" \
    -m 128M \
    -display none \
    -monitor "unix:$monitor_sock,server,nowait"
}

# Talk to the monitor over a unix socket, run `cmd`, print the raw reply.
# Protocol notes: HMP greets each new connection with a banner; sending a
# command before it arrives races QEMU's chardev setup and the command is
# silently dropped, so we always drain the banner first. The CPU is NOT
# resumed here — callers stop/cont explicitly.
monitor_talk() {
  python3 - "$monitor_sock" "$1" <<'EOF'
import socket, sys, time
path, cmd = sys.argv[1], sys.argv[2]
s = socket.socket(socket.AF_UNIX)
s.connect(path)
s.setblocking(False)
def send(c): s.sendall((c + '\n').encode())
def drain(sec=1.5):
    out = b''
    t = time.time()
    while time.time() - t < sec:
        try:
            d = s.recv(65536)
            if not d: break
            out += d
        except BlockingIOError:
            time.sleep(0.03)
    return out
drain(0.5)          # banner; also settles the session before we speak
send(cmd)
out = drain(2.0)
sys.stdout.write(out.decode(errors='replace'))
EOF
}

screen() {
  [[ -S "$monitor_sock" ]] || {
    echo "vm.sh: no monitor at $monitor_sock, run 'make vm agent' first" >&2
    exit 1
  }
  # Stop the CPU, dump the full 80x25 VGA text buffer, resume the VM.
  monitor_talk stop >/dev/null
  local dump
  local dump_err=0
  dump="$(monitor_talk 'xp /4000xb 0xb8000')" || dump_err=$?
  monitor_talk cont >/dev/null || true
  if [[ $dump_err -ne 0 ]]; then
    exit $dump_err
  fi
  python3 - "$dump" <<'EOF'
import sys, re
text = sys.argv[1]
pairs = []
for line in text.splitlines():
    m = re.match(r'^[0-9a-f]+: (.*)$', line.strip())
    if not m:
        continue
    for tok in m.group(1).split():
        if len(tok) == 4 and tok.startswith('0x'):
            try:
                pairs.append(int(tok, 16))
            except ValueError:
                pass
pairs = pairs[:4000]
out = ''
for i in range(0, len(pairs) - 1, 2):
    ch = pairs[i]
    out += chr(ch) if 32 <= ch < 127 else ' '
for i in range(0, len(out), 80):
    print(out[i:i + 80].rstrip())
EOF
}

# Quit the VM via its monitor socket and drop the run dir.
stop() {
  if [[ -f "$run_dir/vm.pid" ]]; then
    local pid
    pid="$(cat "$run_dir/vm.pid" 2>/dev/null || true)"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      if [[ -S "$monitor_sock" ]]; then
        monitor_talk quit >/dev/null 2>&1 || true
      fi
      if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
      fi
    fi
  elif [[ -S "$monitor_sock" ]]; then
    monitor_talk quit >/dev/null 2>&1 || true
  else
    echo "vm.sh: no monitor at $monitor_sock or running VM found" >&2
    exit 1
  fi
  rm -rf "$run_dir"
  echo "[*] VM stopped"
}

# Print usage and exit non-zero (unknown or missing subcommand).
usage() {
  echo "usage: vm.sh launch|agent|foreground|screen|stop" >&2
  exit 1
}

case "${1:-}" in
  launch) exec qemu-system-i386 -cdrom "$image" -display gtk ;;
  agent) start ;;
  foreground) serve ;;
  screen) screen ;;
  stop) stop ;;
  *) usage ;;
esac
