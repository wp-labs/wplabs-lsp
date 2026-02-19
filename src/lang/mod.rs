pub mod oml;
pub mod wfg;
pub mod wfl;
pub mod wfs;
pub mod wpl;

use lsp_types::*;
use tree_sitter::Tree;

/// Information about a built-in function, type, or constant.
pub struct BuiltinInfo {
    pub name: &'static str,
    pub signature: &'static str,
    pub documentation: &'static str,
    pub kind: CompletionItemKind,
}

/// A language-agnostic symbol extracted from the tree-sitter AST.
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<SymbolInfo>,
}

/// Trait implemented by each language (WFL, WFS, WPL, OML, WFG).
///
/// Provides language-specific data (keywords, builtins, grammar) and
/// tree-sitter-based analysis methods (symbol extraction, definition
/// resolution, formatting).
pub trait LangHandler: Send + Sync {
    /// The tree-sitter Language for this DSL.
    fn language(&self) -> tree_sitter::Language;

    /// The LSP language identifier (e.g. "wfl").
    fn lang_id(&self) -> &str;

    /// File extensions handled by this language (without leading dot).
    fn extensions(&self) -> &[&str];

    /// Language keywords for completion.
    fn keywords(&self) -> &[&str];

    /// Built-in functions, types, and constants.
    fn builtins(&self) -> &[BuiltinInfo];

    /// Extract document symbols (rule declarations, window definitions, etc.).
    fn document_symbols(&self, tree: &Tree, src: &str) -> Vec<SymbolInfo>;

    /// Find all declaration sites for a given symbol name.
    fn find_definitions(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range>;

    /// Find all reference sites for a given symbol name (excluding declarations).
    fn find_references(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range>;

    /// Format the entire document, returning the formatted text.
    fn format_document(&self, tree: &Tree, src: &str) -> Option<String>;
}
