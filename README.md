# bullet-bs58

Fast fixed-size base58 encoding and decoding for 32-byte arrays.

```rust
use bullet_bs58::{encode32, parse32};

let bytes: [u8; 32] = [0; 32];
let encoded: [u8; 48] = encode32(&bytes);
let decoded: [u8; 32] = parse32(&encoded).unwrap();
```

## License

MIT
