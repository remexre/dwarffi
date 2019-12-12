mod base_type;
mod function;
mod structure;

use crate::item::Item;
use anyhow::{anyhow, bail, Context, Result};
use fallible_iterator::FallibleIterator;
use gimli::{AttributeValue, Dwarf, EndianSlice, EntriesTreeNode, RunTimeEndian, Unit};
use log::trace;
use object::Object;
use std::{borrow::Cow, str};

pub fn get_items(file: &[u8]) -> Result<Vec<(usize, Item)>> {
    let elf = object::File::parse(&file)
        .map_err(|e| anyhow!("{}", e).context("Failed to parse file as ELF"))?;
    let endianess = if elf.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };
    let dwarf = Dwarf::load(
        |section| -> Result<_> {
            Ok(elf
                .section_data_by_name(section.name())
                .unwrap_or(Cow::Borrowed(&[][..])))
        },
        |_section| Ok(Cow::Borrowed(&[][..])),
    )
    .context("Failed to parse debug info")?;
    let dwarf = dwarf.borrow(|section| EndianSlice::new(&section, endianess));

    dwarf
        .units()
        .take(1) // TODO: Remove after debugging done
        .map_err(|err| anyhow::Error::from(err).context("Error getting next unit"))
        .flat_map(|header| {
            let unit = dwarf.unit(header).context("Failed to call unit()")?;

            let mut items = Vec::new();

            let mut tree = unit
                .entries_tree(None)
                .context("Failed to get entries tree")?;
            let node = tree.root().context("Failed to get root of entries tree")?;
            handle_node(&dwarf, &unit, &mut Vec::new(), &mut items, node)?;
            Ok(fallible_iterator::convert(items.into_iter().map(Ok)))
        })
        .collect()
}

fn handle_node(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    module: &mut Vec<String>,
    items: &mut Vec<(usize, Item)>,
    node: EntriesTreeNode<EndianSlice<RunTimeEndian>>,
) -> Result<()> {
    let offset = node.entry().offset().0;
    match node.entry().tag() {
        gimli::DW_TAG_compile_unit => {
            if node.entry().attr_value(gimli::DW_AT_language)?
                != Some(AttributeValue::Language(gimli::DW_LANG_Rust))
            {
                if let Some(name) = node.entry().attr_value(gimli::DW_AT_name)? {
                    let name = dwarf.attr_string(unit, name)?;
                    let name = str::from_utf8(&name)?;
                    bail!("The compilation unit {:?} doesn't appear to be Rust.", name);
                } else {
                    bail!("The compilation unit doesn't appear to be Rust.");
                }
            }

            let mut iter = node.children();
            while let Some(node) = iter.next()? {
                handle_node(dwarf, unit, module, items, node)?;
            }
        }
        gimli::DW_TAG_namespace => {
            if let Some(name) = node.entry().attr_value(gimli::DW_AT_name)? {
                let name = str::from_utf8(&dwarf.attr_string(unit, name)?)?.to_string();
                module.push(name);
            }

            let mut iter = node.children();
            while let Some(node) = iter.next()? {
                handle_node(dwarf, unit, module, items, node)?;
            }

            module.pop();
        }
        gimli::DW_TAG_subprogram => {
            let func = function::from_subprogram(dwarf, unit, &module, node.entry())?;
            let mut func = if let Some(func) = func {
                func
            } else {
                return Ok(());
            };

            let mut iter = node.children();
            while let Some(node) = iter.next()? {
                function::modify(dwarf, unit, &mut func, node.entry())?;
            }

            items.push((offset, Item::Function(func)));
        }
        gimli::DW_TAG_base_type => {
            let bt = base_type::from_base_type(dwarf, unit, &module, node.entry())?;
            items.push((offset, Item::BaseType(bt)));
        }
        gimli::DW_TAG_structure_type => {
            let mut ty = structure::from_structure_type(dwarf, unit, &module, node.entry())?;

            let mut iter = node.children();
            while let Some(node) = iter.next()? {
                structure::modify(dwarf, unit, &mut ty, node.entry())?;
            }

            items.push((offset, Item::Structure(ty)));
        }
        tag => trace!("[{:x}] unsupported tag: {}", offset, tag),
    }
    Ok(())
}
