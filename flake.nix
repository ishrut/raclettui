{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;

        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo
            rustc
            rustfmt
            pre-commit
            rust-analyzer
            rustPackages.clippy
            libglvnd
            mesa
            wayland
            libevdev
            libxkbcommon
            fontconfig
            llvmPackages.clang
            llvmPackages.libclang
          ];

          nativeBuildInputs = [
            pkg-config
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.mesa
            pkgs.vulkan-loader
            pkgs.libglvnd
          ];

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          shellHook = ''
          '';
        };
      }
    );
}
