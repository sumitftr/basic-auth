#![allow(dead_code, clippy::missing_transmute_annotations)]

pub struct Scanner<R> {
    reader: R,
    buffer: Vec<u8>,
    iter: std::str::SplitAsciiWhitespace<'static>,
}

impl<R: std::io::BufRead> Scanner<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, buffer: vec![], iter: "".split_ascii_whitespace() }
    }

    pub fn next<T: std::str::FromStr>(&mut self) -> T {
        loop {
            if let Some(token) = self.iter.next() {
                return unsafe { token.parse().unwrap_unchecked() };
            }
            self.buffer.clear();
            self.reader.read_until(b'\n', &mut self.buffer).unwrap();
            self.iter = unsafe {
                let slice = std::str::from_utf8_unchecked(&self.buffer);
                std::mem::transmute(slice.split_ascii_whitespace())
            }
        }
    }

    pub fn next_line<T: std::str::FromStr>(&mut self) -> T {
        if self.iter.nth(0).is_none() {
            self.buffer.clear();
            self.reader.read_until(b'\n', &mut self.buffer).unwrap();
        }
        self.iter = "".split_ascii_whitespace();
        let result = std::mem::take(&mut self.buffer);
        unsafe {
            let s = std::str::from_utf8_unchecked(&result);
            s.trim().parse().unwrap_unchecked()
        }
    }
}
