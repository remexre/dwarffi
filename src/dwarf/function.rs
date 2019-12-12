use crate::item::Function;
use anyhow::{anyhow, bail, Result};
use gimli::{
    AttributeValue, DebuggingInformationEntry, Dwarf, EndianSlice, RunTimeEndian, Unit, UnitOffset,
};
use log::trace;
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
    let mut full_name = None;
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
                let demangled = demangle(&name).to_string();
                // Likely nix some of these.
                if demangled.starts_with("alloc::")
                    || demangled.starts_with("backtrace::")
                    || demangled.starts_with("compiler_builtins::")
                    || demangled.starts_with("core::")
                    || demangled.starts_with("libc::")
                    || demangled.starts_with("panic_unwind::")
                    || demangled.starts_with("rust_")
                    || demangled.starts_with("rustc_demangle::")
                    || demangled.starts_with("std::")
                    || demangled.starts_with("<")
                    || demangled.starts_with("__")
                {
                    return Ok(None);
                }
                linkage_name = Some(name);
                full_name = Some(demangled);
            }
            _ => {}
        }
    }

    Ok(Some(Function {
        name,
        linkage_name: linkage_name.ok_or_else(|| anyhow!("Missing DW_AT_linkage_name"))?,
        full_name: full_name.ok_or_else(|| anyhow!("Missing DW_AT_linkage_name"))?,
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

            let name =
                name.ok_or_else(|| anyhow!("Missing DW_AT_name from DW_TAG_formal_parameter"))?;
            let ty = ty.ok_or_else(|| anyhow!("Missing DW_AT_type"))?;
            function.arguments.push((name, ty));
        }
        tag => trace!("[{:x}]<mf> unsupported tag: {}", die.offset().0, tag),
    }
    Ok(())
}
