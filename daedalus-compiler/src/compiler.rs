use std::{ops::Range, str::FromStr};

use daedalus_bytecode::{Bytecode, Label};
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
    encoding::{script_string, symbol_name},
    error::{Diagnostics, Error, ErrorKind, Result},
    files::{File, FileId, Files},
    span::SpanKind,
    symbol_indices::{SymbolIndex, SymbolIndices, SymbolKind, function_locals, item_symbols},
};

pub struct Compiler {
    symbol_indices: SymbolIndices,
    const_values: ConstValues,
    symbol_table: DatSymbolTable,
    bytecode: Bytecode,
    /// Calls to patch with the address of the function, once every body has one
    calls: Vec<(Label, u32)>,
}

impl Compiler {
    pub fn new(symbol_indices: SymbolIndices, const_values: ConstValues) -> Self {
        Self {
            symbol_table: DatSymbolTable::new(&symbol_indices),
            symbol_indices,
            const_values,
            bytecode: Bytecode::new(),
            calls: Vec::new(),
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
        let items = || {
            files
                .iter()
                .flat_map(|file| file.ast.items.iter().map(|item| (file.id, item)))
        };

        // Symbols first, so that bodies can refer to any symbol and know its type
        let mut declared = Vec::new();
        for (file_id, item) in items() {
            let len = self.symbol_table.len() + item_symbols(item).len();

            let result = self.declare_item(span_files, file_id, item);
            declared.push(result.is_ok());
            if let Err(err) = result {
                diagnostics.report(file_id, err);
                self.symbol_table.recover_to(len);
            }
        }

        for ((file_id, item), declared) in items().zip(declared) {
            if declared && let Err(err) = self.define_item(item) {
                diagnostics.report(file_id, err);
            }
        }

        for (label, function) in std::mem::take(&mut self.calls) {
            if let Some(address) = self.symbol_table.body_address(function) {
                self.bytecode.patch(label, address);
            } else {
                // Functions that failed to compile have no address, which was already reported
            }
        }

        let mut out = Vec::new();
        self.symbol_table.encode(&mut out);
        self.bytecode
            .encode(&mut out)
            .expect("writing to a Vec can not fail");
        out
    }

    /// Emit the symbols of an item
    fn declare_item(
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
            daedalus_parser::Item::Var(var) => self.global_var(files, file_id, var),
        }
    }

    /// Emit the body of an item, if it has one
    fn define_item(&mut self, item: &daedalus_parser::Item) -> Result<()> {
        match item {
            daedalus_parser::Item::Instance(instance) => self.define_instance(instance),
            daedalus_parser::Item::Func(func) => self.define_func(func),
            daedalus_parser::Item::Prototype(prototype) => self.define_prototype(prototype),
            daedalus_parser::Item::ExternFunc(_)
            | daedalus_parser::Item::Class(_)
            | daedalus_parser::Item::Const(_)
            | daedalus_parser::Item::Var(_) => Ok(()),
        }
    }

    /// Set the address of a body that starts at the end of the bytecode
    fn start_body(&mut self, ident: &daedalus_parser::Ident) -> Result<()> {
        let id = self
            .lookup(&ident.span, &ident.raw.to_ascii_uppercase())?
            .id;
        let address = self.bytecode.next_available_address();
        self.symbol_table.set_body_address(id, address);
        Ok(())
    }

