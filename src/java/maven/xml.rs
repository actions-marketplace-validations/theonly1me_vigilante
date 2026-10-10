use quick_xml::Reader;
use quick_xml::events::Event;

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
                stack.push(Element {
                    name: start.local_name().as_ref().to_string(),
                    ..Element::default()
                });
            }
            Event::Empty(empty) => stack.last_mut()?.children.push(Element {
                name: empty.local_name().as_ref().to_string(),
                ..Element::default()
            }),
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
