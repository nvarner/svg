use crate::node::element::tag::Type;
use crate::node::Attributes;

pub mod composer;
pub mod parser;

/// An event.
#[derive(Debug)]
pub enum Event<'l> {
    /// A tag.
    Tag(&'l str, Type, Attributes),
    /// A text.
    Text(&'l str),
    /// A comment. `(content, is content padded with spaces inside comment)`
    Comment(&'l str, bool),
    /// A declaration.
    Declaration(&'l str),
    /// An instruction.
    Instruction(&'l str),
}

impl<'l> Event<'l> {}
