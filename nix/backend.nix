{
  self,
  pkgs,
  crane,
  ...
}:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

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
    name = "kybe-backend-wrapped";

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

  devShell = craneLib.devShell {
    checks = checks;
    packages = with pkgs; [
      sqlx-cli
    ];
  };
}
