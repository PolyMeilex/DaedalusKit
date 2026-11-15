use std::ops::Range;

use daedalus_bytecode::Instruction;
use daedalus_parser::{AssocOp, BlockItem, Expr, ExprKind, FunctionCall, Ident, Lit, LitKind};
use dat_file::properties::{DataType, PropFlag};
use zstring::ZString;

use crate::{
    dat_symbol_table::DatSymbolTable,
    error::{Error, Result},
    symbol_indices::{SymbolIndex, SymbolIndices, SymbolKind},
};

pub struct BlockBuilder<'a, 'b> {
    pub(crate) parent: &'a str,
    pub(crate) symbol_indices: &'a SymbolIndices,
    pub(crate) symbol_table: &'a mut DatSymbolTable,
    pub(crate) block: &'a mut daedalus_bytecode::BytecodeBlockBuilder<'b>,
}

impl<'a, 'b> BlockBuilder<'a, 'b> {
    pub fn visit_instance(self, instance: &daedalus_parser::Instance) -> Result<()> {
        self.visit_block(&instance.block)
    }

    pub fn visit_prototype(self, prototype: &daedalus_parser::Prototype) -> Result<()> {
        self.visit_block(&prototype.block)
    }

    pub fn visit_function(self, function: &daedalus_parser::FunctionDefinition) -> Result<()> {
        self.visit_block(&function.block)
    }

    fn visit_block(mut self, block: &daedalus_parser::Block) -> Result<()> {
        for item in block.items.iter() {
            self.visit_block_item(item)?;
        }

        if block.is_implicit_return() {
            self.block.ret();
        }

        Ok(())
    }

    fn visit_block_item(&mut self, item: &BlockItem) -> Result<()> {
        match item {
            BlockItem::Expr(expr) => self.visit_expr(expr),
            BlockItem::Var(var) => Err(Error::unsupported(&var.span, "local variables")),
            BlockItem::If(stmt) => Err(Error::unsupported(&stmt.span, "if statements")),
            BlockItem::Return(stmt) => Err(Error::unsupported(&stmt.span, "return statements")),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<()> {
        match &expr.kind {
            ExprKind::Binary(AssocOp::Assign, left, right) => self.visit_assign(left, right),
            ExprKind::Call(call) => self.visit_call(call),
            _ => Err(Error::unsupported(&expr.span, "expression statement")),
        }
    }

    fn visit_assign(&mut self, left: &Expr, right: &Expr) -> Result<()> {
        match &left.kind {
            ExprKind::Ident(ident) => {
                let value = literal(right)?;

                let name = format!("{}.{}", self.parent, ident.raw.to_ascii_uppercase());
                let destination = self.lookup(&ident.span, &name)?.id;

                let destination_symbol = self.symbol_table.get(destination).ok_or_else(|| {
                    Error::invalid(&ident.span, "assignment to a symbol that failed to compile")
                })?;

                let props = &destination_symbol.props.elem_props;
                if !props.flags().contains(PropFlag::CLASS_VAR) {
                    return Err(Error::unsupported(
                        &left.span,
                        "assignment to a non class variable",
                    ));
                }

                match (props.data_type(), &value.kind) {
                    (DataType::Float, LitKind::Intager(value)) => {
                        self.block.var_assign_float((destination, 0), *value as f32);
                    }
                    (DataType::Float, LitKind::Float(value)) => {
                        self.block.var_assign_float((destination, 0), *value);
                    }
                    (DataType::Int, LitKind::Intager(value)) => {
                        self.block.var_assign_int((destination, 0), *value);
                    }
                    (ty, _) => {
                        return Err(Error::unsupported(
                            &right.span,
                            format!("assignment of this value to a {ty:?} variable"),
                        ));
                    }
                };
            }
            ExprKind::Index(array, index) => {
                let ExprKind::Ident(array) = &array.kind else {
                    return Err(Error::unsupported(&array.span, "indexing a non identifier"));
                };
                let name = format!("{}.{}", self.parent, array.raw.to_ascii_uppercase());
                let array_id = self.lookup(&array.span, &name)?.id;

                let LitKind::Intager(index_value) = literal(index)?.kind else {
                    return Err(Error::invalid(&index.span, "non integer array index"));
                };
                let index_value = u8::try_from(index_value)
                    .map_err(|_| Error::invalid(&index.span, "array index out of range"))?;

                let LitKind::Intager(value) = literal(right)?.kind else {
                    return Err(Error::unsupported(&right.span, "non integer array element"));
                };

                self.block.var_assign_int((array_id, index_value), value);
            }
            _ => return Err(Error::unsupported(&left.span, "assignment destination")),
        }

        Ok(())
    }

    fn lookup(&self, span: &Range<usize>, name: &str) -> Result<&'a SymbolIndex> {
        let symbol_indices: &'a SymbolIndices = self.symbol_indices;
        symbol_indices
            .get(name)
            .ok_or_else(|| Error::undefined_symbol(span, name))
    }

    fn visit_reference(&self, ident: &Ident) -> Result<&'a SymbolIndex> {
        self.lookup(&ident.span, &ident.raw.to_ascii_uppercase())
    }

    fn visit_call_arg(&mut self, arg: &Expr) -> Result<()> {
        match &arg.kind {
            ExprKind::Ident(arg) => {
                let symbol = self.visit_reference(arg)?;
                self.block
                    .push_instruction(if SymbolKind::Instance == symbol.kind {
                        Instruction::push_var_instance(symbol.id)
                    } else {
                        Instruction::push_var(symbol.id)
                    });
            }
            ExprKind::Lit(lit) => match &lit.kind {
                LitKind::Intager(v) => {
                    self.block.push_instruction(Instruction::push_int(v.abs()));
                    if v.is_negative() {
                        self.block.push_instruction(Instruction::negate());
                    }
                }
                LitKind::Float(v) => {
                    // Well that's fun, it turns out floats were ints all along
                    let v = v.to_le_bytes();
                    let v = i32::from_le_bytes(v);
                    self.block.push_instruction(Instruction::push_int(v));
                }
                LitKind::String(v) => {
                    self.block.push_instruction(Instruction::push_var(
                        self.symbol_table.string(ZString::from(v.as_bytes())),
                    ));
                }
            },
            _ => return Err(Error::unsupported(&arg.span, "call argument")),
        };

        Ok(())
    }

    // Mdl_SetVisual(self, "HUMANS.MDS")
    // Mdl_SetVisualBody(self, "hum_body_Naked0", 9, 0, "Hum_Head_Pony", 18, 0, -1);
    fn visit_call(&mut self, call: &FunctionCall) -> Result<()> {
        for arg in call.args.iter() {
            self.visit_call_arg(arg)?;
        }

        let symbol = self.visit_reference(&call.ident)?;
        match symbol.kind {
            SymbolKind::ExternFunction => {
                self.block.extend(&[Instruction::call_extern(symbol.id)]);
            }
            SymbolKind::Function => {
                self.block.extend(&[Instruction::call(symbol.id)]);
            }
            _ => {
                return Err(Error::invalid(
                    &call.ident.span,
                    format!("`{}` is not a function", call.ident.raw),
                ));
            }
        }

        Ok(())
    }
}

fn literal(expr: &Expr) -> Result<&Lit> {
    match &expr.kind {
        ExprKind::Lit(lit) => Ok(lit),
        _ => Err(Error::unsupported(&expr.span, "non literal value")),
    }
}
