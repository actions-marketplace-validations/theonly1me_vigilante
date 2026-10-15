use super::paths::{MemberCall, member};
use crate::code::SyntaxNode;
use crate::deep::program::UseTarget;

pub enum Pending {
    Ready(UseTarget),
    Path {
        text: String,
        impl_type: Option<String>,
    },
}

pub struct PendingCall<'a, 'r> {
    pub function: &'a SyntaxNode<'r>,
    pub impl_type: Option<&'a str>,
}

pub fn pending_call(call: PendingCall) -> Option<Pending> {
    let PendingCall {
        function,
        impl_type,
    } = call;
    match function.kind().as_ref() {
        "identifier" => Some(Pending::Ready(UseTarget::Name(function.text().to_string()))),
        "scoped_identifier" => Some(Pending::Path {
            text: function.text().to_string(),
            impl_type: impl_type.map(str::to_string),
        }),
        "generic_function" => pending_call(PendingCall {
            function: &function.field("function")?,
            impl_type,
        }),
        "field_expression" => {
            let property = function.field("field")?.text().to_string();
            let value = function.field("value")?;
            Some(Pending::Ready(match impl_type {
                Some(owner) if value.kind() == "self" => member(MemberCall {
                    object: owner.to_string(),
                    property,
                }),
                _ => UseTarget::Method(property),
            }))
        }
        _ => None,
    }
}
