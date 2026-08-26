{
  nixpkgs,
  rust-overlay,
  system,
}: let
  pkgs = import nixpkgs {
    inherit system;
    overlays = [rust-overlay.overlays.default];
  };

  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [
      "rust-src"
      "llvm-tools"
    ];
  };

  rustfmtNightly = pkgs.rust-bin.nightly.latest.rustfmt;
in
  pkgs.mkShell {
    packages = with pkgs; [
      rustfmtNightly
      rustToolchain
      rust-analyzer
      bacon
      sqlx-cli
      just
      nodejs_24
      pnpm_11
      chromium
      mpv
      cage
      grim
      ddcutil
      pkg-config
    ];

    CHROMIUM_BINARY = "${pkgs.chromium}/bin/chromium";
    MPV_BINARY = "${pkgs.mpv}/bin/mpv";

    shellHook = ''
      just
    '';
  }
