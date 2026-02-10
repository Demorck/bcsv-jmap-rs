use thiserror::Error;

/// Result type alias for JMap operations
pub type Result<T> = std::result::Result<T, JMapError>;

/// Errors that can occur during JMap operations
#[derive(Error, Debug)]
pub enum JMapError {
    /// Invalid field type ID encountered during parsing
    #[error("Invalid field type ID: 0x{0:02X}")]
    InvalidFieldType(u8),

    /// Field not found in the container
    #[error("Field not found: {0}")]
    FieldNotFound(String),

    /// Field already exists in the container
    #[error("Field already exists: {0}")]
    FieldAlreadyExists(String),

    /// Type mismatch when setting a field value
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        expected: &'static str,
        got: &'static str,
    },

    /// Entry index out of bounds
    #[error("Entry index out of bounds: {index} (len: {len})")]
    EntryIndexOutOfBounds { index: usize, len: usize },

    /// Buffer too small to contain valid BCSV data
    #[error("Buffer too small: expected at least {expected} bytes, got {got}")]
    BufferTooSmall { expected: usize, got: usize },

    /// Invalid BCSV header
    #[error("Invalid BCSV header")]
    InvalidHeader,

    /// String encoding error
    #[error("String encoding error: {0}")]
    EncodingError(String),

    /// CSV parsing error
    #[error("CSV error: {0}")]
    CsvError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Lookup file not found
    #[error("Lookup file not found: {0}")]
    LookupFileNotFound(String),

    /// Invalid CSV field descriptor format
    #[error("Invalid CSV field descriptor: {0}")]
    InvalidCsvFieldDescriptor(String),
}

impl From<csv::Error> for JMapError {
    fn from(err: csv::Error) -> Self {
        JMapError::CsvError(err.to_string())
    }
}
