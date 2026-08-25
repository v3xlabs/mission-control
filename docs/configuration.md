# Configuration

This is the schema reference for every file missiond reads. What the daemon writes back, what the
web UI exports, and what the NixOS module generates are the same documents.

## Locations

| Path | Contents |
| --- | --- |
| `$MISSIOND_CONFIG_DIR`, else `$XDG_CONFIG_HOME/missiond`, else `~/.config/missiond` | configuration |
| `$MISSIOND_STATE_DIR`, else `$XDG_STATE_HOME/missiond`, else `~/.local/state/missiond` | volatile runtime state |
| `$MISSIOND_CACHE_DIR`, else `$XDG_CACHE_HOME/missiond`, else `~/.cache/missiond` | the Chromium profile |

```text
~/.config/missiond/
  device.toml
  display.toml
  tabs.toml
  playlists.toml
```

A document that does not exist loads its defaults, so a fresh install starts with no tabs rather
than with an error. Every document carries `version`, and the daemon writes the current version
back.

State and cache are both reconstructible. `runtime.sqlite3` holds what was on screen and whether
the display was on; deleting it costs one restart. Configuration is the only directory worth
backing up.

Writes are atomic: the daemon writes `<name>.toml.tmp` and renames.

## Read-only mode

The config directory is the source of truth. When the NixOS module generates it, the directory is
a Nix store path and every write fails, so missiond treats it as read-only. It detects this two
ways: `MISSIOND_CONFIG_READ_ONLY=1`, which the module sets, or a directory with no write bits.

In read-only mode a change from the web UI still applies. The display reacts at once, and the API
answers `{"persisted": false}` so the UI can say the change lasts until the next restart. Use
`GET /api/config/export` to get the effective configuration back as TOML.

This is the deliberate trade for a declarative setup, and it is why the UI shows a badge rather
than failing a save.

## device.toml

```toml
version = 1
name = "Lobby Display"
device_id = "lobby-display"

[http]
host = "0.0.0.0"
port = 3000

[admin_key]
file = "/run/secrets/missiond_admin_key"

[chromium]
enabled = true
fullscreen = false
binary_path = "/run/current-system/sw/bin/chromium"
extra_args = ["--force-dark-mode"]

[homeassistant]
mqtt_url = "mqtt://broker.example:1883"
username = "missiond"
password = { env = "MISSIOND_MQTT_PASSWORD" }
```

| Field | Type | Notes |
| --- | --- | --- |
| `name` | string | Display name, shown in the web UI and to Home Assistant. |
| `device_id` | string | Stable identity. MQTT discovery topics are built from it. |
| `http.host`, `http.port` | string, integer | Where the API and web UI listen. Defaults are `0.0.0.0` and `3000`. |
| `admin_key` | secret, optional | Required on every mutating request as `authorization: Bearer <key>`. Without one, anything that can reach the port can change the display. |
| `chromium.enabled` | boolean | A disabled browser leaves the API and web UI running, which is how the daemon is tested without a compositor. |
| `chromium.fullscreen` | boolean | Defaults to `false`. See below. |
| `chromium.binary_path` | string, optional | Falls back to `$CHROMIUM_BINARY`, then to `chromium` on `PATH`. |
| `chromium.extra_args` | array | Appended to the argument list. |
| `homeassistant` | table, optional | Omit it and MQTT stays off. |

### fullscreen

`fullscreen = true` passes `--kiosk`, and the browser covers the output.

The default is `false`, which starts a maximised window the compositor tiles. A full-screen window
is not subject to the space a layer surface reserves, so a reserved overlay sidebar only works
with a tiled window. On niri, hide the decorations with a window rule.

## display.toml

```toml
version = 1
output = "DP-1"

power_on  = ["niri", "msg", "action", "power-on-monitors"]
power_off = ["niri", "msg", "action", "power-off-monitors"]
brightness = ["ddcutil", "setvcp", "10", "{percent}", "--display", "1"]

idle_timeout = "20m"

[[schedule]]
days = ["mon", "tue", "wed", "thu", "fri"]
from = "07:30"
to   = "23:00"
```

The commands live here rather than in a match arm inside the daemon, so a compositor missiond has
never heard of needs a config line rather than a new code path. `{percent}` and `{output}` are
substituted. The defaults are the niri commands above.

| Field | Type | Notes |
| --- | --- | --- |
| `output` | string, optional | Substituted for `{output}`. |
| `power_on`, `power_off` | array | Argument vectors, not shell strings. |
| `brightness` | array | Receives `{percent}` from 0 through 100. |
| `idle_timeout` | duration, optional | How long before the screen sleeps inside an on window. |
| `schedule` | array of tables | Weekly windows during which the screen is on. |

A window whose `to` is earlier than its `from` runs past midnight. A day with no window is off.
With no windows at all the schedule has no opinion and never touches the screen.

The schedule sets the baseline. A command from the API or from MQTT overrides it, and the override
holds until the next schedule boundary rather than forever, so turning the screen on at midnight
does not keep it on until Monday.

## tabs.toml

