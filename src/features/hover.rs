use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::LangHandler;
use crate::util::{find_node_at_position, node_range, node_text};

/// Generate hover information for the symbol at the given position.
pub fn hover(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
    pos: Position,
) -> Option<Hover> {
    let node = find_node_at_position(tree, pos)?;
    let text = node_text(&node, src);

    // Check if hovering over a keyword
    if handler.keywords().contains(&text) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Keyword** `{}`", text),
            }),
            range: Some(node_range(&node)),
        });
    }

    // Check if hovering over a builtin
    for bi in handler.builtins() {
        if bi.name == text {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```\n{}\n```\n{}", bi.signature, bi.documentation),
                }),
                range: Some(node_range(&node)),
            });
        }
    }

    // Show node type and content for identifiers
    if node.kind() == "identifier" {
        // Try to find the definition context
        let context = find_definition_context(&node, tree, src);
        let value = if let Some(ctx) = context {
            format!("`{}` — {}", text, ctx)
        } else {
            format!("`{}` ({})", text, node.kind())
        };

        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: Some(node_range(&node)),
        });
    }

    // For type keywords (field types)
    let type_nodes = [
        "base_type",
        "field_type",
        "data_type",
    ];
    if type_nodes.contains(&node.kind()) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Type** `{}`", text),
            }),
            range: Some(node_range(&node)),
        });
    }

    None
}

/// Try to find context about what a symbol is (e.g. "rule", "window", "event alias").
fn find_definition_context(
    node: &tree_sitter::Node,
    _tree: &Tree,
    _src: &str,
) -> Option<String> {
    let parent = node.parent()?;

    match parent.kind() {
        "rule_declaration" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("rule declaration".to_string());
            }
        }
        "contract_block" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("contract block".to_string());
            }
        }
        "window_declaration" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("window declaration".to_string());
            }
        }
        "scenario_declaration" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("scenario declaration".to_string());
            }
        }
        "event_declaration" => {
            if parent.child_by_field_name("alias").map(|n| n.id()) == Some(node.id()) {
                return Some("event alias".to_string());
            }
        }
        "stream_block" => {
            if parent.child_by_field_name("alias").map(|n| n.id()) == Some(node.id()) {
                return Some("stream alias".to_string());
            }
        }
        "field_declaration" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("field declaration".to_string());
            }
        }
        "package_decl" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("package declaration".to_string());
            }
        }
        "rule_decl" => {
            if parent.child_by_field_name("name").map(|n| n.id()) == Some(node.id()) {
                return Some("rule declaration".to_string());
            }
        }
        _ => {}
    }

    None
}
