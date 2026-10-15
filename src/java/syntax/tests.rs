use ast_grep_core::tree_sitter::LanguageExt;
use ast_grep_language::SupportLang;

use super::index;
use crate::deep::program::Supertypes;

#[test]
fn supertypes_leave_out_generic_arguments_and_keep_qualified_names() {
    let document = SupportLang::Java.ast_grep(
        "class Shell extends Base<Helper> implements com.acme.Tool<Other>, Runnable {}\n\ninterface Task extends Callable<Result> {}\n",
    );

    assert_eq!(
        index(&document.root()).supertypes,
        vec![
            Supertypes {
                class: "Shell".to_string(),
                names: vec![
                    "Base".to_string(),
                    "com.acme.Tool".to_string(),
                    "Runnable".to_string()
                ],
            },
            Supertypes {
                class: "Task".to_string(),
                names: vec!["Callable".to_string()],
            },
        ]
    );
}
