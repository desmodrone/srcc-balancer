# Comet Contracts

Contracts written using Soroban

## How to Test

### Without logs

```cargo test```

### With logs

```cargo test -- --nocapture```

## Create a WASM Release Build

```cargo build --target wasm32-unknown-unknown --release```

## Coding Best Practices Used

1. All Rust code is linted with Clippy with the command `cargo clippy`. If preferred to ignore its advice, do so explicitly:
   `#[allow(clippy::too_many_arguments)]`

2. All rust code is formatted with `cargo fmt`. rustfmt.toml defines the expected format.

3. Function and local variable names follow snake_case. Structs or Enums follow CamelCase and Constants have all capital letters.
