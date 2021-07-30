use std::io;
use std::io::Write;

use crate::node::element::tag::Type;
use crate::node::{Attributes, Value};
use crate::parser::Event;

pub struct Writer<T>
where
    T: Write,
{
    destination: T,
    initial_event_written: bool,
}

impl<T> Writer<T>
where
    T: Write,
{
    #[inline]
    pub fn new(destination: T) -> Self {
        Self {
            destination,
            initial_event_written: false,
        }
    }

    fn write_attribute(&mut self, name: &str, value: &Value) -> io::Result<()> {
        match (value.contains('\''), value.contains('"')) {
            (true, false) | (false, false) => {
                write!(self.destination, r#" {}="{}""#, name, value)?;
            }
            (false, true) => {
                write!(self.destination, r#" {}='{}'"#, name, value)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn write_attributes(&mut self, attributes: &Attributes) -> io::Result<()> {
        let mut attributes = attributes.iter().collect::<Vec<_>>();
        attributes.sort_by_key(|pair| pair.0.as_str());
        for (name, value) in attributes {
            self.write_attribute(name, value)?;
        }
        Ok(())
    }

    fn initial_newline(&mut self) -> io::Result<()> {
        if self.initial_event_written {
            write!(self.destination, "\n")?;
        } else {
            self.initial_event_written = true;
        }
        Ok(())
    }

    fn write_start_tag(&mut self, name: &str, attributes: &Attributes) -> io::Result<()> {
        self.initial_newline()?;
        write!(self.destination, "<{}", name)?;
        self.write_attributes(attributes)?;
        write!(self.destination, ">")
    }

    fn write_empty_tag(&mut self, name: &str, attributes: &Attributes) -> io::Result<()> {
        self.initial_newline()?;
        write!(self.destination, "<{}", name)?;
        self.write_attributes(attributes)?;
        write!(self.destination, "/>")
    }

    fn write_end_tag(&mut self, name: &str) -> io::Result<()> {
        self.initial_newline()?;
        write!(self.destination, "</{}>", name)
    }

    fn write_text(&mut self, text: &str) -> io::Result<()> {
        self.initial_newline()?;
        write!(self.destination, "{}", text)
    }

    pub fn write_event(&mut self, event: &Event) -> io::Result<()> {
        match event {
            Event::Tag(name, Type::Start, attributes) => self.write_start_tag(name, attributes),
            Event::Tag(name, Type::Empty, attributes) => self.write_empty_tag(name, attributes),
            Event::Tag(name, Type::End, _) => self.write_end_tag(name),
            Event::Text(text) => self.write_text(text),
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::node::element::tag::Type;
    use crate::node::{Node, Value};
    use crate::parser::writer::Writer;
    use crate::parser::Event;

    #[test]
    fn event_display() {
        let mut foo_attributes = HashMap::new();
        foo_attributes.insert("x".into(), Value::from(-10));
        foo_attributes.insert("y".into(), Value::from("10px"));
        foo_attributes.insert("s".into(), Value::from((12.5, 13.0)));
        foo_attributes.insert("c".into(), Value::from("green"));
        let foo = Event::Tag("foo", Type::Start, foo_attributes);

        let bar = Event::Tag("bar", Type::Empty, HashMap::new());

        let foo_end = Event::Tag("foo", Type::End, HashMap::new());

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        for event in &[foo, bar, foo_end] {
            writer.write_event(event).unwrap();
        }

        assert_eq!(
            String::from_utf8_lossy(&output),
            "<foo c=\"green\" s=\"12.5 13\" x=\"-10\" y=\"10px\">\n\
             <bar/>\n\
             </foo>\
             "
        );
    }

    #[test]
    fn event_display_quotes() {
        let mut foo_attributes = HashMap::new();
        foo_attributes.insert("s".into(), Value::from("'single'"));
        foo_attributes.insert("d".into(), Value::from(r#""double""#));
        foo_attributes.insert("m".into(), Value::from(r#""mixed'"#));
        let foo = Event::Tag("foo", Type::Empty, foo_attributes);

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        writer.write_event(&foo).unwrap();

        assert_eq!(
            String::from_utf8_lossy(&output),
            r#"<foo d='"double"' s="'single'"/>"#
        );
    }

    #[test]
    fn style_display() {
        let style = Event::Tag("style", Type::Start, HashMap::new());

        let style_text = Event::Text("* { font-family: foo; }");

        let style_end = Event::Tag("style", Type::End, HashMap::new());

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        for event in &[style, style_text, style_end] {
            writer.write_event(event).unwrap();
        }

        assert_eq!(
            String::from_utf8_lossy(&output),
            "<style>\n\
             * { font-family: foo; }\n\
             </style>\
             "
        );
    }
}
