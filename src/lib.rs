use zewif::mod_use;

mod binary;
mod error;

pub use error::{ParseError, Result};

mod_use!(wallet_capability);
mod_use!(zingo_parser);
mod_use!(zingo_wallet);
