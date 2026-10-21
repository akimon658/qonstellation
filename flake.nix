{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-analyzer" "rust-src" ];
        };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "qonstellation";
          version = "0.9.1";

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./.sqlx
              ./migrations
              ./src
              ./Cargo.lock
              ./Cargo.toml
            ];
          };

          cargoLock.lockFile = ./Cargo.lock;

          SQLX_OFFLINE = "true";

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];
        };

        apps = {
          fmt = {
            type = "app";
            program = toString (pkgs.writeShellScript "fmt" ''
              ${pkgs.taplo}/bin/taplo fmt *.toml
              ${rustToolchain}/bin/cargo fmt
            '');
          };
        };
        devShells.default = pkgs.mkShell {
          packages =  [
            pkgs.sqlx-cli
            pkgs.taplo
            rustToolchain
          ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
