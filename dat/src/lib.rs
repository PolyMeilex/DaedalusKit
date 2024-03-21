use bytecode::Bytecode;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive as _;
use properties::Properties;
use std::io::{Read, Write};

mod zstring;
pub use zstring::ZString;

use crate::properties::DataType;

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub name: Option<ZString>,
    pub props: Properties,
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
                    let address = r.read_u32::<LittleEndian>()?;
                    SymbolData::Address(address)
                }
                DataType::Prototype => {
                    let address = r.read_u32::<LittleEndian>()?;
                    SymbolData::Address(address)
                }
                DataType::Instance => {
                    let address = r.read_u32::<LittleEndian>()?;
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
                w.write_u32::<LittleEndian>(*v)?;
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
    Address(u32),
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
    #![allow(dead_code)]

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
    #[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
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

    type BitRange = (usize, usize);

    fn set_field(field: &mut [u8], int: u32, bit_range: BitRange) {
        fn zero_bit(byte: &mut u8, n_bit: u64) {
            let bit = 1 << n_bit;
            *byte &= !bit as u8;
        }

        fn one_bit(byte: &mut u8, n_bit: u64) {
            let bit = 1 << n_bit;
            *byte |= bit as u8;
        }

        fn get_bit(int: u32, bit: usize) -> bool {
            ((int >> bit) & 1) == 1
        }

        let (lhs_bit, rhs_bit) = bit_range;

        for (i, bit_index) in (lhs_bit..=rhs_bit).enumerate() {
            let byte_index = bit_index / 8;
            let byte = &mut field[byte_index];

            if get_bit(int, i) {
                one_bit(byte, (bit_index % 8) as u64);
            } else {
                zero_bit(byte, (bit_index % 8) as u64);
            }
        }
    }

    fn get_field(field: &[u8], bit_range: (usize, usize)) -> u32 {
        let (lhs_bit, rhs_bit) = bit_range;
        let mut val = 0;

        for (i, bit_index) in (lhs_bit..=rhs_bit).enumerate() {
            let byte_index = bit_index / 8;
            let byte = field[byte_index];
            let bit = 1 << (bit_index % 8);
            let read_bit = byte & bit;

            if read_bit != 0 {
                let write_bit = 1 << i;
                val |= write_bit;
            }
        }

        val
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct Properties {
        pub off_cls_ret: i32,
        pub elem_props: ElemProps,
        pub file_index: u18,
        pub line_start: u18,
        pub line_count: u18,
        pub char_start: u23,
        pub char_count: u23,
    }

    impl Properties {
        pub fn decode(mut r: impl Read) -> std::io::Result<Self> {
            let mut this = Self {
                off_cls_ret: r.read_i32::<LittleEndian>()?,
                ..Default::default()
            };

            r.read_exact(&mut this.elem_props.0)?;

            r.read_exact(&mut this.file_index.0)?;

            r.read_exact(&mut this.line_start.0)?;
            r.read_exact(&mut this.line_count.0)?;

            r.read_exact(&mut this.char_start.0)?;
            r.read_exact(&mut this.char_count.0)?;

            Ok(this)
        }

        pub fn encode(&self, mut w: impl Write) -> std::io::Result<()> {
            w.write_i32::<LittleEndian>(self.off_cls_ret)?;
            w.write_all(&self.elem_props.0)?;

            w.write_all(&self.file_index.0)?;

            w.write_all(&self.line_start.0)?;
            w.write_all(&self.line_count.0)?;

            w.write_all(&self.char_start.0)?;
            w.write_all(&self.char_count.0)?;

            Ok(())
        }
    }

    #[derive(Default, Copy, Clone, PartialEq, Eq)]
    pub struct ElemProps([u8; 4]);

    impl ElemProps {
        const COUNT: BitRange = (0, 11);
        const DATA_TYPE: BitRange = (12, 15);
        const FLAGS: BitRange = (16, 21);
        const SPACE: BitRange = (22, 22);
        const RESERVED: BitRange = (23, 31);

        fn set_field(&mut self, int: u32, bit_range: BitRange) {
            set_field(&mut self.0, int, bit_range)
        }

        fn get_field(&self, bit_range: BitRange) -> u32 {
            get_field(&self.0, bit_range)
        }

        pub fn set_count(&mut self, int: u32) {
            self.set_field(int, Self::COUNT);
        }

        pub fn count(&self) -> u32 {
            self.get_field(Self::COUNT)
        }

        pub fn set_data_type(&mut self, ty: DataType) {
            self.set_field(ty as u32, Self::DATA_TYPE);
        }

        pub fn data_type(&self) -> DataType {
            DataType::from_u32(self.get_field(Self::DATA_TYPE)).unwrap()
        }

        pub fn set_flags(&mut self, flags: PropFlag) {
            self.set_field(flags.bits(), Self::FLAGS);
        }

        pub fn flags(&self) -> PropFlag {
            PropFlag::from_bits_retain(self.get_field(Self::FLAGS))
        }

        pub fn set_space(&mut self, int: u32) {
            self.set_field(int, Self::SPACE);
        }

        pub fn space(&self) -> u32 {
            self.get_field(Self::SPACE)
        }

        pub fn reserved(&self) -> u32 {
            self.get_field(Self::RESERVED)
        }
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

    #[derive(Default, Copy, Clone, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    pub struct u18([u8; 4]);

    impl u18 {
        const VALUE: BitRange = (0, 18);
        const RESERVED: BitRange = (19, 31);

        pub fn set_value(&mut self, int: u32) {
            set_field(&mut self.0, int, Self::VALUE)
        }

        pub fn value(&self) -> u32 {
            get_field(&self.0, Self::VALUE)
        }

        pub fn reserved(&self) -> u32 {
            get_field(&self.0, Self::RESERVED)
        }
    }

    impl u18 {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }
    }

    impl std::fmt::Debug for u18 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    #[derive(Default, Copy, Clone, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    pub struct u23([u8; 4]);

    impl u23 {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }

        const VALUE: BitRange = (0, 23);
        const RESERVED: BitRange = (24, 31);

        pub fn set_value(&mut self, int: u32) {
            set_field(&mut self.0, int, Self::VALUE)
        }

        pub fn value(&self) -> u32 {
            get_field(&self.0, Self::VALUE)
        }

        pub fn reserved(&self) -> u32 {
            get_field(&self.0, Self::RESERVED)
        }
    }

    impl std::fmt::Debug for u23 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }
}
