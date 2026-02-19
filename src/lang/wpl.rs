use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{BuiltinInfo, LangHandler, SymbolInfo};
use crate::util::{node_range, node_text};

pub struct WplHandler;

impl LangHandler for WplHandler {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_wpl::language()
    }

    fn lang_id(&self) -> &str {
        "wpl"
    }

    fn extensions(&self) -> &[&str] {
        &["wpl"]
    }

    fn keywords(&self) -> &[&str] {
        &[
            "package", "rule", "field", "metadata", "length", "format",
            "pipe", "separator", "target", "type", "tag", "copy_raw",
            "alt", "opt", "some_of", "seq", "not",
        ]
    }

    fn builtins(&self) -> &[BuiltinInfo] {
        &BUILTINS
    }

    fn document_symbols(&self, tree: &Tree, src: &str) -> Vec<SymbolInfo> {
        let mut symbols = vec![];
        let root = tree.root_node();

        for i in 0..root.named_child_count() {
            let Some(child) = root.named_child(i) else {
                continue;
            };
            match child.kind() {
                "package_decl" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let mut children = vec![];
                        collect_rule_symbols(&child, src, &mut children);
                        symbols.push(SymbolInfo {
                            name: node_text(&name_node, src).to_string(),
                            kind: SymbolKind::NAMESPACE,
                            range: node_range(&child),
                            selection_range: node_range(&name_node),
                            children,
                        });
                    }
                }
                "rule_decl" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        symbols.push(SymbolInfo {
                            name: node_text(&name_node, src).to_string(),
                            kind: SymbolKind::FUNCTION,
                            range: node_range(&child),
                            selection_range: node_range(&name_node),
                            children: vec![],
                        });
                    }
                }
                _ => {}
            }
        }

        symbols
    }

    fn find_definitions(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut defs = vec![];
        find_decl_defs(tree.root_node(), src, name, &mut defs);
        defs
    }

    fn find_references(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut refs = vec![];
        collect_identifier_refs(tree.root_node(), src, name, &mut refs);
        refs
    }

    fn format_document(&self, _tree: &Tree, src: &str) -> Option<String> {
        Some(simple_indent_format(src))
    }
}

fn collect_rule_symbols(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        if child.kind() == "rule_decl" {
            if let Some(name_node) = child.child_by_field_name("name") {
                symbols.push(SymbolInfo {
                    name: node_text(&name_node, src).to_string(),
                    kind: SymbolKind::FUNCTION,
                    range: node_range(&child),
                    selection_range: node_range(&name_node),
                    children: vec![],
                });
            }
        }
    }
}

fn find_decl_defs(node: tree_sitter::Node, src: &str, name: &str, defs: &mut Vec<Range>) {
    match node.kind() {
        "package_decl" | "rule_decl" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                if node_text(&name_node, src) == name {
                    defs.push(node_range(&name_node));
                }
            }
        }
        _ => {}
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            find_decl_defs(child, src, name, defs);
        }
    }
}

fn collect_identifier_refs(
    node: tree_sitter::Node,
    src: &str,
    name: &str,
    refs: &mut Vec<Range>,
) {
    if (node.kind() == "identifier" || node.kind() == "path_name")
        && node_text(&node, src) == name
    {
        refs.push(node_range(&node));
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            collect_identifier_refs(child, src, name, refs);
        }
    }
}

fn simple_indent_format(src: &str) -> String {
    let mut result = String::with_capacity(src.len());
    let mut indent = 0usize;

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }
        if trimmed.starts_with('}') || trimmed.starts_with(')') {
            indent = indent.saturating_sub(1);
        }
        for _ in 0..indent {
            result.push_str("  ");
        }
        result.push_str(trimmed);
        result.push('\n');
        if trimmed.ends_with('{') || trimmed.ends_with('(') {
            indent += 1;
        }
    }

    result
}

static BUILTINS: [BuiltinInfo; 10] = [
    BuiltinInfo {
        name: "chars",
        signature: "chars",
        documentation: "String/text type.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "digit",
        signature: "digit",
        documentation: "Integer numeric type.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "float",
        signature: "float",
        documentation: "Floating-point numeric type.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "bool",
        signature: "bool",
        documentation: "Boolean type.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "time",
        signature: "time",
        documentation: "Timestamp type (various formats: time, time_3339, time/clf).",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "ip",
        signature: "ip",
        documentation: "IP address type.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "hex",
        signature: "hex",
        documentation: "Hexadecimal data type.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "json",
        signature: "json(type@name)",
        documentation: "JSON data type with optional typed extraction.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "kvarr",
        signature: "kvarr(type@name, ...)",
        documentation: "Key-value array type with typed fields.",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
    BuiltinInfo {
        name: "http/request",
        signature: "http/request",
        documentation: "HTTP request line type (method + URI + version).",
        kind: CompletionItemKind::TYPE_PARAMETER,
    },
];
