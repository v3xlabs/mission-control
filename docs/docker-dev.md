## Local dev in Docker (X11-first)

- Use `make dev-x11` from the repo root. This runs `scripts/run-x11docker.sh`.
- The script builds `docker/dev-x11.Dockerfile` (Debian + Chromium + GTK/X11 libs).
- By default it uses `x11docker` + Xephyr (nested X11). If your host has `WAYLAND_DISPLAY` and you pass no extra opts, it switches to `--weston-xwayland`.
- Ports: `3000` is published to the host.
- Env knobs:
  - `MC_DISPLAY_SIZE` for window size (default `1280x720`).
  - `MC_X11DOCKER_OPTS` to override backend (e.g. `--weston-xwayland`).
  - `CHROMIUM_EXTRA_ARGS` is set for the container; adjust in the script if needed.
- Limitations: Wayland-native features (sway `dpms`, `ddcutil` on host) are not testable inside this X11 container; use a real sway session on the host for those.

