use std::ops::Range;

use daedalus_bytecode::{BytecodeBlockBuilder, Instruction, Label, Opcode};
use daedalus_parser::{
    AssocOp, BlockItem, Expr, ExprKind, FunctionCall, Ident, IfStatement, LitKind, UnaryOp,
};
use dat_file::properties::{DataType, PropFlag};
use num_traits::FromPrimitive;

use crate::{
    const_eval::{ConstValues, Value},
    dat_symbol_table::DatSymbolTable,
    encoding::script_string,
    error::{Error, Result},
    symbol_indices::{SymbolIndices, SymbolKind},
};

#[derive(Debug, Clone, Copy)]
pub enum Scope<'a> {
    /// Upper cased name of the function, whose parameters and locals are in scope
    Function(&'a str),
    /// Instance or prototype, whose class fields are in scope
    Instance {
        /// Upper cased name of the class
        class: &'a str,
        /// Upper cased name of the instance, aka. `self`
        instance: Option<&'a str>,
    },
}

pub struct BlockBuilder<'a, 'b> {
    pub(crate) scope: Scope<'a>,
    pub(crate) symbol_indices: &'a SymbolIndices,
    pub(crate) symbol_table: &'a mut DatSymbolTable,
    pub(crate) const_values: &'a ConstValues,
    pub(crate) block: &'a mut BytecodeBlockBuilder<'b>,
    /// Calls to patch with the address of the function
    pub(crate) calls: &'a mut Vec<(Label, u32)>,
}

#[derive(Debug, Clone, Copy)]
struct Symbol {
    id: u32,
    kind: SymbolKind,
    data_type: DataType,
}

#[derive(Debug, Clone, Copy)]
struct Destination {
    /// Instance whose field is assigned
    instance: Option<u32>,
    symbol: u32,
    index: u8,
    data_type: DataType,
}

impl BlockBuilder<'_, '_> {
    pub fn visit_instance(self, instance: &daedalus_parser::Instance) -> Result<()> {
        self.visit_body(&instance.block)
    }

    pub fn visit_prototype(self, prototype: &daedalus_parser::Prototype) -> Result<()> {
        self.visit_body(&prototype.block)
    }

    pub fn visit_function(self, function: &daedalus_parser::FunctionDefinition) -> Result<()> {
        for param in function.args.iter().rev() {
            let param = self.resolve(&param.ident)?;
            let instructions = match param.data_type {
                DataType::Instance => [
                    Instruction::push_var_instance(param.id),
                    Instruction::op(Opcode::MovVI),
                ],
                data_type => [Instruction::push_var(param.id), move_instruction(data_type)],
            };
            self.block.extend(&instructions);
        }

        self.visit_body(&function.block)
    }

    fn visit_body(mut self, block: &daedalus_parser::Block) -> Result<()> {
        self.visit_block(block)?;
        self.block.ret();
        Ok(())
    }

    fn visit_block(&mut self, block: &daedalus_parser::Block) -> Result<()> {
        for item in block.items.iter() {
            match item {
                BlockItem::Expr(expr) => self.visit_statement(expr)?,
                BlockItem::If(stmt) => self.visit_if(stmt)?,
                BlockItem::Return(stmt) => self.visit_return(stmt)?,
                // Locals are symbols, they don't generate any code
                BlockItem::Var(_) => {}
            }
        }
        Ok(())
    }

    fn visit_statement(&mut self, expr: &Expr) -> Result<()> {
        match &expr.kind {
            ExprKind::Binary(op, left, right) if is_assignment(op) => {
                self.visit_assignment(op, left, right)
            }
            // The value is left on the stack
            _ => self.push_value(expr, DataType::Void).map(|_| ()),
        }
    }

