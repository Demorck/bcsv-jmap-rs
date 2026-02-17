use std::path::Path;

use pyo3::prelude::*;
use crate::{
    from_csv, from_file, smg_hash_table_with_lookup, to_csv, to_file, FileHashTable, IoOptions,
    JMapInfo as RustJMapInfo,
};

/// A Python wrapper for JMapInfo.
#[pyclass(name = "JMap")]
pub struct PyJMap {
    inner: RustJMapInfo<FileHashTable>,
}

#[pymethods]
impl PyJMap {
    /// Create a JMap from a BCSV file.
    #[staticmethod]
    pub fn from_file(hash_table_path: &str, bcsv_path: &str) -> PyResult<Self> {
        let hash_table = smg_hash_table_with_lookup(Path::new(hash_table_path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        
        let jmap = from_file(hash_table, Path::new(bcsv_path), &IoOptions::default())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        Ok(PyJMap { inner: jmap })
    }

    /// Create a JMap from a CSV file.
    #[staticmethod]
    pub fn from_csv(hash_table_path: &str, csv_path: &str) -> PyResult<Self> {
        let hash_table = smg_hash_table_with_lookup(Path::new(hash_table_path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        
        // Use default delimiter
        let jmap = from_csv(hash_table, Path::new(csv_path), None)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

        Ok(PyJMap { inner: jmap })
    }

    /// Write the JMap to a BCSV file.
    pub fn to_file(&self, path: &str) -> PyResult<()> {
        to_file(&self.inner, Path::new(path), &IoOptions::default())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(())
    }

    /// Write the JMap to a CSV file.
    pub fn to_csv(&self, path: &str) -> PyResult<()> {
        to_csv(&self.inner, Path::new(path), None)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(())
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get the number of fields.
    pub fn num_fields(&self) -> usize {
        self.inner.num_fields()
    }
    
    /// Recalculate offsets in memory (useful for debugging).
    pub fn recalculate_offsets(&mut self) {
        self.inner.recalculate_offsets();
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn lib_bcsv_jmap(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyJMap>()?;
    Ok(())
}
