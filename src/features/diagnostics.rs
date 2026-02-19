use lsp_types::*;
use tree_sitter::Tree;

/// Collect diagnostics from tree-sitter ERROR and MISSING nodes.
pub fn collect(tree: &Tree, src: &str) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];
    let root = tree.root_node();
    walk_errors(root, src, &mut diagnostics);
    diagnostics
}

fn walk_errors(node: tree_sitter::Node, src: &str, diags: &mut Vec<Diagnostic>) {
    if node.is_error() {
        let range = crate::util::node_range(&node);
        let text = crate::util::node_text(&node, src);
        let message = if text.len() > 40 {
            format!("Syntax error near `{}...`", &text[..40])
        } else {
            format!("Syntax error near `{}`", text)
        };
        diags.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("wplabs-lsp".to_string()),
            message,
            ..Default::default()
        });
        return; // Don't recurse into error nodes
    }

    if node.is_missing() {
        let range = crate::util::node_range(&node);
        let message = format!("Missing `{}`", node.kind());
        diags.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("wplabs-lsp".to_string()),
            message,
            ..Default::default()
        });
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            walk_errors(child, src, diags);
        }
    }
}
