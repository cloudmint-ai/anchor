#[cfg(feature = "runtime")]
mod utils;
#[cfg(feature = "runtime")]
pub use utils::*;

#[cfg(feature = "runtime")]
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

#[cfg(feature = "runtime")]
mod table;
#[cfg(feature = "runtime")]
pub use crate::test_table as table;
#[cfg(feature = "runtime")]
pub use table::*;

pub use mockall::predicate::*;
