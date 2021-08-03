use crate::events::Event;
use crate::Document;

use super::Result;
use crate::node::element::tag::Type;
use crate::node::element::GenericElement;
use crate::node::Node;
use std::borrow::Cow;
use std::iter::Peekable;

pub mod error;

pub struct Parser<'l, T: Iterator<Item = Event<'l>>> {
    events: Peekable<T>,
}

macro_rules! raise(
    ($($argument:tt)*) => (
        return Err(crate::node::Error::new(format!($($argument)*)));
    );
);

impl<'l, T: Iterator<Item = Event<'l>>> Parser<'l, T> {
    #[inline]
    pub fn new(events: T) -> Parser<'l, T> {
        Parser {
            events: events.peekable(),
        }
    }

    fn process_prolog(&mut self) -> Result<Vec<Node<'l>>> {
        let mut prolog = Vec::new();
        loop {
            let node = match self.events.peek() {
                None => raise!("no tags in document"),
                Some(Event::Tag(_, Type::Start | Type::Empty, _)) => break,
                Some(Event::Tag(name, Type::End, _)) => {
                    raise!("found </{}> tag before <{}> tag", name, name)
                }
                Some(Event::Text(content)) => {
                    let node = Node::Text(Cow::Borrowed(content));
                    self.events.next();
                    node
                }
                Some(Event::Comment(content)) => {
                    let node = Node::Comment(Cow::Borrowed(content));
                    self.events.next();
                    node
                }
                Some(Event::UnpaddedComment(content)) => {
                    let node = Node::UnpaddedComment(Cow::Borrowed(content));
                    self.events.next();
                    node
                }
                Some(Event::Declaration(content)) => {
                    let node = Node::Declaration(Cow::Borrowed(content));
                    self.events.next();
                    node
                }
                Some(Event::Instruction(content)) => {
                    let node = Node::Instruction(Cow::Borrowed(content));
                    self.events.next();
                    node
                }
            };
            prolog.push(node);
        }
        Ok(prolog)
    }

    fn process_misc_followers(&mut self) -> Result<Vec<Node<'l>>> {
        let mut followers = Vec::new();
        while let Some(event) = self.events.next() {
            let node = match event {
                Event::Tag(_, _, _) => raise!("unexpected second top-level tag"),
                Event::Text(content) => Node::Text(Cow::Borrowed(content)),
                Event::Comment(content) => Node::Comment(Cow::Borrowed(content)),
                Event::UnpaddedComment(content) => Node::UnpaddedComment(Cow::Borrowed(content)),
                Event::Declaration(content) => Node::Declaration(Cow::Borrowed(content)),
                Event::Instruction(content) => Node::Instruction(Cow::Borrowed(content)),
            };
            followers.push(node);
        }
        Ok(followers)
    }

    fn process_node(&mut self) -> Result<Node<'l>> {
        match self.events.peek() {
            None => raise!("expected more nodes"),
            Some(Event::Tag(_, Type::Start | Type::Empty, _)) => {
                self.process_tag().map(Node::Element)
            }
            Some(Event::Tag(name, Type::End, _)) => {
                raise!("found </{}> tag before <{}> tag", name, name)
            }
            Some(Event::Text(content)) => {
                let node = Ok(Node::Text(Cow::Borrowed(content)));
                self.events.next();
                node
            }
            Some(Event::Comment(content)) => {
                let node = Ok(Node::Comment(Cow::Borrowed(content)));
                self.events.next();
                node
            }
            Some(Event::UnpaddedComment(content)) => {
                let node = Ok(Node::UnpaddedComment(Cow::Borrowed(content)));
                self.events.next();
                node
            }
            Some(Event::Declaration(content)) => {
                let node = Ok(Node::Declaration(Cow::Borrowed(content)));
                self.events.next();
                node
            }
            Some(Event::Instruction(content)) => {
                let node = Ok(Node::Instruction(Cow::Borrowed(content)));
                self.events.next();
                node
            }
        }
    }

    fn process_tag(&mut self) -> Result<GenericElement<'l>> {
        match self.events.next() {
            Some(Event::Tag(name, Type::Empty, attributes)) => Ok(GenericElement::new_from(
                Cow::Borrowed(name),
                attributes.clone(),
                Vec::new(),
            )),
            Some(Event::Tag(name, Type::Start, attributes)) => {
                let mut children = Vec::new();
                while !matches!(self.events.peek(), Some(Event::Tag(_, Type::End, _)) | None) {
                    children.push(self.process_node()?);
                }
                match self.events.next() {
                    Some(Event::Tag(closing_name, Type::End, _)) if closing_name == name => Ok(
                        GenericElement::new_from(Cow::Borrowed(name), attributes.clone(), children),
                    ),
                    Some(Event::Tag(closing_name, Type::End, _)) => {
                        raise!("expected </{}>, found </{}>", name, closing_name)
                    }
                    None => raise!("missing </{}>", name),
                    _ => {
                        unreachable!("preceding loop can only exit when next is None or an end tag")
                    }
                }
            }
            Some(event) => raise!("expected a tag, found event {:?}", event),
            None => raise!("expected a tag"),
        }
    }

    pub fn process(&mut self) -> Result<Document<'l>> {
        let prolog = self.process_prolog()?;
        let svg = self.process_tag()?;
        let misc_followers = self.process_misc_followers()?;

        Ok(Document {
            prolog,
            svg,
            misc_followers,
        })
    }
}
