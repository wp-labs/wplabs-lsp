use lsp_types::*;
use tree_sitter::Tree;

use crate::lang::{BuiltinInfo, LangHandler, SymbolInfo};
use crate::util::{node_range, node_text};

pub struct GxlHandler;

impl LangHandler for GxlHandler {
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_gxl::language()
    }

    fn lang_id(&self) -> &str {
        "gxl"
    }

    fn extensions(&self) -> &[&str] {
        &["gxl"]
    }

    fn keywords(&self) -> &[&str] {
        &[
            "mod",
            "env",
            "flow",
            "fn",
            "activity",
            "extern",
            "path",
            "git",
            "channel",
            "gx.echo",
            "gx.vars",
            "gx.cmd",
            "gx.read",
            "gx.tpl",
            "gx.assert",
            "gx.ver",
            "gx.read_cmd",
            "gx.read_stdin",
            "gx.read_file",
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
                "module" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let mut children = vec![];
                        collect_module_children(&child, src, &mut children);
                        symbols.push(SymbolInfo {
                            name: node_text(&name_node, src).to_string(),
                            kind: SymbolKind::MODULE,
                            range: node_range(&child),
                            selection_range: node_range(&name_node),
                            children,
                        });
                    }
                }
                "extern_module" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        symbols.push(SymbolInfo {
                            name: node_text(&name_node, src).to_string(),
                            kind: SymbolKind::MODULE,
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
        find_defs(tree.root_node(), src, name, &mut defs);
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

fn collect_module_children(node: &tree_sitter::Node, src: &str, symbols: &mut Vec<SymbolInfo>) {
    for i in 0..node.named_child_count() {
        let Some(child) = node.named_child(i as u32) else {
            continue;
        };
        match child.kind() {
            "environment" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    symbols.push(SymbolInfo {
                        name: node_text(&name_node, src).to_string(),
                        kind: SymbolKind::NAMESPACE,
                        range: node_range(&child),
                        selection_range: node_range(&name_node),
                        children: vec![],
                    });
                }
            }
            "flow_definition" | "flow_reference" => {
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
            "function_def" => {
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
            "activity" => {
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
}

fn find_defs(node: tree_sitter::Node, src: &str, name: &str, defs: &mut Vec<Range>) {
    match node.kind() {
        "module" | "environment" | "flow_definition" | "flow_reference" | "function_def"
        | "activity" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                if node_text(&name_node, src) == name {
                    defs.push(node_range(&name_node));
                }
            }
        }
        _ => {}
    }
    for i in 0..node.named_child_count() {
        if let Some(child) = node.named_child(i as u32) {
            find_defs(child, src, name, defs);
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

static BUILTINS: [BuiltinInfo; 10] = [
    BuiltinInfo {
        name: "gx.echo",
        signature: "gx.echo(value: STRING)",
        documentation: "Output a value to the console.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.vars",
        signature: "gx.vars { key: value, ... }",
        documentation: "Define a block of variables.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.cmd",
        signature: "gx.cmd(cmd: STRING)",
        documentation: "Execute a shell command.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.read",
        signature: "gx.read(name: STRING, ...)",
        documentation: "Read input into a variable.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.tpl",
        signature: "gx.tpl(src: STRING, dest: STRING)",
        documentation: "Render a template from src to dest.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.assert",
        signature: "gx.assert(expr: STRING)",
        documentation: "Assert that an expression is true.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.ver",
        signature: "gx.ver(name: STRING)",
        documentation: "Check the version of a tool or dependency.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.read_cmd",
        signature: "gx.read_cmd(name: STRING, cmd: STRING)",
        documentation: "Read a variable from the output of a shell command.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.read_stdin",
        signature: "gx.read_stdin(name: STRING)",
        documentation: "Read a variable from standard input.",
        kind: CompletionItemKind::FUNCTION,
    },
    BuiltinInfo {
        name: "gx.read_file",
        signature: "gx.read_file(name: STRING, path: STRING)",
        documentation: "Read a variable from a file.",
        kind: CompletionItemKind::FUNCTION,
    },
];
