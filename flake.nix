{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      toolchainFile =
        if builtins.pathExists ./rust-toolchain.toml then
          ./rust-toolchain.toml
        else if builtins.pathExists ./toolchain.toml then
          ./toolchain.toml
        else
          throw "No rust toolchain file found";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
          (_: prev: {
            rust-toolchain = prev.rust-bin.fromRustupToolchainFile toolchainFile;
          })
        ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        strictDeps = true;

        nativeBuildInputs = [
          pkgs.rust-toolchain
          pkgs.pkg-config
          pkgs.openssl
          pkgs.sqlx-cli
          pkgs.gcc
        ];

        shellHook = ''
          export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
        '';
      };
    };
}
