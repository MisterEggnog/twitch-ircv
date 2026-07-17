use std::io;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WriteLockBuf(Arc<Mutex<Vec<u8>>>);
impl WriteLockBuf {
    pub fn new() -> Self {
        WriteLockBuf(Arc::new(Mutex::new(vec![])))
    }

    pub fn get_data(&self) -> String {
        String::from(std::str::from_utf8(&self.0.lock().unwrap()).unwrap())
    }
}

impl Write for WriteLockBuf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}
