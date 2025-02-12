```sh
cargo fmt --all -- --check
cargo clippy --all-features --all-targets -- -D warnings
```

```rust
#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::style,
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    rustdoc::all,
    clippy::undocumented_unsafe_blocks
)]
```
