{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  packages = with pkgs; [
    git
    cargo-binutils
    cargo-generate
    gdb
    openocd
    usbutils
    renode
    probe-rs-tools
    # qemu_full
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
