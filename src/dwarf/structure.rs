use crate::item::{Structure, StructureMember};
use anyhow::{anyhow, bail, Result};
use gimli::{
    AttributeValue, DebuggingInformationEntry, Dwarf, EndianSlice, RunTimeEndian, Unit, UnitOffset,
};
use log::trace;
use std::str;

pub fn from_structure_type(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    module: &[String],
    die: &DebuggingInformationEntry<EndianSlice<RunTimeEndian>>,
) -> Result<Structure> {
    let mut name = None;
    let mut size = None;
    let mut alignment = None;

    let mut attrs = die.attrs();
    while let Some(attr) = attrs.next()? {
        match attr.name() {
            gimli::DW_AT_name => {
                name = Some(str::from_utf8(&dwarf.attr_string(unit, attr.value())?)?.to_string());
            }
            gimli::DW_AT_byte_size => {
                size = attr.value().udata_value();
            }
            gimli::DW_AT_alignment => {
                alignment = attr.value().udata_value();
            }
            _ => {}
        }
    }

    Ok(Structure {
        name: name.ok_or_else(|| anyhow!("Missing DW_AT_name"))?,
        module: module.to_vec(),
        size: size.ok_or_else(|| anyhow!("Missing or invalid DW_AT_byte_size"))?,
        alignment: alignment.ok_or_else(|| anyhow!("Missing or invalid DW_AT_alignment"))?,
        members: Vec::new(),
    })
}

pub fn modify(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    structure: &mut Structure,
    die: &DebuggingInformationEntry<EndianSlice<RunTimeEndian>>,
) -> Result<()> {
    let string =
        |val| -> Result<_> { Ok(str::from_utf8(&dwarf.attr_string(unit, val)?)?.to_string()) };

    match die.tag() {
        gimli::DW_TAG_member => {
            let mut name = None;
            let mut ty = None;
            let mut offset = None;
            let mut alignment = None;

            let mut attrs = die.attrs();
            while let Some(attr) = attrs.next()? {
                match attr.name() {
                    gimli::DW_AT_name => {
                        name = Some(string(attr.value())?);
                    }
                    gimli::DW_AT_type => {
                        ty = Some(match attr.value() {
                            AttributeValue::UnitRef(UnitOffset(n)) => n,
                            val => bail!("Unexpected DW_AT_type value: {:?}", val),
                        });
                    }
                    gimli::DW_AT_data_member_location => {
                        offset = attr.value().udata_value();
                    }
                    gimli::DW_AT_alignment => {
                        alignment = attr.value().udata_value();
                    }
                    _ => {}
                }
            }

            structure.members.push(StructureMember {
                name: name.ok_or_else(|| anyhow!("Missing DW_AT_name"))?,
                type_index: ty.ok_or_else(|| anyhow!("Missing DW_AT_type"))?,
                offset: offset
                    .ok_or_else(|| anyhow!("Missing or invalid DW_AT_data_member_location"))?,
                alignment: alignment
                    .ok_or_else(|| anyhow!("Missing or invalid DW_AT_alignment"))?,
            });
        }
        tag => trace!("[{:x}]<ms> unsupported tag: {}", die.offset().0, tag),
    }
    Ok(())
}
