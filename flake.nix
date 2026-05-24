{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane.url = "github:ipetkov/crane";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      crane,
      nixpkgs,
      flake-utils,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        backend = pkgs.callPackage ./nix/backend.nix { inherit self crane; };
        image = pkgs.callPackage ./nix/image.nix { backend = backend.package; };
        treefmt-eval = treefmt-nix.lib.evalModule pkgs ./nix/treefmt.nix;
      in
      {
        checks = {
          inherit (backend.checks) kybe-backend kybe-backend-clippy;
          formatting = treefmt-eval.config.build.check self;
        };
        packages = {
          backend = backend.package;
          inherit image;
        };
        devShells.backend = backend.devShell;
        formatter = treefmt-eval.config.build.wrapper;
      }
    );
}
