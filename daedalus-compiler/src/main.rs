use byteorder::{LittleEndian, WriteBytesExt};
use daedalus_bytecode::{Bytecode, Instruction};
use daedalus_parser::DaedalusLexer;
use dat_file::{
    properties::{DataType, ElemProps, PropFlag, Properties, SymbolCodeSpan},
    DatFile, Symbol, SymbolData, ZString,
};
use indexmap::IndexMap;
use std::{io::Cursor, str::FromStr};

mod builtin;
mod files;
use files::{FileId, Files};

struct Stage1 {
    symbol_table: IndexMap<ZString, u32>,
}

impl Stage1 {
    fn push_symbol(&mut self, ident: ZString) {
        self.symbol_table
            .insert(ident.clone(), self.symbol_table.len() as u32);
    }

    fn build_symbol_table(&mut self, _file_id: FileId, item: &daedalus_parser::Item) {
        match item {
            daedalus_parser::Item::ExternFunc(func) => {
                let ident = ZString::from(func.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident.clone());

                for var in func.args.iter() {
                    let mut arg = ident.clone();
                    arg.0.push(b'.');
                    arg.0.extend(var.ident.raw.as_bytes());
                    self.push_symbol(arg);
                }
            }
            daedalus_parser::Item::Class(class) => {
                let ident = ZString::from(class.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident.clone());

                for var in class.fields.iter() {
                    let mut arg = ident.clone();
                    arg.0.push(b'.');
                    arg.0.extend(var.ident.raw.as_bytes());
                    self.push_symbol(arg);
                }
            }
            daedalus_parser::Item::Instance(instance) => {
                let ident = ZString::from(instance.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            daedalus_parser::Item::Func(func) => {
                let ident = ZString::from(func.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            got => todo!("Got: {got:?}"),
        }
    }

    pub fn run(files: &[(FileId, daedalus_parser::File)]) -> Stage2 {
        let symbol_table = IndexMap::new();
        let mut stage1 = Self { symbol_table };

        stage1.push_symbol(ZString::from(b"\xFFINSTANCE_HELP"));
        for (id, ast) in files.iter() {
            for item in ast.items.iter() {
                stage1.build_symbol_table(*id, item);
            }
        }

        Stage2::new(stage1)
    }
}

struct Stage2 {
    symbol_table: IndexMap<ZString, u32>,
    dat: DatBuilder,
    todo_string_constants: Vec<Symbol>,
}

impl Stage2 {
    pub fn new(stage1: Stage1) -> Self {
        Self {
            symbol_table: stage1.symbol_table,
            dat: DatBuilder::new(),
            todo_string_constants: Vec::new(),
        }
    }

    fn handle_item(&mut self, files: &Files, file_id: FileId, item: &daedalus_parser::Item) {
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

                self.dat.gen_extern_func(
                    name,
                    SymbolCodeSpan::empty(file_id.raw()),
                    &args,
                    ty,
                    addr,
                );
            }

            daedalus_parser::Item::Class(class) => {
                let name = ZString::from(class.ident.raw.as_bytes());
                let span = &class.span;

                let line_start = files.line_index(file_id, span.start as u32).0;
                let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

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
                                file_id.raw(),
                                (line_start + 1, line_count + 1),
                                (span.start as u32, span.end as u32 - span.start as u32 + 3),
                            )
                        } else {
                            // Path for sane spans without compatibility with zengin ones

                            let span = &var.span;
                            let line_start = files.line_index(file_id, span.start as u32).0;
                            let line_count =
                                files.line_index(file_id, span.end as u32).0 - line_start;

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

                self.dat.gen_class(name, span, &fields, 800, 288);
            }

            daedalus_parser::Item::Instance(instance) => {
                let ident = ZString::from(instance.ident.raw.as_bytes().to_ascii_uppercase());
                let parent = ZString::from(instance.parent.raw.as_bytes().to_ascii_uppercase());
                let parent = self.symbol_table.get(&parent).expect("TODO");
                let span = &instance.span;

                let line_start = files.line_index(file_id, span.start as u32).0;
                let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    file_id.raw(),
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 2),
                );

                let address = self.dat.bytecode.next_available_address();

                let pc_hero = *self.symbol_table.get(&ident).unwrap();

                self.dat.gen_instance(ident, span, address, *parent);

                let mdl_set_visual = self
                    .symbol_table
                    .get(&ZString::from(b"MDL_SETVISUAL")) // TODO
                    .unwrap();

                let file_name = (self.symbol_table.len() + self.todo_string_constants.len()) as u32;
                self.todo_string_constants.push(Symbol {
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
                });

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

                let line_start = files.line_index(file_id, span.start as u32).0;
                let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

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

    pub fn run(
        mut self,
        files: &[(FileId, daedalus_parser::File)],
        span_files: &Files,
    ) -> DatBuilder {
        self.dat.push_symbol(Symbol {
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

        for (id, ast) in files.iter() {
            for item in ast.items.iter() {
                self.handle_item(span_files, *id, item);
            }
        }

        for symbol in self.todo_string_constants {
            self.dat.push_symbol(symbol);
        }

        self.dat
    }
}

#[derive(Debug)]
struct DatBuilder {
    symbols: Vec<Symbol>,
    bytecode: Bytecode,
}

impl DatBuilder {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
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

    fn generate_sort_table(&self) -> Vec<u32> {
        let mut symbol_ids: Vec<_> = self
            .symbols
            .iter()
            .enumerate()
            .map(|(i, s)| (i, &s.name))
            .collect();

        // Symbols map is sorted in alphabetical order
        symbol_ids.sort_by_key(|v| v.1.as_slice());
        symbol_ids.iter().map(|(id, _)| *id as u32).collect()
    }

    pub fn push_symbol(&mut self, symbol: Symbol) -> u32 {
        let id = self.symbols.len();
        self.symbols.push(symbol);
        id as u32
    }

    pub fn build(&self) -> Vec<u8> {
        let sort_idx = self.generate_sort_table();
        let mut out = vec![];

        out.write_u8(b'2').unwrap();
        out.write_u32::<LittleEndian>(self.symbols.len() as u32)
            .unwrap();

        for id in sort_idx.iter() {
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
}

fn main() {
    let mut compiler = Compiler {
        files: Files::new(),
    };

    fn read_file(path: &str) -> (&str, String) {
        (path, std::fs::read_to_string(path).unwrap())
    }

    let builtin = read_file("./test_data/builtin-gothic.d");
    let classes = read_file("./test_data/classes.d");
    let startup = read_file("./test_data/startup.d");

    let files = [builtin, classes, startup];
    let files: Vec<_> = files
        .iter()
        .map(|(path, src)| {
            let id = compiler.files.add(path, src);
            let ast = daedalus_parser::File::parse(&mut DaedalusLexer::new(src)).unwrap();
            (id, ast)
        })
        .collect();

    let stage2 = Stage1::run(&files);
    let dat = stage2.run(&files, &compiler.files);

    let data = dat.build();
    std::fs::write("./OUT2.DAT", &data).unwrap();

    let dat = DatFile::decode(&mut Cursor::new(data)).unwrap();
    dat_file::debug_print(&dat);
}
