{
  description = "Mission Control display daemon";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachSystem [
      "aarch64-linux"
      "x86_64-linux"
    ] (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      missiond = pkgs.callPackage ./nix/package.nix {};
    in {
      packages = {
        inherit missiond;
        default = missiond;
      };

      apps = {
        missiond = {
          type = "app";
          program = "${missiond}/bin/missiond";
        };
        default = {
          type = "app";
          program = "${missiond}/bin/missiond";
        };
      };

      devShells.default = import ./nix/devshell.nix {
        inherit nixpkgs rust-overlay system;
      };
    })
    // {
      nixosModules.default = import ./nix/module.nix {inherit self;};
    };
}
