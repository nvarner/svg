//! An SVG composer and parser.
//!
//! ## Example: Composing
//!
//! ```
//! # extern crate svg;
//! use svg::Document;
//! use svg::node::element::Path;
//! use svg::node::element::path::Data;
//!
//! # fn main() {
//! let data = Data::new()
//!     .move_to((10, 10))
//!     .line_by((0, 50))
//!     .line_by((50, 0))
//!     .line_by((0, -50))
//!     .close();
//!
//! let path = Path::new()
//!     .set("fill", "none")
//!     .set("stroke", "black")
//!     .set("stroke-width", 3)
//!     .set("d", data);
//!
//! let document = Document::new()
//!     .set("viewBox", (0, 0, 70, 70))
//!     .add(path);
//!
//! svg::save("image.svg", &document).unwrap();
//! # ::std::fs::remove_file("image.svg");
//! # }
//! ```
//!
//! ## Example: Parsing
//!
//! ```
//! # extern crate svg;
//! use svg::node::element::path::{Command, Data};
//! use svg::node::element::tag::Path;
//! use svg::events::Event;
//!
//! # fn main() {
//! let path = "image.svg";
//! # let path = "tests/fixtures/benton.svg";
//! let mut content = String::new();
//! for event in svg::open(path, &mut content).unwrap() {
//!     match event {
//!         Ok(Event::Tag(Path, _, attributes)) => {
//!             let data = attributes.get("d").unwrap();
//!             let data = Data::parse(data).unwrap();
//!             for command in data.iter() {
//!                 match command {
//!                     &Command::Move(..) => println!("Move!"),
//!                     &Command::Line(..) => println!("Line!"),
//!                     _ => {}
//!                 }
//!             }
//!         }
//!         _ => {}
//!     }
//! }
//! # }
//! ```

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub mod events;
pub mod node;

pub use crate::events::composer::Composer;
pub use crate::events::parser::Parser;
pub use crate::node::Element;

pub use node::Document;

/// Open a document.
pub fn open<'l, T>(path: T, mut content: &'l mut String) -> io::Result<Parser<'l>>
where
    T: AsRef<Path>,
{
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;
    read(content)
}

/// Read a document.
pub fn read<'l>(content: &'l str) -> io::Result<Parser<'l>> {
    Ok(Parser::new(content))
}

/// Save a document.
pub fn save<'l, T, U>(path: T, document: U) -> io::Result<()>
where
    T: AsRef<Path>,
    U: Into<&'l Document<'l>>,
{
    let file = File::create(path)?;
    write(file, document)
}

/// Write a document.
pub fn write<'l, T, U>(mut target: T, document: U) -> io::Result<()>
where
    T: Write,
    U: Into<&'l Document<'l>>,
{
    let document: &Document = document.into();
    let mut composer = Composer::new(&mut target);
    document
        .to_events()
        .try_for_each(|event| composer.write_event(&event))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use crate::events::parser::Parser;
    use crate::events::Event;

    const TEST_PATH: &'static str = "tests/fixtures/benton.svg";

    #[test]
    fn open() {
        let mut content = String::new();
        exercise(crate::open(self::TEST_PATH, &mut content).unwrap());
    }

    #[test]
    fn read() {
        let mut content = String::new();
        let mut file = File::open(self::TEST_PATH).unwrap();
        file.read_to_string(&mut content).unwrap();

        exercise(crate::read(&content).unwrap());
    }

    fn exercise<'l>(mut parser: Parser<'l>) {
        macro_rules! test(
            ($matcher:pat) => (match parser.next().unwrap() {
                Ok($matcher) => {}
                _ => unreachable!(),
            });
        );

        test!(Event::Instruction(r#"xml version="1.0" encoding="utf-8""#));
        test!(Event::Comment(
            "Generator: Adobe Illustrator 18.0.0, SVG Export Plug-In . SVG Version: 6.00 Build 0) "
        ));
        test!(Event::Declaration(
            r#"DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd""#
        ));
        test!(Event::Tag("svg", _, _));
        test!(Event::Tag("path", _, _));
        test!(Event::Tag("path", _, _));
        test!(Event::Tag("path", _, _));
        test!(Event::Tag("path", _, _));
        test!(Event::Tag("svg", _, _));

        assert!(parser.next().is_none());
    }
}
