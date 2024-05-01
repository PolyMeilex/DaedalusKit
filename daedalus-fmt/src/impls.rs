use daedalus_parser::{
    Block, BlockItem, Class, Const, ConstKind, Expr, ExprKind, ExternFunctionDefinition, File,
    FunctionCall, FunctionDefinition, Ident, IfStatement, Instance, Item, Lit, LitKind, Prototype,
    ReturnStatement, Ty, UnaryOp, Var, VarKind,
};

use crate::fmt::{DaedalusDisplay, DaedalusFormatter};
use std::fmt::Write;

impl DaedalusDisplay for Block {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        if self.items.is_empty() {
            return write!(f, "{{}}");
        }

        writeln!(f, "{{")?;

        f.push_indent();
        for item in self.items.iter() {
            match item {
                BlockItem::Var(var) => {
                    var.fmt(f)?;
                    writeln!(f, ";")?;
                }
                BlockItem::If(i) => {
                    i.fmt(f)?;
                }
                BlockItem::Return(ret) => {
                    ret.fmt(f)?;
                }
                BlockItem::Expr(expr) => {
                    f.write_indent()?;
                    expr.fmt(f)?;
                    writeln!(f, ";")?;
                }
            }
        }
        f.pop_indent();

        f.write_indent()?;
        write!(f, "}}")?;

        Ok(())
    }
}

impl DaedalusDisplay for Class {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        writeln!(f, "class ")?;
        self.ident.fmt(f)?;
        writeln!(f, " {{")?;

        f.push_indent();
        for var in self.fields.iter() {
            var.fmt(f)?;
            writeln!(f, ";")?;
        }
        f.pop_indent();

        writeln!(f, "}};")?;
        writeln!(f)?;

        Ok(())
    }
}

impl DaedalusDisplay for Const {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "const ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;

        match &self.kind {
            ConstKind::Value { init } => {
                write!(f, " = ")?;
                init.fmt(f)?;
            }
            ConstKind::Array { size_init, init } => {
                write!(f, "[")?;
                size_init.fmt(f)?;
                write!(f, "]")?;

                write!(f, " = {{")?;

                let mut iter = init.iter().peekable();
                while let Some(expr) = iter.next() {
                    expr.fmt(f)?;
                    if iter.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "}}")?;
            }
        }

        Ok(())
    }
}

impl DaedalusDisplay for Expr {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        match &self.kind {
            ExprKind::Binary(op, left, right) => {
                left.fmt(f)?;
                write!(f, " {} ", op.as_str())?;
                right.fmt(f)?;
            }
            ExprKind::Unary(op, v) => {
                match op {
                    UnaryOp::Not => write!(f, "!")?,
                    UnaryOp::Negative => write!(f, "-")?,
                }
                v.fmt(f)?;
            }
            ExprKind::Lit(Lit {
                kind: LitKind::String(lit),
            }) => {
                write!(f, "\"{}\"", lit)?;
            }
            ExprKind::Lit(Lit {
                kind: LitKind::Intager(lit),
            }) => {
                write!(f, "{}", lit)?;
            }
            ExprKind::Lit(Lit {
                kind: LitKind::Float(lit),
            }) => {
                write!(f, "{}", lit)?;
            }
            ExprKind::Call(call) => {
                call.fmt(f)?;
            }
            ExprKind::Ident(i) => {
                i.fmt(f)?;
            }
            ExprKind::Paren(p) => {
                write!(f, "(")?;
                p.fmt(f)?;
                write!(f, ")")?;
            }
            ExprKind::Field(obj, field) => {
                obj.fmt(f)?;
                write!(f, ".")?;
                field.fmt(f)?;
            }
            ExprKind::Index(a, b) => {
                a.fmt(f)?;
                write!(f, "[")?;
                b.fmt(f)?;
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}

impl DaedalusDisplay for ExternFunctionDefinition {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "extern func ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;
        write!(f, "(")?;

        let mut iter = self.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        writeln!(f, ");")?;
        Ok(())
    }
}

impl DaedalusDisplay for FunctionDefinition {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "func ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;
        write!(f, "(")?;

        let mut iter = self.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        write!(f, ") ")?;
        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl DaedalusDisplay for FunctionCall {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        self.ident.fmt(f)?;
        write!(f, "(")?;
        let mut iter = self.args.iter().peekable();
        while let Some(arg) = iter.next() {
            arg.fmt(f)?;
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl DaedalusDisplay for Ident {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)?;
        Ok(())
    }
}

impl DaedalusDisplay for IfStatement {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        if let Some(condition) = self.condition.as_ref() {
            if self.has_else {
                write!(f, " else if ")?;
            } else {
                f.write_indent()?;
                write!(f, "if ")?;
            }

            condition.fmt(f)?;

            write!(f, " ")?;
        } else if self.has_else {
            write!(f, " else ")?;
        }

        self.block.fmt(f)?;

        if let Some(next) = self.next.as_ref() {
            next.fmt(f)?;
        } else if self.has_semi {
            writeln!(f, ";")?;
        }

        Ok(())
    }
}

impl DaedalusDisplay for Instance {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "instance ")?;
        self.ident.fmt(f)?;
        write!(f, "(")?;
        self.parent.fmt(f)?;
        write!(f, ") ")?;

        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl DaedalusDisplay for File {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        for item in &self.items {
            match item {
                Item::Class(v) => {
                    v.fmt(f)?;
                }
                Item::Instance(v) => {
                    v.fmt(f)?;
                }
                Item::Prototype(v) => {
                    v.fmt(f)?;
                }
                Item::Var(v) => {
                    v.fmt(f)?;
                    writeln!(f, ";")?;
                }
                Item::Const(v) => {
                    v.fmt(f)?;
                    writeln!(f, ";")?;
                }
                Item::Func(v) => {
                    v.fmt(f)?;
                }
                Item::ExternFunc(v) => {
                    v.fmt(f)?;
                }
            }
        }

        Ok(())
    }
}

impl DaedalusDisplay for Prototype {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "prototype ")?;
        self.ident.fmt(f)?;
        write!(f, "(")?;
        self.parent.fmt(f)?;
        write!(f, ") ")?;

        self.block.fmt(f)?;
        writeln!(f, ";")?;
        writeln!(f)?;
        Ok(())
    }
}

impl DaedalusDisplay for ReturnStatement {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;
        write!(f, "return")?;
        if let Some(expr) = self.expr.as_ref() {
            write!(f, " ")?;
            expr.fmt(f)?;
        }
        writeln!(f, ";")?;
        Ok(())
    }
}

impl DaedalusDisplay for Ty {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)?;
        Ok(())
    }
}

impl DaedalusDisplay for Var {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result {
        f.write_indent()?;

        write!(f, "var ")?;
        self.ty.fmt(f)?;
        write!(f, " ")?;
        self.ident.fmt(f)?;

        match &self.kind {
            VarKind::Value { init: Some(init) } => {
                write!(f, " = ")?;
                init.fmt(f)?;
            }
            VarKind::Array { size_init, init } => {
                write!(f, "[")?;
                size_init.fmt(f)?;
                write!(f, "]")?;

                if let Some(init) = init {
                    write!(f, " = {{")?;

                    let mut iter = init.iter().peekable();
                    while let Some(expr) = iter.next() {
                        expr.fmt(f)?;
                        if iter.peek().is_some() {
                            write!(f, ", ")?;
                        }
                    }

                    write!(f, "}}")?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
