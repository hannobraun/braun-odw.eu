use std::{
    io::{self, Write},
    slice,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Element {
    pub name: &'static str,
    pub attributes: Vec<(&'static str, &'static str)>,
    pub content: Content,
}

impl Element {
    pub fn write_to(&self, target: &mut impl Write) -> io::Result<()> {
        write!(target, "<{}", self.name)?;

        for (name, value) in &self.attributes {
            write!(target, " {}=\"{}\"", name, value)?;
        }

        if self.content.is_empty() {
            write!(target, " />")?;
        } else {
            write!(target, ">")?;

            for child in &self.content {
                child.write_to(target)?;
            }

            write!(target, "</{}>", self.name)?;
        }

        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Content(Vec<Node>);

impl Content {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_iter<Iter, Item>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = Item>,
        Item: Into<Node>,
    {
        let mut content = Vec::new();
        content.extend(iter.into_iter().map(|value| value.into()));
        Self(content)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, node: Node) {
        self.0.push(node)
    }
}

impl From<Element> for Content {
    fn from(element: Element) -> Self {
        Self(vec![Node::Element(element)])
    }
}

impl From<&'static str> for Content {
    fn from(text: &'static str) -> Self {
        Self(vec![Node::Text(text)])
    }
}

impl From<Node> for Content {
    fn from(node: Node) -> Self {
        Self(vec![node])
    }
}

impl<T> From<Vec<T>> for Content
where
    T: Into<Node>,
{
    fn from(nodes: Vec<T>) -> Self {
        Self::from_iter(nodes.into_iter())
    }
}

macro_rules! content_from_tuple {
    ($($($ty:ident),*;)*) => {
        $(
            impl<$($ty,)*> From<($($ty,)*)> for Content
                where
                    $($ty: Into<Content>,)*
            {
                #[allow(non_snake_case)]
                fn from(($($ty,)*): ($($ty,)*)) -> Self {
                    #[allow(unused_mut)]
                    let mut content = Vec::new();

                    $(
                        content.extend($ty.into().0);
                    )*

                    Self(content)
                }
            }
        )*
    };
}

content_from_tuple!(
    ;
    A;
    A, B;
    A, B, C;
    A, B, C, D;
    A, B, C, D, E;
    A, B, C, D, E, F;
);

impl<'a> IntoIterator for &'a Content {
    type Item = &'a Node;
    type IntoIter = slice::Iter<'a, Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// An HTML node
///
/// A node can either be an element or text. Please note that HTML in text nodes
/// is not escaped at this point, so this can be used to inject HTML into the
/// document.
#[derive(Debug, Eq, PartialEq)]
pub enum Node {
    Element(Element),
    Raw(String),
    Text(&'static str),
}

impl Node {
    pub fn write_to(&self, target: &mut impl Write) -> io::Result<()> {
        match self {
            Self::Element(element) => element.write_to(target)?,
            Self::Raw(html) => write!(target, "{}", html)?,
            // TASK: Escape text before injecting HTML into document.
            Self::Text(text) => write!(target, "{}", text)?,
        }

        Ok(())
    }
}

impl From<Element> for Node {
    fn from(element: Element) -> Self {
        Self::Element(element)
    }
}

impl From<&'static str> for Node {
    fn from(text: &'static str) -> Self {
        Self::Text(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{Content, Element, Node};

    #[test]
    fn element_should_write_html_code() {
        let element = Element {
            name: "p",
            attributes: vec![("class", "class")],
            content: Content::from(vec![
                Node::Element(Element {
                    name: "strong",
                    attributes: Vec::new(),
                    content: Content::from(vec![Node::Text(
                        "This is a paragraph.",
                    )]),
                }),
                Node::Element(Element {
                    name: "br",
                    attributes: Vec::new(),
                    content: Content::new(),
                }),
            ]),
        };

        let mut output = Vec::new();
        element.write_to(&mut output).unwrap();

        let expected = "\
            <p class=\"class\">\
                <strong>This is a paragraph.</strong>\
                <br />\
            </p>\
        ";

        println!("expected: {}", expected);
        println!("actual: {}", String::from_utf8(output.clone()).unwrap());

        assert_eq!(output, expected.as_bytes().to_vec());
    }
}
