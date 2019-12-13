use crate::{dwarf::dump_die, item::Function};
use anyhow::{anyhow, bail, Result};
use gimli::{
    AttributeValue, DebuggingInformationEntry, Dwarf, EndianSlice, RunTimeEndian, Unit, UnitOffset,
};
use log::debug;
use rustc_demangle::demangle;
use std::str;

pub fn from_subprogram(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    module: &[String],
    die: &DebuggingInformationEntry<EndianSlice<RunTimeEndian>>,
) -> Result<Option<Function>> {
    let string =
        |val| -> Result<_> { Ok(str::from_utf8(&dwarf.attr_string(unit, val)?)?.to_string()) };

    let mut name = None;
    let mut linkage_name = None;
    let mut ret_type_index = None;

    let mut attrs = die.attrs();
    while let Some(attr) = attrs.next()? {
        match attr.name() {
            gimli::DW_AT_name => {
                name = Some(string(attr.value())?);
            }
            gimli::DW_AT_external => match attr.value() {
                AttributeValue::Flag(true) => {}
                _ => return Ok(None),
            },
            gimli::DW_AT_type => {
                ret_type_index = Some(match attr.value() {
                    AttributeValue::UnitRef(UnitOffset(n)) => n,
                    val => bail!("Unexpected DW_AT_type value: {:?}", val),
                });
            }
            gimli::DW_AT_linkage_name => {
                let name = string(attr.value())?;
                linkage_name = Some(name);
            }
            _ => {}
        }
    }

    let linkage_name = linkage_name.ok_or_else(|| {
        let _ = dump_die(dwarf, unit, die, 0, "<ef> ");
        anyhow!(
            "Missing DW_AT_linkage_name from {:?} in {:?} at 0x{:x}",
            name,
            module,
            die.offset().0
        )
    })?;
    let full_name = demangle(&linkage_name).to_string();
    // Likely nix some of these.
    if full_name.starts_with("alloc::")
        || full_name.starts_with("backtrace::")
        || full_name.starts_with("compiler_builtins::")
        || full_name.starts_with("core::")
        || full_name.starts_with("libc::")
        || full_name.starts_with("panic_unwind::")
        || full_name.starts_with("rust_")
        || full_name.starts_with("rustc_demangle::")
        || full_name.starts_with("std::")
        || full_name.starts_with("<")
        || full_name.starts_with("__")
    {
        return Ok(None);
    }

    Ok(Some(Function {
        name,
        linkage_name,
        full_name,
        module: module.to_vec(),
        ret_type_index,
        arguments: Vec::new(),
    }))
}

pub fn modify(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    function: &mut Function,
    die: &DebuggingInformationEntry<EndianSlice<RunTimeEndian>>,
) -> Result<()> {
    let string =
        |val| -> Result<_> { Ok(str::from_utf8(&dwarf.attr_string(unit, val)?)?.to_string()) };

    match die.tag() {
        gimli::DW_TAG_formal_parameter => {
            let mut name = None;
            let mut ty = None;

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
                    _ => {}
                }
            }

            let ty =
                ty.ok_or_else(|| anyhow!("Missing DW_AT_type from DW_TAG_formal_parameter"))?;
            function.arguments.push((name, ty));
        }
        tag => {
            debug!("In function: {}", function.full_name);
            debug!("Unsupported tag: {}", tag);
            dump_die(dwarf, unit, die, 0, "<mf> ")?
        }
    }
    Ok(())
}
