{
  self,
  pkgs,
  crane,
  ...
}:
let
  craneLib = crane.mkLib pkgs;

  inherit (pkgs) lib;

  unfilteredRoot = ../.;

  src = lib.fileset.toSource {
    root = unfilteredRoot;
    fileset = lib.fileset.unions [
      (craneLib.fileset.commonCargoSources unfilteredRoot)
      ../migrations
      ../assets
      ../static
      ../.sqlx
    ];
  };

  commonArgs = {
    inherit src;
    strictDeps = true;

    # no tests
    doCheck = false;

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    KYBE_BACKEND_STATIC_DIR = ../static;
    KYBE_BACKEND_GIT_SHA = self.rev or self.dirtyRev;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  kybe-backend = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      meta.mainProgram = "kybe-backend";
    }
  );

  checks = {
    inherit kybe-backend;
    kybe-backend-clippy = craneLib.cargoClippy (
      commonArgs
      // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      }
    );
  };
in
{
  inherit checks;

  package = kybe-backend;

  devShell = craneLib.devShell {
    checks = checks;
    packages = with pkgs; [
      sqlx-cli
    ];
  };
}
