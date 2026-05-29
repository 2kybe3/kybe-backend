{ pkgs, ... }:
pkgs.buildEnv {
  name = "ci";

  paths = with pkgs; [
    attic-client
  ];
}
