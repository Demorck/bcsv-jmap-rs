use indexmap::IndexMap;

use crate::entry::Entry;
use crate::error::{JMapError, Result};
use crate::field::{Field, FieldType, FieldValue};
use crate::hash::HashTable;

/// The main JMap that holds fields and entries. So basically the in-memory representation of a BCSV file
///
/// This is a table-like structure where each field represents a column
/// and each entry represents a row of data
///
/// Basically implemented what i see on [this page](https://www.lumasworkshop.com/wiki/BCSV_(File_format))
#[derive(Debug)]
pub struct JMapInfo<H: HashTable> {
    /// The hash table used for field name lookups
    hash_table: H,
    /// Fields indexed by their hash
    fields: IndexMap<u32, Field>,
    /// List of entries
    entries: Vec<Entry>,
    /// Size of a single entry in bytes
    pub(crate) entry_size: u32,
}

impl<H: HashTable> JMapInfo<H> {
    /// Create a new empty JMapInfo with the given hash table
    ///
    /// # Arguments
    /// - `hash_table` - The hash table to use for field name lookups
    pub fn new(hash_table: H) -> Self {
        Self {
            hash_table,
            fields: IndexMap::new(),
            entries: Vec::new(),
            entry_size: 0,
        }
    }

    /// Get a reference to the hash table
    pub fn hash_table(&self) -> &H {
        &self.hash_table
    }

    /// Get a mutable reference to the hash table
    pub fn hash_table_mut(&mut self) -> &mut H {
        &mut self.hash_table
    }

    /// Get the number of fields (columns)
    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    /// Get the number of entries (rows)
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if entries are empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get an iterator over all fields
    pub fn fields(&self) -> impl Iterator<Item = &Field> {
        self.fields.values()
    }

    /// Get an iterator over all field hashes
    pub fn field_hashes(&self) -> impl Iterator<Item = &u32> {
        self.fields.keys()
    }

