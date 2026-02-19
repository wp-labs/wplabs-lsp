use std::collections::HashMap;

use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::LangHandler;
use crate::util::{find_node_at_position, node_text};

/// Prepare a rename operation: find the symbol at the cursor and return its range.
#[allow(dead_code)]
pub fn prepare_rename(tree: &Tree, _src: &str, pos: Position) -> Option<Range> {
    let node = find_node_at_position(tree, pos)?;
    if node.kind() != "identifier" {
        return None;
    }
    Some(crate::util::node_range(&node))
}

/// Execute a rename: find all occurrences of the symbol and replace them.
pub fn rename(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
    pos: Position,
    uri: &Url,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let node = find_node_at_position(tree, pos)?;
    let old_name = node_text(&node, src);

    // Collect all definition and reference sites
    let mut ranges = handler.find_definitions(tree, src, old_name);
    ranges.extend(handler.find_references(tree, src, old_name));

    if ranges.is_empty() {
        return None;
    }

    let edits: Vec<TextEdit> = ranges
        .into_iter()
        .map(|range| TextEdit {
            range,
            new_text: new_name.to_string(),
        })
        .collect();

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), edits);

    Some(WorkspaceEdit {
        changes: Some(changes),
        ..Default::default()
    })
}
