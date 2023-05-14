//! A storage backend

pub use fallback::{ColumnFamily, ColumnFamilyDefinition, Db, Iter, Reader, Transaction};

mod fallback;
