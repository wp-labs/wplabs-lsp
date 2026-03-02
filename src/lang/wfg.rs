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
            "use",
            "scenario",
            "traffic",
            "stream",
            "gen",
            "wave",
            "burst",
            "timeline",
            "injection",
            "hit",
            "near_miss",
            "miss",
            "seq",
            "with",
            "expect",
            "duration",
            "tick",
            "rows",
            "emit",
            "auto",
            "base",
            "amp",
            "period",
            "shape",
            "peak",
            "every",
            "hold",
            "true",
            "false",
            "deterministic",
            "poisson",
            "sine",
            "triangle",
            "square",
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
        let Some(child) = node.named_child(i as u32) else {
            continue;
        };
        if child.kind() == "stream_statement" {
            if let Some(stream) = child.child_by_field_name("stream") {
                symbols.push(SymbolInfo {
                    name: node_text(&stream, src).to_string(),
                    kind: SymbolKind::VARIABLE,
                    range: node_range(&child),
                    selection_range: node_range(&stream),
                    children: vec![],
                });
            }
        }
        collect_stream_symbols(&child, src, symbols);
    }
}

fn collect_inject_symbols(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i as u32) else {
            continue;
        };
        if child.kind() == "injection_case" {
            let Some(mode_node) = child.child_by_field_name("mode") else {
                continue;
            };
            let Some(stream_node) = child.child_by_field_name("stream") else {
                continue;
            };
            symbols.push(SymbolInfo {
                name: format!(
                    "{} {}",
                    node_text(&mode_node, src),
                    node_text(&stream_node, src)
                ),
                kind: SymbolKind::EVENT,
                range: node_range(&child),
                selection_range: node_range(&stream_node),
                children: vec![],
            });
        }
        collect_inject_symbols(&child, src, symbols);
    }
}

fn find_scenario_defs(node: tree_sitter::Node, src: &str, name: &str, defs: &mut Vec<Range>) {
    match node.kind() {
        "scenario_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                if node_text(&name_node, src) == name {
                    defs.push(node_range(&name_node));
                }
            }
        }
        "stream_statement" => {
            if let Some(stream) = node.child_by_field_name("stream") {
                if node_text(&stream, src) == name {
                    defs.push(node_range(&stream));
                }
            }
        }
        _ => {}
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i as u32) {
            find_scenario_defs(child, src, name, defs);
        }
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

static BUILTINS: [BuiltinInfo; 0] = [];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tree_sitter::Parser;

    fn parse_ok(src: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_wfg::language())
            .expect("load tree-sitter-wfg language");
        let tree = parser.parse(src, None).expect("parse source");

        let mut errors = Vec::new();
        collect_errors(tree.root_node(), src, &mut errors);
        assert!(
            !tree.root_node().has_error(),
            "source should parse without ERROR nodes: {errors:?}"
        );

        tree
    }

    fn collect_errors(node: tree_sitter::Node, src: &str, errors: &mut Vec<String>) {
        if node.is_error() || node.is_missing() {
            let start = node.start_position();
            let end = node.end_position();
            let snippet = &src[node.byte_range()];
            errors.push(format!(
                "{} [{}:{}-{}:{}] {:?}",
                node.kind(),
                start.row,
                start.column,
                end.row,
                end.column,
                snippet
            ));
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i as u32) {
                collect_errors(child, src, errors);
            }
        }
    }

    #[test]
    fn wfg_keywords_follow_new_design_terms() {
        let handler = WfgHandler;
        let keywords: HashSet<&str> = handler.keywords().iter().copied().collect();

        for required in [
            "use",
            "scenario",
            "traffic",
            "stream",
            "gen",
            "injection",
            "hit",
            "near_miss",
            "miss",
            "expect",
            "seq",
            "with",
            "duration",
            "tick",
            "rows",
            "emit",
            "auto",
            "deterministic",
            "poisson",
            "wave",
            "burst",
            "timeline",
        ] {
            assert!(
                keywords.contains(required),
                "missing required keyword: {required}"
            );
        }

        for removed in ["inject", "non_hit", "oracle", "faults", "time", "total"] {
            assert!(
                !keywords.contains(removed),
                "legacy keyword should not appear in completions: {removed}"
            );
        }
    }

    #[test]
    fn wfg_builtins_are_empty_for_keyword_driven_dsl() {
        let handler = WfgHandler;
        assert!(handler.builtins().is_empty());
    }

    #[test]
    fn parse_and_extract_symbols_from_new_wfg_design_example() {
        let source = r#"
use "../schemas/security.wfs"
use "../rules/brute_force.wfl"

#[duration=10m]
scenario brute_force_detect<seed=42> {
  traffic {
    stream auth_events gen 100/s
    stream auth_events gen wave(base=80/s, amp=40/s, period=2m, shape=sine)
    stream auth_events gen burst(base=20/s, peak=120/s, every=5m, hold=30s)
    stream auth_events gen timeline {
      0s..2m=20/s
      2m..6m=120/s
    }
  }

  injection {
    hit<30%> auth_events {
      user seq {
        use(login="failed") with(3,2m)
        use(action="port_scan") with(1,1m)
      }
    }

    near_miss<10%> auth_events {
      user seq {
        use(login="failed") with(2,2m)
      }
    }

    miss<60%> auth_events {
      user seq {
        use(login="success") with(1,30s)
      }
    }
  }

  expect {
    hit(brute_force_then_scan) >= 95%
    near_miss(brute_force_then_scan) <= 1%
    miss(brute_force_then_scan) <= 0.1%
  }
}
"#;

        let tree = parse_ok(source);
        let handler = WfgHandler;
        let symbols = handler.document_symbols(&tree, source);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "brute_force_detect");

        let child_names: HashSet<&str> = symbols[0]
            .children
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        assert!(child_names.contains("auth_events"));
        assert!(child_names.contains("hit auth_events"));
        assert!(child_names.contains("near_miss auth_events"));
        assert!(child_names.contains("miss auth_events"));
    }
}
