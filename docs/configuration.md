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
fullscreen = true
binary_path = "/run/current-system/sw/bin/chromium"
extra_args = ["--force-dark-mode"]

[mpv]
binary_path = "/run/current-system/sw/bin/mpv"
extra_args = ["--hwdec=auto-safe"]

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
| `chromium.fullscreen` | boolean | Defaults to `true`. See below. |
| `chromium.binary_path` | string, optional | Falls back to `$CHROMIUM_BINARY`, then to `chromium` on `PATH`. |
| `chromium.extra_args` | array | Appended to the argument list. |
| `mpv.binary_path` | string, optional | Falls back to `$MPV_BINARY`, then to `mpv` on `PATH`. Only camera tabs use it. |
| `mpv.extra_args` | array | Appended to the argument list. |
| `homeassistant` | table, optional | Omit it and MQTT stays off. |

### fullscreen

Defaults to `true`. The browser covers the output and draws none of its own interface: no tab
strip, no omnibox, no profile button.

This takes two things, not one. `--kiosk` covers the output, but a tab created over CDP needs a
tab strip to live in and drags the window back to its decorated form, so the daemon also puts the
window itself into fullscreen through `Browser.setWindowBounds`. That is the presentation change
`F11` makes, and it is what actually hides the browser interface.

Setting it to `false` starts a maximised window the compositor tiles instead. A full-screen window
is not subject to the space a layer surface reserves, so a reserved overlay sidebar will need this
off. It is off by default only once something draws overlays, because a tiled window on niri opens
at its column width, which is half the screen, and shows the browser's full interface. If you turn
it off today, add a niri window rule:

```kdl
window-rule {
    match app-id="chromium-browser"
    open-maximized true
    draw-border-with-background false
}
```

## display.toml

```toml
version = 1
output = "DP-1"

power_on  = ["niri", "msg", "action", "power-on-monitors"]
power_off = ["niri", "msg", "action", "power-off-monitors"]
brightness = ["ddcutil", "setvcp", "10", "{percent}", "--display", "1"]
screenshot = ["grim", "-t", "jpeg", "-q", "80", "-"]

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
| `screenshot` | array | Writes the output's current contents to stdout. The default uses `grim`, which speaks wlr-screencopy. |
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
| `url` | string | What the page loads. A tab has this or `rtsp`, never both. |
| `rtsp` | secret, optional | A camera stream. The whole url is treated as a secret, because the credential is part of it. |
| `persist` | boolean | Whether the page stays loaded when the playlist moves on. Defaults to `true`. |
| `scale` | float, optional | Device scale factor for this page only, so a dashboard and a departure board can differ on the same panel. |
| `stinger` | string, optional | A clip played while this tab loads, by name from `notifications.toml`. |

### Cameras

A browser has no `rtsp://` handler, so a camera is not a page. missiond plays it with mpv in its
own window, and the compositor puts that window over the browser. Everything else about a camera
is an ordinary tab: it sits in a playlist, it rotates, and an alert can take over with it.

```toml
[[tabs]]
tab_id = "front-door"
name = "Front door"
stinger = "doorbell"

[tabs.rtsp]
file = "/run/secrets/front-door-rtsp-url"
```

The file holds the whole url on one line, credential included:

```
rtsp://admin:hunter2@10.0.0.40:554/stream1
```

`env = "NAME"` reads it from the environment instead. An inline string works and is what a test
uses, but it puts a credential in the config file.

missiond hands the url to mpv over a control socket rather than as an argument, so it does not
appear in the process list. It never reaches the web UI, Home Assistant or a log line either: the
API reports a camera by leaving `url` out.

Two consequences follow from a camera being outside the browser. `GET /api/preview/:tab_id` has
no frame for one, because previews come from the browser's own protocol, while `GET /api/screen`
still shows it, because that reads the compositor. And `POST /api/tabs` cannot create one: a
stream url is a credential, and the API writes what it is given back to disk.

mpv has to be on the daemon's PATH. Under the NixOS module that means `extraPackages`. The window
carries the app id `missiond-camera`, so a compositor rule can match it.

A camera that drops leaves its last frame on the wall rather than closing its window, which would
put whatever page is behind it on screen instead.

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

`750ms`, `30s`, `5m`, `1h`. A bare number is seconds. These are what the daemon writes back, so a
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

