use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::LangHandler;
use crate::util::{find_node_at_position, node_text};

/// Find the definition of the symbol at the given position.
pub fn goto_definition(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
    pos: Position,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    let node = find_node_at_position(tree, pos)?;
    let name = node_text(&node, src);

    let defs = handler.find_definitions(tree, src, name);
    if defs.is_empty() {
        return None;
    }

    let locations: Vec<Location> = defs
        .into_iter()
        .map(|range| Location {
            uri: uri.clone(),
            range,
        })
        .collect();

    if locations.len() == 1 {
        Some(GotoDefinitionResponse::Scalar(
            locations.into_iter().next().unwrap(),
        ))
    } else {
        Some(GotoDefinitionResponse::Array(locations))
    }
}
