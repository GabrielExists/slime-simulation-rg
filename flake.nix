{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-24.11";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in
      with pkgs; rec {
        devShell = mkShell rec {
          buildInputs = [
            libxkbcommon
            libGL
            vulkan-loader

            # vulkan-headers
            # vulkan-validation-layers

            # WINIT_UNIX_BACKEND=wayland
            wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";

          # If it doesn’t get picked up through nix magic
          # VULKAN_SDK = "${vulkan-validation-layers}/share/vulkan/explicit_layer.d"
        };
      });
}
