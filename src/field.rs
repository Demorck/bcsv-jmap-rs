use std::fmt;

/// Data types supported by BCSV format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FieldType {
    /// Signed 32-bit integer - (4 bytes)
    Long = 0,
    /// Inline string - (32 bytes fixed). Deprecated
    String = 1,
    /// 32-bit floating point (4 bytes)
    Float = 2,
    /// Unsigned 32-bit integer (4 bytes)
    UnsignedLong = 3,
    /// Signed 16-bit integer (2 bytes)
    Short = 4,
    /// Signed 8-bit integer (1 byte)
    Char = 5,
    /// String stored in string table (4 byte offset)
    StringOffset = 6,
}

impl FieldType {
    /// Size in bytes for this field type
    pub const fn size(&self) -> usize {
        match self {
            FieldType::Long => 4,
            FieldType::String => 32,
            FieldType::Float => 4,
            FieldType::UnsignedLong => 4,
            FieldType::Short => 2,
            FieldType::Char => 1,
            FieldType::StringOffset => 4,
        }
    }

    /// Default bitmask for this field type
    pub const fn default_mask(&self) -> u32 {
        match self {
            FieldType::Long => 0xFFFFFFFF,
            FieldType::String => 0x00000000,
            FieldType::Float => 0xFFFFFFFF,
            FieldType::UnsignedLong => 0xFFFFFFFF,
            FieldType::Short => 0x0000FFFF,
            FieldType::Char => 0x000000FF,
            FieldType::StringOffset => 0xFFFFFFFF,
        }
    }

    /// Sorting order for field layout (used when calculating offsets)
    pub const fn order(&self) -> u8 {
        match self {
            FieldType::String => 0,
            FieldType::Float => 1,
            FieldType::Long => 2,
            FieldType::UnsignedLong => 3,
            FieldType::Short => 4,
            FieldType::Char => 5,
            FieldType::StringOffset => 6,
        }
    }

    /// Parse field type from raw byte value
    pub fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(FieldType::Long),
            1 => Some(FieldType::String),
            2 => Some(FieldType::Float),
            3 => Some(FieldType::UnsignedLong),
            4 => Some(FieldType::Short),
            5 => Some(FieldType::Char),
            6 => Some(FieldType::StringOffset),
            _ => None,
        }
    }

    /// Get the name of this field type for CSV export
    pub fn csv_name(&self) -> &'static str {
        match self {
            FieldType::Long => "Int",
            FieldType::String => "EmbeddedString",
            FieldType::Float => "Float",
            FieldType::UnsignedLong => "UnsignedInt",
            FieldType::Short => "Short",
            FieldType::Char => "Char",
            FieldType::StringOffset => "String",
        }
    }

    /// Parse field type from CSV name
    pub fn from_csv_name(name: &str) -> Option<Self> {
        match name {
            "Int" => Some(FieldType::Long),
            "EmbeddedString" => Some(FieldType::String),
            "Float" => Some(FieldType::Float),
            "UnsignedInt" => Some(FieldType::UnsignedLong),
            "Short" => Some(FieldType::Short),
            "Char" => Some(FieldType::Char),
            "String" => Some(FieldType::StringOffset),
            _ => None,
        }
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.csv_name())
    }
}

/// A value that can be stored in a JMap field
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Integer value (for Long, UnsignedLong, Short, Char)
    Int(i32),
    /// Floating point value
    Float(f32),
    /// String value (for String or StringOffset)
    String(String),
}

impl FieldValue {
    /// Get the default value for a field type
    pub fn default_for(field_type: FieldType) -> Self {
        match field_type {
            FieldType::Long
            | FieldType::UnsignedLong
            | FieldType::Short
            | FieldType::Char => FieldValue::Int(0),
            FieldType::Float => FieldValue::Float(0.0),
            FieldType::String | FieldType::StringOffset => FieldValue::String(String::new()),
        }
    }

    /// Check if this value is compatible with a field type
    pub fn is_compatible_with(&self, field_type: FieldType) -> bool {
        match (self, field_type) {
            (FieldValue::Int(_), FieldType::Long)
            | (FieldValue::Int(_), FieldType::UnsignedLong)
            | (FieldValue::Int(_), FieldType::Short)
            | (FieldValue::Int(_), FieldType::Char) => true,
            (FieldValue::Float(_), FieldType::Float) => true,
            (FieldValue::String(_), FieldType::String)
            | (FieldValue::String(_), FieldType::StringOffset) => true,
            _ => false,
        }
    }

    /// Get as integer, if this is an Int value
    pub fn as_int(&self) -> Option<i32> {
        match self {
            FieldValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as float, if this is a Float value
    pub fn as_float(&self) -> Option<f32> {
        match self {
            FieldValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as string reference, if this is a String value
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::String(v) => Some(v),
            _ => None,
        }
    }

    /// Get the type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            FieldValue::Int(_) => "Int",
            FieldValue::Float(_) => "Float",
            FieldValue::String(_) => "String",
        }
    }
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Int(v) => write!(f, "{}", v),
            FieldValue::Float(v) => write!(f, "{}", v),
            FieldValue::String(v) => write!(f, "{}", v),
        }
    }
}

impl From<i32> for FieldValue {
    fn from(v: i32) -> Self {
        FieldValue::Int(v)
    }
}

impl From<f32> for FieldValue {
    fn from(v: f32) -> Self {
        FieldValue::Float(v)
    }
}

impl From<String> for FieldValue {
    fn from(v: String) -> Self {
        FieldValue::String(v)
    }
}

impl From<&str> for FieldValue {
    fn from(v: &str) -> Self {
        FieldValue::String(v.to_string())
    }
}

/// Definition of a field (column) in a BCSV
#[derive(Debug, Clone)]
pub struct Field {
    /// Hash of the field name
    pub hash: u32,
    /// Bitmask for the field value
    pub mask: u32,
    /// Offset within an entry (set during packing/unpacking)
    pub offset: u16,
    /// Data type of the field
    pub field_type: FieldType,
    /// Bit shift amount
    pub shift: u8,

    /// Default value for new entries
    pub default: FieldValue,
}

impl Field {
    /// Create a new field with the given parameters
    pub fn new(hash: u32, field_type: FieldType) -> Self {
        Self {
            hash,
            field_type,
            mask: field_type.default_mask(),
            shift: 0,
            offset: 0,
            default: FieldValue::default_for(field_type),
        }
    }

    /// Create a new field with a custom default value
    pub fn with_default(hash: u32, field_type: FieldType, default: FieldValue) -> Self {
        Self {
            hash,
            field_type,
            mask: field_type.default_mask(),
            shift: 0,
            offset: 0,
            default,
        }
    }

    /// Size of this field in bytes
    pub fn size(&self) -> usize {
        self.field_type.size()
    }
}
