use bytecode::{Bytecode, Instruction};
use byteorder::{LittleEndian, WriteBytesExt};
use dat::{
    properties::{u19, u24, DataType, ElemProps, PropFlag, Properties},
    Symbol, SymbolData, ZString,
};
use std::io::Cursor;

#[derive(Debug)]
struct DatBuilder {
    symbols: Vec<Symbol>,
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
        self.push_symbol(Symbol {
            name: Some(ZString::from("MDL_SETVISUAL")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(2);
                    default.set_data_type(DataType::Func);
                    default.set_flags(PropFlag::CONST | PropFlag::EXTERNAL);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(0),
                line_start: u19::new(235),
                line_count: u19::new(1),
                char_start: u24::new(0),
                char_count: u24::new(66),
            },
            data: SymbolData::Address(-1i32),
            parent: None,
        });
        self.push_symbol(Symbol {
            name: Some(ZString::from("MDL_SETVISUAL.PAR0")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Instance);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                file_index: u19::new(0),
                line_start: u19::new(235),
                line_count: u19::new(1),
                char_start: u24::new(31),
                char_count: u24::new(17),
            },
            data: SymbolData::Address(0),
            parent: None,
        });
        self.push_symbol(Symbol {
            name: Some(ZString::from("MDL_SETVISUAL.PAR1")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::String);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                file_index: u19::new(0),
                line_start: u19::new(235),
                line_count: u19::new(1),
                char_start: u24::new(50),
                char_count: u24::new(15),
            },
            data: SymbolData::None,
            parent: None,
        });
    }

    fn gen_c_npc(&mut self) -> (u32, u32) {
        let c_npc = self.push_symbol(Symbol {
            name: Some(ZString::from("C_NPC")),
            props: Properties {
                off_cls_ret: 800,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(3);
                    default.set_data_type(DataType::Class);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(1),
                line_count: u19::new(37),
                char_start: u24::new(1),
                char_count: u24::new(826),
            },
            data: SymbolData::Address(288),
            parent: None,
        });
        self.push_symbol(Symbol {
            name: Some(ZString::from("C_NPC.PADDING1")),
            props: Properties {
                off_cls_ret: 288,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(38);
                    default.set_data_type(DataType::Int);
                    default.set_flags(PropFlag::CLASS_VAR);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(9),
                line_count: u19::new(1),
                char_start: u24::new(12),
                char_count: u24::new(8),
            },
            data: SymbolData::None,
            parent: Some(4),
        });
        let c_npc_attribute = self.push_symbol(Symbol {
            name: Some(ZString::from("C_NPC.ATTRIBUTE")),
            props: Properties {
                off_cls_ret: 440,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(8);
                    default.set_data_type(DataType::Int);
                    default.set_flags(PropFlag::CLASS_VAR);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(10),
                line_count: u19::new(1),
                char_start: u24::new(12),
                char_count: u24::new(9),
            },
            data: SymbolData::None,
            parent: Some(4),
        });
        self.push_symbol(Symbol {
            name: Some(ZString::from("C_NPC.PADDING2")),
            props: Properties {
                off_cls_ret: 472,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(154);
                    default.set_data_type(DataType::Int);
                    default.set_flags(PropFlag::CLASS_VAR);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(11),
                line_count: u19::new(1),
                char_start: u24::new(12),
                char_count: u24::new(8),
            },
            data: SymbolData::None,
            parent: Some(4),
        });
        (c_npc, c_npc_attribute)
    }

    fn gen_pc_hero(&mut self) -> u32 {
        self.push_symbol(Symbol {
            name: Some(ZString::from("PC_HERO")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Instance);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(61),
                line_count: u19::new(5),
                char_start: u24::new(0),
                char_count: u24::new(111),
            },
            data: SymbolData::Address(0),
            parent: Some(4),
        })
    }

    pub fn gen_startup_global(&mut self) -> u32 {
        self.push_symbol(Symbol {
            name: Some(ZString::from("STARTUP_GLOBAL")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Func);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(67),
                line_count: u19::new(1),
                char_start: u24::new(0),
                char_count: u24::new(29),
            },
            data: SymbolData::Address(0),
            parent: None,
        })
    }

    pub fn gen_init_global(&mut self) -> u32 {
        self.push_symbol(Symbol {
            name: Some(ZString::from("INIT_GLOBAL")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(0);
                    default.set_data_type(DataType::Func);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(68),
                line_count: u19::new(1),
                char_start: u24::new(0),
                char_count: u24::new(26),
            },
            data: SymbolData::Address(0),
            parent: None,
        })
    }

    pub fn gen_f10000(&mut self) -> u32 {
        self.push_symbol(Symbol {
            name: Some(ZString::from(b"\xFF10000")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(1);
                    default.set_data_type(DataType::String);
                    default.set_flags(PropFlag::CONST);
                    default.set_space(1);
                    default
                },
                file_index: u19::new(1),
                line_start: u19::new(64),
                line_count: u19::new(1),
                char_start: u24::new(23),
                char_count: u24::new(12),
            },
            data: SymbolData::String(vec![ZString::from("HUMANS.MDS")]),
            parent: None,
        })
    }

    pub fn example() -> Self {
        let mut dat = Self::new();

        dat.push_symbol(Symbol {
            name: Some(ZString::from(b"\xFFINSTANCE_HELP")),
            props: Properties {
                off_cls_ret: 0,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(1);
                    default.set_data_type(DataType::Instance);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
                ..Default::default()
            },
            data: SymbolData::Address(0),
            parent: None,
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
        dat.symbols[pc_hero_id as usize].data = SymbolData::Address(pc_hero_addr as i32);

        let startup_global_addr = dat.bytecode.block(&[Instruction::rsr()]);
        dat.symbols[startup_global_id as usize].data =
            SymbolData::Address(startup_global_addr as i32);

        let init_global_addr = dat.bytecode.block(&[Instruction::rsr()]);
        dat.symbols[init_global_id as usize].data = SymbolData::Address(init_global_addr as i32);

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

    pub fn push_symbol(&mut self, symbol: Symbol) -> u32 {
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
            symbol.encode(&mut out).unwrap();
        }

        self.bytecode.encode(&mut out).unwrap();

        out
    }
}

fn main() {
    // let data = std::fs::read("OUT.DAT").unwrap();
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
    let dat = dat::DatFile::decode(&mut Cursor::new(data)).unwrap();

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
        // println!("{props:#?}");

        println!("    index: {sym_index} 0x{sym_index:x?}");
        println!("    type: {:?}", props.elem_props.data_type());
        println!("    flags: {:?}", props.elem_props.flags());

        match &symbol.data {
            dat::SymbolData::Float(v) => {
                for v in v {
                    println!("    float: {v}");
                }
            }
            dat::SymbolData::Int(v) => {
                for v in v {
                    println!("    int: {v}");
                }
            }
            dat::SymbolData::String(v) => {
                for v in v {
                    println!("    string: {v:?}");
                }
            }
            dat::SymbolData::ClassOffset(v) => {
                println!("    class_offset: {v} 0x{v:x?}");
            }
            dat::SymbolData::Address(v) => {
                println!("    address: {v} 0x{v:x?}");
            }
            dat::SymbolData::None => {}
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
