use daedalus_bytecode::Instruction;
use daedalus_parser::{AssocOp, BlockItem, Expr, ExprKind, FunctionCall, Ident, LitKind};
use dat_file::properties::{DataType, PropFlag};
use zstring::ZString;

use crate::{
    dat_symbol_table::DatSymbolTable,
    symbol_indices::{SymbolIndex, SymbolIndices, SymbolKind},
};

pub struct BlockBuilder<'a, 'b> {
    pub(crate) parent: &'a str,
    pub(crate) symbol_indices: &'a SymbolIndices,
    pub(crate) symbol_table: &'a mut DatSymbolTable,
    pub(crate) block: &'a mut daedalus_bytecode::BytecodeBlockBuilder<'b>,
}

impl<'a, 'b> BlockBuilder<'a, 'b> {
    pub fn visit_instance(mut self, instance: &daedalus_parser::Instance) {
        for item in instance.block.items.iter() {
            self.visit_block_item(item);
        }

        self.close(&instance.block);
    }

    pub fn visit_prototype(mut self, prototype: &daedalus_parser::Prototype) {
        for item in prototype.block.items.iter() {
            self.visit_block_item(item);
        }

        self.close(&prototype.block);
    }

    pub fn visit_function(mut self, function: &daedalus_parser::FunctionDefinition) {
        for item in function.block.items.iter() {
            self.visit_block_item(item);
        }

        self.close(&function.block);
    }

    fn visit_block_item(&mut self, item: &BlockItem) {
        match item {
            BlockItem::Expr(expr) => self.visit_expr(expr),
            _ => todo!(),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Binary(op, left, right) => self.visit_binary_op(op, left, right),
            ExprKind::Call(call) => self.visit_call(call),
            _ => todo!(),
        }
    }

    fn visit_binary_op(&mut self, op: &AssocOp, left: &Expr, right: &Expr) {
        assert_eq!(*op, AssocOp::Assign);

        match &left.kind {
            ExprKind::Ident(ident) => {
                let ExprKind::Lit(value) = &right.kind else {
                    todo!()
                };

                let symbol = ident.raw.to_ascii_uppercase();
                let symbol = format!("{}.{symbol}", self.parent);

                let destination = self.symbol_indices.get(&symbol).unwrap().id;

                let destination_symbol = &self.symbol_table.symbols[destination as usize];
                assert!(
                    destination_symbol
                        .props
                        .elem_props
                        .flags()
                        .contains(PropFlag::CLASS_VAR),
                    "TODO"
                );

                match destination_symbol.props.elem_props.data_type() {
                    DataType::Void => todo!(),
                    DataType::Float => {
                        let value = match value.kind {
                            LitKind::Intager(value) => value as f32,
                            LitKind::Float(value) => value,
                            _ => todo!(),
                        };
                        self.block.var_assign_float((destination, 0), value)
                    }
                    DataType::Int => {
                        let value = match value.kind {
                            LitKind::Intager(value) => value,
                            _ => todo!(),
                        };
                        self.block.var_assign_int((destination, 0), value)
                    }
                    DataType::String => todo!(),
                    DataType::Class => todo!(),
                    DataType::Func => todo!(),
                    DataType::Prototype => todo!(),
                    DataType::Instance => todo!(),
                };
            }
            ExprKind::Index(symbol, id) => {
                let ExprKind::Ident(symbol) = &symbol.kind else {
                    todo!()
                };
                let symbol = symbol.raw.to_ascii_uppercase();
                let symbol = format!("{}.{symbol}", self.parent);

                let ExprKind::Lit(id) = &id.kind else { todo!() };
                let LitKind::Intager(id) = id.kind else {
                    todo!()
                };
                let id = u8::try_from(id).expect("TODO");

                let ExprKind::Lit(value) = &right.kind else {
                    todo!()
                };
                let LitKind::Intager(value) = value.kind else {
                    todo!()
                };

                // "C_NPC.ATTRIBUTE"
                let npc_attributes = self.symbol_indices.get(&symbol).unwrap().id;

                self.block.var_assign_int((npc_attributes, id), value);
            }
            expr => todo!("{expr:?}"),
        }
    }

    fn visit_reference(&self, ident: &Ident) -> &SymbolIndex {
        let name = ident.raw.to_ascii_uppercase();
        self.symbol_indices.get(&name).expect("TODO")
    }

    fn visit_call_arg(&mut self, arg: &Expr) {
        match &arg.kind {
            ExprKind::Ident(arg) => {
                let symbol = self.visit_reference(arg);
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
            _ => {
                todo!()
            }
        };
    }

    // Mdl_SetVisual(self, "HUMANS.MDS")
    // Mdl_SetVisualBody(self, "hum_body_Naked0", 9, 0, "Hum_Head_Pony", 18, 0, -1);
    fn visit_call(&mut self, call: &FunctionCall) {
        let ident = call.ident.raw.to_ascii_uppercase();

        for arg in call.args.iter() {
            self.visit_call_arg(arg);
        }

        let symbol = self.symbol_indices.get(&ident).unwrap();
        match symbol.kind {
            SymbolKind::ExternFunction => {
                self.block.extend(&[Instruction::call_extern(symbol.id)]);
            }
            SymbolKind::Function => {
                self.block.extend(&[Instruction::call(symbol.id)]);
            }
            kind => todo!("{kind:?}"),
        }
    }

    fn close(self, block: &daedalus_parser::Block) {
        if block.is_implicit_return() {
            self.block.ret();
        }
    }
}
