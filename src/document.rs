use tree_sitter::Tree;

/// Represents the state of an open document.
pub struct DocumentState {
    pub text: String,
    pub tree: Tree,
    pub version: i32,
    pub lang_id: String,
}
