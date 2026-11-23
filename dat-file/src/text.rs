//! Human readable representation of `.DAT` file

use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{self, Write as _},
};

use daedalus_bytecode::InstructionData;
use zstring::bstr::ByteSlice as _;

use crate::{properties::PropFlag, DatFile, Symbol, SymbolData};

fn quoted(bytes: &[u8]) -> String {
    format!("{:?}", bytes.as_bstr())
}

fn name(bytes: &[u8]) -> String {
    let bare = bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'.');
    match bare {
        true => String::from_utf8(bytes.to_vec()).unwrap(),
        false => quoted(bytes),
    }
}

/// Whether the address of a symbol is the start of its body
fn owns_body(symbol: &Symbol) -> bool {
    let flags = symbol.props.elem_props.flags();
    matches!(symbol.data, SymbolData::Address(a) if a >= 0)
        && flags.contains(PropFlag::CONST)
        && !flags.intersects(PropFlag::EXTERNAL | PropFlag::CLASS_VAR)
}

struct Printer<'a> {
    dat: &'a DatFile,
    /// Name of each symbol, or `#index` if it has none
    names: Vec<String>,
    /// Symbol owning the body that starts at an address
    bodies: BTreeMap<u32, u32>,
    reserved: bool,
}

impl Printer<'_> {
    fn symbol(&self, id: u32) -> Cow<'_, str> {
        match self.names.get(id as usize) {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(format!("#{id}")),
        }
    }

    /// `BODY+offset` of an address, or the plain address if it's before any body
    fn address(&self, address: u32) -> String {
        match self.bodies.range(..=address).next_back() {
            None => address.to_string(),
            Some((&start, &id)) if start == address => self.symbol(id).into_owned(),
            Some((&start, &id)) => format!("{}+{}", self.symbol(id), address - start),
        }
    }

    fn print(&self, out: &mut String) -> fmt::Result {
        writeln!(out, "version {}", self.dat.version)?;
        for (id, symbol) in self.dat.symbols.iter().enumerate() {
            self.print_symbol(out, id as u32, symbol)?;
        }

        writeln!(out, "\ncode")?;
        self.print_code(out)?;

        writeln!(out, "\nsort")?;
        for &id in &self.dat.sort_indexes {
            writeln!(out, "  {}", self.symbol(id))?;
        }
        Ok(())
    }

    fn print_symbol(&self, out: &mut String, id: u32, symbol: &Symbol) -> fmt::Result {
        let props = symbol.props.elem_props;

        write!(out, "\nsymbol {} {:?}", self.symbol(id), props.data_type())?;
        for (flag, _) in props.flags().iter_names() {
            write!(out, " {flag}")?;
        }
        writeln!(out)?;

        if props.count() != 0 {
            writeln!(out, "  count {}", props.count())?;
        }
        if props.space() != 0 {
            writeln!(out, "  space {}", props.space())?;
        }
        if symbol.props.off_cls_ret != 0 {
            writeln!(out, "  off_cls_ret {}", symbol.props.off_cls_ret)?;
        }

        let span = &symbol.code_span;
        writeln!(
            out,
            "  span file={} line={}+{} char={}+{}",
            span.file_index.value(),
            span.line_start.value(),
            span.line_count.value(),
            span.char_start.get(),
            span.char_count.get(),
        )?;

        if let Some(parent) = symbol.parent {
            writeln!(out, "  parent {}", self.symbol(parent))?;
        }

        match &symbol.data {
            SymbolData::Float(values) => {
                for v in values {
                    writeln!(out, "  value {v:?}")?;
                }
            }
            SymbolData::Int(values) => {
                for v in values {
                    writeln!(out, "  value {v}")?;
                }
            }
            SymbolData::String(values) => {
                for v in values {
                    writeln!(out, "  value {}", quoted(v))?;
                }
            }
            SymbolData::ClassOffset(offset) => writeln!(out, "  class_offset {offset}")?,
            SymbolData::Address(address) if owns_body(symbol) => {
                writeln!(out, "  address @{}", self.address(*address as u32))?
            }
            SymbolData::Address(address) => writeln!(out, "  address {address}")?,
            SymbolData::None => {}
        }

        let reserved = [
            ("props", props.reserved() << 23),
            ("file", span.file_index.reserved()),
            ("line_start", span.line_start.reserved()),
            ("line_count", span.line_count.reserved()),
            ("char_start", span.char_start.reserved()),
            ("char_count", span.char_count.reserved()),
        ];
        if self.reserved && reserved.iter().any(|(_, v)| *v != 0) {
            write!(out, "  reserved")?;
            for (field, v) in reserved.iter().filter(|(_, v)| *v != 0) {
                write!(out, " {field}=0x{v:08X}")?;
            }
            writeln!(out)?;
        }
        Ok(())
    }

    fn print_code(&self, out: &mut String) -> fmt::Result {
        let mut address = 0;
        for instruction in self.dat.bytecode.instructions() {
            if let Some(&id) = self.bodies.get(&address) {
                writeln!(out, "\n{}:", self.symbol(id))?;
            }
            address += instruction.size() as u32;

            write!(out, "  {:?}", instruction.opcode)?;
            match instruction.data {
                InstructionData::Address(target) => write!(out, " {}", self.address(target)),
                InstructionData::Immediate(v) => write!(out, " {v}"),
                InstructionData::Symbol(id) => write!(out, " {}", self.symbol(id)),
                InstructionData::SymbolIndex { symbol, index } => {
                    write!(out, " {} {index}", self.symbol(symbol))
                }
                InstructionData::None => Ok(()),
            }?;
            writeln!(out)?;
        }
        Ok(())
    }
}

pub fn print(dat: &DatFile, reserved: bool) -> String {
    let names = (dat.symbols.iter().enumerate())
        .map(|(id, s)| s.name.as_ref().map_or(format!("#{id}"), |n| name(n)))
        .collect();

    let mut bodies = BTreeMap::new();
    for (id, symbol) in dat.symbols.iter().enumerate() {
        if let SymbolData::Address(address) = symbol.data {
            if owns_body(symbol) {
                // The first symbol owning an address labels its body
                bodies.entry(address as u32).or_insert(id as u32);
            }
        }
    }

    let printer = Printer {
        dat,
        names,
        bodies,
        reserved,
    };
    let mut out = String::new();
    printer
        .print(&mut out)
        .expect("writing to a String can't fail");
    out
}
