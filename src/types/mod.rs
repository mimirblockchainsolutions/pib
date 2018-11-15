//! Commonly used types.
mod include;
mod enode;
mod error;
mod files;

pub use self::include::Include;
pub use self::enode::{EnodeAddr,ParseEnodeError};
pub use self::error::Error;
pub use self::files::{
    ProjectFiles,
    SetupFiles,
    BuildFiles,
    NodeFiles,
    Files
};

use std::collections::HashSet;

/// Collection of tags associated with an entity
pub type Tags = HashSet<String>;

