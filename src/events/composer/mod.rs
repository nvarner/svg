//! The composer.

use std::io;
use std::io::Write;

use crate::events::Event;

#[doc(hidden)]
pub use self::writer::Writer;

mod writer;

pub struct Composer<T: Write> {
    writer: Writer<T>,
}

impl<T: Write> Composer<T> {
    #[inline]
    pub fn new(destination: T) -> Self {
        Composer {
            writer: Writer::new(destination),
        }
    }

    pub fn write_event(&mut self, event: &Event) -> io::Result<()> {
        self.writer.write_event(event)
    }
}