```toml
version = 1

[[tabs]]
tab_id = "grafana-overview"
name = "Overview"
url = "http://127.0.0.1:3001/d/mission-overview?kiosk"
persist = true
scale = 1.25
```

| Field | Type | Notes |
| --- | --- | --- |
| `tab_id` | string | The identity. A playlist references it, and changing it orphans those references. |
| `name` | string, optional | Web UI label. Defaults to `tab_id`. |
| `url` | string | What the page loads. |
| `persist` | boolean | Whether the page stays loaded when the playlist moves on. Defaults to `true`. |
| `scale` | float, optional | Device scale factor for this page only, so a dashboard and a departure board can differ on the same panel. |

## playlists.toml

```toml
version = 1

[[playlists]]
playlist_id = "mission-display"
name = "Mission Display"
interval = "1m"
hold = "5m"
is_default = true
tabs = ["grafana-overview", "homelab-uptime", "indexer-prices"]
disabled_tabs = ["indexer-prices"]
```

| Field | Type | Notes |
| --- | --- | --- |
| `playlist_id` | string | The identity. |
| `name` | string, optional | Defaults to `playlist_id`. |
| `interval` | duration | How long each tab holds the screen. |
| `hold` | duration, optional | How long a tab chosen by hand suspends rotation. Without it, pressing a Stream Deck key can be undone by the rotation timer a second later. |
| `is_default` | boolean | Which playlist starts at boot. With none set, the first one starts. |
| `tabs` | array of `tab_id` | **The list order is the play order.** Reordering in the web UI reorders this array. |
| `disabled_tabs` | array of `tab_id` | Skipped without being removed, so re-enabling keeps the position. |

A `tab_id` with no tab behind it is skipped rather than treated as an error, so deleting a tab
cannot break a playlist.

Rotation aligns to the wall clock: a one minute interval changes on the minute.

## Durations

`30s`, `5m`, `1h`. A bare number is seconds. These are what the daemon writes back, so a
round trip through the web UI does not turn `1m` into `60`.

## Secrets

Any secret field accepts three forms:

```toml
admin_key = { env = "MISSIOND_ADMIN_KEY" }    # read from the environment at start
admin_key = { file = "/run/secrets/key" }     # read from disk at start, trimmed
admin_key = "inline-is-permitted"             # discouraged
```

Resolution happens once, at start. A missing variable or an unreadable file stops the daemon with
a readable message rather than starting it with an empty credential, because a display daemon that
silently accepts every request is worse than one that does not start.

**An export never contains an inline secret.** `GET /api/config/export` replaces an inline value
with a reference placeholder derived from the field, so the exported document is directly usable
given the environment variable rather than merely redacted.

## The API

`GET` is open. Every other method requires the admin key when one is configured.

| Method | Path | Does |
| --- | --- | --- |
| GET | `/api/status` | What is on screen, plus `config_read_only`. |
| GET | `/api/events` | Server sent events. One message per change. |
| GET | `/api/playlists` | Every playlist. |
| GET | `/api/playlists/:playlist_id/tabs` | A playlist's tabs, in play order. |
| GET | `/api/tabs` | Every configured tab. |
| GET | `/api/preview/:tab_id` | One JPEG frame. |
| GET | `/api/preview_live/:tab_id` | An MJPEG stream. |
| GET | `/api/config/export` | Every document as TOML. |
| POST | `/api/playlists/:playlist_id/activate` | Put a playlist on screen. |
| POST | `/api/playlists/:playlist_id/tabs/:tab_id/activate` | Put a tab on screen, and hold it. |
| POST | `/api/playback/{next,previous,pause,resume}` | Drive the rotation. |
| PUT | `/api/playlists/:playlist_id/reorder` | Reorder, with the full tab list. |
| PUT | `/api/playlists/:playlist_id/tabs/:tab_id/enabled` | Include or exclude a tab. |
| PUT | `/api/tabs/:tab_id` | Create a tab, or replace one with that id. |
| DELETE | `/api/tabs/:tab_id` | Remove a tab and every reference to it. |
| POST | `/api/display/power/:on` | Turn the screen on or off. |
| PUT | `/api/display/brightness` | Set panel brightness over DDC. |
| POST | `/api/system/{poweroff,reboot,suspend}` | Power actions over logind. |

Subscribing to a preview starts the capture, and dropping the subscription stops it, so a display
nobody is watching does not encode JPEG in the background. The tab on screen keeps a slow capture
running so `/api/status` always has something recent.

Failures answer with a status code and `{"message": "..."}`. Nothing returns HTTP 200 carrying an
error.

Swagger UI is at `/docs`, and the OpenAPI document at `/docs/spec`.

## Previews

A preview is the tab's own rendered output, captured over CDP. Two properties follow from that
and are worth knowing before you read a blank thumbnail as a bug.

A page has no frame until it has been rendered at least once, so a tab that has never been on
screen shows nothing. The rotation fixes that within one cycle.

A tab that is not on screen and that no browser is watching stops capturing. Opening the web UI
starts it again, so an unattended display does no JPEG encoding for tabs nobody can see.
