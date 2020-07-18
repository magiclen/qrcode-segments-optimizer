QR Code Segments Optimizer
====================

[![Build Status](https://travis-ci.org/magiclen/qrcode-segments-optimizer.svg?branch=master)](https://travis-ci.org/magiclen/qrcode-segments-optimizer)

This library is used for optimizing the QR code segments.

## Examples

```rust
extern crate qrcode_generator;
extern crate qrcode_segments_optimizer;
extern crate url;

use qrcode_generator::QrCodeEcc;
use url::Url;

let url = "https://magiclen.org/path/to/12345";
let ecc = QrCodeEcc::Low;

let naive_matrix = qrcode_generator::to_matrix(url, ecc).unwrap();

let url = Url::parse(url).unwrap();

let optimized_matrix = qrcode_generator::to_matrix_from_segments(
    qrcode_segments_optimizer::make_segments_from_url(&url, ecc)
        .unwrap()
        .as_slice(),
    ecc,
)
.unwrap();

assert!(optimized_matrix.len() < naive_matrix.len());
```

## No Std

Disable the default features to compile this crate without std.

```toml
[dependencies.qrcode-segments-optimizer]
version = "*"
default-features = false
```

## Crates.io

TDB

## Documentation

TDB

## License

[MIT](LICENSE)