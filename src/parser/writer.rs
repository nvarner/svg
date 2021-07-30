use std::io::Write;

use crate::node::element::tag::Type;
use crate::node::Attributes;
use crate::parser::Event;
use std::io;

pub struct Writer<'l, T>
where
    T: Write,
{
    destination: T,
}

impl<'l, T> Writer<'l, T>
where
    T: Write,
{
    #[inline]
    pub fn new(destination: T) -> Self {
        Self { destination }
    }

    pub fn write_event(&mut self, event: &Event) {}
}
