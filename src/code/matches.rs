use ast_grep_core::Node;
use ast_grep_core::tree_sitter::StrDoc;
use ast_grep_language::SupportLang;

use super::Rules;
use crate::Capability;

pub type SyntaxNode<'r> = Node<'r, StrDoc<SupportLang>>;

pub struct RuleMatch<'r> {
    pub rule: String,
    pub capability: Capability,
    pub node: SyntaxNode<'r>,
}

pub struct MatchRequest<'a, 'r> {
    pub path: &'a str,
    pub language: SupportLang,
    pub root: &'a SyntaxNode<'r>,
}

impl Rules {
    pub fn matches<'r>(&self, request: MatchRequest<'_, 'r>) -> Vec<RuleMatch<'r>> {
        let root = request.root;
        self.rules
            .iter()
            .filter(|rule| rule.language == request.language && rule.files.allows(request.path))
            .flat_map(|rule| {
                root.find_all(&rule.config.matcher)
                    .map(move |found| RuleMatch {
                        rule: rule.id.clone(),
                        capability: rule.capability,
                        node: found.get_node().clone(),
                    })
            })
            .collect()
    }
}