    /// Get a field by hash
    pub fn get_field_by_hash(&self, hash: u32) -> Option<&Field> {
        self.fields.get(&hash)
    }

    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        let hash = self.hash_table.calc(name);
        self.fields.get(&hash)
    }

    /// Check if a field exists by hash
    pub fn contains_field_hash(&self, hash: u32) -> bool {
        self.fields.contains_key(&hash)
    }

    /// Check if a field exists by name
    pub fn contains_field(&self, name: &str) -> bool {
        let hash = self.hash_table.calc(name);
        self.fields.contains_key(&hash)
    }

    /// Get the name of a field by its hash
    pub fn field_name(&self, hash: u32) -> String {
        self.hash_table.find(hash)
    }

    /// Create a new field with the given name and type
    ///
    /// # Arguments
    /// - `name` - The name of the field to create
    /// - `field_type` - The type of the field to create
    /// - `default` - The default value for the field to create
    ///
    /// # Errors
    /// - `JMapError::TypeMismatch` if the default value is not compatible with the field type
    /// - `JMapError::FieldAlreadyExists` if a field with the same name already exists
    ///
    /// # Returns
    /// Ok(()) if the field was created successfully, or an error if the field could not be created
    pub fn create_field(
        &mut self,
        name: &str,
        field_type: FieldType,
        default: FieldValue,
    ) -> Result<()> {
        if !default.is_compatible_with(field_type) {
            return Err(JMapError::TypeMismatch {
                expected: field_type.csv_name(),
                got: default.type_name(),
            });
        }

        let hash = self.hash_table.add(name);

        if self.fields.contains_key(&hash) {
            return Err(JMapError::FieldAlreadyExists(name.to_string()));
        }

        let field = Field::with_default(hash, field_type, default.clone());
        self.fields.insert(hash, field);

        // Add default value to all existing entries
        for entry in &mut self.entries {
            entry.set_by_hash(hash, default.clone());
        }

        Ok(())
    }

    /// Remove a field from the container
    ///
    /// # Arguments
    /// - `name` - The name of the field to remove
    ///
    /// # Errors
    /// - `JMapError::FieldNotFound` if the field does not exist
    ///
    /// # Returns
    /// Ok(()) if the field was removed successfully, or an error if the field could not be found
    pub fn drop_field(&mut self, name: &str) -> Result<()> {
        let hash = self.hash_table.calc(name);

        if !self.fields.contains_key(&hash) {
            return Err(JMapError::FieldNotFound(name.to_string()));
        }

        self.fields.swap_remove(&hash);

        for entry in &mut self.entries {
            entry.data_mut().remove(&hash);
        }

        Ok(())
    }

    /// Get a slice of all entries
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Get a mutable slice of all entries
    pub fn entries_mut(&mut self) -> &mut [Entry] {
        &mut self.entries
    }

    /// Get an entry by index
    pub fn get_entry(&self, index: usize) -> Option<&Entry> {
        self.entries.get(index)
    }

    /// Get a mutable entry by index
    pub fn get_entry_mut(&mut self, index: usize) -> Option<&mut Entry> {
        self.entries.get_mut(index)
    }

    /// Create a new entry with default values for all fields
    pub fn create_entry(&mut self) -> &mut Entry {
        let mut entry = Entry::with_capacity(self.fields.len());

        for field in self.fields.values() {
            entry.set_by_hash(field.hash, field.default.clone());
        }

        self.entries.push(entry);
        self.entries.last_mut().unwrap()
    }

    /// Remove an entry by index
    ///
    /// # Arguments
    /// - `index` - The index of the entry to remove
    ///
    /// # Errors
    /// - `JMapError::EntryIndexOutOfBounds` if the index is out of bounds
    ///
    /// # Returns
    /// Ok(Entry) if the entry was removed successfully, or an error if the index was out of bounds
    pub fn remove_entry(&mut self, index: usize) -> Result<Entry> {
        if index >= self.entries.len() {
            return Err(JMapError::EntryIndexOutOfBounds {
                index,
                len: self.entries.len(),
            });
        }

        Ok(self.entries.remove(index))
    }

    /// Clear all entries but keep the field definitions
    pub fn clear_entries(&mut self) {
        self.entries.clear();
    }

    /// Sort entries by a custom key function
    ///
    /// # Arguments
    /// - `f` - The key function to sort by
    ///
    /// # Types
    /// - `F` - The type of the key function, which must be a function that takes a reference to an `Entry` and returns a key of type `K`
    /// - `K` - The type of the key returned by the key function, which must implement the `Ord` trait for sorting
    pub fn sort_entries_by<F, K>(&mut self, f: F)
    where
        F: FnMut(&Entry) -> K,
        K: Ord,
    {
        self.entries.sort_by_key(f);
    }

    /// Iterate over entries
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }

    /// Iterate over entries mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entry> {
        self.entries.iter_mut()
    }

    /// Get internal access to fields (for I/O operations)
    pub(crate) fn fields_map(&self) -> &IndexMap<u32, Field> {
        &self.fields
    }

    /// Get mutable internal access to fields (for I/O operations)
    pub(crate) fn fields_map_mut(&mut self) -> &mut IndexMap<u32, Field> {
        &mut self.fields
    }

    /// Get mutable access to entries (for I/O operations)
    pub(crate) fn entries_vec_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.entries
    }

    /// Recalculate field offsets and entry size based on field types.
    pub fn recalculate_offsets(&mut self) {
        let mut fields_with_hashes: Vec<(u32, Field)> = self
            .fields
            .iter()
            .map(|(h, f)| (*h, f.clone()))
            .collect();

        fields_with_hashes.sort_by_key(|(_, f)| f.field_type.order());

        let mut current_offset: u16 = 0;
        for (hash, field) in &mut fields_with_hashes {
            field.offset = current_offset;
            current_offset += field.field_type.size() as u16;

            if let Some(f) = self.fields.get_mut(hash) {
                f.offset = field.offset;
            }
        }

        self.entry_size = ((current_offset as u32 + 3) & !3) as u32;
    }
}

/// Implement IntoIterator for JMapInfo to allow iterating over entries directly
/// This allows using `for entry in jmap` syntax to iterate over entries, as well as iterating over references and mutable references to JMapInfo
/// The item type is `Entry` for owned iteration, `&Entry` for reference iteration, and `&mut Entry` for mutable reference iteration
impl<H: HashTable> IntoIterator for JMapInfo<H> {
    type Item = Entry;
    type IntoIter = std::vec::IntoIter<Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

/// Implement IntoIterator for references to JMapInfo to allow iterating over entries by reference
/// # Types
/// - `H` - The type of the hash table used by the JMapInfo, which must implement the `HashTable` trait
///
/// # Lifetime
/// - `'a` - The lifetime of the reference to the JMapInfo
impl<'a, H: HashTable> IntoIterator for &'a JMapInfo<H> {
    type Item = &'a Entry;
    type IntoIter = std::slice::Iter<'a, Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<'a, H: HashTable> IntoIterator for &'a mut JMapInfo<H> {
    type Item = &'a mut Entry;
    type IntoIter = std::slice::IterMut<'a, Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter_mut()
    }
}
