use core::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use validators::prelude::*;
use validators_prelude::{EmailError, Host};

#[derive(Debug, Clone, Validator)]
#[validator(email)]
pub struct Email {
    pub(crate) local_part:                 String,
    pub(crate) need_quoted:                bool,
    pub(crate) domain_part:                Host,
    pub(crate) comment_before_local_part:  Option<String>,
    pub(crate) comment_after_local_part:   Option<String>,
    pub(crate) comment_before_domain_part: Option<String>,
    pub(crate) comment_after_domain_part:  Option<String>,
}

impl FromStr for Email {
    type Err = EmailError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Email::parse_str(s)
    }
}

impl Display for Email {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.to_email_string())
    }
}
