# jmap-rs

A Rust library for reading and writing Nintendo's BCSV/JMap format. This format is primarily used by *Super Mario Galaxy* and other Nintendo titles.

## Features

- **Full Format Support**: Read and write BCSV files with complete fidelity.
- **Endianness Support**: Handle both big-endian and little-endian data seamlessly.
- **String Encoding**: Support for Shift-JIS (Japanese) and UTF-8 string encodings.
- **CSV Integration**: Import from and export to CSV files for easy editing.
- **Hash Table Management**: Utilize hash tables for efficient field name lookups.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lib-bcsv-jmap = "0.1.0"
```

## Quick Start

### Reading a BCSV File

```rust
use jmap::{from_file, smg_hash_table_with_lookup, IoOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a hash table with known field names
    let hash_table = smg_hash_table_with_lookup("assets/strings_SMG.txt")?;

    // 2. Read a BCSV file
    let jmap = from_file(hash_table, "scenariodata.bcsv", &IoOptions::default())?;

    println!("Loaded BCSV with {} entries", jmap.len());

    // 3. Iterate over entries
    for entry in jmap.entries() {
        if let Some(name) = entry.get_string(jmap.hash_table(), "ZoneName") {
            println!("Zone: {}", name);
        }
    }

    Ok(())
}
```

### Writing a BCSV File

```rust
use lib_bcsv_jmap::{to_file, IoOptions};

// ... inside main ...
// Export to BCSV
to_file(&jmap, "output.bcsv", &IoOptions::default())?;
```

### Converting BCSV to CSV

```rust
use lib_bcsv_jmap::{to_csv};

// ... inside main ...
// Export to CSV
to_csv(&jmap, "output.csv", None)?;
```

## Python Bindings

This library includes Python bindings using `maturin`.

### Installation

You can install the library directly from source using `maturin`:

```bash
pip install maturin
maturin develop
```

### Usage

```python
import lib_bcsv_jmap as jmap

# Read BCSV
jmap_data = jmap.JMap.from_file("assets/strings_SMG.txt", "scenariodata.bcsv")
print(f"Loaded {len(jmap_data)} entries")

# Write BCSV
jmap_data.to_file("output.bcsv")

# Export to CSV
jmap_data.to_csv("output.csv")
```
