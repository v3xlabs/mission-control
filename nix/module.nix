{self}: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.missiond;
  inherit (lib) types;

  toml = pkgs.formats.toml {};

  # Only the options the module has to reason about are typed. Tabs, playlists and overlay pass
  # through to the TOML generator, so a schema change does not have to be mirrored in Nix twice.
  passthrough = types.attrsOf types.anything;

  scheduleWindow = types.submodule {
    options = {
      days = lib.mkOption {
        type = types.listOf (types.enum ["mon" "tue" "wed" "thu" "fri" "sat" "sun"]);
        description = "Days this window applies to.";
      };
      from = lib.mkOption {
        type = types.strMatching "[0-9]{2}:[0-9]{2}";
        description = "Local time the screen turns on, as HH:MM.";
      };
      to = lib.mkOption {
        type = types.strMatching "[0-9]{2}:[0-9]{2}";
        description = "Local time the screen turns off, as HH:MM.";
      };
    };
  };

  displayOptions = types.submodule {
    options = {
      output = lib.mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Output name substituted into the power and brightness commands.";
      };
      power_on = lib.mkOption {
        type = types.listOf types.str;
        default = ["niri" "msg" "action" "power-on-monitors"];
        description = "Command that turns the screen on.";
      };
      power_off = lib.mkOption {
        type = types.listOf types.str;
        default = ["niri" "msg" "action" "power-off-monitors"];
        description = "Command that turns the screen off.";
      };
      brightness = lib.mkOption {
        type = types.listOf types.str;
        default = ["ddcutil" "setvcp" "10" "{percent}"];
        description = "Command that sets panel brightness. `{percent}` is substituted.";
      };
      idle_timeout = lib.mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "20m";
        description = "How long without input before the screen sleeps inside an on window.";
      };
      schedule = lib.mkOption {
        type = types.listOf scheduleWindow;
        default = [];
        description = "Weekly windows during which the screen is on.";
      };
    };
  };

  # The daemon reads arrays; the module takes attribute sets so an entry can be named and
  # overridden the way the rest of a NixOS config is.
  namedList = key: attrs:
    lib.mapAttrsToList (name: value: {${key} = name;} // value) attrs;

  dropNulls = value:
    if builtins.isAttrs value
    then lib.filterAttrs (_: item: item != null) (lib.mapAttrs (_: dropNulls) value)
    else if builtins.isList value
    then map dropNulls value
    else value;

  settingsDir = pkgs.runCommand "missiond-config" {} ''
    mkdir -p "$out"
    ln -s ${toml.generate "device.toml" (dropNulls ({
        version = 1;
        name = cfg.settings.name;
        device_id = cfg.settings.device_id;
        http = {
          inherit (cfg) host port;
        };
        chromium = cfg.settings.chromium;
      }
      // lib.optionalAttrs (cfg.adminKeyFile != null) {
        admin_key.file = cfg.adminKeyFile;
      }
      // lib.optionalAttrs (cfg.settings.homeassistant != null) {
        homeassistant = cfg.settings.homeassistant;
      }))} "$out/device.toml"
    ln -s ${toml.generate "display.toml" (dropNulls ({version = 1;} // cfg.settings.display))} "$out/display.toml"
    ln -s ${toml.generate "tabs.toml" {
      version = 1;
      tabs = dropNulls (namedList "tab_id" cfg.settings.tabs);
    }} "$out/tabs.toml"
    ln -s ${toml.generate "playlists.toml" {
      version = 1;
      playlists = dropNulls (namedList "playlist_id" cfg.settings.playlists);
    }} "$out/playlists.toml"
  '';

  configDir =
    if cfg.settings == null
    then cfg.configDir
    else settingsDir;
in {
  options.services.missiond = {
    enable = lib.mkEnableOption "missiond, the Mission Control display daemon";

    package = lib.mkPackageOption self.packages.${pkgs.stdenv.hostPlatform.system} "missiond" {};

    user = lib.mkOption {
      type = types.str;
      description = ''
        The user whose graphical session missiond joins. Unlike a network daemon, missiond has to
        start a browser on a compositor, so it runs as a session service rather than a system one.
      '';
      example = "display";
    };

    host = lib.mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Address the web interface and API listen on.";
    };

    port = lib.mkOption {
      type = types.port;
      default = 3000;
      description = "Port the web interface and API listen on.";
    };

    openFirewall = lib.mkEnableOption "opening the missiond port in the firewall";

    adminKeyFile = lib.mkOption {
      type = types.nullOr types.path;
      default = null;
      example = "/run/secrets/missiond_admin_key";
      description = ''
        File holding the key every mutating request must present as a bearer token. Without one,
        anything that can reach the port can change what is on screen.
      '';
    };

    extraPackages = lib.mkOption {
      type = types.listOf types.package;
      default = [];
      description = "Extra packages on the daemon's PATH, for the display commands it runs.";
    };

    configDir = lib.mkOption {
      type = types.path;
      default = "/var/lib/missiond/config";
      description = "Directory holding the TOML documents, when `settings` is not used.";
    };

    settings = lib.mkOption {
      default = null;
      description = ''
        Configuration generated as TOML into the Nix store. The store path is read-only, so the
        web UI applies a change immediately and reports that it will not survive a restart.
      '';
      type = types.nullOr (types.submodule {
        options = {
          name = lib.mkOption {
            type = types.str;
            default = "Mission Control";
            description = "Display name, shown in the web UI and to Home Assistant.";
          };

          device_id = lib.mkOption {
            type = types.str;
            default = "missiond";
            description = "Stable identity, used for MQTT discovery topics.";
          };

          chromium = lib.mkOption {
            type = passthrough;
            default = {};
            description = ''
              Browser settings. `fullscreen = true` restores the old kiosk behaviour, at the cost
              of overlay surfaces no longer being able to reserve space beside the page.
            '';
          };

          homeassistant = lib.mkOption {
            type = types.nullOr passthrough;
            default = null;
            description = "MQTT connection for Home Assistant discovery.";
          };

          display = lib.mkOption {
            type = displayOptions;
            default = {};
            description = "Screen power, brightness and schedule.";
          };

          tabs = lib.mkOption {
            type = passthrough;
            default = {};
            example = lib.literalExpression ''
              {
                grafana-overview.url = "http://127.0.0.1:3001/d/mission-overview?kiosk";
              }
            '';
            description = "Tabs, keyed by tab id.";
          };

          playlists = lib.mkOption {
            type = passthrough;
            default = {};
            example = lib.literalExpression ''
              {
                mission-display = {
                  interval = "1m";
                  is_default = true;
                  tabs = ["grafana-overview"];
                };
              }
            '';
            description = "Playlists, keyed by playlist id. The tab list order is the play order.";
          };
        };
      });
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.settings == null || cfg.configDir == "/var/lib/missiond/config";
        message = "services.missiond.settings cannot be used with services.missiond.configDir.";
      }
      {
        assertion = cfg.adminKeyFile != null || cfg.host == "127.0.0.1";
        message = ''
          services.missiond listens on ${cfg.host} without an adminKeyFile, so anything that can
          reach port ${toString cfg.port} can change the display. Set adminKeyFile, or bind to
          127.0.0.1.
        '';
      }
    ];

    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [cfg.port];

    systemd.user.services.missiond = {
      description = "Mission Control display daemon";
      wantedBy = ["graphical-session.target"];
      partOf = ["graphical-session.target"];
      after = ["graphical-session.target"];

      # The browser and the display commands are found here rather than assumed to be on the
      # user's PATH, which a systemd unit does not inherit.
      path = cfg.extraPackages;

      environment = {
        MISSIOND_CONFIG_DIR = toString configDir;
        MISSIOND_CONFIG_READ_ONLY = lib.boolToString (cfg.settings != null);
        MISSIOND_STATE_DIR = "%S/missiond";
        MISSIOND_CACHE_DIR = "%C/missiond";
      };

      serviceConfig = {
        ExecStart = lib.getExe cfg.package;
        Restart = "on-failure";
        RestartSec = 5;
        StateDirectory = "missiond";
        CacheDirectory = "missiond";
      };
    };

    systemd.user.services.missiond.unitConfig.ConditionUser = cfg.user;
  };
}
