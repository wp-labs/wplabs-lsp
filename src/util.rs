use lsp_types::{Position, Range};
use tree_sitter::{Node, Point};

/// Convert a tree-sitter Point to an LSP Position.
pub fn ts_point_to_position(point: Point) -> Position {
    Position {
        line: point.row as u32,
        character: point.column as u32,
    }
}

/// Convert an LSP Position to a tree-sitter Point.
pub fn position_to_ts_point(pos: Position) -> Point {
    Point {
        row: pos.line as usize,
        column: pos.character as usize,
    }
}

/// Get the LSP Range for a tree-sitter Node.
pub fn node_range(node: &Node) -> Range {
    Range {
        start: ts_point_to_position(node.start_position()),
        end: ts_point_to_position(node.end_position()),
    }
}

/// Extract the source text of a tree-sitter Node.
pub fn node_text<'a>(node: &Node, src: &'a str) -> &'a str {
    &src[node.start_byte()..node.end_byte()]
}

/// Find the smallest named node at the given LSP position.
pub fn find_node_at_position<'a>(tree: &'a tree_sitter::Tree, pos: Position) -> Option<tree_sitter::Node<'a>> {
    let point = position_to_ts_point(pos);
    let root = tree.root_node();
    root.named_descendant_for_point_range(point, point)
}

/// Collect all named identifier nodes in the tree by walking recursively.
#[allow(dead_code)]
pub fn collect_identifiers<'a>(node: Node<'a>, src: &str, results: &mut Vec<(String, Range)>) {
    if node.kind() == "identifier" {
        let name = node_text(&node, src).to_string();
        results.push((name, node_range(&node)));
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            collect_identifiers(child, src, results);
        }
    }
}
