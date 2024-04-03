use bytecode::{Bytecode, Instruction};
use byteorder::{LittleEndian, WriteBytesExt};
use dat::{
    properties::{DataType, ElemProps, PropFlag, Properties, SymbolCodeSpan},
    Symbol, SymbolData, ZString,
};
use std::{collections::HashMap, io::Cursor, str::FromStr};

mod builtin;

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

    /// extern func void name((var type PARN)*)
    fn gen_extern_func(
        &mut self,
        name: ZString,
        span: SymbolCodeSpan,
        args: &[(ZString, DataType, SymbolCodeSpan)],
        ret: DataType,
        address: i32,
    ) -> u32 {
        let fn_symbol = self.push_symbol(Symbol {
            name: Some(name.clone()),
            props: Properties {
                off_cls_ret: ret as i32,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(args.len() as u32);
                    default.set_data_type(DataType::Func);
                    let mut flags = PropFlag::CONST | PropFlag::EXTERNAL;
                    if ret != DataType::Void {
                        flags |= PropFlag::RETURN;
                    }
                    default.set_flags(flags);
                    default.set_space(1);
                    default
                },
            },
            code_span: span,
            data: SymbolData::Address(address),
            parent: None,
        });

        for (ident, arg, span) in args.iter() {
            let mut name = name.clone();
            name.0.push(b'.');
            name.0.extend(ident.as_bytes());

            self.push_symbol(Symbol {
                name: Some(name),
                props: Properties {
                    off_cls_ret: 0,
                    elem_props: {
                        let mut default = ElemProps::default();
                        default.set_count(0);
                        default.set_data_type(*arg);
                        default.set_flags(PropFlag::empty());
                        default.set_space(1);
                        default
                    },
                },
                code_span: *span,
                data: match arg {
                    DataType::Float => SymbolData::Float(vec![]),
                    DataType::Int => SymbolData::Int(vec![]),
                    DataType::String => SymbolData::String(vec![]),
                    DataType::Class => SymbolData::ClassOffset(0),
                    DataType::Func => SymbolData::Address(0),
                    DataType::Prototype => SymbolData::Address(0),
                    DataType::Instance => SymbolData::Address(0),
                    DataType::Void => SymbolData::None,
                },
                parent: None,
            });
        }

        fn_symbol
    }

    /// func void name((var type PARN)*)
    fn gen_func(
        &mut self,
        name: ZString,
        span: SymbolCodeSpan,
        args: &[(ZString, DataType, SymbolCodeSpan)],
        ret: DataType,
        address: u32,
    ) -> u32 {
        let fn_symbol = self.push_symbol(Symbol {
            name: Some(name.clone()),
            props: Properties {
                off_cls_ret: ret as i32,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(args.len() as u32);
                    default.set_data_type(DataType::Func);
                    let mut flags = PropFlag::CONST;
                    if ret != DataType::Void {
                        flags |= PropFlag::RETURN;
                    }
                    default.set_flags(flags);
                    default.set_space(1);
                    default
                },
            },
            code_span: span,
            data: SymbolData::Address(address as i32),
            parent: None,
        });

        for (ident, arg, span) in args.iter() {
            let mut name = name.clone();
            name.0.push(b'.');
            name.0.extend(ident.as_bytes());

            self.push_symbol(Symbol {
                name: Some(name),
                props: Properties {
                    off_cls_ret: 0,
                    elem_props: {
                        let mut default = ElemProps::default();
                        default.set_count(0);
                        default.set_data_type(*arg);
                        default.set_flags(PropFlag::empty());
                        default.set_space(1);
                        default
                    },
                },
                code_span: *span,
                data: match arg {
                    DataType::Float => SymbolData::Float(vec![]),
                    DataType::Int => SymbolData::Int(vec![]),
                    DataType::String => SymbolData::String(vec![]),
                    DataType::Class => SymbolData::ClassOffset(0),
                    DataType::Func => SymbolData::Address(0),
                    DataType::Prototype => SymbolData::Address(0),
                    DataType::Instance => SymbolData::Address(0),
                    DataType::Void => SymbolData::None,
                },
                parent: None,
            });
        }

        fn_symbol
    }

    fn gen_class(
        &mut self,
        name: ZString,
        span: SymbolCodeSpan,
        fields: &[(ZString, DataType, u32, SymbolCodeSpan)],
        offset: i32,
        address: i32,
    ) -> u32 {
        let class_symbol = self.push_symbol(Symbol {
            name: Some(name.clone()),
            props: Properties {
                off_cls_ret: offset,
                elem_props: {
                    let mut default = ElemProps::default();
                    default.set_count(fields.len() as u32);
                    default.set_data_type(DataType::Class);
                    default.set_flags(PropFlag::empty());
                    default.set_space(1);
                    default
                },
            },
            code_span: span,
            data: SymbolData::Address(address),
            parent: None,
        });

        let mut address = address;
        for (ident, data_type, count, span) in fields.iter() {
            let mut name = name.clone();
            name.0.push(b'.');
            name.0.extend(ident.as_bytes());

            self.push_symbol(Symbol {
                name: Some(name),
                props: Properties {
                    off_cls_ret: address,
                    elem_props: {
                        let mut default = ElemProps::default();
                        default.set_count(*count);

                        let size = match data_type {
                            DataType::Void => 0,
                            DataType::Float => 4,
                            DataType::Int => 4,
                            // class zSTRING { int allocater; char* vector; int length; int reserved; }
                            DataType::String => 20,
                            DataType::Class => todo!(),
                            DataType::Func => 4,
                            DataType::Prototype => todo!(),
                            DataType::Instance => todo!(),
                        };

                        address += *count as i32 * size;
                        default.set_data_type(*data_type);
                        default.set_flags(PropFlag::CLASS_VAR);
                        default.set_space(1);
                        default
                    },
                },
                code_span: *span,
                data: SymbolData::None,
                parent: Some(class_symbol),
            });
        }

        class_symbol
    }

    fn gen_instance(
        &mut self,
        name: ZString,
        code_span: SymbolCodeSpan,
        address: u32,
        parent: u32,
    ) -> u32 {
        self.push_symbol(Symbol {
            name: Some(name),
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
            },
            code_span,
            data: SymbolData::Address(address as i32),
            parent: Some(parent),
        })
    }

    pub fn generate_sort_table(&mut self) {
        let mut symbol_ids: Vec<_> = self
            .symbols
            .iter()
            .enumerate()
            .map(|(i, s)| (i, &s.name))
            .collect();

        // Symbols map is sorted in alphabetical order
        symbol_ids.sort_by_key(|v| v.1.as_slice());
        self.sort_idx = symbol_ids.iter().map(|(id, _)| *id as u32).collect();
    }

    pub fn gen_human_mds(&mut self) -> u32 {
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
            },
            code_span: SymbolCodeSpan::empty(),
            data: SymbolData::String(vec![ZString::from("HUMANS.MDS")]),
            parent: None,
        })
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
    let mut dat = DatBuilder::new();

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
        },
        code_span: SymbolCodeSpan::empty(),
        data: SymbolData::Address(0),
        parent: None,
    });

    let mut externs = HashMap::new();
    {
        let builtin = std::fs::read_to_string("./test_data/builtin-gothic.d").unwrap();
        let builtin =
            parser::parse::File::parse(&mut parser::DaedalusLexer::new(&builtin)).unwrap();
        for item in builtin.items {
            if let parser::parse::Item::ExternFunc(func) = item {
                let name = ZString::from(func.ident.raw.as_bytes());
                let ty = DataType::from_str(func.ty.raw.as_str()).unwrap();

                let args: Vec<_> = func
                    .args
                    .into_iter()
                    .map(|var| {
                        let ident = ZString::from(var.ident.raw.as_bytes());
                        let ty = DataType::from_str(var.ty.raw.as_str()).unwrap();

                        (ident, ty, SymbolCodeSpan::empty())
                    })
                    .collect();

                let addr = builtin::get_address(&func.ident.raw).unwrap() as i32;

                let id = dat.gen_extern_func(name, SymbolCodeSpan::empty(), &args, ty, addr);
                externs.insert(func.ident.raw.to_uppercase(), id);
            }
        }
    }

    let mut classes = HashMap::new();
    let mut files = codespan::Files::<&str>::new();

    let src = std::fs::read_to_string("./test_data/classes.d").unwrap();
    {
        let file_id = files.add("./test_data/classes.d", &src);
        let builtin = parser::parse::File::parse(&mut parser::DaedalusLexer::new(&src)).unwrap();
        for item in builtin.items {
            if let parser::parse::Item::Class(class) = item {
                let name = ZString::from(class.ident.raw.as_bytes());
                let span = class.span;

                let line_start = files.line_index(file_id, span.start as u32).0;
                let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    0,
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 2),
                );

                let fields: Vec<_> = class
                    .fields
                    .into_iter()
                    .map(|var| {
                        let ident = ZString::from(var.ident.raw.as_bytes());
                        let ty = DataType::from_str(var.ty.raw.as_str()).unwrap();

                        // Codespans produced by Zengin are either hard for me to understand, or straight
                        // up broken, so let's make compatibility with them an optional feature
                        let span = if cfg!(feature = "code-span-compat") {
                            let mut span = var.span;

                            let line_start = files.line_index(file_id, span.start as u32).0;
                            let line_count =
                                files.line_index(file_id, span.end as u32).0 - line_start;

                            // Don't ask me why field span starts at the beginning of the line, this
                            // is straight up broken, if 2 fields are on the same line, but that's
                            // what zengine does...
                            span.start =
                                files.line_span(file_id, line_start).unwrap().start().0 as usize;

                            // Don't ask me why we add +3 to char_count of a span, we just do as
                            // that makes it compatible with zengin for some reason
                            SymbolCodeSpan::new(
                                0,
                                (line_start + 1, line_count + 1),
                                (span.start as u32, span.end as u32 - span.start as u32 + 3),
                            )
                        } else {
                            // Path for sane spans without compatibility with zengin ones

                            let span = var.span;
                            let line_start = files.line_index(file_id, span.start as u32).0;
                            let line_count =
                                files.line_index(file_id, span.end as u32).0 - line_start;

                            SymbolCodeSpan::new(
                                // Let's start from 1, so in contrast to zengin 0 is reserved for builtins
                                0,
                                (line_start + 1, line_count + 1),
                                (span.start as u32, span.end as u32 - span.start as u32),
                            )
                        };

                        let count = match var.kind {
                            parser::parse::VarKind::Value { .. } => 1,
                            parser::parse::VarKind::Array { size_init, .. } => {
                                match size_init.kind {
                                    parser::parse::ExprKind::Lit(lit) => match lit.kind {
                                        parser::parse::LitKind::Intager(v) => {
                                            let v: u32 = v.parse().expect("TODO");
                                            v
                                        }
                                        lit => todo!("unexpected: {lit:?}"),
                                    },
                                    _ => todo!(),
                                }
                            }
                        };

                        (ident, ty, count, span)
                    })
                    .collect();

                let id = dat.gen_class(name, span, &fields, 800, 288);
                classes.insert(class.ident.raw.to_uppercase(), id);
            }
        }
    }

    let src = std::fs::read_to_string("./test_data/startup.d").unwrap();
    let mut instances = HashMap::new();
    {
        let file_id = files.add("./test_data/startup.d", &src);
        let builtin = parser::parse::File::parse(&mut parser::DaedalusLexer::new(&src)).unwrap();

        for item in builtin.items {
            match item {
                parser::parse::Item::Instance(instance) => {
                    let ident = ZString::from(instance.ident.raw.to_uppercase().as_bytes());
                    let parent = &instance.parent.raw.to_uppercase();
                    let parent = classes.get(parent).expect("TODO");
                    let span = instance.span;

                    let line_start = files.line_index(file_id, span.start as u32).0;
                    let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

                    let span = SymbolCodeSpan::new(
                        1,
                        (line_start + 1, line_count + 1),
                        (span.start as u32, span.end as u32 - span.start as u32 + 2),
                    );

                    let address = dat.bytecode.next_available_address();
                    let pc_hero = dat.gen_instance(ident, span, address, *parent);

                    instances.insert(instance.ident.raw.to_uppercase(), pc_hero);

                    let mdl_set_visual = externs.get("MDL_SETVISUAL").unwrap();
                    let file_name = dat.gen_human_mds();

                    dat.bytecode
                        .block_builder()
                        // attribute[0] = 20
                        // .var_assign_int((todo!(), 0), 20)
                        // // attribute[1] = 40
                        // .var_assign_int((todo!(), 1), 40)
                        // Mdl_SetVisual(self, "HUMANS.MDS")
                        .extend(&[
                            Instruction::push_var_instance(pc_hero),
                            Instruction::push_var(file_name),
                            Instruction::call_extern(*mdl_set_visual),
                        ])
                        .ret()
                        .done();
                }
                parser::parse::Item::Func(func) => {
                    let ident = ZString::from(func.ident.raw.to_uppercase().as_bytes());
                    let span = func.span;

                    let line_start = files.line_index(file_id, span.start as u32).0;
                    let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

                    let span = SymbolCodeSpan::new(
                        1,
                        (line_start + 1, line_count + 1),
                        (span.start as u32, span.end as u32 - span.start as u32 + 2),
                    );

                    let address = dat.bytecode.block_builder().ret().done();

                    dat.gen_func(ident, span, &[], DataType::Void, address);
                }
                got => todo!("Got: {got:?}"),
            }
        }
    }

    dat.generate_sort_table();

    let data = dat.build();
    std::fs::write("./OUT2.DAT", &data).unwrap();

    let dat = dat::DatFile::decode(&mut Cursor::new(data)).unwrap();
    dat::debug_print(&dat);
}
