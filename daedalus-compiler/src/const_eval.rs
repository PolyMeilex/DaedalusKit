#![allow(dead_code)]

use std::collections::HashMap;

use daedalus_parser::{Expr, ExprKind, LitKind};

#[derive(Debug)]
enum Value {
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Default)]
struct ConstsMap<'a> {
    map: HashMap<&'a str, &'a daedalus_parser::Const>,
}

impl<'a> ConstsMap<'a> {
    fn visit_files(&mut self, files: &'a [daedalus_parser::File]) {
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

#[derive(Default)]
pub struct ConstEvaluator {}

impl ConstEvaluator {
    pub fn visit_files(&mut self, files: &[daedalus_parser::File]) {
        let mut map = ConstsMap::default();
        map.visit_files(files);
        dbg!(map.map.len());

        for file in files {
            for item in file.items.iter() {
                if let daedalus_parser::Item::Const(item) = item {
                    let value = self.visit_const(&map, item);
                    println!("{} = {:?}", item.ident.raw, value);
                }
            }
        }
    }

    fn visit_const(&mut self, map: &ConstsMap, item: &daedalus_parser::Const) -> Value {
        match &item.kind {
            daedalus_parser::ConstKind::Value { init } => self.visit_expr(map, init),
            daedalus_parser::ConstKind::Array {
                size_init: _,
                init: _,
            } => todo!(),
        }
    }

    fn visit_expr(&mut self, map: &ConstsMap, expr: &Expr) -> Value {
        match &expr.kind {
            ExprKind::Binary(op, left, right) => {
                let left = self.visit_expr(map, left);
                let right = self.visit_expr(map, right);

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
                let ref_item = map.map.get(ident.raw.as_str()).expect("TODO");
                self.visit_const(map, ref_item)
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

        let mut expr = ConstEvaluator::default();
        expr.visit_files(&[file]);
    }
}
