# bullet-bs58

[![Crates.io](https://img.shields.io/crates/v/bullet-bs58.svg)](https://crates.io/crates/bullet-bs58)
[![Documentation](https://docs.rs/bullet-bs58/badge.svg)](https://docs.rs/bullet-bs58)
[![License: MIT](https://img.shields.io/crates/l/bullet-bs58.svg)](LICENSE)

Fast fixed-size base58 encoding and decoding for 32-byte arrays.

## Install

```sh
cargo add bullet-bs58
```

## Usage

```rust
use bullet_bs58::{encode32, parse32};

let bytes: [u8; 32] = [0; 32];
let encoded: [u8; 48] = encode32(&bytes);
let decoded: [u8; 32] = parse32(&encoded).unwrap();
```

## License

MIT
