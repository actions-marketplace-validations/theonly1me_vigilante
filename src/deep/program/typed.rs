use std::collections::HashMap;

use super::FileKey;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypedTarget {
    pub file: FileKey,
    pub name: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Typed {
    Targets(Vec<TypedTarget>),
    Unanalyzed,
}

pub enum TypeCheck {
    Checked(TypedCalls),
    NotChecked(String),
}

pub struct TypedCall {
    pub file: FileKey,
    pub line: usize,
    pub callee: String,
    pub typed: Typed,
}

pub struct CallSite<'a> {
    pub file: &'a FileKey,
    pub line: usize,
    pub callee: &'a str,
}

#[derive(Default)]
pub struct TypedCalls {
    calls: HashMap<(FileKey, usize, String), Typed>,
}

impl TypedCalls {
    pub fn add(&mut self, call: TypedCall) {
        let key = (call.file, call.line, call.callee);
        let merged = match (self.calls.remove(&key), call.typed) {
            (Some(Typed::Targets(mut known)), Typed::Targets(added)) => {
                for target in added {
                    if !known.contains(&target) {
                        known.push(target);
                    }
                }
                Typed::Targets(known)
            }
            (Some(Typed::Targets(known)), Typed::Unanalyzed)
            | (Some(Typed::Unanalyzed), Typed::Targets(known)) => Typed::Targets(known),
            (_, typed) => typed,
        };
        self.calls.insert(key, merged);
    }

    pub fn call(&self, site: CallSite) -> Option<&Typed> {
        self.calls
            .get(&(site.file.clone(), site.line, site.callee.to_string()))
    }

    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{CallSite, Typed, TypedCall, TypedCalls, TypedTarget};
    use crate::deep::program::{FileKey, Root};

    fn key(path: &str) -> FileKey {
        FileKey {
            root: Root::Repository,
            path: path.to_string(),
        }
    }

    fn target(name: &str) -> TypedTarget {
        TypedTarget {
            file: key("src/a.ts"),
            name: name.to_string(),
            line: 1,
        }
    }

    fn add(calls: &mut TypedCalls, typed: Typed) {
        calls.add(TypedCall {
            file: key("src/a.ts"),
            line: 3,
            callee: "run".to_string(),
            typed,
        });
    }

    fn at_line_three(calls: &TypedCalls) -> Option<&Typed> {
        calls.call(CallSite {
            file: &key("src/a.ts"),
            line: 3,
            callee: "run",
        })
    }

    #[test]
    fn two_calls_with_one_callee_on_one_line_keep_both_targets() {
        let mut calls = TypedCalls::default();
        add(&mut calls, Typed::Targets(vec![target("first")]));
        add(&mut calls, Typed::Targets(vec![target("second")]));

        assert_eq!(
            at_line_three(&calls),
            Some(&Typed::Targets(vec![target("first"), target("second")]))
        );
    }

    #[test]
    fn a_known_target_wins_over_an_unanalyzed_call_on_the_same_line() {
        let mut calls = TypedCalls::default();
        add(&mut calls, Typed::Unanalyzed);
        add(&mut calls, Typed::Targets(vec![target("first")]));

        assert_eq!(
            at_line_three(&calls),
            Some(&Typed::Targets(vec![target("first")]))
        );
    }
}
