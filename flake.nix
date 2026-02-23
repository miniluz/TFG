{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-compat = {
      url = "https://git.lix.systems/lix-project/flake-compat/archive/main.tar.gz";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      inherit (nixpkgs) lib;
    in
    lib.foldl lib.recursiveUpdate { } (
      lib.map
        (
          system:
          let
            overlays = [ (import rust-overlay) ];
            pkgs = import nixpkgs { inherit system overlays; };
          in
          {
            devShells.${system}.default = pkgs.mkShell {
              nativeBuildInputs = with pkgs; [
                (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)

                cargo-binutils
                cargo-expand
                cargo-generate
                cargo-nextest
                cargo-bloat
                bacon

                just
                prek
                (octave.withPackages (octavePackages: with octavePackages; [ signal ]))

                lldb
                openocd
                usbutils
                probe-rs-tools

                alsa-utils
                pavucontrol
                audacity

                typst
                typstyle
              ];
              buildInputs = [
              ];
            };
          }
        )
        [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-darwin"
          "x86_64-linux"
        ]
    );
}
