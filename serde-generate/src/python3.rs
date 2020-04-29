// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_reflection::{ContainerFormat, Format, Named, Registry, VariantFormat};
use std::collections::BTreeMap;
use std::io::{Result, Write};

pub fn output(out: &mut dyn Write, registry: &Registry) -> Result<()> {
    output_preambule(out)?;
    for (name, format) in registry {
        output_container(out, name, format)?;
    }
    Ok(())
}

fn output_preambule(out: &mut dyn Write) -> Result<()> {
    writeln!(
        out,
        r#"# pyre-ignore-all-errors
from dataclasses import dataclass
import numpy as np
import typing

class char(str):
    pass

class int128(typing.Tuple[np.int64, np.uint64]):
    pass

class uint128(typing.Tuple[np.uint64, np.uint64]):
    pass
"#
    )
}

fn quote_type(format: &Format) -> String {
    use Format::*;
    match format {
        TypeName(x) => format!("\"{}\"", x), // Need quotes because of circular dependencies.
        Unit => "None".into(),
        Bool => "np.bool".into(),
        I8 => "np.int8".into(),
        I16 => "np.int16".into(),
        I32 => "np.int32".into(),
        I64 => "np.int64".into(),
        I128 => "int128".into(),
        U8 => "np.uint8".into(),
        U16 => "np.uint16".into(),
        U32 => "np.uint32".into(),
        U64 => "np.uint64".into(),
        U128 => "uint128".into(),
        F32 => "np.float32".into(),
        F64 => "np.float64".into(),
        Char => "char".into(),
        Str => "str".into(),
        Bytes => "bytes".into(),

        Option(format) => format!("typing.Optional[{}]", quote_type(format)),
        Seq(format) => format!("typing.Sequence[{}]", quote_type(format)),
        Map { key, value } => format!("typing.Dict[{}, {}]", quote_type(key), quote_type(value)),
        Tuple(formats) => format!("typing.Tuple[{}]", quote_types(formats)),
        TupleArray { content, size } => format!(
            "typing.Tuple[{}]",
            quote_types(&vec![content.as_ref().clone(); *size])
        ), // Sadly, there are no fixed-size arrays in python.

        Variable(_) => panic!("unexpected value"),
    }
}

fn quote_types(formats: &[Format]) -> String {
    formats
        .iter()
        .map(quote_type)
        .collect::<Vec<_>>()
        .join(", ")
}

fn output_fields(out: &mut dyn Write, indentation: usize, fields: &[Named<Format>]) -> Result<()> {
    let tab = " ".repeat(indentation);
    for field in fields {
        writeln!(out, "{}{}: {}", tab, field.name, quote_type(&field.value))?;
    }
    Ok(())
}

fn output_variant(
    out: &mut dyn Write,
    base: &str,
    name: &str,
    index: u32,
    variant: &VariantFormat,
) -> Result<()> {
    use VariantFormat::*;
    match variant {
        Unit => writeln!(
            out,
            "@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n",
            base, name, base, index,
        ),
        NewType(format) => writeln!(
            out,
            "@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n    value: {}\n",
            base,
            name,
            base,
            index,
            quote_type(format)
        ),
        Tuple(formats) => writeln!(
            out,
            "@dataclass\nclass _{}_{}({}):\n    INDEX = {}\n    value: typing.Tuple[{}]\n",
            base,
            name,
            base,
            index,
            quote_types(formats)
        ),
        Struct(fields) => {
            writeln!(
                out,
                "@dataclass\nclass _{}_{}({}):\n    INDEX = {}",
                base, name, base, index
            )?;
            output_fields(out, 4, fields)?;
            writeln!(out)
        }
        Variable(_) => panic!("incorrect value"),
    }
}

fn output_variants(
    out: &mut dyn Write,
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    for (index, variant) in variants {
        output_variant(out, base, &variant.name, *index, &variant.value)?;
    }
    Ok(())
}

fn output_variant_aliases(
    out: &mut dyn Write,
    base: &str,
    variants: &BTreeMap<u32, Named<VariantFormat>>,
) -> Result<()> {
    for variant in variants.values() {
        writeln!(
            out,
            "{}.{} = _{}_{}",
            base, &variant.name, base, &variant.name
        )?;
    }
    Ok(())
}

fn output_container(out: &mut dyn Write, name: &str, format: &ContainerFormat) -> Result<()> {
    use ContainerFormat::*;
    match format {
        UnitStruct => writeln!(out, "@dataclass\nclass {}:\n    pass\n", name),
        NewTypeStruct(format) => writeln!(
            out,
            "@dataclass\nclass {}:\n    value: {}\n",
            name,
            quote_type(format)
        ),
        TupleStruct(formats) => writeln!(
            out,
            "@dataclass\nclass {}:\n    value: typing.Tuple[{}]\n",
            name,
            quote_types(formats)
        ),
        Struct(fields) => {
            writeln!(out, "@dataclass\nclass {}:", name)?;
            output_fields(out, 4, fields)?;
            writeln!(out)
        }
        Enum(variants) => {
            writeln!(out, "class {}:\n    pass\n", name)?;
            output_variants(out, name, variants)?;
            output_variant_aliases(out, name, variants)?;
            writeln!(
                out,
                "{}.VARIANTS = [\n{}]\n",
                name,
                variants
                    .iter()
                    .map(|(_, v)| format!("    {}.{},\n", name, v.name))
                    .collect::<Vec<_>>()
                    .join("")
            )
        }
    }
}
