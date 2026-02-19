use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{LangHandler, SymbolInfo};

/// Extract document symbols from the tree.
pub fn document_symbols(
    handler: &dyn LangHandler,
    tree: &Tree,
    src: &str,
) -> DocumentSymbolResponse {
    let symbols = handler.document_symbols(tree, src);
    let lsp_symbols = symbols.into_iter().map(to_lsp_symbol).collect();
    DocumentSymbolResponse::Nested(lsp_symbols)
}

fn to_lsp_symbol(sym: SymbolInfo) -> DocumentSymbol {
    let children = if sym.children.is_empty() {
        None
    } else {
        Some(sym.children.into_iter().map(to_lsp_symbol).collect())
    };

    #[allow(deprecated)]
    DocumentSymbol {
        name: sym.name,
        detail: None,
        kind: sym.kind,
        tags: None,
        deprecated: None,
        range: sym.range,
        selection_range: sym.selection_range,
        children,
    }
}
