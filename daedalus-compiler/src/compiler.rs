use std::{ops::Range, str::FromStr};

use daedalus_bytecode::Bytecode;
use daedalus_parser::{Expr, ExprKind, LitKind};
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
    error::{Diagnostics, Error, ErrorKind, Result},
    files::{File, FileId, Files},
    span::SpanKind,
    symbol_indices::{SymbolIndex, SymbolIndices, SymbolKind, item_symbols},
};

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

    /// Encoded `.DAT` file.
    ///
    /// Compilation goes on after an item fails, to report as many errors as possible, but the
    /// output is only meaningful if no errors were reported.
    pub fn build(
        mut self,
        files: &[File],
        span_files: &Files,
        diagnostics: &mut Diagnostics,
    ) -> Vec<u8> {
        for File { id, ast } in files.iter() {
            for item in ast.items.iter() {
                let len = self.symbol_table.len() + item_symbols(item).len();

                if let Err(err) = self.handle_item(span_files, *id, item) {
                    diagnostics.report(*id, err);
                    self.symbol_table.recover_to(len);
                }
            }
        }

        let mut out = Vec::new();
        self.symbol_table.encode(&mut out);
        self.bytecode
            .encode(&mut out)
            .expect("writing to a Vec can not fail");
        out
    }

    fn handle_item(
        &mut self,
        files: &Files,
        file_id: FileId,
        item: &daedalus_parser::Item,
    ) -> Result<()> {
        match item {
            daedalus_parser::Item::ExternFunc(func) => self.extern_func(file_id, func),
            daedalus_parser::Item::Class(class) => self.class(files, file_id, class),
            daedalus_parser::Item::Instance(instance) => self.instance(files, file_id, instance),
            daedalus_parser::Item::Func(func) => self.func(files, file_id, func),
            daedalus_parser::Item::Const(item) => self.const_item(files, file_id, item),
            daedalus_parser::Item::Prototype(prototype) => {
                self.prototype(files, file_id, prototype)
            }
            daedalus_parser::Item::Var(var) => {
                Err(Error::unsupported(&var.span, "global variables"))
            }
        }
    }

    fn extern_func(
        &mut self,
        file_id: FileId,
        func: &daedalus_parser::ExternFunctionDefinition,
    ) -> Result<()> {
        let name = ZString::from(func.ident.raw.as_bytes().to_ascii_uppercase());
        let ty = data_type(&func.ident.span, &func.ty)?;

        let args = func
            .args
            .iter()
            .map(|var| {
                let ident = ZString::from(var.ident.raw.as_bytes());
                let ty = data_type(&var.span, &var.ty)?;

                Ok((ident, ty, SymbolCodeSpan::empty(file_id.raw())))
            })
            .collect::<Result<Vec<_>>>()?;

        let addr = builtin::get_address(&func.ident.raw).ok_or_else(|| {
            Error::new(
                &func.ident.span,
                ErrorKind::UnknownExtern(func.ident.raw.clone()),
            )
        })?;

        self.symbol_table.extern_func(
            name,
            SymbolCodeSpan::empty(file_id.raw()),
            &args,
            ty,
            addr as i32,
        );

        Ok(())
    }

    fn class(
        &mut self,
        files: &Files,
        file_id: FileId,
        class: &daedalus_parser::Class,
    ) -> Result<()> {
        let name = ZString::from(class.ident.raw.as_bytes().to_ascii_uppercase());
        let span = files.symbol_span(file_id, &class.span, SpanKind::Block);

        let fields = class
            .fields
            .iter()
            .map(|var| {
                let ident = ZString::from(var.ident.raw.as_bytes().to_ascii_uppercase());
                let ty = data_type(&var.span, &var.ty)?;
                if !matches!(
                    ty,
                    DataType::Int | DataType::Float | DataType::String | DataType::Func
                ) {
                    return Err(Error::invalid(
                        &var.span,
                        format!("class field of type {ty:?}"),
                    ));
                }

                let span = files.symbol_span(file_id, &var.span, SpanKind::ClassField);

                let count = match &var.kind {
                    daedalus_parser::VarKind::Value { .. } => 1,
                    daedalus_parser::VarKind::Array { size_init, .. } => {
                        self.array_size(size_init)?
                    }
                };

                Ok((ident, ty, count, span))
            })
            .collect::<Result<Vec<_>>>()?;

        // TODO: Proper full map of those addresses
        match name.as_slice() {
            b"CCAMSYS" => self.symbol_table.class(name, span, &fields, 92, 0),
            b"C_NPC" => self.symbol_table.class(name, span, &fields, 800, 288),
            _ => {
                return Err(Error::unsupported(
                    &class.ident.span,
                    format!("class `{name}` of unknown size and offset"),
                ));
            }
        };

        Ok(())
    }

    fn array_size(&self, size: &Expr) -> Result<u32> {
        let value = match &size.kind {
            ExprKind::Lit(lit) => match &lit.kind {
                LitKind::Intager(v) => *v,
                _ => return Err(Error::invalid(&size.span, "non integer array size")),
            },
            ExprKind::Ident(ident) => {
                match self.const_values.map.get(&ident.raw.to_ascii_uppercase()) {
                    Some(Value::Int(v)) => *v,
                    Some(_) => return Err(Error::invalid(&size.span, "non integer array size")),
                    None => return Err(Error::undefined_symbol(&ident.span, &ident.raw)),
                }
            }
            _ => return Err(Error::unsupported(&size.span, "array size expression")),
        };

        u32::try_from(value).map_err(|_| Error::invalid(&size.span, "negative array size"))
    }

    fn instance(
        &mut self,
        files: &Files,
        file_id: FileId,
        instance: &daedalus_parser::Instance,
    ) -> Result<()> {
        let ident = ZString::from(instance.ident.raw.as_bytes().to_ascii_uppercase());
        let parent = instance.parent.raw.to_ascii_uppercase();
        let parent_data = *self.lookup(&instance.parent.span, &parent)?;
        let span = files.symbol_span(file_id, &instance.span, SpanKind::Block);

        let prototype = if parent_data.kind == SymbolKind::Prototype {
            Some(self.prototype_of_instance(&instance.parent.span, parent_data.id)?)
        } else {
            None
        };

        let mut block = self.bytecode.block_builder();

        self.symbol_table
            .instance(ident, span, block.addr(), parent_data.id);

        if let Some((_, addr)) = prototype.as_ref() {
            block.call(*addr);
        }

        let builder = BlockBuilder {
            parent: match prototype.as_ref() {
                Some((class_name, _)) => class_name,
                None => &parent,
            },
            symbol_indices: &self.symbol_indices,
            symbol_table: &mut self.symbol_table,
            block: &mut block,
        };

        builder.visit_instance(instance)
    }

    /// Name of the class and the body address of prototype `id`, that an instance inherits from
    fn prototype_of_instance(&self, span: &Range<usize>, id: u32) -> Result<(String, u32)> {
        let prototype = self
            .symbol_table
            .get(id)
            .ok_or_else(|| Error::invalid(span, "prototype failed to compile"))?;

        let class_id = prototype
            .parent
            .expect("prototype symbols always have a parent class");

        let (class_name, class) = self
            .symbol_indices
            .iter()
            .find(|(_, v)| v.id == class_id)
            .expect("parent of a symbol is in the symbol indices");

        if class.kind != SymbolKind::Class {
            return Err(Error::unsupported(span, "multi level inheritance"));
        }

        let SymbolData::Address(addr) = prototype.data else {
            unreachable!("prototype symbols always have an address");
        };

        Ok((
            class_name.clone(),
            u32::try_from(addr).expect("body address is positive"),
        ))
    }

    fn func(
        &mut self,
        files: &Files,
        file_id: FileId,
        func: &daedalus_parser::FunctionDefinition,
    ) -> Result<()> {
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

        builder.visit_function(func)
    }

    fn const_item(
        &mut self,
        files: &Files,
        file_id: FileId,
        item: &daedalus_parser::Const,
    ) -> Result<()> {
        let name = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
        let span = files.symbol_span(file_id, &item.span, SpanKind::Const);

        let data = match self
            .const_values
            .map
            .get(&item.ident.raw.to_ascii_uppercase())
        {
            Some(Value::Int(v)) => SymbolData::Int(vec![*v]),
            Some(Value::Float(v)) => SymbolData::Float(vec![*v]),
            Some(Value::String(_)) => {
                return Err(Error::unsupported(&item.span, "string constants"));
            }
            Some(Value::Array(_)) => {
                return Err(Error::unsupported(&item.span, "array constants"));
            }
            Some(Value::Symbol(_)) => {
                return Err(Error::unsupported(&item.span, "symbol reference constants"));
            }
            // Evaluation failed, which was already reported
            None => return Ok(()),
        };

        self.symbol_table.const_item(name, span, data);

        Ok(())
    }

    fn prototype(
        &mut self,
        files: &Files,
        file_id: FileId,
        prototype: &daedalus_parser::Prototype,
    ) -> Result<()> {
        let ident = ZString::from(prototype.ident.raw.as_bytes().to_ascii_uppercase());
        let parent = prototype.parent.raw.to_ascii_uppercase();
        let parent_id = self.lookup(&prototype.parent.span, &parent)?.id;
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

        builder.visit_prototype(prototype)
    }

    fn lookup(&self, span: &Range<usize>, name: &str) -> Result<&SymbolIndex> {
        self.symbol_indices
            .get(name)
            .ok_or_else(|| Error::undefined_symbol(span, name))
    }
}

fn data_type(span: &Range<usize>, ty: &daedalus_parser::Ty) -> Result<DataType> {
    DataType::from_str(&ty.raw)
        .map_err(|()| Error::new(span, ErrorKind::UnknownType(ty.raw.clone())))
}
