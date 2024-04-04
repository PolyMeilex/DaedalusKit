use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use daedalus_bytecode::Bytecode;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive as _;
use properties::{Properties, SymbolCodeSpan};
use std::io::{Read, Write};
use zstring::ZString;

use crate::properties::{DataType, PropFlag};

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub name: Option<ZString>,
    pub props: Properties,
    pub code_span: SymbolCodeSpan,
    pub data: SymbolData,
    pub parent: Option<u32>,
}

impl Symbol {
    pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
        let named = r.read_u32::<LittleEndian>()?;

        assert!(named == 0 || named == 1, "has_name: 0x{named:x?}");

        let name = if named == 1 {
            Some(ZString::decode(&mut r)?)
        } else {
            None
        };

        let props = properties::Properties::decode(&mut r)?;
        let code_span = SymbolCodeSpan::decode(&mut r)?;

        let flags = props.elem_props.flags();

        let data = if !flags.contains(properties::PropFlag::CLASS_VAR) {
            let count = props.elem_props.count() as usize;
            match props.elem_props.data_type() {
                DataType::Float => {
                    let mut floats = Vec::with_capacity(count);
                    for _ in 0..count {
                        let v = r.read_f32::<LittleEndian>()?;
                        floats.push(v);
                    }

                    SymbolData::Float(floats)
                }
                DataType::Int => {
                    let mut ints = Vec::with_capacity(count);
                    for _ in 0..count {
                        let v = r.read_i32::<LittleEndian>()?;
                        ints.push(v);
                    }

                    SymbolData::Int(ints)
                }
                DataType::String => {
                    let mut strings = Vec::with_capacity(count);
                    for _ in 0..count {
                        strings.push(ZString::decode(&mut r)?);
                    }

                    SymbolData::String(strings)
                }
                DataType::Class => {
                    let class_offset = r.read_u32::<LittleEndian>()?;
                    SymbolData::ClassOffset(class_offset)
                }
                DataType::Func => {
                    let address = r.read_i32::<LittleEndian>()?;
                    SymbolData::Address(address)
                }
                DataType::Prototype => {
                    let address = r.read_i32::<LittleEndian>()?;
                    SymbolData::Address(address)
                }
                DataType::Instance => {
                    let address = r.read_i32::<LittleEndian>()?;
                    SymbolData::Address(address)
                }
                DataType::Void => SymbolData::None,
            }
        } else {
            SymbolData::None
        };

        let parent = r.read_i32::<LittleEndian>()?;
        let parent = if parent >= 0 {
            Some(parent as u32)
        } else {
            None
        };

        // TODO: Handle this incorrect escape: https://github.com/GothicKit/ZenKit/commit/0e7e507de92e8da4ec28513e6be56e4043329990

