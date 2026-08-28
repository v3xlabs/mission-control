#!/usr/bin/env bash
# The log is kept, because niri's own stdout is not somewhere you can read afterwards.
exec "$MISSIOND_BINARY" > >(tee -a "$MISSIOND_LAB_LOG") 2>&1
