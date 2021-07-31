//! The nodes.

use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;

use crate::node::element::Element;

mod value;

pub use self::value::Value;

/// Attributes.
pub type Attributes = HashMap<String, Value>;

/// Child nodes.
pub type Children<'l> = Vec<Node_<'l>>;

/// Complete SVG document.
pub struct Document<'l> {
    /// The prolog. Name for the metadata before `<svg>`, like `<?xml ... ?>` and `<!DOCTYPE ...>`.
    /// See also [the XML spec](https://www.w3.org/TR/REC-xml/#sec-prolog-dtd).
    prolog: Vec<Node_<'l>>,
    /// The `<svg>` element.
    svg: Node_<'l>,
    /// All elements following `</svg>`.
    misc_followers: Vec<Node_<'l>>,
}

#[derive(Debug, Clone, Hash)]
pub enum Node_<'l> {
    /// An element.
    Element(Element<'l>),
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

impl<'l> Node_<'l> {
    pub fn new_element<T: Into<Cow<'l, str>>>(name: T) -> Self {
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
        Node_::Comment(content.into())
    }

    /// Create a comment node. The content will be unpadded (eg. `new_comment("foo")` would be
    /// `<!--foo-->` in XML).
    #[inline]
    pub fn new_unpadded_comment<T: Into<Cow<'l, str>>>(content: T) -> Self {
        Node_::UnpaddedComment(content.into())
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
pub trait Node<'l>:
    'l + fmt::Debug + fmt::Display + NodeClone<'l> + NodeDefaultHash + Send + Sync
{
    /// Append a child node.
    fn append<T>(&mut self, _: T)
    where
        Self: Sized,
        T: Node<'l>;

    /// Assign an attribute.
    fn assign<T, U>(&mut self, _: T, _: U)
    where
        Self: Sized,
        T: Into<String>,
        U: Into<Value>;
}

#[doc(hidden)]
pub trait NodeClone<'l> {
    fn clone(&self) -> Box<dyn Node<'l>>;
}

#[doc(hidden)]
pub trait NodeDefaultHash {
    fn default_hash(&self, state: &mut DefaultHasher);
}

impl<'l, T> NodeClone<'l> for T
where
    T: Node<'l> + Clone,
{
    #[inline]
    fn clone(&self) -> Box<dyn Node<'l>> {
        Box::new(Clone::clone(self))
    }
}

impl<'l> NodeDefaultHash for Box<dyn Node<'l>> {
    #[inline]
    fn default_hash(&self, state: &mut DefaultHasher) {
        NodeDefaultHash::default_hash(&**self, state)
    }
}

impl<'l> Clone for Box<dyn Node<'l>> {
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
                T: crate::node::Node<'l>,
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
            pub fn get_inner(&'l self) -> &'l Element {
                &self.inner
            }
        }

        impl<'l> crate::node::Node<'l> for $struct_name<'l> {
            #[inline]
            fn append<T>(&mut self, node: T)
            where
                T: crate::node::Node<'l>,
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

        impl<'l> Into<Element<'l>> for $struct_name<'l> {
            #[inline]
            fn into(self) -> Element<'l> {
                self.inner
            }
        }
    );
);

pub mod element;
