# Mission Control

Mission Control is a tool for managing your information displays.
It is a standalone binary that you can run on any linux (x86 or arm) machine using xorg.

We recommend you use a setup [as described here](https://env.md/networks/information-displays.html).

## Installation

Simply download the latest binary from the releases page and run it on your machine.
You could also easily grab the [latest x86 binary](https://github.com/v3xlabs/mission-control/releases/latest/download/v3x-mission-control-x86_64) or the [latest arm binary](https://github.com/v3xlabs/mission-control/releases/latest/download/v3x-mission-control-arm64) from these links.

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
```
