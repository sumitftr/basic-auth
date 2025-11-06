use std::io::Write;
use std::io::{BufWriter, StdoutLock};

pub struct Printer<'a> {
    pub inner: BufWriter<StdoutLock<'a>>,
}

impl<'a> Printer<'a> {
    pub fn new() -> Self {
        Printer {
            inner: BufWriter::new(std::io::stdout().lock()),
        }
    }

    pub fn write(&mut self, message: &str) {
        write!(self.inner, "{message}");
        self.inner.flush();
    }
}
