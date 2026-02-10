use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::entry::Entry;
use crate::error::{JMapError, Result};
use crate::field::{Field, FieldType, FieldValue};
use crate::hash::HashTable;
use crate::jmap::JMapInfo;

/// Options for reading/writing BCSV files
#[derive(Debug, Clone)]
pub struct IoOptions {
    /// Whether data is big-endian or little-endian
    pub big_endian: bool,
    /// String encoding: "shift_jis" (for japanese language) or "utf-8"
    pub encoding: Encoding,
}

/// String encoding options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Shift-JIS encoding
    ShiftJis,
    /// UTF-8 encoding
    Utf8,
}

impl Default for IoOptions {
    fn default() -> Self {
        Self {
            big_endian: true,
            encoding: Encoding::ShiftJis,
        }
    }
}

impl IoOptions {
    /// Options for Super Mario Galaxy (Wii)
    pub fn super_mario_galaxy() -> Self {
        Self {
            big_endian: true,
            encoding: Encoding::ShiftJis,
        }
    }
}

/// Read a JMapInfo from a byte buffer
///
/// # Arguments
/// - `hash_table` - The hash table to use for field name lookups
/// - `data` - The byte buffer containing the BCSV data
/// - `options` - Options for endianness and string encoding
///
/// # TYpe
/// - `H` - The type of hash table to use, which must implement the `HashTable` trait
///
/// # Returns
/// A `JMapInfo` instance populated with the data from the buffer, or an error if parsing fails
pub fn from_buffer<H: HashTable>(
    hash_table: H,
    data: &[u8],
    options: &IoOptions,
) -> Result<JMapInfo<H>> {
    let mut jmap = JMapInfo::new(hash_table);

    // The header size is 16 bytes, so we need at least that much to read the header
    if data.len() < 0x10 {
        return Err(JMapError::BufferTooSmall {
            expected: 0x10,
            got: data.len(),
        });
    }

    // Read header
    let (num_entries, num_fields, off_data, entry_size) = if options.big_endian {
        (
            BigEndian::read_u32(&data[0x00..0x04]),
            BigEndian::read_u32(&data[0x04..0x08]),
            BigEndian::read_u32(&data[0x08..0x0C]),
            BigEndian::read_u32(&data[0x0C..0x10]),
        )
    } else {
        (
            LittleEndian::read_u32(&data[0x00..0x04]),
            LittleEndian::read_u32(&data[0x04..0x08]),
            LittleEndian::read_u32(&data[0x08..0x0C]),
            LittleEndian::read_u32(&data[0x0C..0x10]),
        )
    };

    jmap.entry_size = entry_size;

    // Calculate string table offset
    // string table starts immediately after the entries, which start at off_data and each entry is entry_size bytes
    // So the string table is at off_data + (num_entries * entry_size)
    let off_strings = off_data as usize + (num_entries as usize * entry_size as usize);

    // Read fields (each field is 0xC bytes)
    let mut off = 0x10_usize;
    for _ in 0..num_fields {
        let field = read_field(data, off, options.big_endian)?;
        jmap.fields_map_mut().insert(field.hash, field);
        off += 0x0C;
    }

    // Read entries
    off = off_data as usize;
    for _ in 0..num_entries {
        let entry = read_entry(data, off, off_strings, &jmap, options)?;
        jmap.entries_vec_mut().push(entry);
        off += entry_size as usize;
    }

    Ok(jmap)
}

/// Read a JMapInfo from a file
///
/// # Arguments
/// - `hash_table` - The hash table to use for field name lookups
/// - `path` - The path to the BCSV file to read
/// - `options` - Options for endianness and string encoding
///
/// # Type
/// - `H` - The type of hash table to use, which must implement the `HashTable` trait
/// - `P` - A type that can be converted to a `Path` reference, such as `&str` or `String`
///
/// # Returns
/// A `JMapInfo` instance populated with the data from the file, or an error if the file cannot be read or parsed
pub fn from_file<H: HashTable, P: AsRef<Path>>(
    hash_table: H,
    path: P,
    options: &IoOptions,
) -> Result<JMapInfo<H>> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    from_buffer(hash_table, &data, options)
}

