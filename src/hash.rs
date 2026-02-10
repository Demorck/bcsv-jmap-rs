use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::{JMapError, Result};

/// The hash function used by Super Mario Galaxy 1
///
/// # Arguments
/// - `field_name` - The ASCII field name to hash
///
/// # Returns
/// A 32-bit hash value
pub fn calc_hash(field_name: &str) -> u32 {
    let mut hash: u32 = 0;

    for byte in field_name.bytes() {
        let ch = if byte & 0x80 != 0 {
            byte as i8 as i32
        } else {
            byte as i32
        };

        hash = hash.wrapping_mul(31).wrapping_add(ch as u32);
    }

    hash
}

/// Trait for hash table implementations
pub trait HashTable {
    /// Calculate the hash for a field name
    ///
    /// # Arguments
    /// - `field_name` - The field name to hash
    ///
    /// # Returns
    /// The hash value corresponding to the given field name
    fn calc(&self, field_name: &str) -> u32;

    /// Find the field name for a given hash
    /// Returns a hex representation like `[DEADBEEF]` if not found
    /// # Arguments
    /// - `hash` - The hash value to look up
    ///
    /// # Returns
    /// The field name corresponding to the given hash, or a hex string if not found
    fn find(&self, hash: u32) -> String;

    /// Add a field name to the lookup table and return its hash
    ///
    /// # Arguments
    /// - `field_name` - The field name to add to the lookup table
    ///
    /// # Returns
    /// The hash value corresponding to the added field name
    fn add(&mut self, field_name: &str) -> u32;
}

/// Type of hash algorithm to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    SMG,
}

impl HashAlgorithm {
    /// Calculate hash using this algorithm
    ///
    /// # Arguments
    /// - `field_name` - The field name to hash
    ///
    /// # Returns
    /// The calculated hash value base of the hash algorithm
    pub fn calc(&self, field_name: &str) -> u32 {
        match self {
            HashAlgorithm::SMG => calc_hash(field_name),
        }
    }
}

/// A hash lookup table backed by a file of known field names
#[derive(Debug, Clone)]
pub struct FileHashTable {
    algorithm: HashAlgorithm,
    lookup: HashMap<u32, String>,
}

impl FileHashTable {
    /// Create a new empty hash table with the given algorithm
    ///
    /// # Arguments
    /// - `algorithm` - The hash algorithm to use for calculating hashes
    ///
    /// # Returns
    /// A new `FileHashTable` instance with the specified algorithm and an empty lookup table
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self {
            algorithm,
            lookup: HashMap::new(),
        }
    }

    /// Create a new hash table with the given algorithm and lookup file
    ///
    /// The lookup file should contain one field name per line
    /// Lines starting with '#' are treated as comments
    ///
    /// # Arguments
    /// - `algorithm` - The hash algorithm to use for calculating hashes
    /// - `path` - The path to the lookup file containing field names
    ///
    /// # Types
    /// - `P` - A type that can be converted to a `Path` reference, such as `&str` or `String`
    ///
    /// # Errors
    /// - If the file cannot be opened, a `JMapError::LookupFileNotFound` error is returned with the file path
    ///
    /// # Returns
    /// A `Result` containing the new `FileHashTable` instance if successful, or a `JMapError` if the file cannot be read
    pub fn from_file<P: AsRef<Path>>(algorithm: HashAlgorithm, path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|_| {
            JMapError::LookupFileNotFound(path.display().to_string())
        })?;

        let reader = BufReader::new(file);
        let mut lookup = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let hash = algorithm.calc(line);
            lookup.insert(hash, line.to_string());
        }

        Ok(Self { algorithm, lookup })
    }

    /// Get the hash algorithm used by this table
    ///
    /// # Returns
    /// The `HashAlgorithm` instance representing the hash algorithm used by this `FileHashTable`
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }
}

/// Implementation of the `HashTable` trait for `FileHashTable`
/// This allows `FileHashTable` to be used wherever a `HashTable` is expected, providing methods to calculate hashes, find field names by hash, and add new field names to the lookup table
impl HashTable for FileHashTable {
    fn calc(&self, field_name: &str) -> u32 {
        self.algorithm.calc(field_name)
    }

    fn find(&self, hash: u32) -> String {
        self.lookup
            .get(&hash)
            .cloned()
            .unwrap_or_else(|| format!("[{:08X}]", hash))
    }

    fn add(&mut self, field_name: &str) -> u32 {
        let hash = self.calc(field_name);
        self.lookup.entry(hash).or_insert_with(|| field_name.to_string());
        hash
    }
}

/// Create a hash table configured for Super Mario Galaxy 1/2
///
/// This uses the JGadget hash algorithm and loads the lookup file
/// from the default location if available
pub fn smg_hash_table() -> FileHashTable {
    FileHashTable::new(HashAlgorithm::SMG)
}

/// Create a hash table for Super Mario Galaxy with a custom lookup file
pub fn smg_hash_table_with_lookup<P: AsRef<Path>>(path: P) -> Result<FileHashTable> {
    FileHashTable::from_file(HashAlgorithm::SMG, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        /// Known hash values from Super Mario Galaxy (verified with this [hash calculator](https://mariogalaxy.org/hash))
        assert_eq!(calc_hash("ScenarioNo"), 0xED08B591);
        assert_eq!(calc_hash("ZoneName"), 0x3666C077);
    }

    #[test]
    fn test_hash_table() {
        let mut table = smg_hash_table();
        
        // Add a field and verify lookup
        let hash = table.add("TestField");
        assert_eq!(table.find(hash), "TestField");
        
        // Unknown hash should return hex representation
        let unknown = table.find(0xDEADBEEF);
        assert_eq!(unknown, "[DEADBEEF]");
    }
}
