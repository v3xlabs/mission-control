# missiond

missiond is the daemon that owns an information display. It holds the content, the screen power,
and one control surface that a browser, Home Assistant and launchpi all drive.

It runs on a Wayland session, starts Chromium, and rotates a playlist of tabs. What is on screen
is decided here rather than by the pages themselves.

Features:

- Playlists of tabs, rotated on a wall-clock aligned interval
- Screen power and DDC brightness, on a weekly schedule
- Poweroff, reboot and suspend over logind
- Home Assistant MQTT discovery for the screen, the playlist and the tab
- A web UI with live previews, driven by a server sent event stream
- An OpenAPI-documented REST API
- A NixOS module that declares the whole configuration

## Configuration

missiond reads a directory of TOML documents. See [docs/configuration.md](docs/configuration.md)
for the schema.

| Directory | Resolution order | Holds |
| --- | --- | --- |
| config | `MISSIOND_CONFIG_DIR`, `$XDG_CONFIG_HOME/missiond`, `~/.config/missiond` | The TOML documents. The only directory worth backing up. |
| state | `MISSIOND_STATE_DIR`, `$XDG_STATE_HOME/missiond`, `~/.local/state/missiond` | `runtime.sqlite3`. Deleting it costs one restart. |
| cache | `MISSIOND_CACHE_DIR`, `$XDG_CACHE_HOME/missiond`, `~/.cache/missiond` | The Chromium profile. |

## NixOS

```nix
{
  inputs.missiond.url = "github:v3xlabs/missiond";

  # in your configuration
  imports = [inputs.missiond.nixosModules.default];

  services.missiond = {
    enable = true;
    user = "display";
    host = "0.0.0.0";
    openFirewall = true;
    adminKeyFile = config.sops.secrets.missiond_admin_key.path;

    settings = {
      name = "Lobby Display";
      device_id = "lobby-display";

      display.output = "DP-1";
      display.schedule = [
        {
          days = ["mon" "tue" "wed" "thu" "fri"];
          from = "07:30";
          to = "23:00";
        }
      ];

      tabs.grafana-overview.url = "http://127.0.0.1:3001/d/mission-overview?kiosk";

      playlists.mission-display = {
        interval = "1m";
        is_default = true;
        tabs = ["grafana-overview"];
      };
    };
  };
}
```

`settings` generates the config directory into the Nix store, which is read-only. The web UI can
still change anything, and says so: a change applies to the running display immediately and does
not survive a restart. `GET /api/config/export` returns the effective configuration as TOML, so a
change worth keeping can be moved back into your Nix.

Leave `settings` unset to let missiond own its config directory and write to it.

## Development

Everything is in the flake devshell.

```bash
nix develop
just          # list the recipes
just dev      # run the daemon
just web      # run the web UI against it on :5173
just kiosk    # run the daemon inside a nested cage session
just check    # clippy, tests, typecheck, lint
just build    # build the web UI, embed it, build the release binary
```

`just schema` regenerates `web/src/api/schema.gen.ts` from a running daemon.
