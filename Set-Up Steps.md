# Set-Up Steps

1. On Linux, udev rules are required, which will vary system to system.
   <https://docs.rust-embedded.org/book/intro/install/linux.html#udev-rules>.
2. An embassy-compatible HAL for the architecture, like
   <https://lib.rs/crates/embassy-stm32>, is required. I think it also must be
   single-core.
3. A memory.x for the specific chip is required. You can probably get one from
   the embassy HAL. You can also get one from the
   [embassy examples](https://github.com/embassy-rs/embassy/blob/main/examples/stm32h723/memory.x).
4. Inputs that require EXTI (external interrupts) must be on different channels:

> Pins PA5, PB5, PC5… all use EXTI channel 5, so you can’t use EXTI on, say, PA5
> and PC5 at the same time.