        Ok(Symbol {
            name,
            props,
            code_span,
            data,
            parent,
        })
    }

    pub fn encode(&self, mut w: impl Write) -> std::io::Result<()> {
        if let Some(name) = self.name.as_ref() {
            w.write_u32::<LittleEndian>(1)?;
            name.encode(&mut w)?;
        } else {
            w.write_u32::<LittleEndian>(0)?;
        }

        self.props.encode(&mut w)?;
        self.code_span.encode(&mut w)?;

        match &self.data {
            SymbolData::Float(v) => {
                for v in v {
                    w.write_f32::<LittleEndian>(*v)?;
                }
            }
            SymbolData::Int(v) => {
                for v in v {
                    w.write_i32::<LittleEndian>(*v)?;
                }
            }
            SymbolData::String(v) => {
                for v in v {
                    v.encode(&mut w)?;
                }
            }
            SymbolData::ClassOffset(v) => {
                w.write_u32::<LittleEndian>(*v)?;
            }
            SymbolData::Address(v) => {
                w.write_i32::<LittleEndian>(*v)?;
            }
            SymbolData::None => {}
        }

        match self.parent {
            Some(parent) => {
                w.write_i32::<LittleEndian>(parent as i32)?;
            }
            None => {
                w.write_i32::<LittleEndian>(-1)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum SymbolData {
    Float(Vec<f32>),
    Int(Vec<i32>),
    String(Vec<ZString>),
    ClassOffset(u32),
    Address(i32),
    None,
}

#[derive(Debug, PartialEq)]
pub struct DatFile {
    pub version: u8,
    pub sort_indexes: Vec<u32>,
    pub symbols: Vec<Symbol>,
    pub bytecode: Bytecode,
}

impl DatFile {
    pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
        let version = r.read_u8()?;
        let count = r.read_u32::<LittleEndian>()?;

        let mut sort_indexes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let sort_idx = r.read_u32::<LittleEndian>()?;
            sort_indexes.push(sort_idx);
        }

        // Read symbols
        let mut symbols = Vec::with_capacity(count as usize);
        for _sym_index in 0..count {
            symbols.push(Symbol::decode(&mut r)?);
        }

        let bytecode = Bytecode::decode(&mut r)?;

        Ok(Self {
            version,
            sort_indexes,
            symbols,
            bytecode,
        })
    }

    pub fn encode(&self, mut w: impl Write) -> std::io::Result<()> {
        w.write_u8(self.version)?;
        w.write_u32::<LittleEndian>(self.symbols.len() as u32)?;

        for id in self.sort_indexes.iter() {
            w.write_u32::<LittleEndian>(*id)?;
        }

        for symbol in self.symbols.iter() {
            symbol.encode(&mut w)?;
        }

        self.bytecode.encode(w)?;

        Ok(())
    }
}

pub mod properties {
    use super::*;

    bitflags::bitflags! {
        #[derive(Debug, Clone, Copy)]
        pub struct PropFlag: u32 {
            const CONST = 1 << 0;
            const RETURN = 1 << 1;
            const CLASS_VAR = 1 << 2;
            const EXTERNAL = 1 << 3;
            const MERGED = 1 << 4;
        }
    }

    #[repr(u32)]
    #[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
    pub enum DataType {
        Void = 0,
        Float = 1,
        Int = 2,
        String = 3,
        Class = 4,
        Func = 5,
        Prototype = 6,
        Instance = 7,
    }

    impl std::str::FromStr for DataType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let ty = match s {
                "void" => DataType::Void,
                "float" => DataType::Float,
                "int" => DataType::Int,
                "string" => DataType::String,
                "class" => DataType::Class,
                "func" => DataType::Func,
                "prototype" => DataType::Prototype,
                "instance" => DataType::Instance,
                _ => return Err(()),
            };
            Ok(ty)
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct SymbolCodeSpan {
        pub file_index: u19,
        pub line_start: u19,
        pub line_count: u19,
        pub char_start: u24,
        pub char_count: u24,
    }

    impl SymbolCodeSpan {
        pub fn new(
            id: u32,
            (line_start, line_count): (u32, u32),
            (col_start, col_count): (u32, u32),
        ) -> Self {
            Self {
                file_index: u19::new(id),
                line_start: u19::new(line_start),
                line_count: u19::new(line_count),
                char_start: u24::new(col_start),
                char_count: u24::new(col_count),
            }
        }

        pub fn empty(file_id: u32) -> Self {
            Self::new(file_id, (0, 0), (0, 0))
        }

        pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
            let file_index = u19(r.read_u32::<LittleEndian>()?);
            let line_start = u19(r.read_u32::<LittleEndian>()?);
            let line_count = u19(r.read_u32::<LittleEndian>()?);
            let char_start = u24(r.read_u32::<LittleEndian>()?);
            let char_count = u24(r.read_u32::<LittleEndian>()?);

            Ok(Self {
                file_index,
                line_start,
                line_count,
                char_start,
                char_count,
            })
        }

        pub fn encode(&self, mut w: impl Write) -> std::io::Result<()> {
            w.write_u32::<LittleEndian>(self.file_index.0)?;

            w.write_u32::<LittleEndian>(self.line_start.0)?;
            w.write_u32::<LittleEndian>(self.line_count.0)?;

            w.write_u32::<LittleEndian>(self.char_start.0)?;
            w.write_u32::<LittleEndian>(self.char_count.0)?;

            Ok(())
        }
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct Properties {
        pub off_cls_ret: i32,
        pub elem_props: ElemProps,
    }

    impl Properties {
        pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
            let mut this = Self {
                off_cls_ret: r.read_i32::<LittleEndian>()?,
                ..Default::default()
            };

            this.elem_props.0 = r.read_u32::<LittleEndian>()?;

            Ok(this)
        }

        pub fn encode(&self, mut w: impl Write) -> std::io::Result<()> {
            w.write_i32::<LittleEndian>(self.off_cls_ret)?;
            w.write_u32::<LittleEndian>(self.elem_props.0)?;
            Ok(())
        }
    }

    #[derive(Default, Copy, Clone, Eq)]
    pub struct ElemProps(u32);

    impl PartialEq for ElemProps {
        fn eq(&self, other: &Self) -> bool {
            self.0 | Self::E == other.0 | Self::E
        }
    }

    impl ElemProps {
        const A: u32 = 0b00000000000000000000111111111111; // u12 |0
        const B: u32 = 0b00000000000000001111000000000000; // u4  |12
        const C: u32 = 0b00000000001111110000000000000000; // u6  |16
        const D: u32 = 0b00000000010000000000000000000000; // u1  |22
        const E: u32 = 0b11111111100000000000000000000000; // u9  |23

        pub fn set_count(&mut self, v: u32) {
            self.0 |= v & Self::A;
        }
        pub fn set_data_type_raw(&mut self, v: u32) {
            self.0 |= (v << 12) & Self::B;
        }
        pub fn set_flags_raw(&mut self, v: u32) {
            self.0 |= (v << 16) & Self::C;
        }
        pub fn set_space(&mut self, v: u32) {
            self.0 |= (v << 22) & Self::D;
        }
        pub fn set_reserved(&mut self, v: u32) {
            self.0 |= (v << 23) & Self::E;
        }

        pub fn count(&self) -> u32 {
            self.0 & Self::A
        }

        pub fn data_type_raw(&self) -> u32 {
            (self.0 & Self::B) >> 12
        }

        pub fn flags_raw(&self) -> u32 {
            (self.0 & Self::C) >> 16
        }

        pub fn space(&self) -> u32 {
            (self.0 & Self::D) >> 22
        }

        pub fn reserved(&self) -> u32 {
            (self.0 & Self::E) >> 23
        }

        pub fn set_data_type(&mut self, ty: DataType) {
            self.set_data_type_raw(ty as u32);
        }

        pub fn data_type(&self) -> DataType {
            DataType::from_u32(self.data_type_raw()).unwrap()
        }

        pub fn set_flags(&mut self, flags: PropFlag) {
            self.set_flags_raw(flags.bits());
        }

        pub fn flags(&self) -> PropFlag {
            PropFlag::from_bits_retain(self.flags_raw())
        }

        pub fn raw(&self) -> u32 {
            self.0
        }
    }

    #[cfg(test)]
    #[test]
    fn props_bitfields() {
        println!();

        let mut props = ElemProps::default();
        props.set_count(u32::MAX);
        println!("{:032b}", props.raw());
        println!("{:32b}", props.count());

        let mut props = ElemProps::default();
        props.set_data_type_raw(u32::MAX);
        println!("{:032b}", props.raw());
        println!("{:20b}", props.data_type_raw());

        let mut props = ElemProps::default();
        props.set_flags_raw(u32::MAX);
        println!("{:032b}", props.raw());
        println!("{:16b}", props.flags_raw());

        let mut props = ElemProps::default();
        props.set_space(u32::MAX);
        println!("{:032b}", props.raw());
        println!("{:10b}", props.space());

        let mut props = ElemProps::default();
        props.set_reserved(u32::MAX);
        println!("{:032b}", props.raw());
        println!("{:b}", props.reserved());

        println!();
    }

    impl std::fmt::Debug for ElemProps {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ElemProps")
                .field("count", &self.count())
                .field("data_type", &self.data_type())
                .field("flags", &self.flags())
                .field("space", &self.space())
                .finish()
        }
    }

    #[derive(Default, Copy, Clone, Eq)]
    #[allow(non_camel_case_types)]
    pub struct u19(u32);

    impl PartialEq for u19 {
        fn eq(&self, other: &Self) -> bool {
            self.0 | 0xFFF80000 == other.0 | 0xFFF80000
        }
    }

    impl u19 {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }

        pub fn set_value(&mut self, v: u32) {
            self.0 = v & 0x0007FFFF;
        }

        pub fn value(&self) -> u32 {
            self.0 & 0x0007FFFF
        }

        pub fn reserved(&self) -> u32 {
            self.0 & 0xFFF80000
        }
    }

    impl std::fmt::Debug for u19 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    #[derive(Default, Copy, Clone, Eq)]
    #[allow(non_camel_case_types)]
    pub struct u24(u32);

    impl PartialEq for u24 {
        fn eq(&self, other: &Self) -> bool {
            self.0 | 0xFF000000 == other.0 | 0xFF000000
        }
    }

    impl u24 {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set(id);
            this
        }

        pub fn set(&mut self, v: u32) {
            self.0 = v & 0x00FFFFFF;
        }

        pub fn get(&self) -> u32 {
            self.0 & 0x00FFFFFF
        }

        pub fn reserved(&self) -> u32 {
            self.0 & 0xFF000000
        }
    }

    impl std::fmt::Debug for u24 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.get())
        }
    }
}

