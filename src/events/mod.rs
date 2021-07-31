use crate::node::element::tag::Type;
use crate::node::Attributes;
use std::borrow::Cow;

pub mod composer;
pub mod parser;

/// An event.
#[derive(Debug)]
pub enum Event<'l> {
    /// A tag.
    Tag(Cow<'l, str>, Type, Attributes),
    /// A text.
    Text(Cow<'l, str>),
    /// A padded comment (eg. `<!-- foo -->`).
    Comment(Cow<'l, str>),
    /// An unpadded comment (eg. `<!--foo-->`).
    UnpaddedCommend(Cow<'l, str>),
    /// A declaration.
    Declaration(Cow<'l, str>),
    /// An instruction.
    Instruction(Cow<'l, str>),
}

impl<'l> Event<'l> {
    pub fn new_tag<T>(name: T, children: Type, attributes: Attributes) -> Event<'l>
    where
        T: Into<Cow<'l, str>>,
    {
        Event::Tag(name.into(), children, attributes)
    }

    pub fn new_text<T>(content: T) -> Event<'l>
    where
        T: Into<Cow<'l, str>>,
    {
        Event::Text(content.into())
    }

    pub fn new_comment<T>(content: T) -> Event<'l>
    where
        T: Into<Cow<'l, str>>,
    {
        Event::Comment(content.into())
    }

    pub fn new_comment_unpadded<T>(content: T) -> Event<'l>
    where
        T: Into<Cow<'l, str>>,
    {
        Event::UnpaddedCommend(content.into())
    }

    pub fn new_declaration<T>(content: T) -> Event<'l>
    where
        T: Into<Cow<'l, str>>,
    {
        Event::Declaration(content.into())
    }

    pub fn new_instruction<T>(content: T) -> Event<'l>
    where
        T: Into<Cow<'l, str>>,
    {
        Event::Instruction(content.into())
    }
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
