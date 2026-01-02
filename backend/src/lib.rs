#![allow(dead_code)]

pub mod storage;
pub mod intent_stratification;

#[cfg(feature = "persistence")]
pub mod rocksdb_store;

#[cfg(feature = "persistence")]
pub mod accounts;

#[cfg(feature = "persistence")]
pub mod profile;

#[cfg(feature = "persistence")]
pub mod quiz;

#[cfg(feature = "persistence")]
pub mod quiz_api;

#[cfg(feature = "persistence")]
pub mod quiz_parser;

#[cfg(feature = "persistence")]
pub mod quiz_loader;

#[cfg(feature = "persistence")]
pub mod quiz_tagging;

#[cfg(feature = "persistence")]
pub mod profile_synthesis;

#[cfg(feature = "search")]
pub mod search;

// Expose a few internal modules for integration tests and tooling.
pub mod broadcast;
pub mod council_verdict;
pub mod generation_manager;
pub mod archetype_api;
