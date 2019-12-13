use crate::item::PointerType;
use anyhow::{anyhow, bail, Result};
use gimli::{
    AttributeValue, DebuggingInformationEntry, Dwarf, EndianSlice, RunTimeEndian, Unit, UnitOffset,
};
use std::str;

pub fn from_pointer_type(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    module: &[String],
    die: &DebuggingInformationEntry<EndianSlice<RunTimeEndian>>,
) -> Result<PointerType> {
    let mut name = None;
    let mut ty = None;

    let mut attrs = die.attrs();
    while let Some(attr) = attrs.next()? {
        match attr.name() {
            gimli::DW_AT_name => {
                name = Some(str::from_utf8(&dwarf.attr_string(unit, attr.value())?)?.to_string());
            }
            gimli::DW_AT_type => {
                ty = Some(match attr.value() {
                    AttributeValue::UnitRef(UnitOffset(n)) => n,
                    val => bail!("Unexpected DW_AT_type value: {:?}", val),
                });
            }
            _ => {}
        }
    }

    Ok(PointerType {
        name: name.ok_or_else(|| anyhow!("Missing DW_AT_name"))?,
        module: module.to_vec(),
        type_index: ty.ok_or_else(|| anyhow!("Missing DW_AT_type"))?,
    })
}
