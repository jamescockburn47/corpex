#!/bin/bash
set -e

# Start virtual framebuffer (1400x900, 24-bit color)
Xvfb :99 -screen 0 1400x900x24 -ac &
sleep 1

# Start a minimal window manager so the app gets proper window decorations
openbox &
sleep 0.5

# Start VNC server (no password, listen on localhost only)
x11vnc -display :99 -forever -nopw -listen 127.0.0.1 -xkb -ncache 10 -shared &
sleep 0.5

# Start noVNC websocket proxy (port 8080 → VNC port 5900)
websockify --web /usr/share/novnc 8080 127.0.0.1:5900 &
sleep 0.5

echo "[corpex-demo] noVNC available at http://localhost:8080/vnc.html?autoconnect=true&resize=scale"

# Launch Corpex — this is the original unmodified binary
exec corpex
