use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use properties::{
    Properties, EPAR_TYPE_CLASS, EPAR_TYPE_INSTANCE, EPAR_TYPE_INT, EPAR_TYPE_STRING,
};
use std::io::{Cursor, Read, Write};

use crate::properties::EPAR_TYPE_FUNC;

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
    instructions: Vec<u8>,
}

impl DatBuilder {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            sort_idx: Vec::new(),
            instructions: Vec::new(),
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
                    default.set_type_0(EPAR_TYPE_FUNC);
                    default.set_flags(9);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(0),
                line_start: properties::LineStart::new(235),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(0),
                char_count: properties::CharCount::new(66),
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
                    default.set_type_0(EPAR_TYPE_INSTANCE);
                    default.set_flags(0);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(0),
                line_start: properties::LineStart::new(235),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(31),
                char_count: properties::CharCount::new(17),
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
                    default.set_type_0(EPAR_TYPE_STRING);
                    default.set_flags(0);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(0),
                line_start: properties::LineStart::new(235),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(50),
                char_count: properties::CharCount::new(15),
            },
            data: vec![],
            parent: -1,
        });
    }

    fn gen_c_npc(&mut self) {
        self.push_symbol(DatSymbol {
            name: b"C_NPC\n".to_vec(),
            props: Properties {
                off_cls_ret: 800,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(3);
                    default.set_type_0(EPAR_TYPE_CLASS);
                    default.set_flags(0);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(1),
                line_count: properties::LineCount::new(37),
                char_start: properties::CharStart::new(1),
                char_count: properties::CharCount::new(826),
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
                    default.set_type_0(EPAR_TYPE_INT);
                    default.set_flags(4);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(9),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(12),
                char_count: properties::CharCount::new(8),
            },
            data: vec![],
            parent: 4,
        });
        self.push_symbol(DatSymbol {
            name: b"C_NPC.ATTRIBUTE\n".to_vec(),
            props: Properties {
                off_cls_ret: 440,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(8);
                    default.set_type_0(EPAR_TYPE_INT);
                    default.set_flags(4);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(10),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(12),
                char_count: properties::CharCount::new(9),
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
                    default.set_type_0(EPAR_TYPE_INT);
                    default.set_flags(4);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(11),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(12),
                char_count: properties::CharCount::new(8),
            },
            data: vec![],
            parent: 4,
        });
    }

    fn gen_pc_hero(&mut self) {
        self.push_symbol(DatSymbol {
            name: b"PC_HERO\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_type_0(EPAR_TYPE_INSTANCE);
                    default.set_flags(1);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(61),
                line_count: properties::LineCount::new(5),
                char_start: properties::CharStart::new(0),
                char_count: properties::CharCount::new(111),
            },
            data: 0i32.to_le_bytes().to_vec(),
            parent: 4,
        });
    }

    pub fn gen_startup_global(&mut self) {
        self.push_symbol(DatSymbol {
            name: b"STARTUP_GLOBAL\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_type_0(EPAR_TYPE_FUNC);
                    default.set_flags(1);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(67),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(0),
                char_count: properties::CharCount::new(29),
            },
            data: 0x27_i32.to_le_bytes().to_vec(),
            parent: -1,
        });
    }

    pub fn gen_init_global(&mut self) {
        self.push_symbol(DatSymbol {
            name: b"INIT_GLOBAL\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(0);
                    default.set_type_0(EPAR_TYPE_FUNC);
                    default.set_flags(1);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(68),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(0),
                char_count: properties::CharCount::new(26),
            },
            data: 0x28_i32.to_le_bytes().to_vec(),
            parent: -1,
        });
    }

    pub fn gen_f10000(&mut self) {
        self.push_symbol(DatSymbol {
            name: b"\xFF10000\n".to_vec(),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = properties::ElemProps::default();
                    default.set_count(1);
                    default.set_type_0(EPAR_TYPE_STRING);
                    default.set_flags(1);
                    default.set_space(1);
                    default
                },
                file_index: properties::FileIndex::new(1),
                line_start: properties::LineStart::new(64),
                line_count: properties::LineCount::new(1),
                char_start: properties::CharStart::new(23),
                char_count: properties::CharCount::new(12),
            },
            data: b"HUMANS.MDS\n".to_vec(),
            parent: -1,
        });
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
                    default.set_type_0(EPAR_TYPE_INSTANCE);
                    default.set_flags(0);
                    default.set_space(1);
                    default
                },
                ..Default::default()
            },
            data: 0i32.to_le_bytes().to_vec(),
            parent: -1,
        });

        dat.gen_mdl_set_visual();
        dat.gen_c_npc();
        dat.gen_pc_hero();
        dat.gen_startup_global();
        dat.gen_init_global();
        dat.gen_f10000();

        dat.sort_idx.push(4);
        dat.sort_idx.push(6);
        dat.sort_idx.push(5);
        dat.sort_idx.push(7);
        dat.sort_idx.push(10);
        dat.sort_idx.push(1);
        dat.sort_idx.push(2);
        dat.sort_idx.push(3);
        dat.sort_idx.push(8);
        dat.sort_idx.push(9);
        dat.sort_idx.push(11);
        dat.sort_idx.push(0);

        // dat.push_return(); // 00
        // dat.push_return(); // 01
        // dat.push_return(); // 02

        dat.instructions = vec![
            0x40, 0x28, 0x0, 0x0, 0x0, 0x41, 0x6, 0x0, 0x0, 0x0, 0x9, 0x40, 0x28, 0x0, 0x0, 0x0,
            0xf5, 0x6, 0x0, 0x0, 0x0, 0x1, 0x9, 0x43, 0x8, 0x0, 0x0, 0x0, 0x41, 0xb, 0x0, 0x0, 0x0,
            0x3e, 0x1, 0x0, 0x0, 0x0, 0x3c, 0x3c, 0x3c,
        ];

        dat
    }

    pub fn push_symbol(&mut self, symbol: DatSymbol) {
        self.symbols.push(symbol);
    }

    pub fn push_return(&mut self) {
        self.instructions.push(0x3c);
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

            let props = &symbol.props as *const _ as *const u8;
            let props =
                unsafe { std::slice::from_raw_parts(props, std::mem::size_of::<Properties>()) };

            out.write_all(props).unwrap();
            out.write_all(&symbol.data).unwrap();
            out.write_i32::<LittleEndian>(symbol.parent).unwrap();
        }

        out.write_u32::<LittleEndian>(self.instructions.len() as u32)
            .unwrap();

        out.write_all(&self.instructions).unwrap();

        out
    }
}

