//! Example: Read a BCSV file and export to CSV

use std::path::Path;
use bcsv::{smg_hash_table_with_lookup, IoOptions, from_csv, to_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lookup_path = Path::new("assets/strings_SMG.txt");
    let hash_table = smg_hash_table_with_lookup(lookup_path)?;

    let csv_path = Path::new("assets/examples/scenariodata.csv");
    let jmap = from_csv(hash_table, csv_path, None)?;

    println!("CSV Info");
    println!("Entries: {}", jmap.len());
    println!("Fields: {}", jmap.num_fields());
    println!();

    to_file(&jmap, "test_output.bcsv", &IoOptions::default())?;
    println!("\nExported to test_output.bcsv");

    Ok(())
}
