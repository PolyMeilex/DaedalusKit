use std::collections::HashMap;

use crate::files::File;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    ExternFunction,
    Function,
    Instance,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct SymbolIndex {
    pub id: u32,
    pub kind: SymbolKind,
}

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
        match item {
            daedalus_parser::Item::ExternFunc(item) => {
                let ident = item.ident.raw.to_uppercase();
                self.push_symbol(ident.clone(), SymbolKind::ExternFunction);

                for var in item.args.iter() {
                    self.push_symbol(
                        format!("{}.{}", ident, var.ident.raw.to_uppercase()),
                        SymbolKind::Other,
                    );
                }
            }
            daedalus_parser::Item::Class(item) => {
                let ident = item.ident.raw.to_uppercase();
                self.push_symbol(ident.clone(), SymbolKind::Other);

                for var in item.fields.iter() {
                    self.push_symbol(
                        format!("{}.{}", ident, var.ident.raw.to_uppercase()),
                        SymbolKind::Other,
                    );
                }
            }
            daedalus_parser::Item::Instance(item) => {
                self.push_symbol(item.ident.raw.to_uppercase(), SymbolKind::Instance);
            }
            daedalus_parser::Item::Func(item) => {
                self.push_symbol(item.ident.raw.to_uppercase(), SymbolKind::Function);
            }
            daedalus_parser::Item::Const(item) => {
                self.push_symbol(item.ident.raw.to_uppercase(), SymbolKind::Other);
            }
            daedalus_parser::Item::Var(item) => {
                self.push_symbol(item.ident.raw.to_uppercase(), SymbolKind::Other);
            }
            daedalus_parser::Item::Prototype(item) => {
                self.push_symbol(item.ident.raw.to_uppercase(), SymbolKind::Other);
            }
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
