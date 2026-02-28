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
            # rust packages
            cargo
            rustc
            rustfmt
            pre-commit
            rust-analyzer
            rustPackages.clippy
            wgsl-analyzer

            # egl packages
            libglvnd

            # vulkan dependencie
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools
            mesa


            # wayland packages
            wayland

            #keyboard events
            libevdev
            libxkbcommon
            libxcb

            # Clang / bindgen
            llvmPackages.clang
            llvmPackages.libclang
          ];

          nativeBuildInputs = [
            pkg-config
          ];

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          shellHook = ''
          '';
        };
      }
    );
}
