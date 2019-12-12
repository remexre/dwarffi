use crate::item::{BaseType, BaseTypeKind};
use anyhow::{anyhow, bail, Result};
use gimli::{AttributeValue, DebuggingInformationEntry, Dwarf, EndianSlice, RunTimeEndian, Unit};
use std::str;

pub fn from_base_type(
    dwarf: &Dwarf<EndianSlice<RunTimeEndian>>,
    unit: &Unit<EndianSlice<RunTimeEndian>>,
    module: &[String],
    die: &DebuggingInformationEntry<EndianSlice<RunTimeEndian>>,
) -> Result<BaseType> {
    let mut name = None;
    let mut size = None;
    let mut kind = None;

    let mut attrs = die.attrs();
    while let Some(attr) = attrs.next()? {
        match attr.name() {
            gimli::DW_AT_name => {
                name = Some(str::from_utf8(&dwarf.attr_string(unit, attr.value())?)?.to_string());
            }
            gimli::DW_AT_byte_size => {
                size = attr.value().udata_value();
            }
            gimli::DW_AT_encoding => {
                kind = Some(match attr.value() {
                    AttributeValue::Encoding(gimli::DW_ATE_float) => BaseTypeKind::Float,
                    AttributeValue::Encoding(gimli::DW_ATE_signed) => BaseTypeKind::SignedInt,
                    AttributeValue::Encoding(gimli::DW_ATE_unsigned) => BaseTypeKind::UnsignedInt,
                    AttributeValue::Encoding(gimli::DW_ATE_unsigned_char) => BaseTypeKind::Char,
                    AttributeValue::Encoding(e) => bail!("Invalid DW_AT_encoding: {}", e),
                    val => bail!("Invalid DW_AT_encoding: {:?}", val),
                });
            }
            _ => {}
        }
    }

    let mut bt = BaseType {
        name: name.ok_or_else(|| anyhow!("Missing DW_AT_name"))?,
        module: module.to_vec(),
        size: size.ok_or_else(|| anyhow!("Missing or invalid DW_AT_byte_size"))?,
        kind: kind.ok_or_else(|| anyhow!("Missing DW_AT_encoding"))?,
    };
    if bt.name == "!" && bt.size == 0 {
        bt.kind = BaseTypeKind::Never;
    }
    if bt.name == "()" && bt.size == 0 {
        bt.kind = BaseTypeKind::Unit;
    }

    Ok(bt)
}
