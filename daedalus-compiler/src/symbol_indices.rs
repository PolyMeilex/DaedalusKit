use std::collections::HashMap;

use crate::files::File;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    ExternFunction,
    Function,
    FunctionArg,
    Instance,
    Prototype,
    Class,
    ClassVar,
    Var,
    Const,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolIndex {
    pub id: u32,
    pub kind: SymbolKind,
}

#[derive(Debug)]
pub struct SymbolIndices(HashMap<String, SymbolIndex>);

impl std::ops::Deref for SymbolIndices {
    type Target = HashMap<String, SymbolIndex>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SymbolIndices {
    fn push_symbol(&mut self, ident: String, kind: SymbolKind) {
        self.0.insert(
            ident,
            SymbolIndex {
                id: self.0.len() as u32,
                kind,
            },
        );
    }

    fn handle_item(&mut self, item: &daedalus_parser::Item) {
        for (name, kind) in item_symbols(item) {
            self.push_symbol(name, kind);
        }
    }

    pub fn build<'a>(files: impl IntoIterator<Item = &'a File>) -> Self {
        let mut symbol_map = Self(HashMap::new());

        symbol_map.push_symbol("$INSTANCE_HELP".to_string(), SymbolKind::Other);
        for file in files {
            for item in file.ast.items.iter() {
                symbol_map.handle_item(item);
            }
        }

        symbol_map
    }
}

/// Names and kinds of the symbols an item defines, in the order they are stored in the symbol table
pub fn item_symbols(item: &daedalus_parser::Item) -> Vec<(String, SymbolKind)> {
    fn members<'a>(
        parent: &str,
        vars: impl IntoIterator<Item = &'a daedalus_parser::Var>,
        kind: SymbolKind,
    ) -> impl Iterator<Item = (String, SymbolKind)> {
        let parent = parent.to_ascii_uppercase();
        vars.into_iter().map(move |var| {
            let name = format!("{parent}.{}", var.ident.raw.to_ascii_uppercase());
            (name, kind)
        })
    }

    let (ident, kind) = match item {
        daedalus_parser::Item::ExternFunc(item) => (&item.ident, SymbolKind::ExternFunction),
        daedalus_parser::Item::Class(item) => (&item.ident, SymbolKind::Class),
        daedalus_parser::Item::Instance(item) => (&item.ident, SymbolKind::Instance),
        daedalus_parser::Item::Func(item) => (&item.ident, SymbolKind::Function),
        daedalus_parser::Item::Const(item) => (&item.ident, SymbolKind::Const),
        daedalus_parser::Item::Var(item) => (&item.ident, SymbolKind::Var),
        daedalus_parser::Item::Prototype(item) => (&item.ident, SymbolKind::Prototype),
    };

    let mut symbols = vec![(ident.raw.to_ascii_uppercase(), kind)];
    match item {
        daedalus_parser::Item::Func(item) => {
            symbols.extend(members(
                &item.ident.raw,
                &item.args,
                SymbolKind::FunctionArg,
            ));
            let locals = function_locals(&item.block);
            symbols.extend(members(&item.ident.raw, locals, SymbolKind::Var));
        }
        daedalus_parser::Item::ExternFunc(item) => {
            symbols.extend(members(
                &item.ident.raw,
                &item.args,
                SymbolKind::FunctionArg,
            ));
        }
        daedalus_parser::Item::Class(item) => {
            symbols.extend(members(&item.ident.raw, &item.fields, SymbolKind::ClassVar));
        }
        _ => {}
    }

    symbols
}

pub fn function_locals(block: &daedalus_parser::Block) -> Vec<&daedalus_parser::Var> {
    fn visit<'a>(block: &'a daedalus_parser::Block, out: &mut Vec<&'a daedalus_parser::Var>) {
        for item in block.items.iter() {
            match item {
                daedalus_parser::BlockItem::Var(var) => out.push(var),
                daedalus_parser::BlockItem::If(stmt) => {
                    let mut branch = Some(stmt);
                    while let Some(stmt) = branch {
                        visit(&stmt.block, out);
                        branch = stmt.next.as_deref();
                    }
                }
                daedalus_parser::BlockItem::Return(_) | daedalus_parser::BlockItem::Expr(_) => {}
            }
        }
    }

    let mut locals = Vec::new();
    visit(block, &mut locals);
    locals
}
