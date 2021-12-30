# SPDX-FileCopyrightText: 2021 localthomas
#
# SPDX-License-Identifier: MIT OR Apache-2.0
{
  description = "This is a standalone binary that listens on the system bus and talks to systemd to identify failed units.";

  inputs = {
    # for eachSystem function
    flake-utils.url = "github:numtide/flake-utils";
    # use flake-compat as side-effect for flake.lock file that is read by shell.nix
    # fill the flake.lock file with `nix flake lock --update-input flake-compat`
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    # get the rust toolchain
    rust-overlay.url = "github:oxalica/rust-overlay";
    # use the rust toolchain for building the binary
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, naersk, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        cargo-metadata = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
        crateName = cargo-metadata.package.name;

        # apply the rust-overlay to nixpkgs
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # setup the rust toolchain based on the rust-toolchain file
        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Override the version used in naersk and using the toolchain from above
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in
      with pkgs;
      {
        devShell = mkShell {
          # tools and dependencies for building and developing
          nativeBuildInputs = [ nixpkgs-fmt rust ];
        };

        checks = {
          format = runCommand "check-format"
            {
              nativeBuildInputs = [ self.devShell.${system}.nativeBuildInputs ];
            }
            ''
              cargo-fmt fmt --manifest-path ${./.}/Cargo.toml -- --check
              nixpkgs-fmt --check ${./.}
              touch $out # touch output file to give the information that check was successful
            '';
        };

        packages.${crateName} = naersk-lib.buildPackage {
          pname = crateName;
          root = ./.;
          # The packages of the devShell are re-used for building
          nativeBuildInputs = [ self.devShell.${system}.nativeBuildInputs ];
          # Configures the target which will be built.
          # ref: https://doc.rust-lang.org/cargo/reference/config.html#buildtarget
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
          doCheck = true;
        };

        defaultPackage = self.packages.${system}.${crateName};
      }
    );
}