pub fn debug_print(dat: &DatFile) {
    println!("version: {}", dat.version);
    println!("count: {}", dat.symbols.len());
    // println!("sorted: {:?}", dat.sort_indexes);

    // Read symbols
    for (sym_index, symbol) in dat.symbols.iter().enumerate() {
        if let Some(name) = symbol.name.as_ref() {
            println!("- {name}");
        } else {
            println!("- ?");
        }

        let props = &symbol.props;

        println!("    index: {sym_index} 0x{sym_index:x?}");
        println!("    type: {:?}", props.elem_props.data_type());
        println!("    flags: {:?}", props.elem_props.flags());
        if props.elem_props.flags().contains(PropFlag::RETURN) {
            println!("    return: {:?}", props.elem_props);
        }
        if props.elem_props.flags().contains(PropFlag::RETURN) {
            println!(
                "    return: {:?}",
                DataType::from_i32(props.off_cls_ret).unwrap()
            );
        }
        if props.elem_props.flags().contains(PropFlag::CLASS_VAR) {
            println!("    count: {:?}", props.elem_props.count());
            println!("    offset: {:?}", props.off_cls_ret);
        }

        match &symbol.data {
            SymbolData::Float(v) => {
                for v in v {
                    println!("    float: {v}");
                }
            }
            SymbolData::Int(v) => {
                for v in v {
                    println!("    int: {v}");
                }
            }
            SymbolData::String(v) => {
                for v in v {
                    println!("    string: {v:?}");
                }
            }
            SymbolData::ClassOffset(v) => {
                println!("    class_offset: {v} 0x{v:x?}");
            }
            SymbolData::Address(v) => {
                println!("    address: {v} 0x{v:x?}");
            }
            SymbolData::None => {}
        }

        if let Some(v) = symbol.parent {
            println!("    parent: {v}");
        }
    }

    for i in dat.bytecode.instructions() {
        println!("{i:?}");
    }

    println!();
}
