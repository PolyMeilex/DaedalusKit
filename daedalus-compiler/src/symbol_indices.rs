use dat_file::ZString;
use std::collections::HashMap;

use crate::files::FileId;

pub struct SymbolIndices(HashMap<ZString, u32>);

impl std::ops::Deref for SymbolIndices {
    type Target = HashMap<ZString, u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SymbolIndices {
    fn push_symbol(&mut self, ident: ZString) {
        self.0.insert(ident.clone(), self.0.len() as u32);
    }

    fn handle_item(&mut self, item: &daedalus_parser::Item) {
        match item {
            daedalus_parser::Item::ExternFunc(func) => {
                let ident = ZString::from(func.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident.clone());

                for var in func.args.iter() {
                    let mut arg = ident.clone();
                    arg.0.push(b'.');
                    arg.0.extend(var.ident.raw.as_bytes());
                    self.push_symbol(arg);
                }
            }
            daedalus_parser::Item::Class(class) => {
                let ident = ZString::from(class.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident.clone());

                for var in class.fields.iter() {
                    let mut arg = ident.clone();
                    arg.0.push(b'.');
                    arg.0.extend(var.ident.raw.as_bytes());
                    self.push_symbol(arg);
                }
            }
            daedalus_parser::Item::Instance(instance) => {
                let ident = ZString::from(instance.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            daedalus_parser::Item::Func(func) => {
                let ident = ZString::from(func.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            got => todo!("Got: {got:?}"),
        }
    }

    pub fn build(files: &[(FileId, daedalus_parser::File)]) -> Self {
        let mut symbol_map = Self(HashMap::new());

        symbol_map.push_symbol(ZString::from(b"\xFFINSTANCE_HELP"));
        for (_id, ast) in files.iter() {
            for item in ast.items.iter() {
                symbol_map.handle_item(item);
            }
        }

        symbol_map
    }
}
