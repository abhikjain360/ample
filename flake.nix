{
  description = "ample";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "aarch64-darwin";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          rustToolchain
          pkgs.bun
          pkgs.mpv
          pkgs.libopus
          pkgs.pkg-config
          pkgs.macdylibbundler
        ];

        shellHook = ''
          export LIBRARY_PATH="${
            pkgs.lib.makeLibraryPath [
              pkgs.mpv
              pkgs.libopus
            ]
          }''${LIBRARY_PATH:+:$LIBRARY_PATH}"
          export DYLD_LIBRARY_PATH="${
            pkgs.lib.makeLibraryPath [
              pkgs.mpv
              pkgs.libopus
            ]
          }''${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
        '';
      };
    };
}