/// Write a JMapInfo to a byte buffer
/// This function serializes the `JMapInfo` into the BCSV format, including the header, field definitions, entries, and string table
///
/// # Arguments
/// - `jmap` - The `JMapInfo` instance to serialize
/// - `options` - Options for endianness and string encoding
///
/// # Type
/// - `H` - The type of hash table used by the `JMapInfo`, which must implement the `HashTable` trait
///
/// # Returns
/// A `Result` containing the serialized byte buffer if successful, or an error if serialization fails
///
/// TODO: This function is pretty complex and could use some refactoring to break it down into smaller functions
pub fn to_buffer<H: HashTable>(jmap: &JMapInfo<H>, options: &IoOptions) -> Result<Vec<u8>> {
    let num_entries = jmap.len() as u32;
    let num_fields = jmap.num_fields() as u32;
    let off_data = 0x10 + num_fields * 0x0C; // Header (16 bytes) + field definitions (12 bytes each)

    // Calculate entry size and field offsets
    let mut fields_with_offsets: Vec<(u32, Field)> = jmap
        .fields()
        .map(|f| (f.hash, f.clone()))
        .collect();

    // Sort by type order and assign offsets
    fields_with_offsets.sort_by_key(|(_, f)| f.field_type.order());

    let mut current_offset: u16 = 0;
    for (_, field) in &mut fields_with_offsets {
        field.offset = current_offset;
        current_offset += field.field_type.size() as u16;
    }

    // Align entry size to 4 bytes
    let entry_size = ((current_offset as u32 + 3) & !3) as u32;

    // Create buffer
    let mut buffer = vec![0u8; (off_data + num_entries * entry_size) as usize];

    // Write header
    if options.big_endian {
        BigEndian::write_u32(&mut buffer[0x00..0x04], num_entries);
        BigEndian::write_u32(&mut buffer[0x04..0x08], num_fields);
        BigEndian::write_u32(&mut buffer[0x08..0x0C], off_data);
        BigEndian::write_u32(&mut buffer[0x0C..0x10], entry_size);
    } else {
        LittleEndian::write_u32(&mut buffer[0x00..0x04], num_entries);
        LittleEndian::write_u32(&mut buffer[0x04..0x08], num_fields);
        LittleEndian::write_u32(&mut buffer[0x08..0x0C], off_data);
        LittleEndian::write_u32(&mut buffer[0x0C..0x10], entry_size);
    }

    // Build a map of hash -> offset for quick lookup
    let field_offsets: std::collections::HashMap<u32, &Field> = fields_with_offsets
        .iter()
        .map(|(hash, field)| (*hash, field))
        .collect();

    // Write fields
    let mut off = 0x10_usize;
    for (hash, field) in &fields_with_offsets {
        write_field(&mut buffer, off, *hash, field, options.big_endian);
        off += 12;
    }

    // Prepare string table for StringOffset fields
    let mut string_table: Vec<u8> = Vec::new();
    let mut string_offsets: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    // Write entries
    off = off_data as usize;
    for entry in jmap.entries() {
        write_entry(
            &mut buffer,
            off,
            entry,
            &field_offsets,
            &mut string_table,
            &mut string_offsets,
            options,
        )?;
        off += entry_size as usize;
    }

    // Append string table
    buffer.extend_from_slice(&string_table);

    // Align to 32 bytes with 0x40 padding
    let len = buffer.len();
    let aligned_len = (len + 31) & !31;
    buffer.resize(aligned_len, 0x40);

    Ok(buffer)
}

/// Write a JMapInfo to a file
///
/// # Arguments
/// - `jmap` - The `JMapInfo` instance to write to the file
/// - `path` - The path to the file where the BCSV data should be written
/// - `options` - Options for endianness and string encoding
///
/// # Type
/// - `H` - The type of hash table used by the `JMapInfo`, which must implement the `HashTable` trait
/// - `P` - A type that can be converted to a `Path` reference, such as `&str` or `String`
///
/// # Returns
/// Ok(()) if the file was successfully written, or an error if the file cannot be created or written to
pub fn to_file<H: HashTable, P: AsRef<Path>>(
    jmap: &JMapInfo<H>,
    path: P,
    options: &IoOptions,
) -> Result<()> {
    let buffer = to_buffer(jmap, options)?;
    let mut file = File::create(path)?;
    file.write_all(&buffer)?;
    file.flush()?;
    Ok(())
}

// Helper functions

