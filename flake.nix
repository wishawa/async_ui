{
  description = "Auth service";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
          config.allowUnfree = true;
        };
        rust =
          (pkgs.rustChannelOf {
            date = "2024-11-06";
            channel = "nightly";
          })
          .default
          .override {
            extensions = ["rust-analyzer" "rust-src" "rustc-codegen-cranelift-preview"];
            targets = ["wasm32-unknown-unknown"];
          };
        craneLib = ((crane.mkLib pkgs).overrideToolchain rust).overrideScope (_final: _prev: {
          inherit (pkgs) wasm-bindgen-cli;
        });
      in rec {
        serverDeps = with pkgs; [
          rustPlatform.bindgenHook

          wasm-bindgen-cli
        ];
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs;
            [
              alejandra
              rust
              cargo-expand
              cargo-watch
              cargo-edit
              gdb
              wasm-pack
              miniserve
            ]
            ++ serverDeps;
          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
        };
      }
    );
}
