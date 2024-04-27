use const_eval::Value;
use daedalus_bytecode::{Bytecode, Instruction};
use daedalus_parser::{ExprKind, LitKind};
use dat_file::{
    properties::{DataType, SymbolCodeSpan},
    DatFile,
};
use std::{io::Cursor, str::FromStr};
use zstring::ZString;

mod builtin;

mod dat_symbol_table;
use dat_symbol_table::DatSymbolTable;

mod symbol_indices;
use symbol_indices::SymbolIndices;

mod files;
use files::{FileId, Files};

use crate::{const_eval::ConstValues, files::File};

mod const_eval;

struct Compiler {
    symbol_indices: SymbolIndices,
    const_values: ConstValues,
    symbol_table: DatSymbolTable,
    bytecode: Bytecode,
}

impl Compiler {
    pub fn new(symbol_indices: SymbolIndices, const_values: ConstValues) -> Self {
        Self {
            symbol_table: DatSymbolTable::new(&symbol_indices),
            symbol_indices,
            const_values,
            bytecode: Bytecode::new(),
        }
    }

    fn handle_item(&mut self, files: &Files, file_id: FileId, item: &daedalus_parser::Item) {
        match item {
            daedalus_parser::Item::ExternFunc(func) => {
                let name = ZString::from(func.ident.raw.as_bytes().to_ascii_uppercase());
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

                self.symbol_table.extern_func(
                    name,
                    SymbolCodeSpan::empty(file_id.raw()),
                    &args,
                    ty,
                    addr,
                );
            }

            daedalus_parser::Item::Class(class) => {
                let name = ZString::from(class.ident.raw.as_bytes().to_ascii_uppercase());
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
                        let ident = ZString::from(var.ident.raw.as_bytes().to_ascii_uppercase());
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
                                    ExprKind::Lit(lit) => match &lit.kind {
                                        LitKind::Intager(v) => {
                                            let v: u32 = v.parse().expect("TODO");
                                            v
                                        }
                                        lit => todo!("unexpected: {lit:?}"),
                                    },
                                    ExprKind::Ident(ident) => {
                                        let value = self
                                            .const_values
                                            .map
                                            .get(&ident.raw.to_uppercase())
                                            .expect("TODO");

                                        if let Value::Int(v) = value {
                                            *v as u32
                                        } else {
                                            todo!()
                                        }
                                    }
                                    _ => todo!(),
                                }
                            }
                        };

