let
  rust-overlay = fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [
      (import rust-overlay)
      (_: prev: {
        rust-toolchain = prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      })
    ];
  };
in
pkgs.callPackage (
  {
    mkShell,
    rust-toolchain,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      rust-toolchain
      pkgs.openssl
      pkgs.gcc
      pkgs.pkg-config
      pkgs.sqlx-cli
    ];
  }
) { }
