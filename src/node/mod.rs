//! The nodes.

use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;
use std::iter::once;

pub use parser::error::Error;

use crate::events;
use crate::events::Event;
use crate::node::element::GenericElement;
use crate::node::parser::Parser;

pub use self::value::Value;

mod parser;
mod value;

/// Attributes.
pub type Attributes = HashMap<String, Value>;

/// Child nodes.
pub type Children<'l> = Vec<Node<'l>>;

/// A complete SVG document.
pub struct Document<'l> {
    /// The prolog. Name for the metadata before `<svg>`, like `<?xml ... ?>` and `<!DOCTYPE ...>`.
    /// See also [the XML spec](https://www.w3.org/TR/REC-xml/#sec-prolog-dtd).
    prolog: Vec<Node<'l>>,
    /// The `<svg>` element.
    svg: GenericElement<'l>,
    /// All elements following `</svg>`.
    misc_followers: Vec<Node<'l>>,
}

impl<'l> Document<'l> {
    #[inline]
    pub fn new() -> Document<'l> {
        Document {
            prolog: Vec::new(),
            svg: GenericElement::new("svg"),
            misc_followers: Vec::new(),
        }
    }

    pub fn from_event_parser(parser: events::parser::Parser<'l>) -> Result<Document<'l>> {
        let events = parser
            .collect::<events::parser::Result<Vec<_>>>()
            .map_err(|err| Error::new(err.to_string()))?;
        Self::from_events(events.into_iter())
    }

    pub fn from_events<T: Iterator<Item = Event<'l>>>(events: T) -> Result<Document<'l>> {
        Parser::new(events).process()
    }

    /// Append a node.
    pub fn add<T>(mut self, node: T) -> Self
    where
        T: Into<Node<'l>>,
    {
        self.svg.append(node.into());
        self
    }

    /// Assign an attribute.
    #[inline]
    pub fn set<T, U>(mut self, name: T, value: U) -> Self
    where
        T: Into<String>,
        U: Into<Value>,
    {
        self.svg.assign(name, value);
        self
    }

    /// Get `<svg>` node.
    #[inline]
    pub fn get_svg(&self) -> &GenericElement {
        &self.svg
    }

    /// Get mutable `<svg>` node.
    #[inline]
    pub fn get_mut_svg(&mut self) -> &mut GenericElement<'l> {
        &mut self.svg
    }

    pub fn to_events(&'l self) -> impl Iterator<Item = Event<'l>> {
        let prolog_events = self.prolog.iter().flat_map(|node| node.to_events());
        let svg_events = self.svg.to_events();
        let misc_follower_events = self.misc_followers.iter().flat_map(|node| node.to_events());

        prolog_events.chain(svg_events).chain(misc_follower_events)
    }
}

impl<'l> From<GenericElement<'l>> for Document<'l> {
    #[inline]
    fn from(element: GenericElement<'l>) -> Self {
        Document {
            prolog: Vec::new(),
            svg: element,
            misc_followers: Vec::new(),
        }
    }
}

impl<'l> AsRef<Document<'l>> for Document<'l> {
    fn as_ref(&self) -> &Document<'l> {
        self
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone, Hash)]
pub enum Node<'l> {
    /// An element.
    Element(GenericElement<'l>),
    /// A text node.
    Text(Cow<'l, str>),
    /// A padded comment (eg. `<!-- foo -->`).
    Comment(Cow<'l, str>),
    /// An unpadded comment (eg. `<!--foo-->`).
    UnpaddedComment(Cow<'l, str>),
    /// A declaration.
    Declaration(Cow<'l, str>),
    /// An instruction.
    Instruction(Cow<'l, str>),
}

impl<'l> Node<'l> {
    pub fn new_element<T: Into<Cow<'l, str>>>(name: T) -> Self {
        Node::Element(GenericElement::new(name))
    }

    /// Creates a text node.
    #[inline]
    pub fn new_text<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node::Text(content.into())
    }

    /// Create a comment node. The content will be padded (eg. `new_comment("foo")` would be
    /// `<!-- foo -->` in XML).
    #[inline]
    pub fn new_comment<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node::Comment(content.into())
    }

    /// Create a comment node. The content will be unpadded (eg. `new_comment("foo")` would be
    /// `<!--foo-->` in XML).
    #[inline]
    pub fn new_unpadded_comment<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node::UnpaddedComment(content.into())
    }

    /// Creates a declaration node.
    #[inline]
    pub fn new_declaration<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node::Declaration(content.into())
    }

    /// Creates an instruction node.
    #[inline]
    pub fn new_instruction<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node::Instruction(content.into())
    }

    pub fn to_events(&'l self) -> Box<dyn Iterator<Item = Event<'l>> + 'l> {
        match self {
            Node::Element(element) => element.to_events(),
            Node::Text(content) => Box::new(once(Event::Text(content))),
            Node::Comment(content) => Box::new(once(Event::Comment(content))),
            Node::UnpaddedComment(content) => Box::new(once(Event::UnpaddedComment(content))),
            Node::Declaration(content) => Box::new(once(Event::Declaration(content))),
            Node::Instruction(content) => Box::new(once(Event::Instruction(content))),
        }
    }
}