/// Read a field definition from the buffer at the given offset
///
/// # Arguments
/// - `data` - The byte buffer containing the field definitions
/// - `offset` - The offset in the buffer where the field definition starts
/// - `big_endian` - Whether the data is big-endian or little-endian
///
/// # Errors
/// - `JMapError::InvalidFieldType` if the field type byte is not a valid `FieldType`
///
/// # Returns
/// A `Field` instance representing the field definition, or an error if the field type is invalid
fn read_field(data: &[u8], offset: usize, big_endian: bool) -> Result<Field> {
    let (hash, mask, field_offset, shift, raw_type) = if big_endian {
        (
            BigEndian::read_u32(&data[offset..offset + 0x04]),
            BigEndian::read_u32(&data[offset + 0x04..offset + 0x08]),
            BigEndian::read_u16(&data[offset + 0x08..offset + 0x0A]),
            data[offset + 0x0A],
            data[offset + 0x0B],
        )
    } else {
        (
            LittleEndian::read_u32(&data[offset..offset + 0x04]),
            LittleEndian::read_u32(&data[offset + 0x04..offset + 0x08]),
            LittleEndian::read_u16(&data[offset + 0x08..offset + 0x0A]),
            data[offset + 0x0A],
            data[offset + 0x0B],
        )
    };

    let field_type = FieldType::from_raw(raw_type)
        .ok_or(JMapError::InvalidFieldType(raw_type))?;

    Ok(Field {
        hash,
        field_type,
        mask,
        shift,
        offset: field_offset,
        default: FieldValue::default_for(field_type),
    })
}

/// Write a field definition to the buffer at the given offset
///
/// # Arguments
/// - `buffer` - The byte buffer where the field definition should be written
/// - `offset` - The offset in the buffer where the field definition should start
/// - `hash` - The hash of the field name
/// - `field` - The `Field` instance containing the field definition to write
/// - `big_endian` - Whether the data should be written in big-endian or little-endian format
fn write_field(buffer: &mut [u8], offset: usize, hash: u32, field: &Field, big_endian: bool) {
    if big_endian {
        BigEndian::write_u32(&mut buffer[offset..offset + 0x04], hash);
        BigEndian::write_u32(&mut buffer[offset + 0x04..offset + 0x08], field.mask);
        BigEndian::write_u16(&mut buffer[offset + 0x08..offset + 0x0A], field.offset);
    } else {
        LittleEndian::write_u32(&mut buffer[offset..offset + 0x04], hash);
        LittleEndian::write_u32(&mut buffer[offset + 0x04..offset + 0x08], field.mask);
        LittleEndian::write_u16(&mut buffer[offset + 0x08..offset + 0x0A], field.offset);
    }
    buffer[offset + 0x0A] = field.shift;
    buffer[offset + 0x0B] = field.field_type as u8;
}

/// Read an entry from the buffer at the given offset, using the field definitions from the JMapInfo
///
/// # Arguments
/// - `data` - The byte buffer containing the entry data
/// - `entry_offset` - The offset in the buffer where the entry starts
/// - `string_table_offset` - The offset in the buffer where the string table starts (for StringOffset fields)
/// - `jmap` - The `JMapInfo` instance containing the field definitions to use for parsing the entry
/// - `options` - Options for endianness and string encoding
///
/// # Returns
/// An `Entry` instance representing the parsed entry, or an error if parsing fails
fn read_entry<H: HashTable>(
    data: &[u8],
    entry_offset: usize,
    string_table_offset: usize,
    jmap: &JMapInfo<H>,
    options: &IoOptions,
) -> Result<Entry> {
    let mut entry = Entry::with_capacity(jmap.num_fields());

    for field in jmap.fields() {
        let val_offset = entry_offset + field.offset as usize;
        let value = read_field_value(data, val_offset, string_table_offset, field, options)?;
        entry.set_by_hash(field.hash, value);
    }

    Ok(entry)
}

