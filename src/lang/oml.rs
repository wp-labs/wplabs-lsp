use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{BuiltinInfo, LangHandler, SymbolInfo};
use crate::util::{node_range, node_text};

pub struct OmlHandler;

impl LangHandler for OmlHandler {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_oml::language()
    }

    fn lang_id(&self) -> &str {
        "oml"
    }

    fn extensions(&self) -> &[&str] {
        &["oml"]
    }

    fn keywords(&self) -> &[&str] {
        &[
            "name", "rule", "read", "take", "pipe", "fmt", "object", "collect", "match", "select",
            "from", "where", "and", "or", "not", "in", "auto", "ip", "chars", "digit", "float",
            "time", "bool", "obj", "array",
        ]
    }

    fn builtins(&self) -> &[BuiltinInfo] {
        &BUILTINS
    }

    fn document_symbols(&self, tree: &Tree, src: &str) -> Vec<SymbolInfo> {
        let mut symbols = vec![];
        let root = tree.root_node();

        // Header name declaration
        for i in 0..root.named_child_count() {
            let Some(child) = root.named_child(i as u32) else {
                continue;
            };
            if child.kind() == "header" {
                for j in 0..child.named_child_count() {
                    let Some(hc) = child.named_child(j as u32) else {
                        continue;
                    };
                    if hc.kind() == "header_name" || hc.kind() == "header_rule" {
                        // Extract the value after the colon
                        if let Some(val) = hc.named_child(0u32) {
                            symbols.push(SymbolInfo {
                                name: node_text(&val, src).to_string(),
                                kind: if hc.kind() == "header_name" {
                                    SymbolKind::MODULE
                                } else {
                                    SymbolKind::FUNCTION
                                },
                                range: node_range(&hc),
                                selection_range: node_range(&val),
                                children: vec![],
                            });
                        }
                    }
                }
            }

            // Aggregate items as symbols
            if child.kind() == "aggregate_item" {
                collect_target_symbols(&child, src, &mut symbols);
            }
        }

        symbols
    }

    fn find_definitions(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut defs = vec![];
        collect_target_defs(tree.root_node(), src, name, &mut defs);
        defs
    }

    fn find_references(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut refs = vec![];
        collect_refs(tree.root_node(), src, name, &mut refs);
        refs
    }

    fn format_document(&self, _tree: &Tree, src: &str) -> Option<String> {
        // OML has a unique format with --- separator; preserve it
        Some(src.to_string())
    }
}

fn collect_target_symbols(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i as u32) else {
            continue;
        };
        if child.kind() == "target" {
            if let Some(name_node) = child.child_by_field_name("name") {
                let name = node_text(&name_node, src);
                if name != "_" {
                    symbols.push(SymbolInfo {
                        name: name.to_string(),
                        kind: SymbolKind::VARIABLE,
                        range: node_range(&child),
                        selection_range: node_range(&name_node),
                        children: vec![],
                    });
                }
            }
        }
    }
}

fn collect_target_defs(node: tree_sitter::Node, src: &str, name: &str, defs: &mut Vec<Range>) {
    if node.kind() == "target" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if node_text(&name_node, src) == name {
                defs.push(node_range(&name_node));
            }
        }
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i as u32) {
            collect_target_defs(child, src, name, defs);
        }
    }
}

fn collect_refs(node: tree_sitter::Node, src: &str, name: &str, refs: &mut Vec<Range>) {
    // Match @ref references and identifiers
    if (node.kind() == "identifier" || node.kind() == "at_ref")
        && node_text(&node, src).trim_start_matches('@') == name
    {
        refs.push(node_range(&node));
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i as u32) {
            collect_refs(child, src, name, refs);
        }
    }
}

static BUILTINS: [BuiltinInfo; 16] = [
    BuiltinInfo {
        name: "to_json",
        signature: "to_json",
        documentation: "Convert value to JSON string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "base64_decode",
        signature: "base64_decode(encoding)",
        documentation: "Decode a Base64-encoded string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "base64_encode",
        signature: "base64_encode",
        documentation: "Encode a string to Base64.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "url",
        signature: "url(component)",
        documentation: "Extract a URL component (domain, host, uri, path, params).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "nth",
        signature: "nth(index)",
        documentation: "Get the nth element from an array or object.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "get",
        signature: "get(path)",
        documentation: "Get a nested value by path.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "path",
        signature: "path(component)",
        documentation: "Extract a file path component (name, path).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "json_escape",
        signature: "json_escape",
        documentation: "Escape a string for JSON.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "json_unescape",
        signature: "json_unescape",
        documentation: "Unescape a JSON string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "html_unescape",
        signature: "html_unescape",
        documentation: "Unescape HTML entities.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "html_escape",
        signature: "html_escape",
        documentation: "Escape HTML entities.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "str_escape",
        signature: "str_escape",
        documentation: "Escape special characters in a string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "to_str",
        signature: "to_str",
        documentation: "Convert value to string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "skip_empty",
        signature: "skip_empty",
        documentation: "Skip the value if it is empty.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "ip4_to_int",
        signature: "ip4_to_int",
        documentation: "Convert an IPv4 address to an integer.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Time::to_ts_ms",
        signature: "Time::to_ts_ms",
        documentation: "Convert time to Unix timestamp in milliseconds.",
        kind: CompletionItemKind::FUNCTION,
    },
];
