//! The parser.

use crate::events::Event;
use crate::node::element::tag::Tag;

pub use self::error::Error;

#[doc(hidden)]
pub use self::reader::Reader;

mod error;
mod reader;

/// A parser.
pub struct Parser<'l> {
    reader: Reader<'l>,
}

/// A result.
pub type Result<T> = ::std::result::Result<T, Error>;

macro_rules! raise(
    ($parser:expr, $($argument:tt)*) => (
        return Some(Err(Error::new($parser.reader.position(), format!($($argument)*))));
    );
);

impl<'l> Parser<'l> {
    /// Create a parser.
    #[inline]
    pub fn new(content: &'l str) -> Self {
        Parser {
            reader: Reader::new(content),
        }
    }

    fn next_angle(&mut self) -> Option<Result<Event<'l>>> {
        let content: String = self.reader.peek_many().take(4).collect();
        if content.is_empty() {
            None
        } else if content.starts_with("<!--") {
            self.read_comment()
        } else if content.starts_with("<!") {
            self.read_declaration()
        } else if content.starts_with("<?") {
            self.read_instruction()
        } else if content.starts_with('<') {
            self.read_tag()
        } else {
            raise!(self, "found an unknown sequence");
        }
    }

    fn next_text(&mut self) -> Option<Result<Event<'l>>> {
        self.reader
            .capture(|reader| reader.consume_until_char('<'))
            .map(|content| Ok(Event::new_text(content)))
    }

    fn parse_comment_body(body: &'l str) -> Event {
        let stripped_content = body
            .strip_prefix(" ")
            .and_then(|content| content.strip_suffix(" "));
        if let Some(content) = stripped_content {
            Event::new_comment(content)
        } else {
            Event::new_comment_unpadded(body)
        }
    }

    fn read_comment(&mut self) -> Option<Result<Event<'l>>> {
        match self.reader.capture(|reader| reader.consume_comment()) {
            None => raise!(self, "found a malformed comment"),
            Some(content) => Some(Ok(Self::parse_comment_body(&content[4..content.len() - 3]))),
        }
    }

    fn read_declaration(&mut self) -> Option<Result<Event<'l>>> {
        match self.reader.capture(|reader| reader.consume_declaration()) {
            None => raise!(self, "found a malformed declaration"),
            Some(content) => Some(Ok(Event::new_declaration(&content[2..content.len() - 1]))),
        }
    }

    fn read_instruction(&mut self) -> Option<Result<Event<'l>>> {
        match self.reader.capture(|reader| reader.consume_instruction()) {
            None => raise!(self, "found a malformed instruction"),
            Some(content) => Some(Ok(Event::new_instruction(&content[2..content.len() - 2]))),
        }
    }

    fn read_tag(&mut self) -> Option<Result<Event<'l>>> {
        match self.reader.capture(|reader| reader.consume_tag()) {
            None => raise!(self, "found a malformed tag"),
            Some(content) => Some(
                Tag::parse(&content[1..content.len() - 1])
                    .map(|Tag(name, kind, attributes)| Event::new_tag(name, kind, attributes)),
            ),
        }
    }
}

impl<'l> Iterator for Parser<'l> {
    type Item = Result<Event<'l>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_text().or_else(|| self.next_angle())
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::events::Event;

    #[test]
    fn next_tag() {
        macro_rules! test(
            ($content:expr, $value:expr) => ({
                let mut parser = Parser::new($content);
                match parser.next().unwrap().unwrap() {
                    Event::Tag(value, _, _) => assert_eq!(value, $value),
                    _ => unreachable!(),
                }
            })
        );

        test!("<foo>", "foo");
        test!("<foo/>", "foo");
        test!("  <foo/>", "foo");
    }

    #[test]
    fn next_text() {
        macro_rules! test(
            ($content:expr, $value:expr) => ({
                let mut parser = Parser::new($content);
                match parser.next().unwrap().unwrap() {
                    Event::Text(value) => assert_eq!(value, $value),
                    _ => unreachable!(),
                }
            })
        );

        test!("foo <bar>", "foo");
        test!("  foo<bar>", "foo");
        test!("foo> <bar>", "foo>");
    }
}