/// Read a field value from the buffer at the given offset, applying the field's mask and shift, and using the string table for StringOffset fields
///
/// # Arguments
/// - `data` - The byte buffer containing the field value data
/// - `offset` - The offset in the buffer where the field value starts
/// - `string_table_offset` - The offset in the buffer where the string table starts (for StringOffset fields)
/// - `field` - The `Field` instance containing the field definition to use for parsing the value
/// - `options` - Options for endianness and string encoding
///
/// # Returns
/// A `FieldValue` instance representing the parsed field value, or an error if parsing fails
///
/// TODO: This function is quite big and could be refactored by implementation of a trait for reading/writing field values based on the field type, to reduce the amount of code
fn read_field_value(
    data: &[u8],
    offset: usize,
    string_table_offset: usize,
    field: &Field,
    options: &IoOptions,
) -> Result<FieldValue> {
    let value = match field.field_type {
        FieldType::Long | FieldType::UnsignedLong => {
            let raw = if options.big_endian {
                BigEndian::read_u32(&data[offset..offset + 4])
            } else {
                LittleEndian::read_u32(&data[offset..offset + 4])
            };
            let masked = (raw & field.mask) >> field.shift;
            // Sign extend for signed types
            let signed = if masked & 0x80000000 != 0 {
                masked as i32
            } else {
                masked as i32
            };
            FieldValue::Int(signed)
        }

        FieldType::Float => {
            let val = if options.big_endian {
                BigEndian::read_f32(&data[offset..offset + 4])
            } else {
                LittleEndian::read_f32(&data[offset..offset + 4])
            };
            FieldValue::Float(val)
        }

        FieldType::Short => {
            let raw = if options.big_endian {
                BigEndian::read_u16(&data[offset..offset + 2])
            } else {
                LittleEndian::read_u16(&data[offset..offset + 2])
            };
            let masked = ((raw as u32) & field.mask) >> field.shift;
            let signed = if masked & 0x8000 != 0 {
                (masked | 0xFFFF0000) as i32
            } else {
                masked as i32
            };
            FieldValue::Int(signed)
        }

        FieldType::Char => {
            let raw = data[offset];
            let masked = ((raw as u32) & field.mask) >> field.shift;
            let signed = if masked & 0x80 != 0 {
                (masked | 0xFFFFFF00) as i32
            } else {
                masked as i32
            };
            FieldValue::Int(signed)
        }

        FieldType::String => {
            // Read up to 32 bytes until null terminator
            let end = data[offset..offset + 32]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(32);
            let bytes = &data[offset..offset + end];
            let s = decode_string(bytes, options.encoding)?;
            FieldValue::String(s)
        }

        FieldType::StringOffset => {
            let str_offset = if options.big_endian {
                BigEndian::read_u32(&data[offset..offset + 4])
            } else {
                LittleEndian::read_u32(&data[offset..offset + 4])
            };
            let str_start = string_table_offset + str_offset as usize;
            let end = data[str_start..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(0);
            let bytes = &data[str_start..str_start + end];
            let s = decode_string(bytes, options.encoding)?;
            FieldValue::String(s)
        }
    };

    Ok(value)
}

/// Write an entry to the buffer at the given offset, using the field definitions from the JMapInfo, and updating the string table for StringOffset fields
///
/// # Arguments
/// - `buffer` - The byte buffer where the entry should be written
/// - `entry_offset` - The offset in the buffer where the entry should start
/// - `entry` - The `Entry` instance containing the field values to write
/// - `field_offsets` - A map of field hash to `Field` instance, used for looking up field definitions when writing values
/// - `string_table` - A mutable byte vector representing the string table, which will be updated with new strings for StringOffset fields
/// - `string_offsets` - A mutable map of string to offset in the string table, used for reusing existing strings and avoiding duplicates in the string table
/// - `options` - Options for endianness and string encoding
///
/// # Returns
/// Ok(()) if the entry was successfully written to the buffer, or an error if writing fails (e.g. due to type mismatch or encoding errors)
fn write_entry(
    buffer: &mut [u8],
    entry_offset: usize,
    entry: &Entry,
    field_offsets: &std::collections::HashMap<u32, &Field>,
    string_table: &mut Vec<u8>,
    string_offsets: &mut std::collections::HashMap<String, u32>,
    options: &IoOptions,
) -> Result<()> {
    for (hash, value) in entry.iter() {
        if let Some(field) = field_offsets.get(hash) {
            let val_offset = entry_offset + field.offset as usize;
            write_field_value(
                buffer,
                val_offset,
                value,
                field,
                string_table,
                string_offsets,
                options,
            )?;
        }
    }
    Ok(())
}

/// Write a field value to the buffer at the given offset, applying the field's mask and shift, and updating the string table for StringOffset fields
///
/// # Arguments
/// - `buffer` - The byte buffer where the field value should be written
/// - `offset` - The offset in the buffer where the field value should start
/// - `value` - The `FieldValue` instance representing the value to write
/// - `field` - The `Field` instance containing the field definition to use for writing the value
/// - `string_table` - A mutable byte vector representing the string table, which will be updated with new strings for StringOffset fields
/// - `string_offsets` - A mutable map of string to offset in the string table, used for reusing existing strings and avoiding duplicates in the string table
/// - `options` - Options for endianness and string encoding
///
/// # Returns
/// Ok(()) if the field value was successfully written to the buffer, or an error if writing fails (e.g. due to type mismatch or encoding errors)
fn write_field_value(
    buffer: &mut [u8],
    offset: usize,
    value: &FieldValue,
    field: &Field,
    string_table: &mut Vec<u8>,
    string_offsets: &mut std::collections::HashMap<String, u32>,
    options: &IoOptions,
) -> Result<()> {
    match (field.field_type, value) {
        (FieldType::Long | FieldType::UnsignedLong, FieldValue::Int(v)) => {
            let existing = if options.big_endian {
                BigEndian::read_u32(&buffer[offset..offset + 4])
            } else {
                LittleEndian::read_u32(&buffer[offset..offset + 4])
            };
            let masked = (existing & !field.mask) | (((*v as u32) << field.shift) & field.mask);
            if options.big_endian {
                BigEndian::write_u32(&mut buffer[offset..offset + 4], masked);
            } else {
                LittleEndian::write_u32(&mut buffer[offset..offset + 4], masked);
            }
        }

        (FieldType::Float, FieldValue::Float(v)) => {
            if options.big_endian {
                BigEndian::write_f32(&mut buffer[offset..offset + 4], *v);
            } else {
                LittleEndian::write_f32(&mut buffer[offset..offset + 4], *v);
            }
        }

        (FieldType::Short, FieldValue::Int(v)) => {
            let existing = if options.big_endian {
                BigEndian::read_u16(&buffer[offset..offset + 2])
            } else {
                LittleEndian::read_u16(&buffer[offset..offset + 2])
            };
            let masked = ((existing as u32 & !field.mask) | (((*v as u32) << field.shift) & field.mask)) as u16;
            if options.big_endian {
                BigEndian::write_u16(&mut buffer[offset..offset + 2], masked);
            } else {
                LittleEndian::write_u16(&mut buffer[offset..offset + 2], masked);
            }
        }

        (FieldType::Char, FieldValue::Int(v)) => {
            let existing = buffer[offset] as u32;
            let masked = ((existing & !field.mask) | (((*v as u32) << field.shift) & field.mask)) as u8;
            buffer[offset] = masked;
        }

        (FieldType::String, FieldValue::String(s)) => {
            let bytes = encode_string(s, options.encoding)?;
            let len = bytes.len().min(32);
            buffer[offset..offset + len].copy_from_slice(&bytes[..len]);
        }

        (FieldType::StringOffset, FieldValue::String(s)) => {
            let str_offset = if let Some(&existing_offset) = string_offsets.get(s) {
                existing_offset
            } else {
                let offset = string_table.len() as u32;
                let bytes = encode_string(s, options.encoding)?;
                string_table.extend_from_slice(&bytes);
                string_table.push(0); // Null terminator
                string_offsets.insert(s.clone(), offset);
                offset
            };

            if options.big_endian {
                BigEndian::write_u32(&mut buffer[offset..offset + 4], str_offset);
            } else {
                LittleEndian::write_u32(&mut buffer[offset..offset + 4], str_offset);
            }
        }

        _ => {
            return Err(JMapError::TypeMismatch {
                expected: field.field_type.csv_name(),
                got: value.type_name(),
            });
        }
    }

    Ok(())
}

/// Decode a byte slice into a string using the specified encoding
///
/// # Arguments
/// - `bytes` - The byte slice to decode
/// - `encoding` - The encoding to use for decoding the bytes (e.g. Shift-JIS or UTF-8)
///
/// # Errors
/// - `JMapError::EncodingError` if the bytes cannot be decoded using the specified encoding
///
/// # Returns
/// A `String` containing the decoded text, or an error if decoding fails
fn decode_string(bytes: &[u8], encoding: Encoding) -> Result<String> {
    match encoding {
        Encoding::Utf8 => String::from_utf8(bytes.to_vec())
            .map_err(|e| JMapError::EncodingError(e.to_string())),
        Encoding::ShiftJis => {
            let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(bytes);
            if had_errors {
                // Try to decode anyway, some bytes might be valid
            }
            Ok(decoded.into_owned())
        }
    }
}

/// Encode a string into a byte vector using the specified encoding
///
/// # Arguments
/// - `s` - The string to encode
/// - `encoding` - The encoding to use for encoding the string (e.g. Shift-JIS or UTF-8)
///
/// # Returns
/// A `Vec<u8>` containing the encoded bytes of the string, or an error if encoding fails
fn encode_string(s: &str, encoding: Encoding) -> Result<Vec<u8>> {
    match encoding {
        Encoding::Utf8 => Ok(s.as_bytes().to_vec()),
        Encoding::ShiftJis => {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(s);
            Ok(encoded.into_owned())
        }
    }
}
