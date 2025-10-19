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

                (texlive.combine {
                  inherit (texlive)
                    scheme-basic
                    # dvisvgm dvipng # for preview and export as html
                    # wrapfig amsmath ulem hyperref capt-of
                    #(setq org-latex-compiler "lualatex")
                    #(setq org-preview-latex-default-process 'dvisvgm)
                    ;
                })

                cargo-binutils
                cargo-expand
                cargo-generate
                cargo-nextest
                bacon
                gdb
                openocd
                usbutils
                (pkgs.rustPlatform.buildRustPackage rec {
                  pname = "probe-rs-tools";
                  version = "0.27.0";

                  src = fetchFromGitHub {
                    owner = "probe-rs";
                    repo = "probe-rs";
                    rev = "70f2e4060a81041f2dac7f388b5e11143696b3af";
                    hash = "sha256-qpHSfn8/dvFC5yjtbD35j5OH2XJQrDI988V7Oidsr8A=";
                  };

                  cargoHash = "sha256-C5ca6uaZ6RBtLoShkat9OoaeNM7iEgiUltcwBCcvvQw=";

                  buildAndTestSubdir = pname;

                  nativeBuildInputs = [
                    cmake
                    pkg-config
                  ];

                  buildInputs = [
                    libusb1
                    openssl
                  ];
                  doCheck = false;
                })
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