    fn extern_func(
        &mut self,
        file_id: FileId,
        func: &daedalus_parser::ExternFunctionDefinition,
    ) -> Result<()> {
        let name = symbol_name(&func.ident.raw);
        let ty = data_type(&func.ident.span, &func.ty)?;

        let args = func
            .args
            .iter()
            .map(|var| {
                let ident = symbol_name(&var.ident.raw);
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
        let name = symbol_name(&class.ident.raw);
        let span = files.symbol_span(file_id, &class.span, SpanKind::Block);

        let fields = class
            .fields
            .iter()
            .map(|var| {
                let ident = symbol_name(&var.ident.raw);
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

                let span = files.symbol_span(file_id, &var.span, SpanKind::Member);

                let count = match &var.kind {
                    daedalus_parser::VarKind::Value { .. } => 1,
                    daedalus_parser::VarKind::Array { size_init, .. } => {
                        self.array_size(size_init)?
                    }
                };

                Ok((ident, ty, count, span))
            })
            .collect::<Result<Vec<_>>>()?;

        let offset = builtin::class_offset(&name.to_string()).unwrap_or(0);
        self.symbol_table.class(name, span, &fields, offset);

        Ok(())
    }

    fn global_var(
        &mut self,
        files: &Files,
        file_id: FileId,
        var: &daedalus_parser::Var,
    ) -> Result<()> {
        let name = symbol_name(&var.ident.raw);
        let span = files.symbol_span(file_id, &var.span, SpanKind::Var);
        let (data_type, parent) = self.var_type(&var.span, &var.ty)?;

        let count = self.var_count(var)?;

        if data_type == DataType::Func {
            return Err(Error::unsupported(&var.span, "global func variables"));
        }

        self.symbol_table.var(name, span, data_type, count, parent);

        Ok(())
    }

    fn var_count(&self, var: &daedalus_parser::Var) -> Result<u32> {
        match &var.kind {
            daedalus_parser::VarKind::Value { init: None } => Ok(1),
            daedalus_parser::VarKind::Array {
                size_init,
                init: None,
            } => self.array_size(size_init),
            _ => Err(Error::invalid(
                &var.span,
                "variables can not be initialized",
            )),
        }
    }

    /// Type of a variable, and the class of instance variables
    fn var_type(
        &self,
        span: &Range<usize>,
        ty: &daedalus_parser::Ty,
    ) -> Result<(DataType, Option<u32>)> {
        if let Ok(data_type) = data_type(span, ty) {
            return Ok((data_type, None));
        }

        let class = self.lookup(span, &ty.raw.to_ascii_uppercase())?;
        if class.kind != SymbolKind::Class {
            return Err(Error::unsupported(
                span,
                format!("variables of type {:?}", class.kind),
            ));
        }

        Ok((DataType::Instance, Some(class.id)))
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
        let ident = symbol_name(&instance.ident.raw);
        let parent = instance.parent.raw.to_ascii_uppercase();
        let parent_id = self.lookup(&instance.parent.span, &parent)?.id;
        let span = files.symbol_span(file_id, &instance.span, SpanKind::Block);

        self.symbol_table.instance(ident, span, 0, parent_id);

        Ok(())
    }

    fn define_instance(&mut self, instance: &daedalus_parser::Instance) -> Result<()> {
        let parent = instance.parent.raw.to_ascii_uppercase();
        let parent = *self.lookup(&instance.parent.span, &parent)?;

        let class = if parent.kind == SymbolKind::Prototype {
            self.class_of_prototype(&instance.parent.span, parent.id)?
        } else {
            instance.parent.raw.to_ascii_uppercase()
        };

        self.start_body(&instance.ident)?;
        let mut block = self.bytecode.block_builder();

        // Instances of a prototype start by running it
        if parent.kind == SymbolKind::Prototype {
            self.calls.push((block.call_later(), parent.id));
        }

        let builder = BlockBuilder {
            parent: &class,
            symbol_indices: &self.symbol_indices,
            symbol_table: &mut self.symbol_table,
            block: &mut block,
            calls: &mut self.calls,
        };

        builder.visit_instance(instance)
    }

    /// Name of the class of prototype `id`
    fn class_of_prototype(&self, span: &Range<usize>, id: u32) -> Result<String> {
        let prototype = self
            .symbol_table
            .get(id)
            .ok_or_else(|| Error::invalid(span, "prototype failed to compile"))?;

        let class_id = prototype
            .parent
            .expect("prototype symbols always have a parent class");

        let class_name = self.symbol_indices.name(class_id);
        if self.symbol_indices.get(class_name).map(|class| class.kind) != Some(SymbolKind::Class) {
            return Err(Error::unsupported(span, "multi level inheritance"));
        }

        Ok(class_name.to_string())
    }

    fn func(
        &mut self,
        files: &Files,
        file_id: FileId,
        func: &daedalus_parser::FunctionDefinition,
    ) -> Result<()> {
        let ident_utf8 = func.ident.raw.to_ascii_uppercase();
        let ident = symbol_name(&ident_utf8);
        let span = files.symbol_span(file_id, &func.span, SpanKind::Block);

        let ret = data_type(&func.ident.span, &func.ty)?;

        self.symbol_table
            .func(ident, span, func.args.len() as u32, ret, 0);

        // ZenGin keeps the class of the last instance parameter for the parameters after it
        let mut param_class = None;
        for param in func.args.iter() {
            let (data_type, class) = self.var_type(&param.span, &param.ty)?;
            param_class = class.or(param_class);

            let span = files.symbol_span(file_id, &param.span, SpanKind::Member);

            let name = member_name(&ident_utf8, &param.ident);
            let count = self.var_count(param)?;
            self.symbol_table
                .var(name, span, data_type, count, param_class);
        }

        for local in function_locals(&func.block) {
            let (data_type, class) = self.var_type(&local.span, &local.ty)?;
            let span = files.symbol_span(file_id, &local.span, SpanKind::Member);

            let name = member_name(&ident_utf8, &local.ident);
            let count = self.var_count(local)?;
            self.symbol_table.var(name, span, data_type, count, class);
        }

        Ok(())
    }

    fn define_func(&mut self, func: &daedalus_parser::FunctionDefinition) -> Result<()> {
        let name = func.ident.raw.to_ascii_uppercase();

        self.start_body(&func.ident)?;
        let mut block = self.bytecode.block_builder();
        let builder = BlockBuilder {
            parent: &name,
            symbol_indices: &self.symbol_indices,
            symbol_table: &mut self.symbol_table,
            block: &mut block,
            calls: &mut self.calls,
        };

        builder.visit_function(func)
    }

    fn const_item(
        &mut self,
        files: &Files,
        file_id: FileId,
        item: &daedalus_parser::Const,
    ) -> Result<()> {
        let name = symbol_name(&item.ident.raw);
        let span = files.symbol_span(file_id, &item.span, SpanKind::Const);

        let Some(value) = self
            .const_values
            .map
            .get(&item.ident.raw.to_ascii_uppercase())
        else {
            // Evaluation failed, which was already reported
            return Ok(());
        };

        let values = match (&item.kind, value) {
            (daedalus_parser::ConstKind::Value { .. }, value) => std::slice::from_ref(value),
            (daedalus_parser::ConstKind::Array { size_init, .. }, Value::Array(values)) => {
                let size = self.array_size(size_init)?;
                if values.len() != size as usize {
                    return Err(Error::invalid(
                        &item.span,
                        format!(
                            "array of size {size} initialized with {} values",
                            values.len()
                        ),
                    ));
                }
                values.as_slice()
            }
            (daedalus_parser::ConstKind::Array { .. }, _) => {
                unreachable!("array constants evaluate to arrays")
            }
        };

        let data_type = data_type(&item.span, &item.ty)?;
        let mismatch = || Error::invalid(&item.span, format!("value is not a {data_type:?}"));

        let (stored_type, data) = match data_type {
            DataType::Int => {
                let values = values.iter().map(|value| match value {
                    Value::Int(v) => Ok(*v),
                    _ => Err(mismatch()),
                });
                (
                    DataType::Int,
                    SymbolData::Int(values.collect::<Result<_>>()?),
                )
            }
            DataType::Float => {
                let values = values.iter().map(|value| match value {
                    Value::Float(v) => Ok(*v),
                    Value::Int(v) => Ok(*v as f32),
                    _ => Err(mismatch()),
                });
                (
                    DataType::Float,
                    SymbolData::Float(values.collect::<Result<_>>()?),
                )
            }
            DataType::String => {
                let values = values.iter().map(|value| match value {
                    Value::String(v) => Ok(script_string(v)),
                    _ => Err(mismatch()),
                });
                (
                    DataType::String,
                    SymbolData::String(values.collect::<Result<_>>()?),
                )
            }
            DataType::Func => {
                let values = values.iter().map(|value| match value {
                    Value::Symbol(id) => Ok(*id as i32),
                    _ => Err(mismatch()),
                });
                (
                    DataType::Int,
                    SymbolData::Int(values.collect::<Result<_>>()?),
                )
            }
            _ => {
                return Err(Error::unsupported(
                    &item.span,
                    format!("constants of type {data_type:?}"),
                ));
            }
        };

        self.symbol_table.const_item(name, span, stored_type, data);

        Ok(())
    }

    fn prototype(
        &mut self,
        files: &Files,
        file_id: FileId,
        prototype: &daedalus_parser::Prototype,
    ) -> Result<()> {
        let ident = symbol_name(&prototype.ident.raw);
        let parent = prototype.parent.raw.to_ascii_uppercase();
        let parent_id = self.lookup(&prototype.parent.span, &parent)?.id;
        let span = files.symbol_span(file_id, &prototype.span, SpanKind::Block);

        self.symbol_table.prototype(ident, span, 0, parent_id);

        Ok(())
    }

    fn define_prototype(&mut self, prototype: &daedalus_parser::Prototype) -> Result<()> {
        let parent = prototype.parent.raw.to_ascii_uppercase();

        self.start_body(&prototype.ident)?;
        let mut block = self.bytecode.block_builder();
        let builder = BlockBuilder {
            parent: &parent,
            symbol_indices: &self.symbol_indices,
            symbol_table: &mut self.symbol_table,
            block: &mut block,
            calls: &mut self.calls,
        };

        builder.visit_prototype(prototype)
    }

    fn lookup(&self, span: &Range<usize>, name: &str) -> Result<&SymbolIndex> {
        self.symbol_indices
            .get(name)
            .ok_or_else(|| Error::undefined_symbol(span, name))
    }
}

fn member_name(parent: &str, member: &daedalus_parser::Ident) -> ZString {
    symbol_name(&format!("{parent}.{}", member.raw))
}

fn data_type(span: &Range<usize>, ty: &daedalus_parser::Ty) -> Result<DataType> {
    DataType::from_str(&ty.raw.to_ascii_lowercase())
        .map_err(|()| Error::new(span, ErrorKind::UnknownType(ty.raw.clone())))
}
