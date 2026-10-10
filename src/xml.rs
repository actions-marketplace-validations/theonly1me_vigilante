use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

const MAXIMUM_DEPTH: usize = 64;
const ENTITIES: &[(&str, &str)] = &[
    ("amp", "&"),
    ("lt", "<"),
    ("gt", ">"),
    ("quot", "\""),
    ("apos", "'"),
];

#[derive(Debug, Default)]
pub struct Element {
    pub name: String,
    pub text: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
}

impl Element {
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }

    pub fn all(&self, name: &str) -> Vec<&Element> {
        self.children
            .iter()
            .filter(|child| child.name == name)
            .collect()
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn descendants(&self, name: &str) -> Vec<&Element> {
        let mut found = Vec::new();
        let mut pending: Vec<&Element> = self.children.iter().collect();
        while let Some(element) = pending.pop() {
            if element.name == name {
                found.push(element);
            }
            pending.extend(element.children.iter());
        }
        found.reverse();
        found
    }

    pub fn value(&self, name: &str) -> Option<String> {
        self.attribute(name)
            .map(str::to_string)
            .or_else(|| self.text_of(name))
    }

    pub fn text_of(&self, name: &str) -> Option<String> {
        self.child(name)
            .map(|child| child.text.trim().to_string())
            .filter(|text| !text.is_empty())
    }
}

pub fn parse(text: &str) -> Option<Element> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(true);
    let mut stack = vec![Element::default()];
    loop {
        match reader.read_event().ok()? {
            Event::Start(start) => {
                if stack.len() > MAXIMUM_DEPTH {
                    return None;
                }
                stack.push(element(&start));
            }
            Event::Empty(empty) => stack.last_mut()?.children.push(element(&empty)),
            Event::End(_) => {
                let done = stack.pop()?;
                stack.last_mut()?.children.push(done);
            }
            Event::Text(text) => stack.last_mut()?.text.push_str(&text.xml10_content()),
            Event::CData(data) => stack.last_mut()?.text.push_str(&data.into_inner()),
            Event::GeneralRef(reference) => {
                let name = reference.as_ref().to_string();
                if let Some((_, value)) = ENTITIES.iter().find(|(entity, _)| *entity == name) {
                    stack.last_mut()?.text.push_str(value);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    stack.pop()?.children.into_iter().next()
}

fn element(start: &BytesStart) -> Element {
    Element {
        name: start.local_name().as_ref().to_string(),
        attributes: start
            .attributes()
            .flatten()
            .map(|attribute| {
                (
                    attribute.key.local_name().as_ref().to_string(),
                    attribute
                        .normalized_value(XmlVersion::Implicit1_0)
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                )
            })
            .collect(),
        ..Element::default()
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn elements_keep_their_attributes_text_and_children() {
        let root = parse(
            "<Project Sdk=\"Microsoft.NET.Sdk\"><ItemGroup><PackageReference Include=\"Serilog\" Version=\"3.1.1\" /><PackageReference Include=\"Polly\"><Version>8.4.0</Version></PackageReference></ItemGroup></Project>",
        )
        .expect("project");

        let group = root.child("ItemGroup").expect("group");
        let references = group.all("PackageReference");
        assert_eq!(root.attribute("Sdk"), Some("Microsoft.NET.Sdk"));
        assert_eq!(references[0].attribute("Include"), Some("Serilog"));
        assert_eq!(references[0].attribute("Version"), Some("3.1.1"));
        assert_eq!(references[1].text_of("Version").as_deref(), Some("8.4.0"));
        assert_eq!(references[1].attribute("Version"), None);
    }
}
