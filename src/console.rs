use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
pub struct Screen {
    events: Arc<RwLock<VecDeque<String>>>,
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
        }
    }
}

impl Screen {
    pub fn read(&self) -> Vec<String> {
        match self.events.read() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl std::io::Write for Screen {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let text = std::str::from_utf8(buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf-8"))?;
        let mut guard = self.events.write().unwrap_or_else(|e| {
            let mut guard = e.into_inner();
            guard.clear();
            guard
        });

        // Prevent the ring buffer from growing.
        if guard.len() == guard.capacity() {
            guard.pop_front();
        }

        guard.push_back(String::from(text));

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Screen {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
