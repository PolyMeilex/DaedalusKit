use daedalus_bytecode::{Bytecode, Instruction};
use byteorder::{LittleEndian, WriteBytesExt};
use daedalus_parser::DaedalusLexer;
use dat::{
    properties::{DataType, ElemProps, PropFlag, Properties, SymbolCodeSpan},
    Symbol, SymbolData, ZString,
};
use std::{collections::HashMap, io::Cursor, str::FromStr};

mod builtin;
mod files;
use files::{FileId, Files};

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
            code_span: SymbolCodeSpan::empty(0),
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

struct Compiler<'a> {
    files: Files<'a>,
    classes: HashMap<String, u32>,
    instances: HashMap<String, u32>,
    externs: HashMap<String, u32>,

    dat: DatBuilder,
}

impl<'a> Compiler<'a> {
    pub fn handle_item(&mut self, file_id: FileId, item: &daedalus_parser::Item) {
        match item {
            daedalus_parser::Item::ExternFunc(func) => {
                let name = ZString::from(func.ident.raw.as_bytes());
                let ty = DataType::from_str(func.ty.raw.as_str()).unwrap();

                let args: Vec<_> = func
                    .args
                    .iter()
                    .map(|var| {
                        let ident = ZString::from(var.ident.raw.as_bytes());
                        let ty = DataType::from_str(var.ty.raw.as_str()).unwrap();

                        (ident, ty, SymbolCodeSpan::empty(file_id.raw()))
                    })
                    .collect();

                let addr = builtin::get_address(&func.ident.raw).unwrap() as i32;

                let id = self.dat.gen_extern_func(
                    name,
                    SymbolCodeSpan::empty(file_id.raw()),
                    &args,
                    ty,
                    addr,
                );
                self.externs.insert(func.ident.raw.to_uppercase(), id);
            }

            daedalus_parser::Item::Class(class) => {
                let name = ZString::from(class.ident.raw.as_bytes());
                let span = &class.span;

                let line_start = self.files.line_index(file_id, span.start as u32).0;
                let line_count = self.files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    file_id.raw(),
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 2),
                );

                let fields: Vec<_> = class
                    .fields
                    .iter()
                    .map(|var| {
                        let ident = ZString::from(var.ident.raw.as_bytes());
                        let ty = DataType::from_str(var.ty.raw.as_str()).unwrap();

                        // Codespans produced by Zengin are either hard for me to understand, or straight
                        // up broken, so let's make compatibility with them an optional feature
                        let span = if cfg!(feature = "code-span-compat") {
                            let mut span = var.span.clone();

                            let line_start = self.files.line_index(file_id, span.start as u32).0;
                            let line_count =
                                self.files.line_index(file_id, span.end as u32).0 - line_start;

                            // Don't ask me why field span starts at the beginning of the line, this
                            // is straight up broken, if 2 fields are on the same line, but that's
                            // what zengine does...
                            span.start =
                                self.files.line_span(file_id, line_start).unwrap().start().0
                                    as usize;

                            // Don't ask me why we add +3 to char_count of a span, we just do as
                            // that makes it compatible with zengin for some reason
                            SymbolCodeSpan::new(
                                file_id.raw(),
                                (line_start + 1, line_count + 1),
                                (span.start as u32, span.end as u32 - span.start as u32 + 3),
                            )
                        } else {
                            // Path for sane spans without compatibility with zengin ones

                            let span = &var.span;
                            let line_start = self.files.line_index(file_id, span.start as u32).0;
                            let line_count =
                                self.files.line_index(file_id, span.end as u32).0 - line_start;

                            SymbolCodeSpan::new(
                                file_id.raw(),
                                (line_start + 1, line_count + 1),
                                (span.start as u32, span.end as u32 - span.start as u32),
                            )
                        };

                        let count = match &var.kind {
                            daedalus_parser::VarKind::Value { .. } => 1,
                            daedalus_parser::VarKind::Array { size_init, .. } => {
                                match &size_init.kind {
                                    daedalus_parser::ExprKind::Lit(lit) => match &lit.kind {
                                        daedalus_parser::LitKind::Intager(v) => {
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

                let id = self.dat.gen_class(name, span, &fields, 800, 288);
                self.classes.insert(class.ident.raw.to_uppercase(), id);
            }

            daedalus_parser::Item::Instance(instance) => {
                let ident = ZString::from(instance.ident.raw.to_uppercase().as_bytes());
                let parent = &instance.parent.raw.to_uppercase();
                let parent = self.classes.get(parent).expect("TODO");
                let span = &instance.span;

                let line_start = self.files.line_index(file_id, span.start as u32).0;
                let line_count = self.files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    file_id.raw(),
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 2),
                );

                let address = self.dat.bytecode.next_available_address();
                let pc_hero = self.dat.gen_instance(ident, span, address, *parent);

                self.instances
                    .insert(instance.ident.raw.to_uppercase(), pc_hero);

                let mdl_set_visual = self.externs.get("MDL_SETVISUAL").unwrap();
                let file_name = self.dat.gen_human_mds();

                self.dat
                    .bytecode
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

            daedalus_parser::Item::Func(func) => {
                let ident = ZString::from(func.ident.raw.to_uppercase().as_bytes());
                let span = &func.span;

                let line_start = self.files.line_index(file_id, span.start as u32).0;
                let line_count = self.files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    file_id.raw(),
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 2),
                );

                let address = self.dat.bytecode.block_builder().ret().done();

                self.dat.gen_func(ident, span, &[], DataType::Void, address);
            }
            got => todo!("Got: {got:?}"),
        }
    }

    pub fn handle_ast(&mut self, file_id: FileId, items: &[daedalus_parser::Item]) {
        for item in items {
            self.handle_item(file_id, item);
        }
    }
}

fn main() {
    let mut compiler = Compiler {
        dat: DatBuilder::new(),
        files: Files::new(),
        classes: HashMap::new(),
        instances: HashMap::new(),
        externs: HashMap::new(),
    };

    compiler.dat.push_symbol(Symbol {
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
        code_span: SymbolCodeSpan::empty(0),
        data: SymbolData::Address(0),
        parent: None,
    });

    let builtin = std::fs::read_to_string("./test_data/builtin-gothic.d").unwrap();
    let builtin_id = compiler.files.add("./test_data/builtin-gothic.d", &builtin);
    let builtin_ast = daedalus_parser::File::parse(&mut DaedalusLexer::new(&builtin)).unwrap();
    compiler.handle_ast(builtin_id, &builtin_ast.items);

    let classes = std::fs::read_to_string("./test_data/classes.d").unwrap();
    let classes_id = compiler.files.add("./test_data/classes.d", &classes);
    let classes_ast = daedalus_parser::File::parse(&mut DaedalusLexer::new(&classes)).unwrap();
    compiler.handle_ast(classes_id, &classes_ast.items);

    let startup = std::fs::read_to_string("./test_data/startup.d").unwrap();
    let startup_id = compiler.files.add("./test_data/startup.d", &startup);
    let startup_ast = daedalus_parser::File::parse(&mut DaedalusLexer::new(&startup)).unwrap();
    compiler.handle_ast(startup_id, &startup_ast.items);

    compiler.dat.generate_sort_table();

    let data = compiler.dat.build();
    std::fs::write("./OUT2.DAT", &data).unwrap();

    let dat = dat::DatFile::decode(&mut Cursor::new(data)).unwrap();
    dat::debug_print(&dat);
}
