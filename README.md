# Mission Control

Mission Control is a tool for managing your information displays.
It is a standalone binary that you can run on any linux (x86 or arm) machine using xorg.

Features:

- 🖥️ **Digital Signage Management**: Control multiple displays, playlists, and tabs
- 🏠 **Home Assistant Integration**: MQTT-based device discovery and control
- 🌐 **Web UI**: Modern React-based dashboard for monitoring and control
- 🎛️ **REST API**: OpenAPI-documented endpoints for programmatic control
- 📱 **Live Previews**: Real-time tab screenshots and live streaming

We recommend you use a setup [as described here](https://env.md/networks/information-displays.html).

## Installation

Simply download the latest binary from the releases page and run it on your machine.
You could also easily grab the [latest x86 binary](https://github.com/v3xlabs/mission-control/releases/latest/download/v3x-mission-control-x86_64) or the [latest arm binary](https://github.com/v3xlabs/mission-control/releases/latest/download/v3x-mission-control-arm64) from these links.

## Web UI

Mission Control includes a built-in web interface accessible at `http://localhost:3000` once the server is running. The web UI provides:

- 📊 **Dashboard**: Overview of all playlists and their current status
- 🖼️ **Live Previews**: Thumbnail previews of each tab with live updates
- ⚡ **Quick Controls**: Click to activate playlists or switch to specific tabs
- 📈 **Device Status**: Real-time monitoring of device state and uptime

## Development

### Building from Source

1. **Build Web UI**:

```bash
cd web
pnpm install
pnpm dev
```

The web development server runs on `http://localhost:5173` and proxies API calls to the backend.

1. **Build Backend**:

```bash
cd app
cargo run
```

## Configuration

The configuration is stored at `./config.toml` or `~/.config/v3x-mission-control/config.toml`.

A sample configuration look as follows:

```toml
[homeassistant]
mqtt_url = "mqtt://localhost:1883"
mqtt_username = "username"
mqtt_password = "password"

[device]
name = "My Display"
id = "my_display_1"

[display]
# sleep time in seconds
sleep_time = 10

[chromium]
enabled = true
# optional
binary_path = "/usr/bin/chromium"

[chromium.tabs.my_homepage]
url = "https://v3x.fyi/s1"
persist = true

[chromium.tabs.google_news]
url = "https://news.google.com/topstories"
persist = true

[chromium.playlists.my_playlist]
tabs = [
    "my_homepage",
    "google_news",
]
# Alternate between tabs every 30 seconds
interval = 30
```
