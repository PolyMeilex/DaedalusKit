use std::collections::HashMap;
use zstring::ZString;

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
            daedalus_parser::Item::ExternFunc(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident.clone());

                for var in item.args.iter() {
                    let mut arg = ident.clone();
                    arg.0.push(b'.');
                    arg.0.extend(var.ident.raw.as_bytes());
                    self.push_symbol(arg);
                }
            }
            daedalus_parser::Item::Class(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident.clone());

                for var in item.fields.iter() {
                    let mut arg = ident.clone();
                    arg.0.push(b'.');
                    arg.0.extend(var.ident.raw.as_bytes());
                    self.push_symbol(arg);
                }
            }
            daedalus_parser::Item::Instance(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            daedalus_parser::Item::Func(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            daedalus_parser::Item::Const(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            daedalus_parser::Item::Var(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
            daedalus_parser::Item::Prototype(item) => {
                let ident = ZString::from(item.ident.raw.as_bytes().to_ascii_uppercase());
                self.push_symbol(ident);
            }
        }
    }

    pub fn build<'a>(files: impl IntoIterator<Item = &'a daedalus_parser::File>) -> Self {
        let mut symbol_map = Self(HashMap::new());

        symbol_map.push_symbol(ZString::from(b"\xFFINSTANCE_HELP"));
        for ast in files {
            for item in ast.items.iter() {
                symbol_map.handle_item(item);
            }
        }

        symbol_map
    }
}
