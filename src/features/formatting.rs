use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::LangHandler;

/// Format the entire document.
pub fn format_document(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
) -> Option<Vec<TextEdit>> {
    let formatted = handler.format_document(tree, src)?;

    if formatted == src {
        return None;
    }

    // Replace the entire document content
    let line_count = src.lines().count() as u32;
    let last_line_len = src.lines().last().map(|l| l.len() as u32).unwrap_or(0);

    Some(vec![TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: line_count,
                character: last_line_len,
            },
        },
        new_text: formatted,
    }])
}
