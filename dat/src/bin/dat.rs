use bstr::BString;
use bytecode::{Bytecode, Instruction};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive as _;
use properties::{DataType, PropFlag, Properties};
use std::io::{Cursor, Read, Write};

#[derive(Debug)]
struct DatSymbol {
    name: Vec<u8>,
    props: Properties,
    data: Vec<u8>,
    parent: i32,
}

#[derive(Debug)]
struct DatBuilder {
    symbols: Vec<DatSymbol>,
    sort_idx: Vec<u32>,
    bytecode: Bytecode,
}

impl DatBuilder {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            sort_idx: Vec::new(),
            bytecode: Bytecode::new(),
        }
    }

    fn gen_mdl_set_visual(&mut self) {
        self.push_symbol(DatSymbol {
            name: b"MDL_SETVISUAL\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(2);
                    default.set_data_type(DataType::Func);
                    default.set_flags(PropFlag::CONST | PropFlag::EXTERNAL);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(0),
                line_start: properties::u18::new(235),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(0),
                char_count: properties::u23::new(66),
            },
            data: (-1i32).to_le_bytes().to_vec(),
            parent: -1,
        });
        self.push_symbol(DatSymbol {
            name: b"MDL_SETVISUAL.PAR0\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Instance);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(0),
                line_start: properties::u18::new(235),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(31),
                char_count: properties::u23::new(17),
            },
            data: 0i32.to_le_bytes().to_vec(),
            parent: -1,
        });
        self.push_symbol(DatSymbol {
            name: b"MDL_SETVISUAL.PAR1\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::String);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(0),
                line_start: properties::u18::new(235),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(50),
                char_count: properties::u23::new(15),
            },
            data: vec![],
            parent: -1,
        });
    }

    fn gen_c_npc(&mut self) -> (u32, u32) {
        let c_npc = self.push_symbol(DatSymbol {
            name: b"C_NPC\n".to_vec(),
            props: Properties {
                off_cls_ret: 800,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(3);
                    default.set_data_type(DataType::Class);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(1),
                line_count: properties::u18::new(37),
                char_start: properties::u23::new(1),
                char_count: properties::u23::new(826),
            },
            data: (288i32).to_le_bytes().to_vec(),
            parent: -1,
        });
        self.push_symbol(DatSymbol {
            name: b"C_NPC.PADDING1\n".to_vec(),
            props: Properties {
                off_cls_ret: 288,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(38);
                    default.set_data_type(DataType::Int);
                    default.set_flags(PropFlag::CLASS_VAR);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(9),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(12),
                char_count: properties::u23::new(8),
            },
            data: vec![],
            parent: 4,
        });
        let c_npc_attribute = self.push_symbol(DatSymbol {
            name: b"C_NPC.ATTRIBUTE\n".to_vec(),
            props: Properties {
                off_cls_ret: 440,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(8);
                    default.set_data_type(DataType::Int);
                    default.set_flags(PropFlag::CLASS_VAR);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(10),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(12),
                char_count: properties::u23::new(9),
            },
            data: vec![],
            parent: 4,
        });
        self.push_symbol(DatSymbol {
            name: b"C_NPC.PADDING2\n".to_vec(),
            props: Properties {
                off_cls_ret: 472,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(154);
                    default.set_data_type(DataType::Int);
                    default.set_flags(PropFlag::CLASS_VAR);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(11),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(12),
                char_count: properties::u23::new(8),
            },
            data: vec![],
            parent: 4,
        });
        (c_npc, c_npc_attribute)
    }

    fn gen_pc_hero(&mut self) -> u32 {
        self.push_symbol(DatSymbol {
            name: b"PC_HERO\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Instance);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(61),
                line_count: properties::u18::new(5),
                char_start: properties::u23::new(0),
                char_count: properties::u23::new(111),
            },
            data: 0u32.to_le_bytes().to_vec(),
            parent: 4,
        })
    }

    pub fn gen_startup_global(&mut self) -> u32 {
        self.push_symbol(DatSymbol {
            name: b"STARTUP_GLOBAL\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Func);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(67),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(0),
                char_count: properties::u23::new(29),
            },
            data: 0u32.to_le_bytes().to_vec(),
            parent: -1,
        })
    }

    pub fn gen_init_global(&mut self) -> u32 {
        self.push_symbol(DatSymbol {
            name: b"INIT_GLOBAL\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Func);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(68),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(0),
                char_count: properties::u23::new(26),
            },
            data: 0u32.to_le_bytes().to_vec(),
            parent: -1,
        })
    }

    pub fn gen_f10000(&mut self) -> u32 {
        self.push_symbol(DatSymbol {
            name: b"\xFF10000\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(1);
                    default.set_data_type(DataType::String);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: properties::u18::new(1),
                line_start: properties::u18::new(64),
                line_count: properties::u18::new(1),
                char_start: properties::u23::new(23),
                char_count: properties::u23::new(12),
            },
            data: b"HUMANS.MDS\n".to_vec(),
            parent: -1,
        })
    }

    pub fn example() -> Self {
        let mut dat = Self::new();

        dat.push_symbol(DatSymbol {
            name: b"\xFFINSTANCE_HELP\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(1);
                    default.set_data_type(DataType::Instance);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                ..Default::default()
            },
            data: 0i32.to_le_bytes().to_vec(),
            parent: -1,
        });

        dat.gen_mdl_set_visual();
        let (_c_npc_id, c_ncp_attribute_id) = dat.gen_c_npc();

        let pc_hero_id = dat.gen_pc_hero();

        let startup_global_id = dat.gen_startup_global();
        let init_global_id = dat.gen_init_global();

        let f10000_id = dat.gen_f10000();

        let pc_hero_addr = dat.bytecode.block(&[
            // attribute[0] = 40
            Instruction::push_i(20),
            Instruction::push_v(c_ncp_attribute_id),
            Instruction::mov_i(),
            // attribute[1] = 40
            Instruction::push_i(40),
            Instruction::push_vv(c_ncp_attribute_id, 1),
            Instruction::mov_i(),
            // Mdl_SetVisual(self, "HUMANS.MDS")
            Instruction::push_vi(pc_hero_id),
            Instruction::push_v(f10000_id),
            Instruction::be(1),
            Instruction::rsr(),
        ]);
        dat.symbols[pc_hero_id as usize].data = pc_hero_addr.to_le_bytes().to_vec();

        let startup_global_addr = dat.bytecode.block(&[Instruction::rsr()]);
        dat.symbols[startup_global_id as usize].data = startup_global_addr.to_le_bytes().to_vec();

        let init_global_addr = dat.bytecode.block(&[Instruction::rsr()]);
        dat.symbols[init_global_id as usize].data = init_global_addr.to_le_bytes().to_vec();

        let mut symbol_ids: Vec<_> = dat
            .symbols
            .iter()
            .enumerate()
            .map(|(i, s)| (i, &s.name))
            .collect();

        // Symbols map is sorted in alphabetical order
        symbol_ids.sort_by_key(|v| v.1.as_slice());
        dat.sort_idx = symbol_ids.iter().map(|(id, _)| *id as u32).collect();

        dat
    }

    pub fn push_symbol(&mut self, symbol: DatSymbol) -> u32 {
        let id = self.symbols.len();
        self.symbols.push(symbol);
        id as u32
    }

    pub fn build(&self) -> Vec<u8> {
        let mut out = vec![];

        out.write_u8(b'2').unwrap();
        out.write_u32::<LittleEndian>(self.symbols.len() as u32)
            .unwrap();

        for id in self.sort_idx.iter() {
            out.write_u32::<LittleEndian>(*id).unwrap();
        }

        for symbol in self.symbols.iter() {
            out.write_u32::<LittleEndian>(1).unwrap();
            out.write_all(&symbol.name).unwrap();

            symbol.props.encode(&mut out).unwrap();

            out.write_all(&symbol.data).unwrap();
            out.write_i32::<LittleEndian>(symbol.parent).unwrap();
        }

        self.bytecode.encode(&mut out).unwrap();

        out
    }
}

