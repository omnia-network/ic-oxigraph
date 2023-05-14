//! A storage backend

#[cfg(target_family = "wasm")]
pub use fallback::{ColumnFamily, ColumnFamilyDefinition, Db, Iter, Reader, Transaction};

#[cfg(target_family = "wasm")]
mod fallback;
