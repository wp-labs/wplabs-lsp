use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{BuiltinInfo, LangHandler, SymbolInfo};
use crate::util::{node_range, node_text};

pub struct WfgHandler;

impl LangHandler for WfgHandler {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_wfg::language()
    }

    fn lang_id(&self) -> &str {
        "wfg"
    }

    fn extensions(&self) -> &[&str] {
        &["wfg"]
    }

    fn keywords(&self) -> &[&str] {
        &[
            "use", "scenario", "seed", "time", "duration", "total",
            "stream", "inject", "for", "on", "faults", "oracle", "hit",
            "near_miss", "non_hit", "true", "false", "out_of_order",
            "late", "duplicate", "drop",
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
            if child.kind() == "scenario_declaration" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let mut children = vec![];
                    collect_stream_symbols(&child, src, &mut children);
                    collect_inject_symbols(&child, src, &mut children);
                    symbols.push(SymbolInfo {
                        name: node_text(&name_node, src).to_string(),
                        kind: SymbolKind::FUNCTION,
                        range: node_range(&child),
                        selection_range: node_range(&name_node),
                        children,
                    });
                }
            }
        }

        symbols
    }

    fn find_definitions(&self, tree: &Tree, src: &str, name: &str) -> Vec<Range> {
        let mut defs = vec![];
        find_scenario_defs(tree.root_node(), src, name, &mut defs);
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

fn collect_stream_symbols(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        if child.kind() == "stream_block" {
            if let Some(alias) = child.child_by_field_name("alias") {
                symbols.push(SymbolInfo {
                    name: node_text(&alias, src).to_string(),
                    kind: SymbolKind::VARIABLE,
                    range: node_range(&child),
                    selection_range: node_range(&alias),
                    children: vec![],
                });
            }
        }
    }
}

fn collect_inject_symbols(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i) else {
            continue;
        };
        if child.kind() == "inject_block" {
            if let Some(rule_node) = child.child_by_field_name("rule") {
                symbols.push(SymbolInfo {
                    name: format!("inject {}", node_text(&rule_node, src)),
                    kind: SymbolKind::EVENT,
                    range: node_range(&child),
                    selection_range: node_range(&rule_node),
                    children: vec![],
                });
            }
        }
    }
}

fn find_scenario_defs(
    node: tree_sitter::Node,
    src: &str,
    name: &str,
    defs: &mut Vec<Range>,
) {
    match node.kind() {
        "scenario_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                if node_text(&name_node, src) == name {
                    defs.push(node_range(&name_node));
                }
            }
        }
        "stream_block" => {
            if let Some(alias) = node.child_by_field_name("alias") {
                if node_text(&alias, src) == name {
                    defs.push(node_range(&alias));
                }
            }
        }
        _ => {}
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i) {
            find_scenario_defs(child, src, name, defs);
        }
    }
}

fn collect_identifier_refs(
    node: tree_sitter::Node,
    src: &str,
    name: &str,
    refs: &mut Vec<Range>,
) {
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
        if trimmed.starts_with('}') || trimmed.starts_with(']') {
            indent = indent.saturating_sub(1);
        }
        for _ in 0..indent {
            result.push_str("  ");
        }
        result.push_str(trimmed);
        result.push('\n');
        if trimmed.ends_with('{') || trimmed.ends_with('[') {
            indent += 1;
        }
    }

    result
}

static BUILTINS: [BuiltinInfo; 7] = [
    BuiltinInfo {
        name: "ipv4",
        signature: "ipv4(pool: N)",
        documentation: "Generate random IPv4 addresses from a pool of N unique IPs.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "pattern",
        signature: "pattern(template)",
        documentation: "Generate values from a template pattern. Supports {seq:NNNN} for sequences.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "uniform",
        signature: "uniform(min, max)",
        documentation: "Generate uniformly distributed random numbers in [min, max].",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "choice",
        signature: "choice([values...])",
        documentation: "Randomly choose from a list of values.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "counter",
        signature: "counter(start, step)",
        documentation: "Generate sequential counter values.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "normal",
        signature: "normal(mean, stddev)",
        documentation: "Generate normally distributed random numbers.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "zipf",
        signature: "zipf(n, exponent)",
        documentation: "Generate Zipf-distributed values from [1, n].",
        kind: CompletionItemKind::FUNCTION,
    },
];
