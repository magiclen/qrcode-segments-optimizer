QR Code Segments Optimizer
====================

[![CI](https://github.com/magiclen/qrcode-segments-optimizer/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/qrcode-segments-optimizer/actions/workflows/ci.yml)

This library is used for optimizing the QR code segments.

## Examples

```rust
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

## Crates.io

TDB

## Documentation

TDB

## License

[MIT](LICENSE)