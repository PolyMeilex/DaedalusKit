use std::fmt::Write;

pub struct DaedalusFormatter<'a> {
    indent: usize,
    writer: Box<dyn Write + 'a>,
}

impl<'a> DaedalusFormatter<'a> {
    pub fn new(writer: impl Write + 'a) -> Self {
        Self {
            indent: 0,
            writer: Box::new(writer),
        }
    }

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
        write!(self.writer, "{:indent$}", "", indent = self.indent)?;
        Ok(())
    }
}

impl<'a> Write for DaedalusFormatter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        write!(self.writer, "{s}")?;
        Ok(())
    }
}

pub trait DaedalusDisplay {
    fn fmt(&self, f: &mut DaedalusFormatter) -> std::fmt::Result;
}

pub struct IoFmt<T>(pub T);

impl<T> std::fmt::Write for IoFmt<T>
where
    T: std::io::Write,
{
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
    }
}
