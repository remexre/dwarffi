use crate::item::Item;
use anyhow::Result;
use std::path::Path;

pub fn make_ffi(path: &Path, items: &[(usize, Item)]) -> Result<()> {
    // ech... this is technically all awful.
    println!("ffi_file = {:?}", path);

    println!("import json");
    println!(
        "ffi_values = json.loads({:?})",
        serde_json::to_string(items)?
    );

    println!("\n\n\n{}", include_str!("template.py"));
    Ok(())
}
