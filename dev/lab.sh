#!/usr/bin/env bash
# A nested niri with missiond running inside it. Mod+Shift+E closes it, and everything missiond
# writes goes to .tmp/lab.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
lab="$root/.tmp/lab"

export MISSIOND_CONFIG_DIR="$lab/config"
export MISSIOND_STATE_DIR="$lab/state"
export MISSIOND_CACHE_DIR="$lab/cache"
export MISSIOND_LAB_LOG="$lab/missiond.log"
export MISSIOND_BINARY="$root/daemon/target/debug/missiond"
export RUST_LOG="${RUST_LOG:-missiond=debug,info}"

mkdir -p "$MISSIOND_STATE_DIR" "$MISSIOND_CACHE_DIR"

if [ ! -e "$MISSIOND_CONFIG_DIR/device.toml" ]; then
    mkdir -p "$MISSIOND_CONFIG_DIR"
    cp "$root"/dev/config/*.toml "$MISSIOND_CONFIG_DIR/"
    echo "seeded $MISSIOND_CONFIG_DIR from dev/config"
fi

if [ ! -d "$root/daemon/web-dist" ]; then
    echo "daemon/web-dist is missing. Run just web-dist first." >&2
    exit 1
fi

# A daemon that cannot bind its port exits inside the compositor, which looks from outside like a
# compositor that came up empty.
port="$(sed -n 's/^port = \([0-9]*\)$/\1/p' "$MISSIOND_CONFIG_DIR/device.toml")"
if [ -n "$port" ] && ss -ltnp "sport = :$port" | grep -q LISTEN; then
    echo "port $port is already in use:" >&2
    ss -ltnp "sport = :$port" | tail -n +2 >&2
    exit 1
fi

cargo build --manifest-path "$root/daemon/Cargo.toml"

echo "lab on http://127.0.0.1:$port, log at $MISSIOND_LAB_LOG"

# niri does not stop the command it started, so a compositor closed with Mod+Shift+E would leave
# the daemon holding its port and its browser.
stop() {
    pkill -f "^$MISSIOND_BINARY$" 2>/dev/null || true
}
trap stop EXIT INT TERM

# The command after -- is started by niri itself, so it inherits WAYLAND_DISPLAY and NIRI_SOCKET.
niri -c "$root/dev/niri.kdl" -- "$root/dev/lab-daemon.sh"
