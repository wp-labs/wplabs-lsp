use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::LangHandler;
use crate::util::{find_node_at_position, node_text};

/// Find all references to the symbol at the given position.
pub fn find_references(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
    pos: Position,
    uri: &Url,
    include_declaration: bool,
) -> Vec<Location> {
    let node = match find_node_at_position(tree, pos) {
        Some(n) => n,
        None => return vec![],
    };
    let name = node_text(&node, src);

    let mut locations = vec![];

    // Include definitions if requested
    if include_declaration {
        for range in handler.find_definitions(tree, src, name) {
            locations.push(Location {
                uri: uri.clone(),
                range,
            });
        }
    }

    // Include references
    for range in handler.find_references(tree, src, name) {
        locations.push(Location {
            uri: uri.clone(),
            range,
        });
    }

    locations
}
