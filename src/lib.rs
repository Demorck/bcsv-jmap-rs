//! # jmap
//!
//! A Rust library for reading and writing Nintendo's BCSV/JMap format
//! This format is used by Super Mario Galaxy and some other games
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use lib_bcsv_jmap::{from_file, to_csv, smg_hash_table_with_lookup, IoOptions};
//!
//! // Create a hash table with known field names
//! let hash_table = smg_hash_table_with_lookup("hashtable_smg.txt").unwrap();
//!
//! // Read a BCSV file
//! let jmap = from_file(hash_table, "scenariodata.bcsv", &IoOptions::default()).unwrap();
//!
//! // Print some data
//! println!("Entries: {}", jmap.len());
//! for entry in jmap.entries() {
//!     if let Some(name) = entry.get_string(jmap.hash_table(), "ZoneName") {
//!         println!("Zone: {}", name);
//!     }
//! }
//!
//! // Export to CSV
//! to_csv(&jmap, "output.csv", None).unwrap();
//! ```
//!
//! ## Features
//!
//! - Read and write BCSV files with full format support
//! - big-endian and little-endian support
//! - Shift-JIS and UTF-8 string encoding
//! - CSV import/export

pub mod csv;
pub mod entry;
pub mod error;
pub mod field;
pub mod hash;
pub mod io;
pub mod jmap;


pub use crate::csv::{from_csv, to_csv};
pub use crate::entry::{Entry, FieldKey};
pub use crate::error::{JMapError, Result};
pub use crate::field::{Field, FieldType, FieldValue};
pub use crate::hash::{
    calc_hash, FileHashTable, HashAlgorithm, HashTable,
    smg_hash_table, smg_hash_table_with_lookup,
};
pub use crate::io::{from_buffer, from_file, to_buffer, to_file, Encoding, IoOptions};
pub use crate::jmap::JMapInfo;
