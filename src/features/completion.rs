use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::LangHandler;
use crate::util::{find_node_at_position, node_text};

/// Generate completion items for the given position.
pub fn complete(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
    pos: Position,
) -> Vec<CompletionItem> {
    let mut items = vec![];

    // Determine prefix at cursor for filtering
    let prefix = get_prefix_at(src, pos);

    // 1. Keyword completions
    for kw in handler.keywords() {
        if prefix.is_empty() || kw.starts_with(&prefix) {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }
    }

    // 2. Builtin completions
    for bi in handler.builtins() {
        if prefix.is_empty() || bi.name.starts_with(&prefix) {
            items.push(CompletionItem {
                label: bi.name.to_string(),
                kind: Some(bi.kind),
                detail: Some(bi.signature.to_string()),
                documentation: Some(Documentation::String(bi.documentation.to_string())),
                ..Default::default()
            });
        }
    }

    // 3. Document symbol completions (identifiers already defined in this file)
    let mut seen = std::collections::HashSet::new();
    collect_identifiers_from_tree(tree.root_node(), src, &prefix, &mut seen, &mut items);

    // 4. Context-aware completions based on cursor node
    if let Some(node) = find_node_at_position(tree, pos) {
        context_completions(&node, src, &prefix, &mut items);
    }

    items
}

fn get_prefix_at(src: &str, pos: Position) -> String {
    let line_idx = pos.line as usize;
    let col = pos.character as usize;

    let line = match src.lines().nth(line_idx) {
        Some(l) => l,
        None => return String::new(),
    };

    let before_cursor = if col <= line.len() {
        &line[..col]
    } else {
        line
    };

    // Walk backwards to find the start of the current identifier
    let start = before_cursor
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
        .map(|i| i + 1)
        .unwrap_or(0);

    before_cursor[start..].to_string()
}

fn collect_identifiers_from_tree(
    node: tree_sitter::Node,
    src: &str,
    prefix: &str,
    seen: &mut std::collections::HashSet<String>,
    items: &mut Vec<CompletionItem>,
) {
    if node.kind() == "identifier" {
        let text = node_text(&node, src).to_string();
        if (prefix.is_empty() || text.starts_with(prefix)) && seen.insert(text.clone()) {
            items.push(CompletionItem {
                label: text,
                kind: Some(CompletionItemKind::VARIABLE),
                ..Default::default()
            });
        }
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            collect_identifiers_from_tree(child, src, prefix, seen, items);
        }
    }
}

fn context_completions(
    node: &tree_sitter::Node,
    _src: &str,
    _prefix: &str,
    _items: &mut Vec<CompletionItem>,
) {
    // Context-aware completions based on parent node type
    let _parent = node.parent();
    // Future: add field completion inside field_override, type completion
    // after ':', etc.
}
