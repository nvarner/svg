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

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use super::composer::Composer;
    use super::parser::Parser;

    #[test]
    fn identity_iterator() {
        let mut destination = Vec::new();
        let mut composer = Composer::new(&mut destination);

        // Prevent line ending issues when comparing
        let contents = fs::read_to_string("tests/fixtures/benton_composer_formatted.svg")
            .unwrap()
            .replace("\r\n", "\n");
        Parser::new(&contents)
            .map(|event| event.unwrap())
            .map(|event| composer.write_event(&event))
            .collect::<io::Result<()>>()
            .unwrap();

        let composed = String::from_utf8(destination).unwrap();
        assert_eq!(contents, composed);
    }
}
