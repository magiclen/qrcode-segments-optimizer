extern crate qrcode_generator;
extern crate qrcode_segments_optimizer;
extern crate url;

#[cfg(feature = "test-image")]
#[macro_use]
extern crate slash_formatter;

#[cfg(feature = "std")]
use core::str::FromStr;

#[cfg(feature = "test-image")]
use std::fs;
#[cfg(feature = "test-image")]
use std::path::Path;

use qrcode_generator::QrCodeEcc;

use url::Url;

#[cfg(feature = "std")]
use qrcode_segments_optimizer::models::Email;

#[cfg(feature = "test-image")]
const FOLDER: &str = concat_with_file_separator!("tests", "data");

#[test]
fn optimize_url() {
    let url = "https://magiclen.org/path/to/12345";
    let ecc = QrCodeEcc::Low;

    let naive_matrix = qrcode_generator::to_matrix(url, ecc).unwrap();

    let url = Url::parse(url).unwrap();

    let optimized_matrix = qrcode_generator::to_matrix_from_segments(
        qrcode_segments_optimizer::make_segments_from_url(&url, ecc).unwrap().as_slice(),
        ecc,
    )
    .unwrap();

    assert!(optimized_matrix.len() < naive_matrix.len());
}

#[cfg(feature = "std")]
#[test]
fn optimize_email() {
    let email = "len@email.abcde.example.org";
    let ecc = QrCodeEcc::High;

    let naive_matrix = qrcode_generator::to_matrix(email, ecc).unwrap();

    let email = Email::from_str(email).unwrap();

    let optimized_matrix = qrcode_generator::to_matrix_from_segments(
        qrcode_segments_optimizer::make_segments_from_email(&email, ecc).unwrap().as_slice(),
        ecc,
    )
    .unwrap();

    assert!(optimized_matrix.len() < naive_matrix.len());
}

#[cfg(feature = "test-image")]
#[test]
fn url_to_png_to_file() {
    let url = "https://magiclen.org/path/to/12345";
    let ecc = QrCodeEcc::Low;

    qrcode_generator::to_png_to_file_from_segments(
        &qrcode_segments_optimizer::make_segments_from_url(&Url::parse(url).unwrap(), ecc).unwrap(),
        ecc,
        256,
        Path::new(FOLDER).join("url_output.png"),
    )
    .unwrap();

    assert_eq!(
        fs::read(Path::new(FOLDER).join("url.png"),).unwrap(),
        fs::read(Path::new(FOLDER).join("url_output.png"),).unwrap()
    );
}

#[cfg(all(feature = "std", feature = "test-image"))]
#[test]
fn email_to_png_to_file() {
    let email = "len@email.abcde.example.org";
    let ecc = QrCodeEcc::High;

    qrcode_generator::to_png_to_file_from_segments(
        &qrcode_segments_optimizer::make_segments_from_email(&Email::from_str(email).unwrap(), ecc)
            .unwrap(),
        ecc,
        256,
        Path::new(FOLDER).join("email_output.png"),
    )
    .unwrap();

    assert_eq!(
        fs::read(Path::new(FOLDER).join("email.png"),).unwrap(),
        fs::read(Path::new(FOLDER).join("email_output.png"),).unwrap()
    );
}
