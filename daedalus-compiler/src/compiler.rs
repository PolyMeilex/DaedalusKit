use std::str::FromStr;

use daedalus_parser::{ExprKind, LitKind};
use dat_file::{
    SymbolData,
    properties::{DataType, SymbolCodeSpan},
};
use zstring::ZString;

use crate::{
    builtin,
    codegen::BlockBuilder,
    const_eval::{ConstValues, Value},
    dat_symbol_table::DatSymbolTable,
    files::{File, FileId, Files},
    span::SpanKind,
    symbol_indices::{SymbolIndices, SymbolKind},
};
use daedalus_bytecode::Bytecode;

pub struct Compiler {
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
                let span = files.symbol_span(file_id, &class.span, SpanKind::Block);

                let fields: Vec<_> = class
                    .fields
                    .iter()
                    .map(|var| {
                        let ident = ZString::from(var.ident.raw.as_bytes().to_ascii_uppercase());
                        let ty = DataType::from_str(var.ty.raw.as_str()).unwrap();

                        let span = files.symbol_span(file_id, &var.span, SpanKind::ClassField);

                        let count = match &var.kind {
                            daedalus_parser::VarKind::Value { .. } => 1,
                            daedalus_parser::VarKind::Array { size_init, .. } => {
                                match &size_init.kind {
                                    ExprKind::Lit(lit) => match &lit.kind {
                                        LitKind::Intager(v) => u32::try_from(*v).expect("TODO"),
                                        lit => todo!("unexpected: {lit:?}"),
                                    },
                                    ExprKind::Ident(ident) => {
                                        let value = self
                                            .const_values
                                            .map
                                            .get(&ident.raw.to_ascii_uppercase())
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

                // TODO: Proper full map of those addresses
                if name.as_slice() == b"CCAMSYS" {
                    self.symbol_table.class(name, span, &fields, 92, 0);
                } else if name.as_slice() == b"C_NPC" {
                    self.symbol_table.class(name, span, &fields, 800, 288);
                } else {
                    todo!("{name}");
                }
            }

            daedalus_parser::Item::Instance(instance) => {
                let ident = ZString::from(instance.ident.raw.as_bytes().to_ascii_uppercase());
                let parent = instance.parent.raw.to_ascii_uppercase();
                let parent_data = self.symbol_indices.get(&parent).expect("TODO");
                let parent_kind = parent_data.kind;
                let parent_id = parent_data.id;
                let span = files.symbol_span(file_id, &instance.span, SpanKind::Block);

                let prototype = if parent_kind == SymbolKind::Prototype {
                    let parent_symbol = &self.symbol_table.symbols[parent_id as usize];

                    let Some(class_id) = parent_symbol.parent else {
                        todo!(" Prototype must have a parent");
                    };

                    let Some((class_name, class)) =
                        self.symbol_indices.iter().find(|(_, v)| v.id == class_id)
                    else {
                        todo!("Missing parent symbol");
                    };

                    if class.kind != SymbolKind::Class {
                        todo!("Multi level inheritance not implemented: {class:#?}");
                    }

                    let SymbolData::Address(addr) = parent_symbol.data else {
                        todo!()
                    };

                    Some((class_name, u32::try_from(addr).expect("TODO")))
                } else {
                    None
                };

                let mut block = self.bytecode.block_builder();

                self.symbol_table
                    .instance(ident, span, block.addr(), parent_id);

                if let Some((_, addr)) = prototype.as_ref() {
                    block.call(*addr);
                }

                let builder = BlockBuilder {
                    parent: if let Some((parent, _)) = prototype.as_ref() {
                        parent
                    } else {
                        &parent
                    },
                    symbol_indices: &self.symbol_indices,
                    symbol_table: &mut self.symbol_table,
                    block: &mut block,
                };

                builder.visit_instance(instance);
            }

            daedalus_parser::Item::Func(func) => {
                let ident_utf8 = func.ident.raw.to_ascii_uppercase();
                let ident = ZString::from(ident_utf8.as_bytes());
                let span = files.symbol_span(file_id, &func.span, SpanKind::Block);

                let mut block = self.bytecode.block_builder();

                self.symbol_table
                    .func(ident, span, &[], DataType::Void, block.addr());

                let builder = BlockBuilder {
                    parent: &ident_utf8,
                    symbol_indices: &self.symbol_indices,
                    symbol_table: &mut self.symbol_table,
                    block: &mut block,
                };

                builder.visit_function(func);
            }
            daedalus_parser::Item::Const(item) => {
                let name = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());

                let span = files.symbol_span(file_id, &item.span, SpanKind::Const);

                let value = self
                    .const_values
                    .map
                    .get(&item.ident.raw.to_ascii_uppercase())
                    .expect("TODO");

                self.symbol_table.const_item(name, span, value);
            }
            daedalus_parser::Item::Prototype(prototype) => {
                let ident = ZString::from(prototype.ident.raw.as_bytes().to_ascii_uppercase());
                let parent = prototype.parent.raw.to_ascii_uppercase();
                let parent_id = self.symbol_indices.get(&parent).expect("TODO").id;
                let span = files.symbol_span(file_id, &prototype.span, SpanKind::Block);

                let mut block = self.bytecode.block_builder();

                self.symbol_table
                    .prototype(ident, span, block.addr(), parent_id);

                let builder = BlockBuilder {
                    parent: &parent,
                    symbol_indices: &self.symbol_indices,
                    symbol_table: &mut self.symbol_table,
                    block: &mut block,
                };

                builder.visit_prototype(prototype);
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
