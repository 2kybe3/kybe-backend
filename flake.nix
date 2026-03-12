{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
         buildInputs = with pkgs; [
          cargo
          rustc
          clippy
          rustfmt
          openssl
          pkg-config
          rust-analyzer

          sqlx-cli
        ];

        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

        shellHook = ''
          export PKG_CONFIG_PATH=${pkgs.openssl.dev}/lib/pkgconfig
        '';
      };
    };
}
