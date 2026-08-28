#!/usr/bin/env bash
# Started by the nested niri, so it inherits WAYLAND_DISPLAY and NIRI_SOCKET. Everything the
# daemon says is kept, because niri's own stdout is not somewhere you can read it afterwards.
exec "$MISSIOND_BINARY" > >(tee -a "$MISSIOND_LAB_LOG") 2>&1