    fn visit_if(&mut self, stmt: &IfStatement) -> Result<()> {
        let mut to_end = Vec::new();

        let mut branch = Some(stmt);
        while let Some(stmt) = branch {
            let to_next = match &stmt.condition {
                Some(condition) => {
                    self.push_value(condition, DataType::Int)?;
                    Some(self.block.forward_jump(Opcode::Bz))
                }
                None => None,
            };

            self.visit_block(&stmt.block)?;

            if stmt.next.is_some() {
                to_end.push(self.block.forward_jump(Opcode::B));
            }
            if let Some(label) = to_next {
                self.block.bind(label);
            }

            branch = stmt.next.as_deref();
        }

        for label in to_end {
            self.block.bind(label);
        }

        Ok(())
    }

    fn visit_return(&mut self, stmt: &daedalus_parser::ReturnStatement) -> Result<()> {
        if let Some(value) = &stmt.expr {
            let Scope::Function(function) = self.scope else {
                return Err(Error::invalid(
                    &stmt.span,
                    "return with a value outside of a function",
                ));
            };
            let function = self
                .symbol_indices
                .get(function)
                .expect("function is declared")
                .id;
            let return_type = self.return_type(function);
            self.push_value(value, return_type)?;
        }

        self.block.ret();
        Ok(())
    }

    fn visit_assignment(&mut self, op: &AssocOp, left: &Expr, right: &Expr) -> Result<()> {
        let destination = self.destination(left)?;

        self.push_value(right, destination.data_type)?;

        if let Some(instance) = destination.instance {
            self.block
                .push_instruction(Instruction::set_instance(instance));
        }
        self.block.push_instruction(match destination.data_type {
            DataType::Instance => Instruction::push_var_instance(destination.symbol),
            _ => push_element(destination.symbol, destination.index),
        });

        let instruction = match op {
            AssocOp::Assign => move_instruction(destination.data_type),
            _ if destination.data_type != DataType::Int => {
                return Err(Error::invalid(
                    &left.span,
                    "compound assignment to a non int",
                ));
            }
            AssocOp::AddAssign => Instruction::op(Opcode::AddMovI),
            AssocOp::SubtractAssign => Instruction::op(Opcode::SubMovI),
            AssocOp::MultiplyAssign => Instruction::op(Opcode::MulMovI),
            AssocOp::DivideAssign => Instruction::op(Opcode::DivMovI),
            _ => unreachable!("{op:?} is not an assignment"),
        };
        self.block.push_instruction(instruction);

        Ok(())
    }

    fn destination(&self, expr: &Expr) -> Result<Destination> {
        let (instance, field, index) = match &expr.kind {
            ExprKind::Ident(ident) => (None, ident, 0),
            ExprKind::Field(instance, field) => (Some(self.instance(instance)?), field, 0),
            ExprKind::Index(array, index) => {
                let index = self.const_index(index)?;
                match &array.kind {
                    ExprKind::Ident(ident) => (None, ident, index),
                    ExprKind::Field(instance, field) => {
                        (Some(self.instance(instance)?), field, index)
                    }
                    _ => {
                        return Err(Error::unsupported(&array.span, "indexing this expression"));
                    }
                }
            }
            _ => return Err(Error::invalid(&expr.span, "can not be assigned to")),
        };

        let symbol = match instance {
            Some(instance) => self.field(instance, field)?,
            None => self.resolve(field)?,
        };

        if !matches!(
            symbol.kind,
            SymbolKind::Var | SymbolKind::FunctionArg | SymbolKind::ClassVar | SymbolKind::Instance
        ) {
            return Err(Error::invalid(&expr.span, "can not be assigned to"));
        }

        Ok(Destination {
            instance: instance.map(|instance| instance.id),
            symbol: symbol.id,
            index,
            data_type: symbol.data_type,
        })
    }

