{
  lib,
  pkgs,
  backend,
  ...
}:
pkgs.dockerTools.buildLayeredImage {
  name = "kybe-backend";
  tag = "latest";

  contents = [ pkgs.cacert ];

  config = {
    Cmd = [ "${lib.getExe backend}" ];
    WorkingDir = "/opt/backend";
  };
}