fn main() {
    let data = std::fs::read("/home/poly/Gothic2/_work/Data/Scripts/_compiled/GOTHIC.DAT").unwrap();
    println!("{:?}", &data.len());
    run(data);

    // println!();

    // let data = DatBuilder::example().build();
    // std::fs::write("./OUT.DAT", &data).unwrap();
    // println!("{:?}", &data.len());
    // run(data);
}

fn run(data: Vec<u8>) {
    let mut data = Cursor::new(data);

    let version = data.read_u8().unwrap();
    let count = data.read_u32::<LittleEndian>().unwrap();

    println!("version: {version}");
    println!("count: {count}");

    for _ in 0..count {
        let sort_idx = data.read_u32::<LittleEndian>().unwrap();
        println!("dat.sort_idx.push({sort_idx})");
    }

    // Read symbols
    for sym_index in 0..count {
        let named = data.read_u32::<LittleEndian>().unwrap();

        assert!(named == 0 || named == 1, "has_name: 0x{named:x?}");

        let mut name;
        let name = if named == 1 {
            name = Vec::new();
            let mut b = data.read_u8().unwrap();
            while b != b'\n' {
                if b == 0xFF {
                    name.push(b'$');
                } else {
                    name.push(b);
                }
                b = data.read_u8().unwrap();
            }

            Some(BString::new(name))
        } else {
            None
        };

        if let Some(name) = name {
            println!("- {name}");
        } else {
            println!("- ?");
        }

        let props = properties::Properties::decode(&mut data).unwrap();
        // println!("{props:#?}");

        println!("    sym_index: {sym_index} 0x{sym_index:x?}");

        // dbg!(&props);

        let flags = props.elem_props.flags();

        if !flags.contains(properties::PropFlag::CLASS_VAR) {
            use properties::*;
            match props.elem_props.data_type() {
                DataType::Float => {
                    let count = props.elem_props.count() as usize;
                    let mut buf = vec![0; std::mem::size_of::<f32>() * count];
                    data.read_exact(&mut buf).unwrap();

                    for _ in 0..count {
                        let int = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        println!("    float: {int}");
                    }
                }
                DataType::Int => {
                    let count = props.elem_props.count() as usize;
                    let mut buf = vec![0; std::mem::size_of::<u32>() * count];
                    data.read_exact(&mut buf).unwrap();

                    for _ in 0..count {
                        let int = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        println!("    int: {int}");
                    }
                }
                DataType::String => {
                    let count = props.elem_props.count() as usize;
                    for _ in 0..count {
                        let mut string = Vec::new();
                        let mut b = data.read_u8().unwrap();
                        while b != b'\n' {
                            if b == 0xFF {
                                string.push(b'$');
                            } else {
                                string.push(b);
                            }
                            b = data.read_u8().unwrap();
                        }

                        let string = BString::new(string);
                        println!("    string: {string:?}");
                    }
                }
                DataType::Class => {
                    let class_offset = data.read_i32::<LittleEndian>().unwrap();
                    println!("    class_offset: {class_offset} 0x{class_offset:x?}");
                }
                DataType::Func => {
                    let address = data.read_i32::<LittleEndian>().unwrap();
                    println!("    func_address: {address} 0x{address:x?}");
                    if flags.contains(PropFlag::EXTERNAL) {
                        println!("    external: true");
                    }
                }
                DataType::Prototype => {
                    let address = data.read_i32::<LittleEndian>().unwrap();
                    println!("    prototype_address: {address}");
                    if flags.contains(PropFlag::EXTERNAL) {
                        println!("    external: true");
                    }
                }
                DataType::Instance => {
                    let address = data.read_i32::<LittleEndian>().unwrap();
                    println!("    instance_address: {address} 0x{address:x?}");
                    if flags.contains(PropFlag::EXTERNAL) {
                        println!("    external: true");
                    }
                }
                DataType::Void => {}
            }
        } else {
            println!("    EPAR_FLAG_CLASS_VAR: true");
        }

        let parent = data.read_i32::<LittleEndian>().unwrap();
        if parent != -1 {
            println!("    parent: {parent}");
        }

        // TODO: Handle this incorrect escape: https://github.com/GothicKit/ZenKit/commit/0e7e507de92e8da4ec28513e6be56e4043329990
    }

    let bytecode = Bytecode::decode(&mut data).unwrap();
    let bytecode_len = bytecode.as_bytes().len();

    let mut addr = 0;
    let mut bytecode = Cursor::new(bytecode.as_bytes());

    loop {
        let i = Instruction::decode(&mut bytecode).unwrap();
        println!("{i:?}");
        addr += i.size();

        if addr >= bytecode_len {
            break;
        }
    }

    println!();
}

// TODO: Replace with just regular manual parsing
mod properties {
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

    #[derive(Debug, Default, Copy, Clone)]
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

    #[derive(Default, Copy, Clone)]
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

    #[derive(Default, Copy, Clone)]
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

    #[derive(Default, Copy, Clone)]
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
