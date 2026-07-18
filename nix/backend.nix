{
  lib,
  pkgs,
  self,
  crane,
  ...
}:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

  unfilteredRoot = ../.;

  src = lib.fileset.toSource {
    root = unfilteredRoot;
    fileset = lib.fileset.unions [
      (craneLib.fileset.commonCargoSources unfilteredRoot)
      ../assets
      ../static
    ];
  };

  commonArgs = {
    inherit src;

    __structuredAttrs = true;
    strictDeps = true;

    # no tests
    doCheck = false;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  kybe-backend-unwrapped = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      meta.mainProgram = "kybe-backend";
    }
  );

  kybe-backend = pkgs.symlinkJoin {
    inherit (kybe-backend-unwrapped) pname version;

    paths = [ kybe-backend-unwrapped ];

    nativeBuildInputs = [ pkgs.makeWrapper ];

    postBuild = ''
      wrapProgram $out/bin/kybe-backend \
        --set KYBE_BACKEND_STATIC_DIR ${../static} \
        --set KYBE_BACKEND_GIT_SHA ${self.rev or self.dirtyRev}
    '';

    meta.mainProgram = "kybe-backend";
  };

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

  devShell = craneLib.devShell { };
}
