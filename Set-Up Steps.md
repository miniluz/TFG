# Set-Up Steps

1. On Linux, udev rules are required, which will vary system to system.
   <https://docs.rust-embedded.org/book/intro/install/linux.html#udev-rules>.
2. A HAL for the architecture, like <https://lib.rs/crates/stm32h7xx-hal>, is
   required.
3. A memory.x for the specific chip is required. I got mine from the
   [embassy examples](https://github.com/embassy-rs/embassy/blob/main/examples/stm32h723/memory.x).
