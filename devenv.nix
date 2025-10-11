{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  packages = with pkgs; [
    cargo-binutils
    cargo-expand
    cargo-generate
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

  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [
      "x86_64-unknown-linux-gnu"
      "thumbv7em-none-eabihf"
    ];
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "llvm-tools"
    ];
  };

  # git-hooks.hooks = {
  #   rustfmt.enable = true;
  #   clippy.enable = true;
  # };

}
