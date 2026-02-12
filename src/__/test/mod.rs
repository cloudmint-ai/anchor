#[cfg(feature = "async")]
mod utils;
#[cfg(feature = "async")]
pub use utils::*;

#[cfg(feature = "async")]
pub use crate::{test_path as path, test_read as read};

mod init;
pub use init::*;

mod config;
pub use config::*;

pub use crate::test_config as config;

pub use macros::case;
pub use macros::cases;

mod tests;

// TODO check cloud api or desktop api or unauth api (比如 token based)
#[cfg(feature = "api")]
mod api;
#[cfg(feature = "api")]
pub use api::*;

#[cfg(feature = "async")]
mod table;
#[cfg(feature = "async")]
pub use crate::test_table as table;
#[cfg(feature = "async")]
pub use table::*;

pub use mockall::predicate::*;
