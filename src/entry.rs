//! Entry (row) implementation for JMap containers

use std::collections::HashMap;

use crate::field::FieldValue;
use crate::hash::HashTable;

/// A key that can be used to access field values
#[derive(Debug, Clone)]
pub enum FieldKey {
    /// Access by hash value
    Hash(u32),
    /// Access by field name (will be hashed)
    Name(String),
}

impl From<u32> for FieldKey {
    fn from(hash: u32) -> Self {
        FieldKey::Hash(hash)
    }
}

impl From<&str> for FieldKey {
    fn from(name: &str) -> Self {
        FieldKey::Name(name.to_string())
    }
}

impl From<String> for FieldKey {
    fn from(name: String) -> Self {
        FieldKey::Name(name)
    }
}

/// An entry (row) in a JMap container
#[derive(Debug, Clone)]
pub struct Entry {
    /// Data stored as hash -> value mappings
    data: HashMap<u32, FieldValue>,
}

impl Entry {
    /// Create a new empty entry
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create an entry with pre-allocated capacity
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    /// Get a value by hash
    pub fn get_by_hash(&self, hash: u32) -> Option<&FieldValue> {
        self.data.get(&hash)
    }

    /// Get a value by name, using the provided hash table
    pub fn get<H: HashTable>(&self, hash_table: &H, name: &str) -> Option<&FieldValue> {
        let hash = hash_table.calc(name);
        self.data.get(&hash)
    }

    /// Get an integer value by hash
    pub fn get_int_by_hash(&self, hash: u32) -> Option<i32> {
        self.get_by_hash(hash).and_then(|v| v.as_int())
    }

    /// Get an integer value by name
    pub fn get_int<H: HashTable>(&self, hash_table: &H, name: &str) -> Option<i32> {
        self.get(hash_table, name).and_then(|v| v.as_int())
    }

    /// Get a float value by hash
    pub fn get_float_by_hash(&self, hash: u32) -> Option<f32> {
        self.get_by_hash(hash).and_then(|v| v.as_float())
    }

    /// Get a float value by name
    pub fn get_float<H: HashTable>(&self, hash_table: &H, name: &str) -> Option<f32> {
        self.get(hash_table, name).and_then(|v| v.as_float())
    }

    /// Get a string value by hash
    pub fn get_string_by_hash(&self, hash: u32) -> Option<&str> {
        self.get_by_hash(hash).and_then(|v| v.as_str())
    }

    /// Get a string value by name
    pub fn get_string<H: HashTable>(&self, hash_table: &H, name: &str) -> Option<&str> {
        self.get(hash_table, name).and_then(|v| v.as_str())
    }

    /// Set a value by hash
    pub fn set_by_hash(&mut self, hash: u32, value: FieldValue) {
        self.data.insert(hash, value);
    }

    /// Set a value by name, using the provided hash table
    pub fn set<H: HashTable>(&mut self, hash_table: &H, name: &str, value: FieldValue) {
        let hash = hash_table.calc(name);
        self.data.insert(hash, value);
    }

    /// Check if this entry contains a field by hash
    pub fn contains_hash(&self, hash: u32) -> bool {
        self.data.contains_key(&hash)
    }

    /// Check if this entry contains a field by name
    pub fn contains<H: HashTable>(&self, hash_table: &H, name: &str) -> bool {
        let hash = hash_table.calc(name);
        self.data.contains_key(&hash)
    }

    /// Get the number of fields in this entry
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if this entry is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Iterate over all hash-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &FieldValue)> {
        self.data.iter()
    }

    /// Get mutable access to the internal data map
    pub(crate) fn data_mut(&mut self) -> &mut HashMap<u32, FieldValue> {
        &mut self.data
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self::new()
    }
}