fn main() {
    // let data = std::fs::read("/home/poly/Gothic2/_work/Data/Scripts/_compiled/GOTHIC.DAT").unwrap();
    // println!("{:?}", &data.len());
    // run(data);

    // println!();

    let data = DatBuilder::example().build();
    std::fs::write("./OUT.DAT", &data).unwrap();
    println!("{:?}", &data.len());
    run(data);
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

        let name = if named == 1 {
            let mut name = Vec::new();
            let mut b = data.read_u8().unwrap();
            while b != b'\n' {
                if b == 0xFF {
                    name.push(b'$');
                } else {
                    name.push(b);
                }
                b = data.read_u8().unwrap();
            }

            Some(String::from_utf8(name).unwrap())
        } else {
            None
        };

        if let Some(name) = name {
            println!("- {name}");
        } else {
            println!("- ?");
        }

        let mut buf = [0; std::mem::size_of::<properties::Properties>()];
        data.read_exact(&mut buf).unwrap();
        let props: properties::Properties = unsafe { std::mem::transmute(buf) };
        println!("{props:#?}");

        println!("    sym_index: {sym_index} 0x{sym_index:x?}");

        // dbg!(&props);

        if (props.elem_props.flags() & properties::EPAR_FLAG_CLASS_VAR) == 0 {
            use properties::*;
            match props.elem_props.type_0() {
                EPAR_TYPE_FLOAT => {
                    let count = props.elem_props.count() as usize;
                    let mut buf = vec![0; std::mem::size_of::<f32>() * count];
                    data.read_exact(&mut buf).unwrap();

                    for _ in 0..count {
                        let int = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        println!("    float: {int}");
                    }
                }
                EPAR_TYPE_INT => {
                    let count = props.elem_props.count() as usize;
                    let mut buf = vec![0; std::mem::size_of::<u32>() * count];
                    data.read_exact(&mut buf).unwrap();

                    for _ in 0..count {
                        let int = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        println!("    int: {int}");
                    }
                }
                EPAR_TYPE_STRING => {
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

                        let string = std::str::from_utf8(&string).unwrap();
                        println!("    string: {string:?}");
                    }
                }
                EPAR_TYPE_CLASS => {
                    let class_offset = data.read_i32::<LittleEndian>().unwrap();
                    println!("    class_offset: {class_offset} 0x{class_offset:x?}");
                }
                EPAR_TYPE_FUNC => {
                    let address = data.read_i32::<LittleEndian>().unwrap();
                    println!("    func_address: {address} 0x{address:x?}");
                    if (props.elem_props.flags() & EPAR_FLAG_EXTERNAL) != 0 {
                        println!("    external_address: {address}");
                    }
                }
                EPAR_TYPE_PROTOTYPE => {
                    let address = data.read_i32::<LittleEndian>().unwrap();
                    println!("    prototype_address: {address}");
                    if (props.elem_props.flags() & EPAR_FLAG_EXTERNAL) != 0 {
                        println!("    external_address: {address}");
                    }
                }
                EPAR_TYPE_INSTANCE => {
                    let address = data.read_i32::<LittleEndian>().unwrap();
                    println!("    instance_address: {address} 0x{address:x?}");
                    if (props.elem_props.flags() & EPAR_FLAG_EXTERNAL) != 0 {
                        println!("    external_address: {address}");
                    }
                }
                _ => {
                    todo!()
                }
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

    let bytecode_size = data.read_u32::<LittleEndian>().unwrap() as usize;
    let mut bytecode = vec![0; bytecode_size];
    data.read_exact(&mut bytecode).unwrap();

    println!();

    for (id, _byte) in bytecode.iter().enumerate() {
        print!("{id:3x?}");
    }

    println!();

    for byte in bytecode.iter() {
        print!("{byte:3x?}");
    }

    println!();
}

// TODO: Replace with just regular manual parsing
mod properties {
    #![allow(dead_code)]

    use c2rust_bitfields::BitfieldStruct;

    pub const EPAR_FLAG_CONST: u32 = 1 << 0;
    pub const EPAR_FLAG_RETURN: u32 = 1 << 1;
    pub const EPAR_FLAG_CLASS_VAR: u32 = 1 << 2;
    pub const EPAR_FLAG_EXTERNAL: u32 = 1 << 3;
    pub const EPAR_FLAG_MERGED: u32 = 1 << 4;

    pub const EPAR_TYPE_VOID: u32 = 0;
    pub const EPAR_TYPE_FLOAT: u32 = 1;
    pub const EPAR_TYPE_INT: u32 = 2;
    pub const EPAR_TYPE_STRING: u32 = 3;
    pub const EPAR_TYPE_CLASS: u32 = 4;
    pub const EPAR_TYPE_FUNC: u32 = 5;
    pub const EPAR_TYPE_PROTOTYPE: u32 = 6;
    pub const EPAR_TYPE_INSTANCE: u32 = 7;

    #[derive(Debug, Default, Copy, Clone)]
    #[repr(C)]
    pub struct Properties {
        pub off_cls_ret: i32,
        pub elem_props: ElemProps,
        pub file_index: FileIndex,
        pub line_start: LineStart,
        pub line_count: LineCount,
        pub char_start: CharStart,
        pub char_count: CharCount,
    }

    #[derive(Default, Copy, Clone, BitfieldStruct)]
    #[repr(C)]
    pub struct ElemProps {
        #[bitfield(name = "count", ty = "u32", bits = "0..=11")]
        #[bitfield(name = "type_0", ty = "u32", bits = "12..=15")]
        #[bitfield(name = "flags", ty = "u32", bits = "16..=21")]
        #[bitfield(name = "space", ty = "u32", bits = "22..=22")]
        #[bitfield(name = "reserved", ty = "u32", bits = "23..=31")]
        pub count_type_0_flags_space_reserved: [u8; 4],
    }

    impl std::fmt::Debug for ElemProps {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ElemProps")
                .field("count", &self.count())
                .field("type_0", &self.type_0())
                .field("flags", &self.flags())
                .field("space", &self.space())
                .finish()
        }
    }

    #[derive(Default, Copy, Clone, BitfieldStruct)]
    #[repr(C)]
    pub struct FileIndex {
        #[bitfield(name = "value", ty = "u32", bits = "0..=18")]
        #[bitfield(name = "reserved", ty = "u32", bits = "19..=31")]
        pub value_reserved: [u8; 4],
    }

    impl FileIndex {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }
    }

    impl std::fmt::Debug for FileIndex {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    #[derive(Default, Copy, Clone, BitfieldStruct)]
    #[repr(C)]
    pub struct LineStart {
        #[bitfield(name = "value", ty = "u32", bits = "0..=18")]
        #[bitfield(name = "reserved", ty = "u32", bits = "19..=31")]
        pub value_reserved: [u8; 4],
    }

    impl LineStart {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }
    }

    impl std::fmt::Debug for LineStart {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    #[derive(Default, Copy, Clone, BitfieldStruct)]
    #[repr(C)]
    pub struct LineCount {
        #[bitfield(name = "value", ty = "u32", bits = "0..=18")]
        #[bitfield(name = "reserved", ty = "u32", bits = "19..=31")]
        pub value_reserved: [u8; 4],
    }

    impl LineCount {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }
    }

    impl std::fmt::Debug for LineCount {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    #[derive(Default, Copy, Clone, BitfieldStruct)]
    #[repr(C)]
    pub struct CharStart {
        #[bitfield(name = "value", ty = "u32", bits = "0..=23")]
        #[bitfield(name = "reserved", ty = "u32", bits = "24..=31")]
        pub value_reserved: [u8; 4],
    }

    impl CharStart {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }
    }

    impl std::fmt::Debug for CharStart {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    #[derive(Default, Copy, Clone, BitfieldStruct)]
    #[repr(C)]
    pub struct CharCount {
        #[bitfield(name = "value", ty = "u32", bits = "0..=23")]
        #[bitfield(name = "reserved", ty = "u32", bits = "24..=31")]
        pub value_reserved: [u8; 4],
    }

    impl CharCount {
        pub fn new(id: u32) -> Self {
            let mut this = Self::default();
            this.set_value(id);
            this
        }
    }

    impl std::fmt::Debug for CharCount {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value())
        }
    }
}
