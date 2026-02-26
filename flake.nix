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
    typst-wrapper = {
      url = "github:miniluz/typst-wrapper";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      nixpkgs,
      rust-overlay,
      typst-wrapper,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      wrapTypst = pkgs: typst-wrapper.lib.${pkgs.stdenv.hostPlatform.system}.wrapTypst { };
    in
    {
      devShells = forAllSystems (
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          wrappedTypst = wrapTypst pkgs;
          typstLive = pkgs.symlinkJoin {
            name = "typst-live";
            paths = [ pkgs.typst-live ];
            buildInputs = [ pkgs.makeWrapper ];
            postBuild = ''
              wrapProgram $out/bin/typst-live \
                --suffix PATH : ${pkgs.lib.makeBinPath [ wrappedTypst ]}
            '';
            meta.mainProgram = "typst-live";
          };
          ciPackages = with pkgs; [
            (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            just
            prek
            typstyle
          ];
        in
        {
          ci = pkgs.mkShell {
            nativeBuildInputs = ciPackages;
            buildInputs = [ ];
          };

          default = pkgs.mkShell {
            nativeBuildInputs =
              ciPackages
              ++ (with pkgs; [
                cargo-binutils
                cargo-expand
                cargo-generate
                cargo-nextest
                cargo-bloat
                bacon

                (octave.withPackages (octavePackages: with octavePackages; [ signal ]))

                lldb
                openocd
                usbutils
                probe-rs-tools

                alsa-utils
                pavucontrol
                audacity

                wrappedTypst
                typstLive
              ]);
            buildInputs = [ ];
          };
        }
      );
    };
}
