use std::fmt::Write;

#[derive(Debug, Default)]
pub struct DaedalusFormatter {
    indent: usize,
}

impl DaedalusFormatter {
    pub fn format<T: DaedalusDisplay>(&mut self, v: T) -> std::fmt::Result {
        v.fmt(self)
    }

    pub fn push_indent(&mut self) {
        self.indent += 4;
    }

    pub fn pop_indent(&mut self) {
        self.indent -= 4;
    }

    pub fn write_indent(&mut self) -> std::fmt::Result {
        print!("{:indent$}", "", indent = self.indent);
        Ok(())
    }
}

impl Write for DaedalusFormatter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        print!("{s}");
        Ok(())
    }
}

pub trait DaedalusDisplay {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result;
}