## The screen, as opposed to a tab

`GET /api/screen` returns what the compositor is putting on the panel, captured through its own
screencopy protocol rather than through the browser. It includes anything drawn over the page,
which a tab preview cannot show.

It runs the `screenshot` command from `display.toml` and nothing else: no background process, no
stream. Repeated requests inside one second reuse the last capture, so holding the page open
cannot spawn a capture per frame.

The default command is `grim`, which needs to be on the daemon's PATH. Under the NixOS module,
add it to `extraPackages`.

## notifications.toml

```toml
version = 1
mode = "takeover"
default_duration = "20s"
sidebar_width = 480

[stingers.doorbell]
file = "doorbell.webm"
max_duration = "2s"
```

| Field | Type | Notes |
| --- | --- | --- |
| `mode` | `takeover` or `sidebar` | What an alert does by default. A single alert can override it. |
| `default_duration` | duration | How long an alert stays up when the caller does not say. |
| `sidebar_width` | integer | How wide the sidebar window asks to be. The compositor has the final say. |
| `stingers` | table | Named clips, keyed by the name a tab or an alert refers to. |

### takeover and sidebar

**Takeover** makes the alert the thing on screen. Rotation stops, and when the alert expires the
display returns to the tab it interrupted rather than to wherever rotation would have reached.

**Sidebar** opens the alert as its own window beside the content and leaves the playlist running.
It is a second browser window in Chromium's app mode, not a layer-shell surface: the compositor is
already a tiling one, so it opens, gets a column, and the display shrinks to make room. The window
carries its own app id, `missiond-sidebar`, so a window rule can place or size it, and its own
profile, so it cannot disturb the display's.

### Stingers

A stinger is a clip played while the screen changes. It is not decoration. A camera feed takes
seconds to connect, and a viewer watching a blank page reads that as broken. The target starts
loading first, the clip covers the wait, and the switch happens behind it.

```toml
[[tabs]]
tab_id = "front-door"
url = "http://camera.example/stream"
stinger = "doorbell"
```

Files live in `media` inside the config directory. That directory can be a Nix store path, which
makes a clip a build input like anything else. `max_duration` cuts the clip off even if it has not
ended, so a mis-encoded file cannot strand the display mid-transition.

Under the NixOS module, `media` is an attribute set keyed by the name a stinger refers to. The
value is any path: a file next to your configuration, or a derivation that produces one.

```nix
services.missiond.settings = {
  notifications.stingers.doorbell = {
    file = "doorbell.webm";
    max_duration = "1500ms";
  };

  media."doorbell.webm" = ./media/doorbell.webm;
};
```

The file has to be in the git tree of the flake it is referenced from, or Nix cannot see it.

For a camera the clip plays first and the stream connects after it, rather than the two
overlapping. mpv connects in well under the time a browser needs to load a camera page, so there
is little left to cover.

### Raising an alert

```bash
curl -X POST http://display.example:3000/api/notify \
  -H "authorization: Bearer $MISSIOND_ADMIN_KEY" \
  -H 'content-type: application/json' \
  -d '{
        "title": "Front door",
        "body": "Someone is at the door",
        "level": "warning",
        "tab_id": "front-door",
        "stinger": "doorbell",
        "duration": "30s"
      }'
```

| Field | Notes |
| --- | --- |
| `title`, `body` | What the alert card shows. `body` is optional. |
| `level` | `info`, `warning` or `critical`. Colours one edge of the card. |
| `mode` | Overrides `mode` for this alert. |
| `duration` | Overrides `default_duration`. |
| `tab_id` | Show this tab instead of a card. This is what turns a doorbell alert into the camera feed rather than the word "doorbell". |
| `stinger` | A clip to cover the change. |

The call returns as soon as the alert is queued. A transition can take seconds and the caller is
usually a doorbell or an automation, so it is not held open while the screen changes.

| Method | Path | Does |
| --- | --- | --- |
| POST | `/api/notify` | Raise an alert. |
| GET | `/api/notifications` | What is currently showing. The alert pages read this. |
| DELETE | `/api/notifications/:notification_id` | Clear one early. |
| GET | `/api/stingers` | The configured clips, so the transition page can resolve a name. |
| GET | `/api/media/:name` | A file from the config directory's `media`. |
