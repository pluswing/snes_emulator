#!/usr/bin/env bash
set -euo pipefail

export DISPLAY="${DISPLAY:-:99}"

XVFB_RESOLUTION="${XVFB_RESOLUTION:-1024x768x24}"
VNC_PORT="${VNC_PORT:-5900}"
NOVNC_PORT="${NOVNC_PORT:-6080}"

pids=()

cleanup() {
  for pid in "${pids[@]:-}"; do
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
    fi
  done
}
trap cleanup EXIT INT TERM

Xvfb "$DISPLAY" -screen 0 "$XVFB_RESOLUTION" -ac -noreset +extension GLX +render &
pids+=("$!")

for _ in $(seq 1 50); do
  if xdpyinfo -display "$DISPLAY" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

fluxbox >/tmp/fluxbox.log 2>&1 &
pids+=("$!")

x11vnc \
  -display "$DISPLAY" \
  -forever \
  -nopw \
  -quiet \
  -rfbport "$VNC_PORT" \
  -shared \
  >/tmp/x11vnc.log 2>&1 &
pids+=("$!")

websockify \
  --web=/usr/share/novnc \
  "0.0.0.0:${NOVNC_PORT}" \
  "127.0.0.1:${VNC_PORT}" \
  >/tmp/novnc.log 2>&1 &
pids+=("$!")

cat <<EOF
noVNC is ready:
  http://localhost:${NOVNC_PORT}/vnc.html?autoconnect=true&resize=scale

Run SDL apps with:
  docker compose exec app cargo run --bin main
EOF

exec "$@"