    /// Push the value of an expression where a value of type `expected` is expected, returns the
    /// type of the pushed value
    fn push_value(&mut self, expr: &Expr, expected: DataType) -> Result<DataType> {
        let data_type = match &expr.kind {
            ExprKind::Lit(lit) => match &lit.kind {
                LitKind::Intager(value) if expected == DataType::Float => {
                    self.push_float(*value as f32);
                    DataType::Float
                }
                LitKind::Intager(value) => {
                    self.block
                        .push_instruction(Instruction::push_int(value.wrapping_abs()));
                    if *value < 0 {
                        self.block.push_instruction(Instruction::op(Opcode::Negate));
                    }
                    DataType::Int
                }
                LitKind::Float(value) => {
                    self.push_float(*value);
                    DataType::Float
                }
                LitKind::String(value) => {
                    let symbol = self.symbol_table.string(script_string(value));
                    self.block.push_instruction(Instruction::push_var(symbol));
                    DataType::String
                }
            },
            ExprKind::Ident(ident) => match self.resolve(ident) {
                Ok(symbol) => {
                    let symbol = self.instance_self(symbol, expected);
                    self.push_symbol(symbol, expected)
                }
                // TODO: Kinda shit hardcoding this here, but I want it to get to compile G2NotR first before cleaning this up
                Err(err) => match ident.raw.to_ascii_uppercase().as_str() {
                    // No function, an invalid symbol id
                    "NOFUNC" => {
                        self.block.push_instruction(Instruction::push_int(-1));
                        DataType::Func
                    }
                    // No instance, the instance helper symbol stands in for it
                    "NULL" => {
                        let instance_help = Symbol {
                            id: 0,
                            kind: SymbolKind::Other,
                            data_type: DataType::Instance,
                        };
                        self.push_symbol(instance_help, expected)
                    }
                    _ => return Err(err),
                },
            },
            ExprKind::Field(instance, field) => {
                let instance = self.instance(instance)?;
                let field = self.field(instance, field)?;
                self.block.extend(&[
                    Instruction::set_instance(instance.id),
                    Instruction::push_var(field.id),
                ]);
                field.data_type
            }
            ExprKind::Index(array, index) => {
                let index = self.const_index(index)?;
                let array = match &array.kind {
                    ExprKind::Ident(ident) => self.resolve(ident)?,
                    ExprKind::Field(instance, field) => {
                        let instance = self.instance(instance)?;
                        self.block
                            .push_instruction(Instruction::set_instance(instance.id));
                        self.field(instance, field)?
                    }
                    _ => {
                        return Err(Error::unsupported(&array.span, "indexing this expression"));
                    }
                };
                self.block.push_instruction(push_element(array.id, index));
                array.data_type
            }
            ExprKind::Call(call) => self.visit_call(call)?,
            ExprKind::Paren(inner) => self.push_value(inner, expected)?,
            ExprKind::Unary(op, operand) => {
                self.push_value(operand, DataType::Int)?;
                self.block.push_instruction(Instruction::op(match op {
                    UnaryOp::Negative => Opcode::Negate,
                    UnaryOp::Not => Opcode::Not,
                    UnaryOp::Complement => Opcode::Cmpl,
                }));
                DataType::Int
            }
            ExprKind::Binary(op, _, _) if is_assignment(op) => {
                return Err(Error::invalid(&expr.span, "assignment used as a value"));
            }
            ExprKind::Binary(op, left, right) => {
                self.push_value(right, DataType::Int)?;
                self.push_value(left, DataType::Int)?;
                self.block
                    .push_instruction(Instruction::op(binary_opcode(op)));
                DataType::Int
            }
        };

        Ok(data_type)
    }

    fn instance_self(&self, symbol: Symbol, expected: DataType) -> Symbol {
        let Scope::Instance {
            instance: Some(instance),
            ..
        } = self.scope
        else {
            return symbol;
        };

        if expected != DataType::Instance || self.symbol_indices.name(symbol.id) != "SELF" {
            return symbol;
        }

        let instance = self
            .symbol_indices
            .get(instance)
            .expect("instance is declared");
        Symbol {
            id: instance.id,
            kind: instance.kind,
            data_type: DataType::Instance,
        }
    }

    fn push_float(&mut self, value: f32) {
        // The VM stack holds ints, floats are pushed as their bits
        let bits = i32::from_le_bytes(value.to_le_bytes());
        self.block.push_instruction(Instruction::push_int(bits));
    }

