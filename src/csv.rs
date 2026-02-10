use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::entry::Entry;
use crate::error::{JMapError, Result};
use crate::field::{Field, FieldType, FieldValue};
use crate::hash::HashTable;
use crate::jmap::JMapInfo;

/// Read a JMapInfo from a CSV file
///
/// The CSV format uses a header row where each column is formatted as:
/// `FieldName:Type:DefaultValue`
///
/// For example: `ScenarioNo:Int:0,ZoneName:String:0`
/// The delimiter between the parts can be customized (default is ':') and should not appear in field names or type names
///
/// # Arguments
/// - `hash_table` - The hash table to use for field name lookups. Field names from the CSV will be added to this hash table
/// - `path` - The path to the CSV file to read
/// - `header_delimiter` - Optional character that separates field name, type, and default value in the header. Default is ':'
///
/// # Returns
/// A JMapInfo populated with fields and entries from the CSV file
pub fn from_csv<H: HashTable, P: AsRef<Path>>(
    hash_table: H,
    path: P,
    header_delimiter: Option<char>,

) -> Result<JMapInfo<H>> {
    let delimiter = header_delimiter.unwrap_or(':');
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader);

    let mut jmap = JMapInfo::new(hash_table);
    let mut records = csv_reader.records();

    // Parse header
    let header = records
        .next()
        .ok_or_else(|| JMapError::CsvError("CSV file is empty".to_string()))??;

    let mut field_infos: Vec<(u32, FieldType)> = Vec::new();

    for field_desc in header.iter() {
        let parts: Vec<&str> = field_desc.split(delimiter).collect();

        if parts.len() != 3 {
            return Err(JMapError::InvalidCsvFieldDescriptor(format!(
                "Expected 3 parts (name{}type{}default), got: {}",
                delimiter,
                delimiter,
                field_desc
            )));
        }

        let field_name = parts[0];
        let type_name = parts[1];
        let _default_str = parts[2];

        if field_name.is_empty() {
            return Err(JMapError::InvalidCsvFieldDescriptor(
                "Field name cannot be empty".to_string(),
            ));
        }

        let field_type = FieldType::from_csv_name(type_name).ok_or_else(|| {
            JMapError::InvalidCsvFieldDescriptor(format!("Unknown field type: {}", type_name))
        })?;

        // Parse hash from [XXXXXXXX] format or compute from name
        let hash = if field_name.starts_with('[') && field_name.ends_with(']') {
            let hex_str = &field_name[1..field_name.len() - 1];
            u32::from_str_radix(hex_str, 16).map_err(|_| {
                JMapError::InvalidCsvFieldDescriptor(format!("Invalid hash: {}", field_name))
            })?
        } else {
            jmap.hash_table_mut().add(field_name)
        };

        let default = FieldValue::default_for(field_type);
        let field = Field::with_default(hash, field_type, default);
        jmap.fields_map_mut().insert(hash, field);
        field_infos.push((hash, field_type));
    }

    // Parse data rows
    for result in records {
        let record = result?;
        let mut entry = Entry::with_capacity(field_infos.len());

        for (i, (hash, field_type)) in field_infos.iter().enumerate() {
            let value_str = record.get(i).unwrap_or("");

            let value = if value_str.is_empty() {
                FieldValue::default_for(*field_type)
            } else {
                parse_field_value(value_str, *field_type)?
            };

            entry.set_by_hash(*hash, value);
        }

        jmap.entries_vec_mut().push(entry);
    }

    Ok(jmap)
}

/// Write a JMapInfo to a CSV file
///
/// The CSV format uses a header row where each column is formatted as:
/// `FieldName:Type:DefaultValue`
///
/// # Arguments
/// - `jmap` - The JMapInfo to export to CSV
/// - `path` - The path to the CSV file to write
/// - `header_delimiter` - Optional character that separates field name, type, and default value in the header. Default is ':'
///
/// # Returns
/// Ok(()) if the export was successful, or an error if the file could not be written
pub fn to_csv<H: HashTable, P: AsRef<Path>>(jmap: &JMapInfo<H>, path: P, header_delimiter: Option<char>) -> Result<()> {
    let delimiter = header_delimiter.unwrap_or(':');
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::Writer::from_writer(writer);

    // Write header
    let headers: Vec<String> = jmap
        .fields()
        .map(|field| {
            let name = jmap.field_name(field.hash);
            let type_name = field.field_type.csv_name();
            let default = default_csv_value(field.field_type);
            format!("{}{}{}{}{}", name, delimiter, type_name, delimiter, default)
        })
        .collect();

    csv_writer.write_record(&headers)?;

    // Write entries
    for entry in jmap.entries() {
        let values: Vec<String> = jmap
            .fields()
            .map(|field| {
                entry
                    .get_by_hash(field.hash)
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            })
            .collect();

        csv_writer.write_record(&values)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn parse_field_value(s: &str, field_type: FieldType) -> Result<FieldValue> {
    match field_type {
        FieldType::Long | FieldType::UnsignedLong | FieldType::Short | FieldType::Char => {
            let v: i32 = s.parse().map_err(|_| {
                JMapError::CsvError(format!("Cannot parse '{}' as integer", s))
            })?;
            Ok(FieldValue::Int(v))
        }
        FieldType::Float => {
            let v: f32 = s.parse().map_err(|_| {
                JMapError::CsvError(format!("Cannot parse '{}' as float", s))
            })?;
            Ok(FieldValue::Float(v))
        }
        FieldType::String | FieldType::StringOffset => Ok(FieldValue::String(s.to_string())),
    }
}

fn default_csv_value(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::Long | FieldType::UnsignedLong | FieldType::Short | FieldType::Char => "0",
        FieldType::Float => "0.0",
        FieldType::String | FieldType::StringOffset => "0",
    }
}
