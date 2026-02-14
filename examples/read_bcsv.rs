//! Example: Read a BCSV file and export to CSV

use std::path::Path;
use lib_bcsv_jmap::{from_file, to_csv, smg_hash_table_with_lookup, IoOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lookup_path = Path::new("assets/strings_SMG.txt");
    let hash_table = smg_hash_table_with_lookup(lookup_path)?;

    let bcsv_path = Path::new("assets/examples/scenariodata.bcsv");
    let jmap = from_file(hash_table, bcsv_path, &IoOptions::default())?;

    println!("BCSV Info");
    println!("Entries: {}", jmap.len());
    println!("Fields: {}", jmap.num_fields());
    println!();

    println!("Fields");
    for field in jmap.fields() {
        let name = jmap.field_name(field.hash);
        println!("+0x{:X} - {} - {}", field.offset, name, field.field_type);
    }
    println!();

    to_csv(&jmap, "test_output.csv", None)?;
    println!("\nExported to test_output.csv");

    Ok(())
}
