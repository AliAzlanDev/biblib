//! Re-exports from either `regex` or `regex_lite`, depending on features.

#[cfg(feature = "lite")]
pub(crate) use regex_lite::{Regex, Captures};
#[cfg(all(feature = "regex", not(feature = "lite")))]
pub(crate) use regex::{Regex, Captures};

#[cfg(not(any(feature = "regex", feature = "lite")))]
compile_error!("biblib requires the \"regex\" or \"lite\" feature to be enabled");
