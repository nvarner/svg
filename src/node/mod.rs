//! The nodes.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;

mod value;

pub use self::value::Value;
use crate::events::Event;
use crate::node::element::tag::Type;
use crate::node::element::Element;
use std::borrow::Cow;
use std::fmt::Debug;

/// Attributes.
pub type Attributes = HashMap<String, Value>;

/// Child nodes.
pub type Children = Vec<Box<dyn Node>>;

/// Complete SVG document.
pub struct Document {
    /// The prolog. Name for the metadata before `<svg>`, like `<?xml ... ?>` and `<!DOCTYPE ...>`.
    /// See also [the XML spec](https://www.w3.org/TR/REC-xml/#sec-prolog-dtd).
    prolog: Vec<Box<dyn Node>>,
    /// The `<svg>` element.
    svg: Box<dyn Node>,
    /// All elements following `</svg>`.
    misc_followers: Vec<Box<dyn Node>>,
}

pub enum Node_<'l> {
    /// An element.
    Element(Element),
    /// A text node.
    Text(Cow<'l, str>),
    /// A comment. `(content, is content padded with spaces inside comment)`
    Comment(Cow<'l, str>, bool),
    /// A declaration.
    Declaration(Cow<'l, str>),
    /// An instruction.
    Instruction(Cow<'l, str>),
}

impl<'l> Node_<'l> {
    pub fn new_element<T: Into<String>>(name: T) -> Self {
        Node_::Element(Element::new(name))
    }

    /// Creates a text node.
    #[inline]
    pub fn new_text<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node_::Text(content.into())
    }

    /// Create a comment node. The content will be padded (eg. `new_comment("foo")` would be
    /// `<!-- foo -->` in XML).
    #[inline]
    pub fn new_comment<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node_::Comment(content.into(), true)
    }

    /// Create a comment node. The content will be unpadded (eg. `new_comment("foo")` would be
    /// `<!--foo-->` in XML).
    #[inline]
    pub fn new_comment_unpadded<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node_::Comment(content.into(), false)
    }

    /// Creates a declaration node.
    #[inline]
    pub fn new_declaration<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node_::Declaration(content.into())
    }

    /// Creates an instruction node.
    #[inline]
    pub fn new_instruction<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node_::Instruction(content.into())
    }
}

/// A node.
pub trait Node:
    'static + fmt::Debug + fmt::Display + NodeClone + NodeDefaultHash + Send + Sync
{
    /// Append a child node.
    fn append<T>(&mut self, _: T)
    where
        Self: Sized,
        T: Node;

    /// Assign an attribute.
    fn assign<T, U>(&mut self, _: T, _: U)
    where
        Self: Sized,
        T: Into<String>,
        U: Into<Value>;
}

#[doc(hidden)]
pub trait NodeClone {
    fn clone(&self) -> Box<dyn Node>;
}

#[doc(hidden)]
pub trait NodeDefaultHash {
    fn default_hash(&self, state: &mut DefaultHasher);
}

impl<T> NodeClone for T
where
    T: Node + Clone,
{
    #[inline]
    fn clone(&self) -> Box<dyn Node> {
        Box::new(Clone::clone(self))
    }
}

impl NodeDefaultHash for Box<dyn Node> {
    #[inline]
    fn default_hash(&self, state: &mut DefaultHasher) {
        NodeDefaultHash::default_hash(&**self, state)
    }
}

impl Clone for Box<dyn Node> {
    #[inline]
    fn clone(&self) -> Self {
        NodeClone::clone(&**self)
    }
}

macro_rules! node(
    ($struct_name:ident::$field_name:ident) => (
        impl $struct_name {
            /// Append a node.
            pub fn add<T>(mut self, node: T) -> Self
            where
                T: crate::node::Node,
            {
                crate::node::Node::append(&mut self, node);
                self
            }

            /// Assign an attribute.
            #[inline]
            pub fn set<T, U>(mut self, name: T, value: U) -> Self
            where
                T: Into<String>,
                U: Into<crate::node::Value>,
            {
                crate::node::Node::assign(&mut self, name, value);
                self
            }

            /// Return the inner element.
            #[inline]
            pub fn get_inner(&self) -> &Element {
                &self.inner
            }
        }

        impl crate::node::Node for $struct_name {
            #[inline]
            fn append<T>(&mut self, node: T)
            where
                T: crate::node::Node,
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

        impl ::std::fmt::Display for $struct_name {
            #[inline]
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                self.$field_name.fmt(formatter)
            }
        }

        impl Into<Element> for $struct_name {
            #[inline]
            fn into(self) -> Element {
                self.inner
            }
        }
    );
);

pub mod element;
