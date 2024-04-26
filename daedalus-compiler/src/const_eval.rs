#![allow(dead_code)]

use std::collections::HashMap;

use daedalus_parser::{Expr, ExprKind, LitKind};

use crate::symbol_indices::SymbolIndices;

#[derive(Debug)]
enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Array(Vec<Value>),
    Symbol(u32),
}

#[derive(Default)]
struct ConstsMap<'a> {
    map: HashMap<&'a str, &'a daedalus_parser::Const>,
}

impl<'a> ConstsMap<'a> {
    fn visit_files(&mut self, files: impl IntoIterator<Item = &'a daedalus_parser::File>) {
        for file in files {
            self.visit_file(file);
        }
    }

    fn visit_file(&mut self, file: &'a daedalus_parser::File) {
        for item in file.items.iter() {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &'a daedalus_parser::Item) {
        if let daedalus_parser::Item::Const(item) = item {
            self.map.insert(&item.ident.raw, item);
        }
    }
}

pub struct ConstEvaluator<'a> {
    map: ConstsMap<'a>,
    indices: &'a SymbolIndices,
}

impl<'a> ConstEvaluator<'a> {
    pub fn build(
        indices: &'a SymbolIndices,
        files: impl IntoIterator<Item = &'a daedalus_parser::File> + Clone,
    ) {
        let mut map = ConstsMap::default();
        map.visit_files(files.clone());

        let mut this = Self { indices, map };

        for file in files {
            for item in file.items.iter() {
                if let daedalus_parser::Item::Const(item) = item {
                    let value = this.visit_const(item);
                    println!("{} = {:?}", item.ident.raw, value);
                }
            }
        }
    }

    fn visit_const(&mut self, item: &daedalus_parser::Const) -> Value {
        match &item.kind {
            daedalus_parser::ConstKind::Value { init } => self.visit_expr(init),
            daedalus_parser::ConstKind::Array { size_init: _, init } => {
                let values: Vec<_> = init.iter().map(|expr| self.visit_expr(expr)).collect();
                // todo!("{}[{}] = {:?}", item.ident.raw, values.len(), values)
                Value::Array(values)
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Value {
        match &expr.kind {
            ExprKind::Binary(op, left, right) => {
                let left = self.visit_expr(left);
                let right = self.visit_expr(right);

                let (left, right) = match (left, right) {
                    (Value::Int(l), Value::Int(r)) => (l, r),
                    _ => todo!(),
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
                    daedalus_parser::AssocOp::ShiftLeft => Value::Int(left << right),
                    daedalus_parser::AssocOp::ShiftRight => Value::Int(left >> right),
                    daedalus_parser::AssocOp::Assign => todo!(),
                    daedalus_parser::AssocOp::AddAssign => todo!(),
                    daedalus_parser::AssocOp::SubtractAssign => todo!(),
                    daedalus_parser::AssocOp::MultiplyAssign => todo!(),
                    daedalus_parser::AssocOp::DivideAssign => todo!(),
                }
            }
            ExprKind::Unary(_op, _expr) => todo!(),
            ExprKind::Lit(lit) => match &lit.kind {
                LitKind::Intager(v) => {
                    let v: i32 = v.parse().expect("TODO");
                    Value::Int(v)
                }
                LitKind::Float(v) => {
                    let v: f32 = v.parse().expect("TODO");
                    Value::Float(v)
                }
                LitKind::String(v) => Value::String(v.clone()),
            },
            ExprKind::Call(_) => todo!(),
            ExprKind::Ident(ident) => {
                dbg!(&ident);
                if let Some(ref_item) = self.map.map.get(ident.raw.to_uppercase().as_str()) {
                    self.visit_const(ref_item)
                } else if let Some(symbol) =
                    self.indices.get(ident.raw.to_ascii_uppercase().as_bytes())
                {
                    Value::Symbol(*symbol)
                } else {
                    todo!()
                }
            }
            ExprKind::Paren(_) => todo!(),
            ExprKind::Field(_, _) => todo!(),
            ExprKind::Index(_, _) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn abc() {
        let src = indoc! {"
        const int ABC = 5;
        const int CBA = ABC + 1;
        "};

        let file =
            daedalus_parser::File::parse(&mut daedalus_parser::DaedalusLexer::new(src)).unwrap();

        let files = [file];

        let indices = SymbolIndices::build(&files);

        ConstEvaluator::build(&indices, &files);
    }
}
