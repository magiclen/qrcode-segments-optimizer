/*!
# QR Code Segments Optimizer

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
*/

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

extern crate qrcode_generator;

#[cfg(feature = "std")]
#[macro_use]
extern crate validators_derive;
#[cfg(feature = "std")]
extern crate validators;

extern crate cow_utils;
extern crate url;

#[cfg(feature = "kanji")]
#[macro_use]
extern crate lazy_static_include;

pub mod models;
mod qr_segment_advanced;

use core::str::from_utf8_unchecked;

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

use qrcode_generator::qrcodegen::{QrCode_MAX_VERSION, QrCode_MIN_VERSION};
use qrcode_generator::{QRCodeError, QrCodeEcc, QrSegment};

use cow_utils::CowUtils;

use url::Url;

#[cfg(feature = "std")]
use validators::models::Host;

#[cfg(feature = "std")]
use models::Email;

/// Make segments from a string slice optimally.
#[inline]
pub fn make_segments_from_str<S: AsRef<str>>(
    text: S,
    ecc: QrCodeEcc,
) -> Result<Vec<QrSegment>, QRCodeError> {
    let chars: Vec<char> = text.as_ref().chars().collect();

    qr_segment_advanced::make_segments_optimally(
        chars.as_slice(),
        ecc,
        QrCode_MIN_VERSION,
        QrCode_MAX_VERSION,
    )
    .ok_or(QRCodeError::DataTooLong)
}

/// Make segments from a url optimally.
#[inline]
pub fn make_segments_from_url(url: &Url, ecc: QrCodeEcc) -> Result<Vec<QrSegment>, QRCodeError> {
    let url_str = url.as_str();

    let mut qrcode_url: Cow<str> = Cow::Borrowed(url_str);

    if let Cow::Owned(scheme) = url.scheme().cow_to_ascii_uppercase() {
        if !url.has_authority() {
            // non-hierarchical
            let mut s = String::with_capacity(url_str.len());

            s.push_str(scheme.as_str());
            s.push_str(unsafe { from_utf8_unchecked(&url_str.as_bytes()[scheme.len()..]) });

            return make_segments_from_str(s.as_str(), ecc);
        }

        let mut s = String::with_capacity(url_str.len());

        s.push_str(scheme.as_str());
        s.push_str("://");

        // userinfo
        if !url.username().is_empty() {
            s.push_str(url.username());

            if let Some(password) = url.password() {
                s.push(':');
                s.push_str(password);
            }

            s.push('@');
        }

        qrcode_url = Cow::Owned(s);
    }

    let host_done = if let Some(domain) = url.domain() {
        if let Cow::Owned(domain) = domain.cow_to_ascii_uppercase() {
            if let Cow::Borrowed(_) = &qrcode_url {
                let mut s = String::with_capacity(url_str.len());

                s.push_str(url.scheme());
                s.push_str("://");

                // userinfo
                if !url.username().is_empty() {
                    s.push_str(url.username());

                    if let Some(password) = url.password() {
                        s.push(':');
                        s.push_str(password);
                    }

                    s.push('@');
                }

                qrcode_url = Cow::Owned(s);
            }

            if let Cow::Owned(s) = &mut qrcode_url {
                s.push_str(domain.as_str());
            }

            true
        } else {
            false
        }
    } else {
        false
    };

    match qrcode_url {
        Cow::Borrowed(qrcode_url) => {
            // nothing change
            make_segments_from_str(qrcode_url, ecc)
        }
        Cow::Owned(mut s) => {
            if !host_done {
                s.push_str(url.host_str().unwrap());
            }

            s.push_str(url.path());

            if let Some(query) = url.query() {
                s.push('?');
                s.push_str(query);
            }

            if let Some(fragment) = url.fragment() {
                s.push('#');
                s.push_str(fragment);
            }

            make_segments_from_str(s.as_str(), ecc)
        }
    }
}

#[cfg(feature = "std")]
/// Make segments from an email address optimally.
#[inline]
pub fn make_segments_from_email(
    email: &Email,
    ecc: QrCodeEcc,
) -> Result<Vec<QrSegment>, QRCodeError> {
    if let Host::Domain(domain) = &email.domain_part {
        if let Cow::Owned(domain) = domain.cow_to_ascii_uppercase() {
            let mut s = String::with_capacity(email.local_part.len() + domain.len() + 32);

            if let Some(comment) = &email.comment_before_local_part {
                s.push('(');
                s.push_str(comment);
                s.push(')');
            }

            if email.need_quoted {
                s.push('"');
            }

            s.push_str(email.local_part.as_str());

            if email.need_quoted {
                s.push('"');
            }

            if let Some(comment) = &email.comment_after_local_part {
                s.push('(');
                s.push_str(comment);
                s.push(')');
            }

            s.push('@');

            if let Some(comment) = &email.comment_before_domain_part {
                s.push('(');
                s.push_str(comment);
                s.push(')');
            }

            s.push_str(domain.as_str());

            if let Some(comment) = &email.comment_after_domain_part {
                s.push('(');
                s.push_str(comment);
                s.push(')');
            }

            return make_segments_from_str(s.as_str(), ecc);
        }
    }

    make_segments_from_str(&email.to_email_string(), ecc)
}
