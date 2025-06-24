//! Re-exports from either `regex` or `regex_lite`, depending on features.

#[cfg(all(feature = "regex", feature = "lite"))]
compile_error!(
    "Cannot enable both \"regex\" and \"lite\" features simultaneously. Please enable only one of them."
);

#[cfg(all(feature = "regex", not(feature = "lite")))]
pub(crate) use regex::{Captures, Regex};
#[cfg(feature = "lite")]
pub(crate) use regex_lite::{Captures, Regex};

#[cfg(not(any(feature = "regex", feature = "lite")))]
compile_error!("biblib requires the \"regex\" or \"lite\" feature to be enabled");
