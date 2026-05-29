{
  lib,
  pkgs,
  backend,
  ...
}:
pkgs.dockerTools.buildImage {
  name = "kybe-backend";
  tag = "latest";

  copyToRoot = [ pkgs.cacert ];

  config = {
    Cmd = [ "${lib.getExe backend}" ];
    WorkingDir = "/opt/backend";
  };
}