                        (ident, ty, count, span)
                    })
                    .collect();

                self.symbol_table.class(name, span, &fields, 800, 288);
            }

            daedalus_parser::Item::Instance(instance) => {
                let ident = ZString::from(instance.ident.raw.as_bytes().to_ascii_uppercase());
                let parent = self
                    .symbol_indices
                    .get(&instance.parent.raw.to_uppercase())
                    .expect("TODO");
                let span = &instance.span;

                let line_start = files.line_index(file_id, span.start as u32).0;
                let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    file_id.raw(),
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 2),
                );

                let address = self.bytecode.next_available_address();

                let this = self.symbol_table.instance(ident, span, address, *parent);
                let humans_mds = self.symbol_table.string(ZString::from("HUMANS.MDS"));
                let hum_body_naked0 = self.symbol_table.string(ZString::from("hum_body_Naked0"));
                let hum_head_pony = self.symbol_table.string(ZString::from("Hum_Head_Pony"));

                let npc_attributes = *self.symbol_indices.get("C_NPC.ATTRIBUTE").unwrap();
                let mdl_set_visual = *self.symbol_indices.get("MDL_SETVISUAL").unwrap();
                let mdl_set_visual_body = *self.symbol_indices.get("MDL_SETVISUALBODY").unwrap();

                self.bytecode
                    .block_builder()
                    // attribute[0] = 20
                    .var_assign_int((npc_attributes, 0), 20)
                    // attribute[1] = 40
                    .var_assign_int((npc_attributes, 1), 40)
                    // Mdl_SetVisual(self, "HUMANS.MDS")
                    .extend(&[
                        Instruction::push_var_instance(this),
                        Instruction::push_var(humans_mds),
                        Instruction::call_extern(mdl_set_visual),
                    ])
                    // Mdl_SetVisualBody(self, "hum_body_Naked0", 9, 0, "Hum_Head_Pony", 18, 0, -1);
                    .extend(&[
                        Instruction::push_var_instance(this),
                        Instruction::push_var(hum_body_naked0),
                        Instruction::push_int(9),
                        Instruction::push_int(0),
                        Instruction::push_var(hum_head_pony),
                        Instruction::push_int(18),
                        Instruction::push_int(0),
                        Instruction::push_int(1),
                        Instruction::negate(),
                        Instruction::call_extern(mdl_set_visual_body),
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

                let address = self.bytecode.block_builder().ret().done();

                self.symbol_table
                    .func(ident, span, &[], DataType::Void, address);
            }
            daedalus_parser::Item::Const(item) => {
                let name = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());

                let span = &item.span;

                let line_start = files.line_index(file_id, span.start as u32).0;
                let line_count = files.line_index(file_id, span.end as u32).0 - line_start;

                let span = SymbolCodeSpan::new(
                    file_id.raw(),
                    (line_start + 1, line_count + 1),
                    (span.start as u32, span.end as u32 - span.start as u32 + 3),
                );

                let value = self
                    .const_values
                    .map
                    .get(&item.ident.raw.to_uppercase())
                    .expect("TODO");

                self.symbol_table.const_item(name, span, value);
            }
            got => todo!("Got: {got:?}"),
        }
    }

    pub fn build(mut self, files: &[File], span_files: &Files) -> Vec<u8> {
        for File { id, ast } in files.iter() {
            for item in ast.items.iter() {
                self.handle_item(span_files, *id, item);
            }
        }

        let mut out = Vec::new();
        self.symbol_table.encode(&mut out);
        self.bytecode.encode(&mut out).unwrap();
        out
    }
}

// fn abc() {
//     let mut files_store = Files::new();
//
//     let base_path = "./test_data/G2MDK-PolishScripts/Content/";
//     let src = src_file::load(format!("{base_path}Gothic.src"));
//
//     let file_sources: Vec<(&std::path::Path, String)> = src
//         .iter()
//         .map(|path| {
//             let bytes = std::fs::read(path).unwrap();
//             let path = path.strip_prefix(base_path).unwrap();
//
//             let (src, _, _) = encoding_rs::WINDOWS_1250.decode(&bytes);
//             (path, src.into())
//         })
//         .collect();
//
//     let files: Vec<_> = file_sources
//         .iter()
//         .map(|(path, src)| files_store.parse(path, src).unwrap())
//         .collect();
//
//     let time = std::time::Instant::now();
//     let symbol_indices = SymbolIndices::build(&files);
//
//     const_eval::ConstValues::build(&symbol_indices, &files);
//     dbg!(time.elapsed().as_millis());
// }

fn main() {
    // abc();
    let mut files_store = Files::new();

    fn read_file(path: &str) -> (&str, String) {
        (path, std::fs::read_to_string(path).unwrap())
    }

    let builtin = read_file("./test_data/builtin-gothic.d");
    let classes = read_file("./test_data/classes.d");
    let startup = read_file("./test_data/startup.d");

    let files = [builtin, classes, startup];
    let files: Vec<_> = files
        .iter()
        .map(|(path, src)| files_store.parse(path, src).unwrap())
        .collect();

    let symbol_map = SymbolIndices::build(&files);
    let const_values = ConstValues::build(&files, &symbol_map);
    let out = Compiler::new(symbol_map, const_values).build(&files, &files_store);

    std::fs::write("./OUT2.DAT", &out).unwrap();

    let dat = DatFile::decode(&mut Cursor::new(out)).unwrap();
    dat_file::debug_print(&dat);
}
