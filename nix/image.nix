{
  pkgs,
  backend,
  ...
}:
let
  inherit (pkgs) lib;
in
pkgs.dockerTools.buildImage {
  name = "kybe-backend";
  tag = "latest";

  copyToRoot = with pkgs; [
    cacert
  ];

  config = {
    Cmd = [ "${lib.getExe backend}" ];
    WorkingDir = "/opt/backend";
  };
}
