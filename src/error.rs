use std::io;

use bip0039::Error as MnemonicError;
use thiserror::Error;

/// Result alias for parsing operations inside `zewif-zingo`.
pub type Result<T> = std::result::Result<T, ParseError>;

/// Semantic error type describing the ways parsing a Zingo wallet can fail.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unsupported wallet version {found} (max supported {max})")]
    UnsupportedWalletVersion { found: u64, max: u64 },

    #[error("invalid boolean value {value} while reading {label}")]
    InvalidBoolean { label: &'static str, value: u8 },

    #[error("length {length} for {label} does not fit into usize")]
    LengthOverflow { label: &'static str, length: u64 },

    #[error("failed to decode UTF-8 string for {label}")]
    InvalidString {
        label: &'static str,
        #[source]
        source: std::string::FromUtf8Error,
    },

    #[error("failed to construct mnemonic from entropy bytes")]
    MnemonicEntropy {
        #[source]
        source: MnemonicError,
    },

    #[error("failed to read {label}")]
    Read {
        label: &'static str,
        #[source]
        source: io::Error,
    },

    #[error("expected {expected} to be consumed but {remaining} bytes remain")]
    RemainingBytes {
        expected: &'static str,
        remaining: usize,
    },
}

impl ParseError {
    pub(crate) fn read(label: &'static str, source: io::Error) -> Self {
        Self::Read { label, source }
    }

    pub(crate) fn length_overflow(label: &'static str, length: u64) -> Self {
        Self::LengthOverflow { label, length }
    }

    pub(crate) fn invalid_boolean(label: &'static str, value: u8) -> Self {
        Self::InvalidBoolean { label, value }
    }
}

impl From<MnemonicError> for ParseError {
    fn from(source: MnemonicError) -> Self {
        Self::MnemonicEntropy { source }
    }
}
