#![allow(dead_code)]

use std::collections::HashMap;

use daedalus_parser::{Expr, ExprKind, LitKind, UnaryOp};

use crate::{
    error::{Diagnostics, Error, Result},
    files::File,
    symbol_indices::SymbolIndices,
};

#[derive(Debug)]
pub enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Array(Vec<Value>),
    Symbol(u32),
}

/// Const nodes in the tree
#[derive(Default)]
struct ConstNodes<'a> {
    map: HashMap<String, &'a daedalus_parser::Const>,
}

impl<'a> ConstNodes<'a> {
    fn visit_files(&mut self, files: impl IntoIterator<Item = &'a File>) {
        for file in files {
            self.visit_file(file);
        }
    }

    fn visit_file(&mut self, file: &'a File) {
        for item in file.ast.items.iter() {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &'a daedalus_parser::Item) {
        if let daedalus_parser::Item::Const(item) = item {
            self.map.insert(item.ident.raw.to_ascii_uppercase(), item);
        }
    }
}

pub struct ConstValues {
    pub map: HashMap<String, Value>,
}

impl ConstValues {
    pub fn build<'a>(
        files: impl IntoIterator<Item = &'a File> + Clone,
        indices: &'a SymbolIndices,
        diagnostics: &mut Diagnostics,
    ) -> Self {
        let mut map = ConstNodes::default();
        map.visit_files(files.clone());

        let mut eval = ConstEvaluator { indices, map };

        let mut map = HashMap::new();
        for file in files {
            for item in file.ast.items.iter() {
                if let daedalus_parser::Item::Const(item) = item {
                    match eval.visit_const(item) {
                        Ok(value) => {
                            map.insert(item.ident.raw.to_ascii_uppercase(), value);
                        }
                        Err(err) => diagnostics.report(file.id, err),
                    }
                }
            }
        }

        Self { map }
    }
}

struct ConstEvaluator<'a> {
    map: ConstNodes<'a>,
    indices: &'a SymbolIndices,
}

impl<'a> ConstEvaluator<'a> {
    fn visit_const(&mut self, item: &daedalus_parser::Const) -> Result<Value> {
        match &item.kind {
            daedalus_parser::ConstKind::Value { init } => self.visit_expr(init),
            daedalus_parser::ConstKind::Array { size_init: _, init } => {
                let values = init.iter().map(|expr| self.visit_expr(expr));
                Ok(Value::Array(values.collect::<Result<_>>()?))
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        let unsupported = |what: &str| Err(Error::unsupported(&expr.span, what));

        let value = match &expr.kind {
            ExprKind::Binary(op, left, right) => {
                let left = self.visit_expr(left)?;
                let right = self.visit_expr(right)?;

                let (left, right) = match (left, right) {
                    (Value::Int(l), Value::Int(r)) => (l, r),
                    _ => return unsupported("non integer operands in a constant expression"),
                };

                match op {
                    daedalus_parser::AssocOp::Add => Value::Int(left + right),
                    daedalus_parser::AssocOp::Subtract => Value::Int(left - right),
                    daedalus_parser::AssocOp::Equal => Value::Int((left == right) as i32),
                    daedalus_parser::AssocOp::NotEqual => Value::Int((left != right) as i32),
                    daedalus_parser::AssocOp::Less => Value::Int((left < right) as i32),
                    daedalus_parser::AssocOp::LessEqual => Value::Int((left <= right) as i32),
                    daedalus_parser::AssocOp::Greater => Value::Int((left > right) as i32),
                    daedalus_parser::AssocOp::GreaterEqual => Value::Int((left >= right) as i32),
                    daedalus_parser::AssocOp::And => Value::Int((left > 0 && right > 0) as i32),
                    daedalus_parser::AssocOp::BitAnd => Value::Int(left & right),
                    daedalus_parser::AssocOp::Or => Value::Int((left > 0 || right > 0) as i32),
                    daedalus_parser::AssocOp::BitOr => Value::Int(left | right),
                    daedalus_parser::AssocOp::Multiply => Value::Int(left * right),
                    daedalus_parser::AssocOp::Divide => Value::Int(left / right),
                    daedalus_parser::AssocOp::Modulo => Value::Int(left % right),
                    daedalus_parser::AssocOp::ShiftLeft => Value::Int(left << right),
                    daedalus_parser::AssocOp::ShiftRight => Value::Int(left >> right),
                    daedalus_parser::AssocOp::Assign
                    | daedalus_parser::AssocOp::AddAssign
                    | daedalus_parser::AssocOp::SubtractAssign
                    | daedalus_parser::AssocOp::MultiplyAssign
                    | daedalus_parser::AssocOp::DivideAssign => {
                        return Err(Error::invalid(
                            &expr.span,
                            "assignment in a constant expression",
                        ));
                    }
                }
            }
            ExprKind::Unary(op, operand) => {
                let Value::Int(value) = self.visit_expr(operand)? else {
                    return unsupported("non integer operand in a constant expression");
                };

                Value::Int(match op {
                    UnaryOp::Not => match value {
                        0 => 1,
                        _ => 0,
                    },
                    UnaryOp::Negative => -value,
                    UnaryOp::Complement => !value,
                })
            }
            ExprKind::Lit(lit) => match &lit.kind {
                LitKind::Intager(v) => Value::Int(*v),
                LitKind::Float(v) => Value::Float(*v),
                LitKind::String(v) => Value::String(v.clone()),
            },
            ExprKind::Call(_) => return unsupported("function call in a constant expression"),
            ExprKind::Ident(ident) => {
                if let Some(ref_item) = self.map.map.get(ident.raw.to_ascii_uppercase().as_str()) {
                    self.visit_const(ref_item)?
                } else if let Some(symbol) = self.indices.get(&ident.raw.to_ascii_uppercase()) {
                    Value::Symbol(symbol.id)
                } else {
                    return Err(Error::undefined_symbol(&ident.span, &ident.raw));
                }
            }
            ExprKind::Paren(inner) => self.visit_expr(inner)?,
            ExprKind::Field(_, _) => return unsupported("field access in a constant expression"),
            ExprKind::Index(_, _) => return unsupported("array index in a constant expression"),
        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Diagnostics, files::Files};
    use indoc::indoc;

    #[test]
    fn abc() {
        let src = indoc! {"
        const int ABC = 5;
        const int CBA = ABC + 1;
        const int PAREN = (CBA + 1) * 2;
        const int lower = 2;
        const int UPPER = LOWER * 3;
        "};

        let mut files_store = Files::new();
        let file = files_store.parse("abc.d", src).unwrap();
        let files = [file];

        let indices = SymbolIndices::build(&files);
        let mut diagnostics = Diagnostics::new(&files_store);

        let values = ConstValues::build(&files, &indices, &mut diagnostics);
        assert!(diagnostics.into_errors().is_empty());
        assert!(matches!(values.map["CBA"], Value::Int(6)));
        assert!(matches!(values.map["PAREN"], Value::Int(14)));
        assert!(matches!(values.map["UPPER"], Value::Int(6)));
    }
}
