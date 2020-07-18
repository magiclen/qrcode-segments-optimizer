#[cfg(feature = "std")]
mod email;

#[cfg(feature = "std")]
pub use email::*;

#[cfg(feature = "std")]
pub use crate::validators::EmailError;
