use std::io::Write;

use bstr::BString;
use indexmap::IndexMap;

#[derive(Debug, Default)]
pub struct SvmClass<'a> {
    fields: IndexMap<&'a str, Option<(BString, BString)>>,
}

impl<'a> SvmClass<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, field: &'a str) {
        self.fields.insert(field, None);
    }

    pub fn new_instance(&self) -> SvmInstance<'a> {
        SvmInstance {
            fields: self.fields.clone(),
        }
    }
}

#[derive(Debug)]
pub struct SvmInstance<'a> {
    fields: IndexMap<&'a str, Option<(BString, BString)>>,
}

impl<'a> SvmInstance<'a> {
    pub fn insert(&mut self, field: &'a str, key: impl Into<BString>, text: impl Into<BString>) {
        self.fields
            .entry(field)
            .or_default()
            .replace((key.into(), text.into()));
    }

    pub fn iter(self) -> impl Iterator<Item = (&'a str, BString, BString)> {
        self.fields.into_iter().filter_map(|(field, v)| {
            let (key, text) = v?;
            Some((field, key, text))
        })
    }
}

#[derive(Debug)]
pub struct Block {
    text: BString,
}

#[derive(Default, Debug)]
pub struct OutputUnits {
    // SVM have to be before everything else
    svm: IndexMap<BString, Block>,
    map: IndexMap<BString, Block>,
}

impl OutputUnits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_svm(&mut self, svm: SvmInstance<'_>) {
        for (_, key, text) in svm.iter() {
            self.svm.insert(key, Block { text });
        }
    }

    pub fn push(&mut self, key: impl Into<BString>, text: impl Into<BString>) {
        self.map.insert(key.into(), Block { text: text.into() });
    }

    pub fn encode(&self, mut w: impl Write) -> std::io::Result<()> {
        let count = self.map.len() + self.svm.len();
        write_header(&mut w, count * 3 + 1)?;

        let mut id = 0;
        writeln!(w, "[% zCCSLib 0 {id}]")?;
        writeln!(w, "\tNumOfItems=int:{count}")?;

        for (key, v) in self.svm.iter().chain(self.map.iter()) {
            id += 1;
            writeln!(w, "\t[% zCCSBlock 0 {id}]")?;

            write!(w, "\t\tblockName=string:")?;
            w.write_all(key.as_slice())?;
            writeln!(w)?;

            writeln!(w, "\t\tnumOfBlocks=int:1")?;
            writeln!(w, "\t\tsubBlock0=float:0")?;
            id += 1;
            writeln!(w, "\t\t[% zCCSAtomicBlock 0 {id}]")?;

            {
                id += 1;
                writeln!(
                    w,
                    "\t\t\t[% oCMsgConversation:oCNpcMessage:zCEventMessage 0 {id}]",
                )?;
                writeln!(w, "\t\t\t\tsubType=enum:0")?;

                write!(w, "\t\t\t\ttext=string:")?;
                w.write_all(v.text.as_slice())?;
                writeln!(w)?;

                write!(w, "\t\t\t\tname=string:")?;
                w.write_all(key.as_slice())?;
                writeln!(w, ".WAV")?;
                writeln!(w, "\t\t\t[]")?;
            }

            writeln!(w, "\t\t[]")?;
            writeln!(w, "\t[]")?;
        }

        writeln!(w, "[]")?;

        Ok(())
    }
}

fn write_header(mut w: impl Write, count: usize) -> std::io::Result<()> {
    let date = "22.3.2024 5:18:34";
    let user = "poly";

    writeln!(w, "ZenGin Archive")?;
    writeln!(w, "ver 1")?;
    writeln!(w, "zCArchiverGeneric")?;
    writeln!(w, "ASCII")?;
    writeln!(w, "saveGame 0")?;
    writeln!(w, "date {date}")?;
    writeln!(w, "user {user}")?;
    writeln!(w, "END")?;
    writeln!(w, "objects {count:<9}")?; // Fill to 9, as it appears that zengin does something similar
    writeln!(w, "END")?;
    writeln!(w)?;

    Ok(())
}
