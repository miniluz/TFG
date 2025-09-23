# How did I set this up?

Based on the
[Cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
chapter of the Rust Book, you just need to add:

```toml
[workspace]
resolver = "3"
```

to the `Cargo.toml` and then you can
`cargo new --lib library-name --name library-name` and
`cargo new --bin entry --name entry`. On the entry, you can declare the library
as a dependency with

```toml
[dependencies]
library-name = { path = "../library-name" }
```

Then, as per the
[Test driven embedded rust development \[Tutorial\]](https://hackaday.io/page/21907-test-driven-embedded-rust-development-tutorial)
on the `lib.rs` you can add to the library-name:

```rs
#![cfg_attr(not(test), no_std)]
```

to make std available on tests but not for the library itself.

Finally, you should make a `.cargo/config.toml` on the `library-name` directory
containing:

```toml
[build]
target="x86_64-unknown-linux-gnu"
```

For running, you just `cargo test` on the `library-name` directory. You might
also wanna make a [Justfile](https://github.com/casey/just) or
[devenv task](https://devenv.sh/tasks/).

Note that the `memory.x` must be on the project root, not on `entry`, and that
to compile for the right target you must `cargo build` on the `entry` directory.
