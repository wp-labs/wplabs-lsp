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
            "use",
            "rule",
            "meta",
            "events",
            "match",
            "key",
            "on",
            "event",
            "close",
            "derive",
            "score",
            "entity",
            "yield",
            "join",
            "conv",
            "limits",
            "test",
            "input",
            "expect",
            "options",
            "snapshot",
            "asof",
            "within",
            "fixed",
            "session",
            "if",
            "then",
            "else",
            "in",
            "not",
            "true",
            "false",
            "for",
            "row",
            "tick",
            "hits",
            "hit",
            "field",
            "close_reason",
            "distinct",
            "count",
            "sum",
            "avg",
            "min",
            "max",
            "max_memory",
            "max_instances",
            "max_throttle",
            "on_exceed",
        ]
    }

    fn builtins(&self) -> &[BuiltinInfo] {
        &BUILTINS
    }

    fn document_symbols(&self, tree: &Tree, src: &str) -> Vec<SymbolInfo> {
        let mut symbols = vec![];
        let root = tree.root_node();

        for i in 0..root.named_child_count() {
            let Some(child) = root.named_child(i as u32) else {
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
                "test_block" => {
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
            let Some(child) = root.named_child(i as u32) else {
                continue;
            };
            match child.kind() {
                "rule_declaration" | "test_block" => {
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
        let Some(child) = node.named_child(i as u32) else {
            continue;
        };
        if child.kind() == "events_block" {
            for j in 0..child.named_child_count() {
                let Some(ev) = child.named_child(j as u32) else {
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
        let Some(child) = node.named_child(i as u32) else {
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
        if let Some(child) = node.named_child(i as u32) {
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

// L1 builtins
static BUILTINS: [BuiltinInfo; 27] = [
    BuiltinInfo {
        name: "count",
        signature: "count(alias) -> digit",
        documentation: "Count events in a window. Argument is a Set-level alias.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "sum",
        signature: "sum(alias.field) -> digit/float",
        documentation: "Sum a numeric field (digit/float) across events.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "avg",
        signature: "avg(alias.field) -> float",
        documentation: "Average a numeric field (digit/float) across events.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "min",
        signature: "min(alias.field) -> T",
        documentation: "Minimum value of a sortable field (digit/float/time/chars).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "max",
        signature: "max(alias.field) -> T",
        documentation: "Maximum value of a sortable field (digit/float/time/chars).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "distinct",
        signature: "distinct(alias.field) -> digit",
        documentation: "Distinct count of a Column-level field projection.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "fmt",
        signature: "fmt(template, ...args) -> chars",
        documentation: "Format a string with {} placeholders. Placeholder count must match args.",
        kind: CompletionItemKind::FUNCTION,
    },
    // L2 builtins
    BuiltinInfo {
        name: "baseline",
        signature: "baseline(expr, duration[, method]) -> float",
        documentation: "Rolling baseline mean. expr must be digit/float. Optional method: \"mean\" (default) / \"ewma\" / \"median\" (L3).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "window.has",
        signature: "window.has(field[, target_field]) -> bool",
        documentation: "Check if current context field value exists in a static/dimension window. Two-arg form maps to a different target field name.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "hit",
        signature: "hit(cond) -> float",
        documentation: "Map boolean condition to score: true -> 1.0, false -> 0.0.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "time_diff",
        signature: "time_diff(t1, t2) -> float",
        documentation: "Difference between two timestamps in seconds. Both args must be time type.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "time_bucket",
        signature: "time_bucket(field, interval) -> time",
        documentation: "Bucket a time field by interval (DURATION literal). Returns time.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "contains",
        signature: "contains(field, pattern) -> bool",
        documentation: "Check if field (chars/ip/hex) contains the pattern substring.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "regex_match",
        signature: "regex_match(field, pattern) -> bool",
        documentation: "Check if field (chars/ip/hex) matches the regex pattern (STRING literal).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "len",
        signature: "len(field) -> digit",
        documentation: "String length of a chars/ip/hex field.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "lower",
        signature: "lower(field) -> chars",
        documentation: "Convert a chars field to lowercase.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "upper",
        signature: "upper(field) -> chars",
        documentation: "Convert a chars field to uppercase.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "coalesce",
        signature: "coalesce(expr, default) -> T",
        documentation: "Return expr if non-null, otherwise default. Both must have the same type.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "try",
        signature: "try(expr, default) -> T",
        documentation: "Evaluate expr; on error return default. Both must have the same type.",
        kind: CompletionItemKind::FUNCTION,
    },
    // L3 builtins
    BuiltinInfo {
        name: "collect_set",
        signature: "collect_set(alias.field) -> array/T",
        documentation: "Collect distinct values within a window (L3). Returns array of field's type.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "collect_list",
        signature: "collect_list(alias.field) -> array/T",
        documentation: "Collect ordered values within a window (L3). Returns array of field's type.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "first",
        signature: "first(alias.field) -> T",
        documentation: "First value of a field within a window (L3).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "last",
        signature: "last(alias.field) -> T",
        documentation: "Last value of a field within a window (L3).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "stddev",
        signature: "stddev(alias.field) -> float",
        documentation: "Standard deviation of a numeric field (digit/float) within a window (L3).",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "percentile",
        signature: "percentile(alias.field, p) -> float",
        documentation: "Percentile of a numeric field (digit/float). p is 0-100 (L3).",
        kind: CompletionItemKind::FUNCTION,
    },
    // Utility
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
];