    fn push_symbol(&mut self, symbol: Symbol, expected: DataType) -> DataType {
        let (instruction, data_type) = match (symbol.kind, symbol.data_type) {
            (SymbolKind::Function | SymbolKind::ExternFunction, _) => {
                (Instruction::push_int(symbol.id as i32), DataType::Func)
            }
            (_, DataType::Instance | DataType::Prototype) if expected == DataType::Instance => (
                Instruction::push_var_instance(symbol.id),
                DataType::Instance,
            ),
            (_, DataType::Instance | DataType::Prototype) => {
                (Instruction::push_int(symbol.id as i32), DataType::Int)
            }
            (_, data_type) => (Instruction::push_var(symbol.id), data_type),
        };

        self.block.push_instruction(instruction);
        data_type
    }

    /// Push the arguments and call, returns the return type of the function
    fn visit_call(&mut self, call: &FunctionCall) -> Result<DataType> {
        let function = self.resolve(&call.ident)?;
        if !matches!(
            function.kind,
            SymbolKind::Function | SymbolKind::ExternFunction
        ) {
            return Err(Error::invalid(
                &call.ident.span,
                format!("`{}` is not a function", call.ident.raw),
            ));
        }

        let param_count = self.symbol(&call.ident.span, function.id)?.count();
        if call.args.len() != param_count as usize {
            return Err(Error::invalid(
                &call.ident.span,
                format!("expected {param_count} arguments, got {}", call.args.len()),
            ));
        }

        // Parameters follow the function in the symbol table
        for (param, arg) in (function.id + 1..).zip(call.args.iter()) {
            let param_type = self.symbol(&arg.span, param)?.data_type();
            self.push_value(arg, param_type)?;
        }

        match function.kind {
            SymbolKind::ExternFunction => {
                self.block
                    .push_instruction(Instruction::call_extern(function.id));
            }
            _ => self.calls.push((self.block.call_later(), function.id)),
        }

        Ok(self.return_type(function.id))
    }

    fn return_type(&self, function: u32) -> DataType {
        let function = self
            .symbol_table
            .get(function)
            .expect("callers checked that the function compiled");

        if function.props.elem_props.flags().contains(PropFlag::RETURN) {
            DataType::from_i32(function.props.off_cls_ret).expect("return type is a data type")
        } else {
            DataType::Void
        }
    }

    /// Symbol an identifier refers to in the current scope
    fn resolve(&self, ident: &Ident) -> Result<Symbol> {
        let name = ident.raw.to_ascii_uppercase();
        let scoped = match self.scope {
            Scope::Function(parent) | Scope::Instance { class: parent, .. } => {
                format!("{parent}.{name}")
            }
        };

        let index = self
            .symbol_indices
            .get(&scoped)
            .or_else(|| self.symbol_indices.get(&name))
            .ok_or_else(|| Error::undefined_symbol(&ident.span, &ident.raw))?;

        Ok(Symbol {
            id: index.id,
            kind: index.kind,
            data_type: self.symbol(&ident.span, index.id)?.data_type(),
        })
    }

    /// Instance whose field is accessed
    fn instance(&self, expr: &Expr) -> Result<Symbol> {
        let ExprKind::Ident(ident) = &expr.kind else {
            return Err(Error::unsupported(
                &expr.span,
                "field access on this expression",
            ));
        };

        let instance = self.resolve(ident)?;
        if instance.data_type != DataType::Instance {
            return Err(Error::invalid(&expr.span, "not an instance"));
        }
        Ok(instance)
    }

    /// Field of the class of an instance
    fn field(&self, instance: Symbol, field: &Ident) -> Result<Symbol> {
        let class = self.class_of(&field.span, instance.id)?;
        let name = format!(
            "{}.{}",
            self.symbol_indices.name(class),
            field.raw.to_ascii_uppercase()
        );

        let index = self
            .symbol_indices
            .get(&name)
            .ok_or_else(|| Error::undefined_symbol(&field.span, &name))?;

        Ok(Symbol {
            id: index.id,
            kind: index.kind,
            data_type: self.symbol(&field.span, index.id)?.data_type(),
        })
    }