impl<'l> From<GenericElement<'l>> for Node<'l> {
    fn from(element: GenericElement<'l>) -> Self {
        Node::Element(element)
    }
}

/// A node.
pub trait Element<'l>:
    'l + fmt::Debug + fmt::Display + NodeClone<'l> + NodeDefaultHash + Send + Sync
{
    /// Append a child node.
    fn append<T>(&mut self, _: T)
    where
        Self: Sized,
        T: Into<Node<'l>>;

    /// Assign an attribute.
    fn assign<T, U>(&mut self, _: T, _: U)
    where
        Self: Sized,
        T: Into<String>,
        U: Into<Value>;
}

#[doc(hidden)]
pub trait NodeClone<'l> {
    fn clone(&self) -> Box<dyn Element<'l>>;
}

#[doc(hidden)]
pub trait NodeDefaultHash {
    fn default_hash(&self, state: &mut DefaultHasher);
}

impl<'l, T> NodeClone<'l> for T
where
    T: Element<'l> + Clone,
{
    #[inline]
    fn clone(&self) -> Box<dyn Element<'l>> {
        Box::new(Clone::clone(self))
    }
}

impl<'l> NodeDefaultHash for Box<dyn Element<'l>> {
    #[inline]
    fn default_hash(&self, state: &mut DefaultHasher) {
        NodeDefaultHash::default_hash(&**self, state)
    }
}

impl<'l> Clone for Box<dyn Element<'l>> {
    #[inline]
    fn clone(&self) -> Self {
        NodeClone::clone(&**self)
    }
}

macro_rules! node(
    ($struct_name:ident::$field_name:ident) => (
        impl<'l> $struct_name<'l> {
            /// Append a node.
            pub fn add<T>(mut self, node: T) -> Self
            where
                T: std::convert::Into<crate::node::Node<'l>>,
            {
                crate::node::Element::append(&mut self, node.into());
                self
            }

            /// Assign an attribute.
            #[inline]
            pub fn set<T, U>(mut self, name: T, value: U) -> Self
            where
                T: Into<String>,
                U: Into<crate::node::Value>,
            {
                crate::node::Element::assign(&mut self, name, value);
                self
            }

            /// Return the inner element.
            #[inline]
            pub fn get_inner(&'l self) -> &'l GenericElement {
                &self.inner
            }
        }

        impl<'l> crate::node::Element<'l> for $struct_name<'l> {
            #[inline]
            fn append<T>(&mut self, node: T)
            where
                T: std::convert::Into<crate::node::Node<'l>>,
            {
                self.$field_name.append(node);
            }

            #[inline]
            fn assign<T, U>(&mut self, name: T, value: U)
            where
                T: Into<String>,
                U: Into<crate::node::Value>,
            {
                self.$field_name.assign(name, value);
            }
        }

        impl<'l> ::std::fmt::Display for $struct_name<'l> {
            #[inline]
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                self.$field_name.fmt(formatter)
            }
        }

        impl<'l> ::std::convert::From<$struct_name<'l>> for GenericElement<'l> {
            #[inline]
            fn from(element: $struct_name<'l>) -> GenericElement<'l> {
                element.inner
            }
        }

        impl<'l> ::std::convert::TryFrom<GenericElement<'l>> for $struct_name<'l> {
            type Error = GenericElement<'l>;

            fn try_from(generic_element: GenericElement<'l>) -> ::std::result::Result<Self, Self::Error> {
                if generic_element.get_name() == crate::node::element::tag::$struct_name {
                    Ok($struct_name {
                        inner: generic_element,
                    })
                } else {
                    Err(generic_element)
                }
            }
        }

        impl<'l> ::std::convert::TryFrom<Node<'l>> for $struct_name<'l> {
            type Error = Node<'l>;

            fn try_from(node: Node<'l>) -> ::std::result::Result<Self, Self::Error> {
                match node {
                    Node::Element(element) => {
                        ::std::convert::TryInto::try_into(element).map_err(|element| Node::Element(element))
                    }
                    _ => Err(node),
                }
            }
        }

        impl<'l> ::std::convert::From<$struct_name<'l>> for Node<'l> {
            #[inline]
            fn from(element: $struct_name<'l>) -> Node<'l> {
                Node::Element(element.into())
            }
        }

        impl<'l> ::std::convert::From<$struct_name<'l>> for crate::Document<'l> {
            #[inline]
            fn from(element: $struct_name<'l>) -> crate::Document<'l> {
                let generic: GenericElement = element.into();
                generic.into()
            }
        }
    );
);

