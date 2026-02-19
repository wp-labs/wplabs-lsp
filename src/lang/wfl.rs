use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{BuiltinInfo, LangHandler, SymbolInfo};
use crate::util::{node_range, node_text};

pub struct WflHandler;

impl LangHandler for WflHandler {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_wfl::language()
    }

    fn lang_id(&self) -> &str {
        "wfl"
    }

    fn extensions(&self) -> &[&str] {
        &["wfl"]
    }

    fn keywords(&self) -> &[&str] {
        &[
            "use", "rule", "meta", "events", "match", "keys", "duration", "on", "event", "close",
            "score", "entity", "yield", "contract", "given", "expect", "derive", "stage", "and",
            "or", "not", "in", "true", "false", "if", "then", "else", "join", "conv",
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
                "rule_declaration" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let mut children = vec![];
                        // Extract event aliases as child symbols
                        collect_event_aliases(&child, src, &mut children);
                        symbols.push(SymbolInfo {
                            name: node_text(&name_node, src).to_string(),
                            kind: SymbolKind::FUNCTION,
                            range: node_range(&child),
                            selection_range: node_range(&name_node),
                            children,
                        });
                    }
                }
                "contract_block" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        symbols.push(SymbolInfo {
                            name: node_text(&name_node, src).to_string(),
                            kind: SymbolKind::CLASS,
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
        let root = tree.root_node();

        for i in 0..root.named_child_count() {
            let Some(child) = root.named_child(i) else {
                continue;
            };
            match child.kind() {
                "rule_declaration" | "contract_block" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        if node_text(&name_node, src) == name {
                            defs.push(node_range(&name_node));
                        }
                    }
                }
                _ => {}
            }
            // Also check event aliases inside rule declarations
            if child.kind() == "rule_declaration" {
                collect_event_def_ranges(&child, src, name, &mut defs);
            }
        }

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

fn collect_event_aliases(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        if child.kind() == "events_block" {
            for j in 0..child.named_child_count() {
                let Some(ev) = child.named_child(j) else {
                    continue;
                };
                if ev.kind() == "event_declaration" {
                    if let Some(alias) = ev.child_by_field_name("alias") {
                        symbols.push(SymbolInfo {
                            name: node_text(&alias, src).to_string(),
                            kind: SymbolKind::VARIABLE,
                            range: node_range(&ev),
                            selection_range: node_range(&alias),
                            children: vec![],
                        });
                    }
                }
            }
        }
        collect_event_aliases(&child, src, symbols);
    }
}

fn collect_event_def_ranges(
    node: &tree_sitter::Node,
    src: &str,
    name: &str,
    defs: &mut Vec<Range>,
) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        if child.kind() == "event_declaration" {
            if let Some(alias) = child.child_by_field_name("alias") {
                if node_text(&alias, src) == name {
                    defs.push(node_range(&alias));
                }
            }
        }
        collect_event_def_ranges(&child, src, name, defs);
    }
}

fn collect_identifier_refs(node: tree_sitter::Node, src: &str, name: &str, refs: &mut Vec<Range>) {
    if node.kind() == "identifier" && node_text(&node, src) == name {
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

        // Decrease indent for closing braces
        if trimmed.starts_with('}') || trimmed.starts_with(')') {
            indent = indent.saturating_sub(1);
        }

        // Write indented line
        for _ in 0..indent {
            result.push_str("  ");
        }
        result.push_str(trimmed);
        result.push('\n');

        // Increase indent for opening braces
        if trimmed.ends_with('{') || trimmed.ends_with('(') {
            indent += 1;
        }
    }

    result
}

static BUILTINS: [BuiltinInfo; 11] = [
    BuiltinInfo {
        name: "count",
        signature: "count(events) -> digit",
        documentation: "Count the number of events in a window.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "sum",
        signature: "sum(field) -> float",
        documentation: "Sum a numeric field across events.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "avg",
        signature: "avg(field) -> float",
        documentation: "Average a numeric field across events.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "min",
        signature: "min(field) -> float",
        documentation: "Minimum value of a numeric field.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "max",
        signature: "max(field) -> float",
        documentation: "Maximum value of a numeric field.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "distinct",
        signature: "distinct(field) -> array",
        documentation: "Get distinct values of a field.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "fmt",
        signature: "fmt(template, ...args) -> chars",
        documentation: "Format a string with placeholders.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Now::time",
        signature: "Now::time() -> time",
        documentation: "Current timestamp.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Now::date",
        signature: "Now::date() -> chars",
        documentation: "Current date string.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "Now::hour",
        signature: "Now::hour() -> digit",
        documentation: "Current hour (0-23).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "has",
        signature: "has(field) -> bool",
        documentation: "Check if a dimension table contains a value.",
        kind: CompletionItemKind::FUNCTION,
    },
];