    /// Class of an instance, which can be defined with a prototype in between
    fn class_of(&self, span: &Range<usize>, instance: u32) -> Result<u32> {
        let mut parent = self
            .symbol(span, instance)?
            .parent
            .ok_or_else(|| Error::invalid(span, "instance has no class"))?;

        if self.symbol(span, parent)?.data_type() == DataType::Prototype {
            parent = self
                .symbol(span, parent)?
                .parent
                .ok_or_else(|| Error::invalid(span, "prototype has no class"))?;
        }

        Ok(parent)
    }

    /// Array indices have to be known at compile time
    fn const_index(&self, expr: &Expr) -> Result<u8> {
        let value = match &expr.kind {
            ExprKind::Lit(lit) => match lit.kind {
                LitKind::Intager(value) => value,
                _ => return Err(Error::invalid(&expr.span, "non integer array index")),
            },
            ExprKind::Ident(ident) => {
                match self.const_values.map.get(&ident.raw.to_ascii_uppercase()) {
                    Some(Value::Int(value)) => *value,
                    _ => {
                        return Err(Error::invalid(&expr.span, "array index is not a constant"));
                    }
                }
            }
            _ => return Err(Error::unsupported(&expr.span, "array index expression")),
        };

        u8::try_from(value).map_err(|_| Error::invalid(&expr.span, "array index out of range"))
    }

    fn symbol(&self, span: &Range<usize>, id: u32) -> Result<&dat_file::Symbol> {
        self.symbol_table
            .get(id)
            .ok_or_else(|| Error::invalid(span, "refers to a symbol that failed to compile"))
    }
}

/// Element props of a symbol
trait SymbolExt {
    fn data_type(&self) -> DataType;
    fn count(&self) -> u32;
}

impl SymbolExt for dat_file::Symbol {
    fn data_type(&self) -> DataType {
        self.props.elem_props.data_type()
    }

    fn count(&self) -> u32 {
        self.props.elem_props.count()
    }
}

fn is_assignment(op: &AssocOp) -> bool {
    matches!(
        op,
        AssocOp::Assign
            | AssocOp::AddAssign
            | AssocOp::SubtractAssign
            | AssocOp::MultiplyAssign
            | AssocOp::DivideAssign
    )
}

fn binary_opcode(op: &AssocOp) -> Opcode {
    match op {
        AssocOp::Add => Opcode::Add,
        AssocOp::Subtract => Opcode::Sub,
        AssocOp::Multiply => Opcode::Mul,
        AssocOp::Divide => Opcode::Div,
        AssocOp::Modulo => Opcode::Mod,
        AssocOp::BitOr => Opcode::Or,
        AssocOp::BitAnd => Opcode::AndB,
        AssocOp::Less => Opcode::Lt,
        AssocOp::Greater => Opcode::Gt,
        AssocOp::LessEqual => Opcode::Lte,
        AssocOp::GreaterEqual => Opcode::Gte,
        AssocOp::Equal => Opcode::Eq,
        AssocOp::NotEqual => Opcode::Neq,
        AssocOp::Or => Opcode::Orr,
        AssocOp::And => Opcode::And,
        AssocOp::ShiftLeft => Opcode::Lsl,
        AssocOp::ShiftRight => Opcode::Lsr,
        AssocOp::Assign
        | AssocOp::AddAssign
        | AssocOp::SubtractAssign
        | AssocOp::MultiplyAssign
        | AssocOp::DivideAssign => unreachable!("{op:?} is an assignment"),
    }
}

/// Move into a variable of a given type
fn move_instruction(data_type: DataType) -> Instruction {
    Instruction::op(match data_type {
        DataType::Int => Opcode::MovInt,
        DataType::Float => Opcode::MovF,
        DataType::String => Opcode::MovS,
        DataType::Func => Opcode::MovVF,
        DataType::Instance => Opcode::MovVI,
        DataType::Void | DataType::Class | DataType::Prototype => {
            unreachable!("{data_type:?} is not a variable type")
        }
    })
}

/// Push an array element, element 0 is pushed as the array itself
fn push_element(symbol: u32, index: u8) -> Instruction {
    match index {
        0 => Instruction::push_var(symbol),
        index => Instruction::push_var_array(symbol, index),
    }
}
