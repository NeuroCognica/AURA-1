#![allow(dead_code)]

#[cfg(feature = "persistence")]
pub use crate::storage::RocksStore as RocksDBStore;

#[cfg(not(feature = "persistence"))]
compile_error!("feature \"persistence\" required for RocksDBStore");