pub mod element;

#[cfg(test)]
mod tests {
    use crate::events::Event;
    use crate::node::element::tag::Type;
    use crate::node::element::{Path, SVG};
    use crate::node::Attributes;
    use crate::{Composer, Document, Parser};

    use std::collections::HashMap;
    use std::convert::TryInto;
    use std::fs;

    #[test]
    fn parse_basic_document() {
        let events = vec![
            Event::Tag("svg", Type::Start, HashMap::new()),
            Event::Tag("svg", Type::End, HashMap::new()),
        ];

        let document = Document::from_events(events.into_iter()).unwrap();

        let svg: SVG = document.svg.try_into().unwrap();
        assert!(svg.get_inner().get_attributes().is_empty());
    }

    #[test]
    fn parse_empty_tag_document() {
        let events = vec![Event::Tag("svg", Type::Empty, HashMap::new())];

        let document = Document::from_events(events.into_iter()).unwrap();

        let svg: SVG = document.svg.try_into().unwrap();
        assert!(svg.get_inner().get_attributes().is_empty());
    }

    #[test]
    fn reject_no_tags() {
        let events = vec![];
        assert!(Document::from_events(events.into_iter()).is_err());
    }

    #[test]
    fn parse_larger_document() {
        // Based on tests/fixtures/benton.svg
        let mut svg_attributes: Attributes = HashMap::new();
        svg_attributes.insert("version".into(), "1.1".into());
        svg_attributes.insert("id".into(), "Layer_1".into());
        svg_attributes.insert("xmlns".into(), "http://www.w3.org/2000/svg".into());
        svg_attributes.insert("xmlns:xlink".into(), "http://www.w3.org/1999/xlink".into());
        svg_attributes.insert("x".into(), "0px".into());
        svg_attributes.insert("y".into(), "0px".into());
        svg_attributes.insert("viewBox".into(), "0 0 800 800".into());
        svg_attributes.insert("enable-background".into(), "new 0 0 800 800".into());
        svg_attributes.insert("xml:space".into(), "preserve".into());

        let mut path1_attributes: Attributes = HashMap::new();
        path1_attributes.insert("d".into(), r#"M249,752c67,0,129-63,129-143c0-107-87-191-199-191c-81,0-149,60-149,125c0,42,29,73,62,73c22,0,44-17,44-40
	c0-25-19-43-41-43c-11,0-26,5-31,11c-17,16-26,3-27-5c-2-45,40-93,123-93c90,0,192,91,192,177c0,81-52,123-99,121c-11-2-21-9-5-24
	c7-4,12-20,12-30c0-22-20-40-45-40s-43,21-43,43C172,724,205,752,249,752z"#.into());

        let mut path2_attributes: Attributes = HashMap::new();
        path2_attributes.insert("d".into(), r#"M544,752c44,0,77-28,77-59c0-22-18-43-43-43s-45,18-45,40c0,10,5,26,12,30c16,15,7,22-5,24c-47,2-99-40-99-121
	c0-86,102-177,192-177c83,0,124,48,123,93c-1,8-11,21-27,5c-5-6-20-11-31-11c-22,0-41,18-41,43c0,23,22,40,44,40c33,0,62-31,62-73
	c0-65-68-125-149-125c-112,0-199,84-199,191C415,689,477,752,544,752z"#.into());

        let mut path3_attributes: Attributes = HashMap::new();
        path3_attributes.insert("d".into(), r#"M249,50c-44,0-77,28-77,59c0,22,18,43,43,43s45-18,45-40c0-10-5-26-12-30c-16-15-6-22,5-24c47-2,99,40,99,121
	c0,86-102,177-192,177c-83,0-125-48-123-93c1-8,10-21,27-5c5,6,20,11,31,11c22,0,41-18,41-43c0-23-22-40-44-40c-33,0-62,31-62,73
	c0,65,68,125,149,125c112,0,199-84,199-191C378,113,316,50,249,50z"#.into());

        let mut path4_attributes: Attributes = HashMap::new();
        path4_attributes.insert("d".into(), r#"M544,50c-67,0-129,63-129,143c0,107,87,191,199,191c81,0,149-60,149-125c0-42-29-73-62-73c-22,0-44,17-44,40
	c0,25,19,43,41,43c11,0,26-5,31-11c16-16,26-3,27,5c1,45-40,93-123,93c-90,0-192-91-192-177c0-81,52-123,99-121c11,2,21,9,5,24
	c-7,4-12,20-12,30c0,22,20,40,45,40s43-21,43-43C621,78,588,50,544,50z"#.into());

        let events = vec![
            Event::Instruction(r#"xml version="1.0" encoding="utf-8""#),
            Event::Comment("Generator: Adobe Illustrator 18.0.0, SVG Export Plug-In . SVG Version: 6.00 Build 0) "),
            Event::Declaration(r#"DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd""#),
            Event::Tag("svg", Type::Start, svg_attributes.clone()),
            Event::Tag("path", Type::Empty, path1_attributes.clone()),
            Event::Tag("path", Type::Empty, path2_attributes.clone()),
            Event::Tag("path", Type::Empty, path3_attributes.clone()),
            Event::Tag("path", Type::Empty, path4_attributes.clone()),
            Event::Tag("svg", Type::End, HashMap::new()),
        ];

        let document = Document::from_events(events.into_iter()).unwrap();

        let svg: SVG = document.svg.try_into().unwrap();
        let svg_inner = svg.get_inner();
        assert_eq!(&svg_attributes, svg_inner.get_attributes());

        let svg_children = svg_inner.get_children();
        assert_eq!(4, svg_children.len());

        let path1: Path = svg_children[0].clone().try_into().unwrap();
        assert_eq!(&path1_attributes, path1.get_inner().get_attributes());

        let path2: Path = svg_children[1].clone().try_into().unwrap();
        assert_eq!(&path2_attributes, path2.get_inner().get_attributes());

        let path3: Path = svg_children[2].clone().try_into().unwrap();
        assert_eq!(&path3_attributes, path3.get_inner().get_attributes());

        let path4: Path = svg_children[3].clone().try_into().unwrap();
        assert_eq!(&path4_attributes, path4.get_inner().get_attributes());
    }

    #[test]
    fn identity_iterator() {
        let mut destination = Vec::new();
        let mut composer = Composer::new(&mut destination);

        // Prevent line ending issues when comparing
        let contents = fs::read_to_string("tests/fixtures/benton_composer_formatted.svg")
            .unwrap()
            .replace("\r\n", "\n");
        let events = Parser::new(&contents).map(|event| event.unwrap());
        let document = Document::from_events(events).unwrap();
        document
            .to_events()
            .try_for_each(|event| composer.write_event(&event))
            .unwrap();

        let composed = String::from_utf8(destination).unwrap();
        assert_eq!(contents, composed);
    }
}
