//! The nodes.

use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;
use std::iter::once;

use crate::node::element::GenericElement;

mod value;

pub use self::value::Value;
use crate::events::Event;

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

            fn try_from(generic_element: GenericElement<'l>) -> Result<Self, Self::Error> {
                if generic_element.get_name() == crate::node::element::tag::$struct_name {
                    Ok($struct_name {
                        inner: generic_element,
                    })
                } else {
                    Err(generic_element)
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
                element.into()
            }
        }
    );
);

pub mod element;
